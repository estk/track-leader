"use client";

import { useEffect, useRef, useState, useMemo } from "react";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { api, FeedActivity, TrackData } from "@/lib/api";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

type DateRange = "7" | "30" | "90" | "all";

interface TeamHeatmapProps {
  teamId: string;
}

interface ActivityWithTrack {
  activity: FeedActivity;
  track: TrackData;
}

const MAX_ACTIVITIES = 50;
const POINT_SAMPLE_INTERVAL = 20; // Sample every Nth point - sparse enough that single tracks show as cool colors

const DATE_RANGE_OPTIONS: { value: DateRange; label: string }[] = [
  { value: "7", label: "Last 7 days" },
  { value: "30", label: "Last 30 days" },
  { value: "90", label: "Last 90 days" },
  { value: "all", label: "All time" },
];

function getDateRangeStart(range: DateRange): string | undefined {
  if (range === "all") return undefined;

  const now = new Date();
  const days = parseInt(range, 10);
  const start = new Date(now.getTime() - days * 24 * 60 * 60 * 1000);
  return start.toISOString().split("T")[0];
}

export function TeamHeatmap({ teamId }: TeamHeatmapProps) {
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<maplibregl.Map | null>(null);
  const [dateRange, setDateRange] = useState<DateRange>("30");
  const [activities, setActivities] = useState<FeedActivity[]>([]);
  const [activitiesWithTracks, setActivitiesWithTracks] = useState<ActivityWithTrack[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Fetch activities for the team within the date range
  useEffect(() => {
    setLoading(true);
    setError("");

    const fetchActivities = async () => {
      try {
        // Get team activities - the API supports limit parameter
        const teamActivities = await api.getTeamActivities(teamId, MAX_ACTIVITIES);

        // Filter by date range if not "all"
        const startDate = getDateRangeStart(dateRange);
        const filteredActivities = startDate
          ? teamActivities.filter((a) => a.submitted_at >= startDate)
          : teamActivities;

        setActivities(filteredActivities.slice(0, MAX_ACTIVITIES));
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load activities");
      }
    };

    fetchActivities();
  }, [teamId, dateRange]);

  // Fetch tracks for all activities
  useEffect(() => {
    if (activities.length === 0) {
      setActivitiesWithTracks([]);
      setLoading(false);
      return;
    }

    const fetchTracks = async () => {
      const results = await Promise.all(
        activities.map(async (activity) => {
          try {
            const track = await api.getActivityTrack(activity.id);
            return { activity, track };
          } catch {
            return null;
          }
        })
      );

      const validResults = results.filter((r): r is ActivityWithTrack => r !== null);
      setActivitiesWithTracks(validResults);
      setLoading(false);
    };

    fetchTracks();
  }, [activities]);

  // Extract and sample heatmap points from all tracks
  const heatmapPoints = useMemo(() => {
    const points: [number, number][] = [];

    activitiesWithTracks.forEach(({ track }) => {
      // Sample every Nth point for performance
      for (let i = 0; i < track.points.length; i += POINT_SAMPLE_INTERVAL) {
        const p = track.points[i];
        points.push([p.lon, p.lat]);
      }
    });

    return points;
  }, [activitiesWithTracks]);

  // Calculate bounds from all track data
  const bounds = useMemo(() => {
    if (activitiesWithTracks.length === 0) return null;

    let minLat = Infinity;
    let maxLat = -Infinity;
    let minLon = Infinity;
    let maxLon = -Infinity;

    activitiesWithTracks.forEach(({ track }) => {
      minLat = Math.min(minLat, track.bounds.min_lat);
      maxLat = Math.max(maxLat, track.bounds.max_lat);
      minLon = Math.min(minLon, track.bounds.min_lon);
      maxLon = Math.max(maxLon, track.bounds.max_lon);
    });

    return { minLat, maxLat, minLon, maxLon };
  }, [activitiesWithTracks]);

  // Initialize and update map
  useEffect(() => {
    if (loading || !mapContainer.current) return;

    // Clean up existing map
    if (map.current) {
      map.current.remove();
      map.current = null;
    }

    if (heatmapPoints.length === 0 || !bounds) return;

    const center: [number, number] = [
      (bounds.minLon + bounds.maxLon) / 2,
      (bounds.minLat + bounds.maxLat) / 2,
    ];

    map.current = new maplibregl.Map({
      container: mapContainer.current,
      style: {
        version: 8,
        sources: {
          opentopomap: {
            type: "raster",
            tiles: ["https://tile.opentopomap.org/{z}/{x}/{y}.png"],
            tileSize: 256,
            attribution:
              '&copy; <a href="https://opentopomap.org">OpenTopoMap</a> (<a href="https://creativecommons.org/licenses/by-sa/3.0/">CC-BY-SA</a>)',
          },
        },
        layers: [
          {
            id: "opentopomap",
            type: "raster",
            source: "opentopomap",
          },
        ],
      },
      center,
      zoom: 10,
    });

    map.current.on("load", () => {
      if (!map.current) return;

      // Create GeoJSON for heatmap points
      const geojsonData: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: heatmapPoints.map(([lon, lat]) => ({
          type: "Feature",
          properties: {},
          geometry: {
            type: "Point",
            coordinates: [lon, lat],
          },
        })),
      };

      // Add heatmap source
      map.current.addSource("heatmap-data", {
        type: "geojson",
        data: geojsonData,
      });

      // Add heatmap layer with color gradient: blue -> green -> yellow -> orange -> red
      // Tuned for activity tracks - single tracks show as cool colors, overlapping tracks show warmer
      map.current.addLayer({
        id: "heatmap-layer",
        type: "heatmap",
        source: "heatmap-data",
        paint: {
          // Very low weight per point - only overlapping tracks from multiple activities show warm
          "heatmap-weight": 0.3,
          // Low intensity - requires many overlapping points for saturation
          "heatmap-intensity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            0, 0.2,
            10, 0.4,
            15, 0.7,
          ],
          // Color gradient - shifted so single tracks appear blue/cyan
          "heatmap-color": [
            "interpolate",
            ["linear"],
            ["heatmap-density"],
            0, "rgba(0, 0, 255, 0)",       // transparent (no density)
            0.15, "rgba(65, 105, 225, 0.4)", // royal blue (single track)
            0.35, "rgba(0, 191, 255, 0.5)", // deep sky blue (low overlap)
            0.5, "rgba(0, 255, 127, 0.6)", // spring green (medium)
            0.65, "rgba(255, 255, 0, 0.7)", // yellow (medium-high)
            0.8, "rgba(255, 140, 0, 0.8)", // dark orange (high)
            1, "rgba(255, 0, 0, 0.9)",     // red (many overlapping tracks)
          ],
          // Small radius - tracks appear as thin lines rather than blobs
          "heatmap-radius": [
            "interpolate",
            ["linear"],
            ["zoom"],
            0, 1,
            10, 5,
            15, 10,
          ],
          // Fade out at high zoom
          "heatmap-opacity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            14, 1,
            17, 0.3,
          ],
        },
      });

      // Fit bounds to show all activity data
      map.current.fitBounds(
        [
          [bounds.minLon, bounds.minLat],
          [bounds.maxLon, bounds.maxLat],
        ],
        { padding: 50 }
      );
    });

    map.current.addControl(new maplibregl.NavigationControl());

    return () => {
      map.current?.remove();
      map.current = null;
    };
  }, [loading, heatmapPoints, bounds]);

  if (error) {
    return (
      <div className="p-4 text-destructive bg-destructive/10 rounded-md">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Activity Heatmap</h3>
        <Select value={dateRange} onValueChange={(v) => setDateRange(v as DateRange)}>
          <SelectTrigger className="w-[150px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {DATE_RANGE_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {loading ? (
        <div className="space-y-4">
          <Skeleton className="h-[500px] w-full rounded-lg" />
          <p className="text-sm text-muted-foreground text-center">
            Loading activity tracks...
          </p>
        </div>
      ) : activitiesWithTracks.length === 0 ? (
        <div className="p-8 text-center text-muted-foreground border rounded-lg">
          No activities found for this date range.
        </div>
      ) : (
        <div className="space-y-2">
          <div
            ref={mapContainer}
            className="w-full h-[500px] rounded-lg overflow-hidden"
          />
          <p className="text-sm text-muted-foreground text-center">
            Showing {activitiesWithTracks.length} activit{activitiesWithTracks.length === 1 ? "y" : "ies"}
            {activities.length >= MAX_ACTIVITIES && ` (limited to ${MAX_ACTIVITIES})`}
          </p>
        </div>
      )}
    </div>
  );
}
