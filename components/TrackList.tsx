"use client";

import { Track } from "@/lib/database";
import { ChevronRightIcon } from "@heroicons/react/24/outline";

interface TrackListProps {
  tracks: Track[];
  loading: boolean;
  onTrackSelected: (track: Track) => void;
}

function formatDistance(meters: number): string {
  const km = meters / 1000;
  return `${km.toFixed(2)} km`;
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString();
}

export default function TrackList({ tracks, loading, onTrackSelected }: TrackListProps) {
  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6">
        <div className="animate-pulse space-y-4">
          <div className="h-4 bg-gray-200 rounded w-1/4"></div>
          <div className="space-y-3">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="h-20 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h2 className="text-xl font-semibold mb-4">Your Tracks</h2>

      {tracks.length === 0 ? (
        <p className="text-gray-500 text-center py-8">No tracks yet. Upload your first GPX file to get started!</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-200">
                <th className="text-left py-3 px-4 font-medium text-gray-700">Track</th>
                <th className="text-left py-3 px-4 font-medium text-gray-700">Date</th>
                <th className="text-left py-3 px-4 font-medium text-gray-700">Stats</th>
                <th className="text-center py-3 px-4 font-medium text-gray-700 w-16">Details</th>
              </tr>
            </thead>
            <tbody>
              {tracks.map((track) => (
                <tr
                  key={track.id}
                  className="border-b border-gray-100 hover:bg-gray-50 cursor-pointer transition-colors"
                  onClick={() => onTrackSelected(track)}
                >
                  <td className="py-4 px-4">
                    <h3 className="font-medium text-gray-900">{track.name}</h3>
                  </td>

                  <td className="py-4 px-4">
                    <span className="text-sm text-gray-600">{formatDate(track.uploadDate)}</span>
                  </td>

                  <td className="py-4 px-4">
                    <div className="flex flex-wrap gap-4 text-sm">
                      <span className="text-gray-600">
                        <span className="font-medium">{formatDistance(track.distance)}</span>
                      </span>
                      <span className="text-gray-600">
                        <span className="font-medium">{formatDuration(track.duration)}</span>
                      </span>
                      <span className="text-gray-600">
                        <span className="font-medium">{Math.round(track.elevationGain)}m ↗</span>
                      </span>
                      <span className="text-gray-600">
                        <span className="font-medium">{track.avgSpeed.toFixed(1)} km/h</span>
                      </span>
                    </div>
                  </td>

                  <td className="py-4 px-4 text-center">
                    <div className="text-gray-400">
                      <ChevronRightIcon className="w-4 h-4 mx-auto" />
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
