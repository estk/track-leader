"use client";

import { useEffect, useRef, useState, useMemo } from "react";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { api, DigHeatmapResponse } from "@/lib/api";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Clock, Hash } from "lucide-react";

type DateRange = "7" | "30" | "90" | "all";
type DisplayMode = "duration" | "frequency";

interface DigHeatmapProps {
  teamId?: string;
}

const DATE_RANGE_OPTIONS: { value: DateRange; label: string }[] = [
  { value: "7", label: "Last 7 days" },
  { value: "30", label: "Last 30 days" },
  { value: "90", label: "Last 90 days" },
  { value: "all", label: "All time" },
];

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

export function DigHeatmap({ teamId }: DigHeatmapProps) {
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<maplibregl.Map | null>(null);
  const [dateRange, setDateRange] = useState<DateRange>("30");
  const [displayMode, setDisplayMode] = useState<DisplayMode>("duration");
  const [data, setData] = useState<DigHeatmapResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Fetch dig heatmap data
  useEffect(() => {
    setLoading(true);
    setError("");

    const days = dateRange === "all" ? undefined : parseInt(dateRange, 10);

    const fetchData = async () => {
      try {
        const response = teamId
          ? await api.getTeamDigHeatmap(teamId, days)
          : await api.getGlobalDigHeatmap(days);
        setData(response);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load dig heatmap data");
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [teamId, dateRange]);

  // Calculate max weight for normalization
  const maxWeight = useMemo(() => {
    if (!data || data.points.length === 0) return 1;
    return Math.max(
      ...data.points.map((p) =>
        displayMode === "duration" ? p.total_duration_seconds : p.frequency
      )
    );
  }, [data, displayMode]);

  // Initialize and update map
  useEffect(() => {
    if (loading || !mapContainer.current) return;

    // Clean up existing map
    if (map.current) {
      map.current.remove();
      map.current = null;
    }

    if (!data || data.points.length === 0 || !data.bounds) return;

    const bounds = data.bounds;
    const center: [number, number] = [
      (bounds.min_lon + bounds.max_lon) / 2,
      (bounds.min_lat + bounds.max_lat) / 2,
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
      if (!map.current || !data) return;

      // Create GeoJSON for heatmap points with weight property
      const geojsonData: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: data.points.map((point) => ({
          type: "Feature",
          properties: {
            weight:
              displayMode === "duration"
                ? point.total_duration_seconds / maxWeight
                : point.frequency / maxWeight,
            duration: point.total_duration_seconds,
            frequency: point.frequency,
          },
          geometry: {
            type: "Point",
            coordinates: [point.lon, point.lat],
          },
        })),
      };

      // Add heatmap source
      map.current.addSource("dig-heatmap-data", {
        type: "geojson",
        data: geojsonData,
      });

      // Add heatmap layer with orange/red color gradient (distinct from activity heatmap)
      map.current.addLayer({
        id: "dig-heatmap-layer",
        type: "heatmap",
        source: "dig-heatmap-data",
        paint: {
          // Weight per point based on normalized value
          "heatmap-weight": ["get", "weight"],
          // Intensity by zoom level
          "heatmap-intensity": [
            "interpolate",
            ["linear"],
            ["zoom"],
            0, 0.6,
            10, 1,
            15, 1.5,
          ],
          // Orange/red color gradient for dig work visualization
          "heatmap-color": [
            "interpolate",
            ["linear"],
            ["heatmap-density"],
            0, "rgba(255, 165, 0, 0)",        // transparent (no density)
            0.1, "rgba(255, 200, 100, 0.6)",  // light orange (low)
            0.25, "rgba(255, 165, 0, 0.7)",   // orange
            0.4, "rgba(255, 140, 0, 0.75)",   // dark orange
            0.55, "rgba(255, 100, 0, 0.8)",   // orange-red
            0.7, "rgba(255, 69, 0, 0.85)",    // red-orange
            0.85, "rgba(220, 20, 60, 0.9)",   // crimson
            1, "rgba(139, 0, 0, 0.95)",       // dark red (high concentration)
          ],
          // Radius by zoom - visible but not blobby
          "heatmap-radius": [
            "interpolate",
            ["linear"],
            ["zoom"],
            0, 4,
            10, 12,
            15, 20,
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

      // Fit bounds to show all dig data
      map.current.fitBounds(
        [
          [bounds.min_lon, bounds.min_lat],
          [bounds.max_lon, bounds.max_lat],
        ],
        { padding: 50 }
      );
    });

    map.current.addControl(new maplibregl.NavigationControl());

    return () => {
      map.current?.remove();
      map.current = null;
    };
  }, [loading, data, displayMode, maxWeight]);

  if (error) {
    return (
      <div className="p-4 text-destructive bg-destructive/10 rounded-md">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <h3 className="text-lg font-semibold">Trail Work Heatmap</h3>
        <div className="flex items-center gap-2">
          {/* Display mode toggle */}
          <div className="flex rounded-md border">
            <Button
              variant={displayMode === "duration" ? "default" : "ghost"}
              size="sm"
              onClick={() => setDisplayMode("duration")}
              className="rounded-r-none gap-1"
            >
              <Clock className="h-4 w-4" />
              Duration
            </Button>
            <Button
              variant={displayMode === "frequency" ? "default" : "ghost"}
              size="sm"
              onClick={() => setDisplayMode("frequency")}
              className="rounded-l-none gap-1"
            >
              <Hash className="h-4 w-4" />
              Frequency
            </Button>
          </div>

          {/* Date range selector */}
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
      </div>

      {loading ? (
        <div className="space-y-4">
          <Skeleton className="h-[750px] w-full rounded-lg" />
          <p className="text-sm text-muted-foreground text-center">
            Loading trail work data...
          </p>
        </div>
      ) : !data || data.points.length === 0 ? (
        <div className="p-8 text-center text-muted-foreground border rounded-lg">
          No trail work data found for this date range.
        </div>
      ) : (
        <div className="space-y-2">
          <div
            ref={mapContainer}
            className="w-full h-[750px] rounded-lg overflow-hidden"
          />
          <p className="text-sm text-muted-foreground text-center">
            {data.total_dig_count} dig segment{data.total_dig_count === 1 ? "" : "s"} totaling {formatDuration(data.total_dig_time_seconds)} of trail work
          </p>
        </div>
      )}
    </div>
  );
}
