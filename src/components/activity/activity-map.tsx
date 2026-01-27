"use client";

import { useEffect, useRef, useState } from "react";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { TrackData } from "@/lib/api";

interface ActivityMapProps {
  trackData: TrackData;
  highlightIndex?: number;
  onHover?: (index: number | null) => void;
}

export function ActivityMap({ trackData, highlightIndex, onHover }: ActivityMapProps) {
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<maplibregl.Map | null>(null);
  const markerRef = useRef<maplibregl.Marker | null>(null);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    if (!mapContainer.current || map.current) return;

    const { bounds } = trackData;
    const center: [number, number] = [
      (bounds.min_lon + bounds.max_lon) / 2,
      (bounds.min_lat + bounds.max_lat) / 2,
    ];

    map.current = new maplibregl.Map({
      container: mapContainer.current,
      style: "https://demotiles.maplibre.org/style.json",
      center,
      zoom: 12,
    });

    map.current.on("load", () => {
      if (!map.current) return;
      setLoaded(true);

      const coordinates = trackData.points.map((p) => [p.lon, p.lat]);

      map.current.addSource("route", {
        type: "geojson",
        data: {
          type: "Feature",
          properties: {},
          geometry: {
            type: "LineString",
            coordinates,
          },
        },
      });

      map.current.addLayer({
        id: "route",
        type: "line",
        source: "route",
        layout: {
          "line-join": "round",
          "line-cap": "round",
        },
        paint: {
          "line-color": "#3b82f6",
          "line-width": 4,
        },
      });

      // Add start marker
      if (coordinates.length > 0) {
        new maplibregl.Marker({ color: "#22c55e" })
          .setLngLat(coordinates[0] as [number, number])
          .addTo(map.current);
      }

      // Add end marker
      if (coordinates.length > 1) {
        new maplibregl.Marker({ color: "#ef4444" })
          .setLngLat(coordinates[coordinates.length - 1] as [number, number])
          .addTo(map.current);
      }

      // Fit bounds
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
  }, [trackData]);

  // Update highlight marker when hovering on elevation chart
  useEffect(() => {
    if (!map.current || !loaded || highlightIndex === undefined) {
      if (markerRef.current) {
        markerRef.current.remove();
        markerRef.current = null;
      }
      return;
    }

    const point = trackData.points[highlightIndex];
    if (!point) return;

    if (!markerRef.current) {
      markerRef.current = new maplibregl.Marker({ color: "#f97316" })
        .setLngLat([point.lon, point.lat])
        .addTo(map.current);
    } else {
      markerRef.current.setLngLat([point.lon, point.lat]);
    }
  }, [highlightIndex, loaded, trackData.points]);

  return (
    <div
      ref={mapContainer}
      className="w-full h-[400px] rounded-lg overflow-hidden"
    />
  );
}
