"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import maplibregl, { GeoJSONSource } from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { api, Segment, SegmentTrackData } from "@/lib/api";
import { Skeleton } from "@/components/ui/skeleton";

interface SegmentWithTrack {
  segment: Segment;
  track: SegmentTrackData;
}

interface SegmentsMapProps {
  segments: Segment[];
}

const MAX_SEGMENTS_TO_FETCH = 20;

// Zoom level threshold for switching between clusters and individual segments
const CLUSTER_MAX_ZOOM = 12;
const SEGMENT_MIN_ZOOM = 10;

// Colors for different segments
const SEGMENT_COLORS = [
  "#3b82f6", // blue
  "#ef4444", // red
  "#22c55e", // green
  "#f59e0b", // amber
  "#8b5cf6", // violet
  "#ec4899", // pink
  "#06b6d4", // cyan
  "#f97316", // orange
];

export function SegmentsMap({ segments }: SegmentsMapProps) {
  const router = useRouter();
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<maplibregl.Map | null>(null);
  const [segmentsWithTracks, setSegmentsWithTracks] = useState<SegmentWithTrack[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Fetch tracks for all segments (limited to MAX_SEGMENTS_TO_FETCH)
  useEffect(() => {
    const segmentsToFetch = segments.slice(0, MAX_SEGMENTS_TO_FETCH);

    if (segmentsToFetch.length === 0) {
      setLoading(false);
      return;
    }

    setLoading(true);
    setError("");

    Promise.all(
      segmentsToFetch.map(async (segment) => {
        try {
          const track = await api.getSegmentTrack(segment.id);
          return { segment, track };
        } catch {
          // Skip segments that fail to load
          return null;
        }
      })
    )
      .then((results) => {
        const validResults = results.filter((r): r is SegmentWithTrack => r !== null);
        setSegmentsWithTracks(validResults);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [segments]);

  // Initialize and update map
  useEffect(() => {
    if (loading || !mapContainer.current) return;

    // Clean up existing map
    if (map.current) {
      map.current.remove();
      map.current = null;
    }

    if (segmentsWithTracks.length === 0) return;

    // Calculate combined bounds
    let minLat = Infinity;
    let maxLat = -Infinity;
    let minLon = Infinity;
    let maxLon = -Infinity;

    segmentsWithTracks.forEach(({ track }) => {
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

      // Create GeoJSON for clustered start points
      const startPointsGeoJSON: GeoJSON.FeatureCollection = {
        type: "FeatureCollection",
        features: segmentsWithTracks.map(({ segment, track }, index) => ({
          type: "Feature",
          properties: {
            id: segment.id,
            name: segment.name,
            colorIndex: index % SEGMENT_COLORS.length,
          },
          geometry: {
            type: "Point",
            coordinates: [track.points[0].lon, track.points[0].lat],
          },
        })),
      };

      // Add clustered source for segment start points
      map.current!.addSource("segment-clusters", {
        type: "geojson",
        data: startPointsGeoJSON,
        cluster: true,
        clusterMaxZoom: CLUSTER_MAX_ZOOM,
        clusterRadius: 50,
      });

      // Add cluster circle layer
      map.current!.addLayer({
        id: "clusters",
        type: "circle",
        source: "segment-clusters",
        filter: ["has", "point_count"],
        paint: {
          "circle-color": [
            "step",
            ["get", "point_count"],
            "#3b82f6", // blue for small clusters
            5,
            "#8b5cf6", // violet for medium clusters
            10,
            "#ef4444", // red for large clusters
          ],
          "circle-radius": [
            "step",
            ["get", "point_count"],
            20,
            5,
            25,
            10,
            30,
          ],
          "circle-stroke-width": 2,
          "circle-stroke-color": "#ffffff",
        },
      });

      // Add cluster count label
      map.current!.addLayer({
        id: "cluster-count",
        type: "symbol",
        source: "segment-clusters",
        filter: ["has", "point_count"],
        layout: {
          "text-field": "{point_count_abbreviated}",
          "text-font": ["Open Sans Bold", "Arial Unicode MS Bold"],
          "text-size": 14,
        },
        paint: {
          "text-color": "#ffffff",
        },
      });

      // Add unclustered point layer (individual segment markers)
      map.current!.addLayer({
        id: "unclustered-point",
        type: "circle",
        source: "segment-clusters",
        filter: ["!", ["has", "point_count"]],
        paint: {
          "circle-color": [
            "match",
            ["get", "colorIndex"],
            0, SEGMENT_COLORS[0],
            1, SEGMENT_COLORS[1],
            2, SEGMENT_COLORS[2],
            3, SEGMENT_COLORS[3],
            4, SEGMENT_COLORS[4],
            5, SEGMENT_COLORS[5],
            6, SEGMENT_COLORS[6],
            7, SEGMENT_COLORS[7],
            SEGMENT_COLORS[0], // default
          ],
          "circle-radius": 8,
          "circle-stroke-width": 2,
          "circle-stroke-color": "#ffffff",
        },
      });

      // Add each segment as a separate source and layer (visible at higher zoom)
      segmentsWithTracks.forEach(({ segment, track }, index) => {
        const coordinates = track.points.map((p) => [p.lon, p.lat]);
        const color = SEGMENT_COLORS[index % SEGMENT_COLORS.length];
        const sourceId = `segment-${segment.id}`;
        const layerId = `segment-line-${segment.id}`;

        map.current!.addSource(sourceId, {
          type: "geojson",
          data: {
            type: "Feature",
            properties: {
              id: segment.id,
              name: segment.name,
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
          minzoom: SEGMENT_MIN_ZOOM,
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

        // Add click handler to navigate to segment detail
        map.current!.on("click", layerId, () => {
          router.push(`/segments/${segment.id}`);
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
            .setHTML(`<strong>${segment.name}</strong>`)
            .addTo(map.current!);
        });

        map.current!.on("mouseleave", layerId, () => {
          popup.remove();
        });
      });

      // Handle cluster click - zoom in to expand
      map.current!.on("click", "clusters", async (e) => {
        const features = map.current!.queryRenderedFeatures(e.point, {
          layers: ["clusters"],
        });
        if (!features.length) return;

        const clusterId = features[0].properties?.cluster_id;
        if (clusterId === undefined) return;

        const source = map.current!.getSource("segment-clusters") as GeoJSONSource;
        try {
          const zoom = await source.getClusterExpansionZoom(clusterId);
          if (zoom === undefined || zoom === null) return;

          const geometry = features[0].geometry;
          if (geometry.type !== "Point") return;

          map.current!.easeTo({
            center: geometry.coordinates as [number, number],
            zoom: zoom,
          });
        } catch {
          // Ignore cluster expansion errors
        }
      });

      // Handle unclustered point click - navigate to segment
      map.current!.on("click", "unclustered-point", (e) => {
        const features = map.current!.queryRenderedFeatures(e.point, {
          layers: ["unclustered-point"],
        });
        if (!features.length) return;

        const segmentId = features[0].properties?.id;
        if (segmentId) {
          router.push(`/segments/${segmentId}`);
        }
      });

      // Cursor styling for clusters and points
      map.current!.on("mouseenter", "clusters", () => {
        map.current!.getCanvas().style.cursor = "pointer";
      });
      map.current!.on("mouseleave", "clusters", () => {
        map.current!.getCanvas().style.cursor = "";
      });
      map.current!.on("mouseenter", "unclustered-point", () => {
        map.current!.getCanvas().style.cursor = "pointer";
      });
      map.current!.on("mouseleave", "unclustered-point", () => {
        map.current!.getCanvas().style.cursor = "";
      });

      // Popup for unclustered points
      const pointPopup = new maplibregl.Popup({
        closeButton: false,
        closeOnClick: false,
      });

      map.current!.on("mouseenter", "unclustered-point", (e) => {
        if (!e.features?.length || !e.lngLat) return;
        const name = e.features[0].properties?.name;
        if (name) {
          pointPopup
            .setLngLat(e.lngLat)
            .setHTML(`<strong>${name}</strong>`)
            .addTo(map.current!);
        }
      });

      map.current!.on("mouseleave", "unclustered-point", () => {
        pointPopup.remove();
      });

      // Fit bounds to show all segments
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
  }, [loading, segmentsWithTracks, router]);

  if (loading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-[500px] w-full rounded-lg" />
        <p className="text-sm text-muted-foreground text-center">
          Loading segment tracks...
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

  if (segmentsWithTracks.length === 0) {
    return (
      <div className="p-8 text-center text-muted-foreground">
        No segments to display on the map.
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div
        ref={mapContainer}
        className="w-full h-[500px] rounded-lg overflow-hidden"
      />
      {segments.length > MAX_SEGMENTS_TO_FETCH && (
        <p className="text-sm text-muted-foreground text-center">
          Showing first {MAX_SEGMENTS_TO_FETCH} of {segments.length} segments.
          Use filters to narrow down results.
        </p>
      )}
    </div>
  );
}
