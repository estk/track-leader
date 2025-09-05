"use client";

import "leaflet/dist/leaflet.css";
import { as_latlon, TrackPoint } from "@/lib/gpx-parser";
import { MapPinIcon } from "@heroicons/react/24/solid";
import { useEffect } from "react";
import { MapContainer, Polyline, TileLayer } from "react-leaflet";

interface TrackMapProps {
  points: TrackPoint[];
}

export default function TrackMap({ points }: TrackMapProps) {
  if (points.length === 0) {
    return (
      <div className="w-full h-96 bg-gray-200 rounded-lg flex items-center justify-center">
        <p className="text-gray-500">No track data available</p>
      </div>
    );
  }

  const positions = points.map((p) => {
    return as_latlon(p);
  });
  const center = positions[0];

  const map = (
    <MapContainer center={center} zoom={13} scrollWheelZoom={false}>
      <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" />
      <Polyline pathOptions={{ fillColor: "red", color: "blue" }} positions={positions} />
    </MapContainer>
  );
  return (
    <div className="w-full h-96 bg-blue-50 rounded-lg border-2 border-blue-200 flex flex-col items-center justify-center">
      <div className="text-blue-600 mb-2">
        <MapPinIcon width={24} height={24} className="inline-block mr-2" />
      </div>
      {map}
      <p className="text-blue-700 font-medium">Track Route ({points.length} points)</p>
    </div>
  );
}
