"use client";

import { useEffect, useRef, useState } from "react";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { TrackData } from "@/lib/api";

interface ActivityMapProps {
  trackData: TrackData;
  highlightIndex?: number;
  onHover?: (index: number | null) => void;
  selectionStart?: number | null;
  selectionEnd?: number | null;
}

export function ActivityMap({
  trackData,
  highlightIndex,
  onHover,
  selectionStart,
  selectionEnd,
}: ActivityMapProps) {
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
      style: {
        version: 8,
        sources: {
          opentopomap: {
            type: "raster",
            tiles: ["https://tile.opentopomap.org/{z}/{x}/{y}.png"],
            tileSize: 256,
            attribution: '&copy; <a href="https://opentopomap.org">OpenTopoMap</a> (<a href="https://creativecommons.org/licenses/by-sa/3.0/">CC-BY-SA</a>)',
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

  // Update selection segment layer
  useEffect(() => {
    if (!map.current || !loaded) return;

    const hasSelection =
      selectionStart !== null &&
      selectionStart !== undefined &&
      selectionEnd !== null &&
      selectionEnd !== undefined;

    // Remove existing selection layer and source
    if (map.current.getLayer("selection-route")) {
      map.current.removeLayer("selection-route");
    }
    if (map.current.getSource("selection")) {
      map.current.removeSource("selection");
    }

    if (hasSelection) {
      const startIdx = Math.min(selectionStart, selectionEnd);
      const endIdx = Math.max(selectionStart, selectionEnd);
      const selectedPoints = trackData.points.slice(startIdx, endIdx + 1);
      const coordinates = selectedPoints.map((p) => [p.lon, p.lat]);

      if (coordinates.length >= 2) {
        map.current.addSource("selection", {
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
          id: "selection-route",
          type: "line",
          source: "selection",
          layout: {
            "line-join": "round",
            "line-cap": "round",
          },
          paint: {
            "line-color": "#22c55e",
            "line-width": 6,
          },
        });
      }
    }
  }, [selectionStart, selectionEnd, loaded, trackData.points]);

  return (
    <div
      ref={mapContainer}
      className="w-full h-[300px] md:h-[400px] rounded-lg overflow-hidden"
    />
  );
}
