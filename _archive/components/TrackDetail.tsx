"use client";

import { Track } from "@/lib/database";
import { TrackPoint } from "@/lib/gpx-parser";
import { ChevronLeftIcon } from "@heroicons/react/24/outline";
import dynamic from "next/dynamic";

const TrackMap = dynamic(() => import("./TrackMap"), {
  ssr: false,
  loading: () => (
    <div className="w-full h-96 bg-gray-200 rounded-lg flex items-center justify-center">
      <p className="text-gray-500">Loading map...</p>
    </div>
  ),
});

interface TrackDetailProps {
  track: Track & { coordinates: TrackPoint[] };
  onBack: () => void;
}

function formatDistance(meters: number): string {
  const km = meters / 1000;
  return `${km.toFixed(2)} km`;
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${minutes}:${secs.toString().padStart(2, "0")}`;
}

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function TrackDetail({ track, onBack }: TrackDetailProps) {
  return (
    <div className="space-y-6">
      <div className="flex items-center space-x-4">
        <button onClick={onBack} className="flex items-center text-blue-600 hover:text-blue-800 transition-colors">
          <ChevronLeftIcon width={25} className="w-44 h-44 mr-1" />
          Back to Tracks
        </button>
      </div>

      <div className="bg-white rounded-lg shadow-md p-6">
        <div className="mb-6">
          <h1 className="text-2xl font-bold mb-2">{track.name}</h1>
          <p className="text-gray-600">{formatDate(track.uploadDate)}</p>
        </div>

        <div className="mb-6">
          <TrackMap points={track.coordinates} />
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600">{formatDistance(track.distance)}</div>
            <div className="text-sm text-gray-500">Distance</div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">{formatDuration(track.duration)}</div>
            <div className="text-sm text-gray-500">Duration</div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-orange-600">{Math.round(track.elevationGain)}m</div>
            <div className="text-sm text-gray-500">Elevation Gain</div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600">{track.avgSpeed.toFixed(1)} km/h</div>
            <div className="text-sm text-gray-500">Average Speed</div>
          </div>
        </div>

        <div className="mt-6 pt-6 border-t border-gray-200">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-semibold mb-2">Additional Statistics</h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">Max Speed:</span>
                  <span className="font-medium">{track.maxSpeed.toFixed(1)} km/h</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Track Points:</span>
                  <span className="font-medium">{track.coordinates.length}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">File:</span>
                  <span className="font-medium">{track.filename}</span>
                </div>
              </div>
            </div>

            <div>
              <h3 className="font-semibold mb-2">Performance</h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">Pace:</span>
                  <span className="font-medium">
                    {track.avgSpeed > 0 ? `${(60 / track.avgSpeed).toFixed(1)} min/km` : "N/A"}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Calories:</span>
                  <span className="font-medium">{Math.round((track.distance / 1000) * 65)} cal</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
