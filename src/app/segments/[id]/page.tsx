"use client";

import { useEffect, useMemo, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { api, Segment, SegmentEffort, SegmentTrackData, TrackData } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ActivityMap } from "@/components/activity/activity-map";
import { ElevationProfile } from "@/components/activity/elevation-profile";

const ACTIVITY_TYPE_LABELS: Record<string, string> = {
  Running: "Run",
  RoadCycling: "Road Cycling",
  MountainBiking: "Mountain Biking",
  Hiking: "Hike",
  Walking: "Walk",
  Unknown: "Other",
};

function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(2)} km`;
  }
  return `${Math.round(meters)} m`;
}

function formatElevation(meters: number | null): string {
  if (meters === null) return "N/A";
  return `${Math.round(meters)} m`;
}

function formatTime(seconds: number): string {
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hrs > 0) {
    return `${hrs}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export default function SegmentDetailPage() {
  const params = useParams();
  const router = useRouter();
  const [segment, setSegment] = useState<Segment | null>(null);
  const [trackData, setTrackData] = useState<SegmentTrackData | null>(null);
  const [efforts, setEfforts] = useState<SegmentEffort[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [highlightIndex, setHighlightIndex] = useState<number | null>(null);

  const segmentId = params.id as string;

  useEffect(() => {
    if (segmentId) {
      Promise.all([
        api.getSegment(segmentId),
        api.getSegmentTrack(segmentId),
        api.getSegmentLeaderboard(segmentId),
      ])
        .then(([seg, track, eff]) => {
          setSegment(seg);
          setTrackData(track);
          setEfforts(eff);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [segmentId]);

  // Convert segment track data to the format expected by ActivityMap/ElevationProfile
  const convertedTrackData: TrackData | null = useMemo(() => {
    if (!trackData) return null;
    return {
      points: trackData.points.map((p) => ({
        lat: p.lat,
        lon: p.lon,
        ele: p.ele,
        time: null,
      })),
      bounds: trackData.bounds,
    };
  }, [trackData]);

  if (loading) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-10 w-64" />
        <Skeleton className="h-48 w-full" />
        <Skeleton className="h-64 w-full" />
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

  if (!segment) {
    return (
      <div className="p-4 text-muted-foreground">Segment not found</div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-2xl md:text-3xl font-bold">{segment.name}</h1>
          <div className="flex flex-wrap items-center gap-2 md:gap-4 mt-2">
            <Badge variant="secondary">
              {ACTIVITY_TYPE_LABELS[segment.activity_type] || segment.activity_type}
            </Badge>
            <span className="text-sm md:text-base text-muted-foreground">
              {new Date(segment.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
        <Button variant="outline" onClick={() => router.push("/segments")}>
          Back to Segments
        </Button>
      </div>

      {segment.description && (
        <Card>
          <CardContent className="py-4">
            <p className="text-muted-foreground">{segment.description}</p>
          </CardContent>
        </Card>
      )}

      {convertedTrackData && (
        <Card>
          <CardHeader>
            <CardTitle>Route</CardTitle>
          </CardHeader>
          <CardContent>
            <ActivityMap
              trackData={convertedTrackData}
              highlightIndex={highlightIndex ?? undefined}
            />
          </CardContent>
        </Card>
      )}

      {convertedTrackData && convertedTrackData.points.some((p) => p.ele !== null) && (
        <Card>
          <CardHeader>
            <CardTitle>Elevation Profile</CardTitle>
          </CardHeader>
          <CardContent>
            <ElevationProfile
              points={convertedTrackData.points}
              onHover={setHighlightIndex}
            />
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Statistics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatItem
              label="Distance"
              value={formatDistance(segment.distance_meters)}
            />
            <StatItem
              label="Elevation Gain"
              value={formatElevation(segment.elevation_gain_meters)}
            />
            <StatItem
              label="Elevation Loss"
              value={formatElevation(segment.elevation_loss_meters)}
            />
            <StatItem
              label="Attempts"
              value={efforts.length.toString()}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Leaderboard</CardTitle>
        </CardHeader>
        <CardContent>
          {efforts.length === 0 ? (
            <p className="text-muted-foreground text-center py-8">
              No efforts recorded yet. Be the first!
            </p>
          ) : (
            <div className="space-y-2">
              <div className="grid grid-cols-4 text-sm font-medium text-muted-foreground pb-2 border-b">
                <span>Rank</span>
                <span>Athlete</span>
                <span>Time</span>
                <span>Date</span>
              </div>
              {efforts.map((effort, index) => (
                <div
                  key={effort.id}
                  className="grid grid-cols-4 py-2 text-sm border-b last:border-b-0"
                >
                  <span className="font-medium">
                    {index === 0 && "🥇 "}
                    {index === 1 && "🥈 "}
                    {index === 2 && "🥉 "}
                    {index + 1}
                  </span>
                  <span className="truncate">
                    {effort.user_id.slice(0, 8)}...
                    {effort.is_personal_record && (
                      <Badge variant="outline" className="ml-2 text-xs">
                        PR
                      </Badge>
                    )}
                  </span>
                  <span className="font-mono">
                    {formatTime(effort.elapsed_time_seconds)}
                  </span>
                  <span className="text-muted-foreground">
                    {new Date(effort.started_at).toLocaleDateString()}
                  </span>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="text-center p-4 bg-muted/50 rounded-lg">
      <p className="text-2xl font-bold">{value}</p>
      <p className="text-sm text-muted-foreground">{label}</p>
    </div>
  );
}
