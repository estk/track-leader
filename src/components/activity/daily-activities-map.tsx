"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { api, FeedActivity, TrackData } from "@/lib/api";
import { Skeleton } from "@/components/ui/skeleton";
import { simplifyRoute, getAdaptiveTolerance } from "@/lib/route-simplify";

interface ActivityWithTrack {
  activity: FeedActivity;
  track: TrackData;
}

interface DailyActivitiesMapProps {
  activities: FeedActivity[];
}

const MAX_ACTIVITIES_TO_FETCH = 20;

// Colors for different activities
const ACTIVITY_COLORS = [
  "#3b82f6", // blue
  "#ef4444", // red
  "#22c55e", // green
  "#f59e0b", // amber
  "#8b5cf6", // violet
  "#ec4899", // pink
  "#06b6d4", // cyan
  "#f97316", // orange
];

export function DailyActivitiesMap({ activities }: DailyActivitiesMapProps) {
  const router = useRouter();
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<maplibregl.Map | null>(null);
  const [activitiesWithTracks, setActivitiesWithTracks] = useState<ActivityWithTrack[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Fetch tracks for all activities (limited to MAX_ACTIVITIES_TO_FETCH)
  useEffect(() => {
    const activitiesToFetch = activities.slice(0, MAX_ACTIVITIES_TO_FETCH);

    if (activitiesToFetch.length === 0) {
      setLoading(false);
      return;
    }

    setLoading(true);
    setError("");

    Promise.all(
      activitiesToFetch.map(async (activity) => {
        try {
          const track = await api.getActivityTrack(activity.id);
          return { activity, track };
        } catch {
          // Skip activities that fail to load
          return null;
        }
      })
    )
      .then((results) => {
        const validResults = results.filter((r): r is ActivityWithTrack => r !== null);
        setActivitiesWithTracks(validResults);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [activities]);

  // Initialize and update map
  useEffect(() => {
    if (loading || !mapContainer.current) return;

    // Clean up existing map
    if (map.current) {
      map.current.remove();
      map.current = null;
    }

    if (activitiesWithTracks.length === 0) return;

    // Calculate combined bounds
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

    const center: [number, number] = [
      (minLon + maxLon) / 2,
      (minLat + maxLat) / 2,
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

      // Add each activity as a separate source and layer
      activitiesWithTracks.forEach(({ activity, track }, index) => {
        // Simplify route for better performance
        const tolerance = getAdaptiveTolerance(track.points.length);
        const displayPoints = tolerance > 0
          ? simplifyRoute(track.points, tolerance)
          : track.points;
        const coordinates = displayPoints.map((p) => [p.lon, p.lat]);
        const color = ACTIVITY_COLORS[index % ACTIVITY_COLORS.length];
        const sourceId = `activity-${activity.id}`;
        const layerId = `activity-line-${activity.id}`;

        map.current!.addSource(sourceId, {
          type: "geojson",
          data: {
            type: "Feature",
            properties: {
              id: activity.id,
              name: activity.name,
              userName: activity.user_name,
            },
            geometry: {
              type: "LineString",
              coordinates,
            },
          },
        });

        map.current!.addLayer({
          id: layerId,
          type: "line",
          source: sourceId,
          layout: {
            "line-join": "round",
            "line-cap": "round",
          },
          paint: {
            "line-color": color,
            "line-width": 4,
          },
        });

        // Add hover effect
        map.current!.on("mouseenter", layerId, () => {
          map.current!.getCanvas().style.cursor = "pointer";
          map.current!.setPaintProperty(layerId, "line-width", 6);
        });

        map.current!.on("mouseleave", layerId, () => {
          map.current!.getCanvas().style.cursor = "";
          map.current!.setPaintProperty(layerId, "line-width", 4);
        });

        // Add click handler to navigate to activity detail
        map.current!.on("click", layerId, () => {
          router.push(`/activities/${activity.id}`);
        });

        // Add popup on hover
        const popup = new maplibregl.Popup({
          closeButton: false,
          closeOnClick: false,
        });

        map.current!.on("mouseenter", layerId, (e) => {
          if (!e.lngLat) return;
          popup
            .setLngLat(e.lngLat)
            .setHTML(`<strong>${activity.name}</strong><br/><span style="color: #666">${activity.user_name}</span>`)
            .addTo(map.current!);
        });

        map.current!.on("mouseleave", layerId, () => {
          popup.remove();
        });
      });

      // Fit bounds to show all activities
      map.current!.fitBounds(
        [
          [minLon, minLat],
          [maxLon, maxLat],
        ],
        { padding: 50 }
      );
    });

    map.current.addControl(new maplibregl.NavigationControl());

    return () => {
      map.current?.remove();
      map.current = null;
    };
  }, [loading, activitiesWithTracks, router]);

  if (loading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-[750px] w-full rounded-lg" />
        <p className="text-sm text-muted-foreground text-center">
          Loading activity tracks...
        </p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-destructive bg-destructive/10 rounded-md">
        {error}
      </div>
    );
  }

  if (activitiesWithTracks.length === 0) {
    return (
      <div className="p-8 text-center text-muted-foreground border rounded-lg">
        No activities to display on the map.
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div
        ref={mapContainer}
        className="w-full h-[750px] rounded-lg overflow-hidden"
      />
      {activities.length > MAX_ACTIVITIES_TO_FETCH && (
        <p className="text-sm text-muted-foreground text-center">
          Showing first {MAX_ACTIVITIES_TO_FETCH} of {activities.length} activities on the map.
        </p>
      )}
    </div>
  );
}
