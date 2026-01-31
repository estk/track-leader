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

// Adaptive sampling based on total point count for performance
function getSampleInterval(totalPoints: number): number {
  if (totalPoints < 5000) return 1;
  if (totalPoints < 10000) return 2;
  if (totalPoints < 20000) return 3;
  if (totalPoints < 40000) return 4;
  return 5;
}

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
    // Calculate total points to determine sampling interval
    const totalPoints = activitiesWithTracks.reduce(
      (sum, { track }) => sum + track.points.length,
      0
    );
    const sampleInterval = getSampleInterval(totalPoints);

    const points: [number, number][] = [];
    activitiesWithTracks.forEach(({ track }) => {
      for (let i = 0; i < track.points.length; i += sampleInterval) {
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
      // Tuned for activity tracks - single tracks show as visible blue/cyan, overlapping tracks show warmer
      map.current.addLayer({
        id: "heatmap-layer",
        type: "heatmap",
        source: "heatmap-data",
        paint: {
          // Weight per point
          "heatmap-weight": 1,
          // Intensity by zoom level
          "heatmap-intensity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            0, 0.6,
            10, 1,
            15, 1.5,
          ],
          // Color gradient - single tracks appear blue/cyan, overlapping show warmer
          "heatmap-color": [
            "interpolate",
            ["linear"],
            ["heatmap-density"],
            0, "rgba(0, 0, 255, 0)",       // transparent (no density)
            0.1, "rgba(65, 105, 225, 0.6)", // royal blue (single track)
            0.25, "rgba(0, 191, 255, 0.7)", // deep sky blue (low overlap)
            0.4, "rgba(0, 255, 127, 0.75)", // spring green (medium)
            0.55, "rgba(255, 255, 0, 0.8)", // yellow (medium-high)
            0.7, "rgba(255, 140, 0, 0.85)", // dark orange (high)
            0.85, "rgba(255, 69, 0, 0.9)", // orange-red
            1, "rgba(255, 0, 0, 0.95)",    // red (many overlapping tracks)
          ],
          // Radius by zoom - visible but not blobby
          "heatmap-radius": [
            "interpolate",
            ["linear"],
            ["zoom"],
            0, 3,
            10, 10,
            15, 18,
          ],
          // Fade out at high zoom
          "heatmap-opacity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            14, 1,
            17, 0.5,
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
          <Skeleton className="h-[750px] w-full rounded-lg" />
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
            className="w-full h-[750px] rounded-lg overflow-hidden"
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
