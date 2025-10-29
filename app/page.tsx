"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import TrackList from "@/components/TrackList";
import TrackUpload from "@/components/TrackUpload";

export default function Home() {
  const [tracks, setTracks] = useState<Track[]>([]);
  const [loading, setLoading] = useState(true);
  const router = useRouter();

  useEffect(() => {
    fetchTracks();
  }, []);

  const fetchTracks = async () => {
    try {
      const response = await fetch("/api/tracks");
      const data = await response.json();
      setTracks(data);
    } catch (error) {
      console.error("Failed to fetch tracks:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleTrackUploaded = (newTrack: Track) => {
    setTracks((prev) => [newTrack, ...prev]);
  };

  const handleTrackSelected = (track: Track) => {
    router.push(`/tracks/${track.id}`);
  };

  return (
    <div className="space-y-6">
      <TrackUpload onTrackUploaded={handleTrackUploaded} />
      <TrackList tracks={tracks} loading={loading} onTrackSelected={handleTrackSelected} />
    </div>
  );
}
