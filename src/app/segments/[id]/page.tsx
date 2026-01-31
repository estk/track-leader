"use client";

import { useEffect, useMemo, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { api, Segment, SegmentEffort, SegmentTrackData, TrackData, getActivityTypeName } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { LazyActivityMap } from "@/components/activity/lazy-activity-map";
import { LazyElevationProfile } from "@/components/activity/lazy-elevation-profile";
import dynamic from "next/dynamic";
import { getClimbCategoryInfo } from "@/lib/utils";

const LazyPRHistoryChart = dynamic(
  () => import("./pr-history-chart").then((mod) => mod.PRHistoryChart),
  {
    loading: () => <div className="h-64 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

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

function formatGrade(grade: number | null): string {
  if (grade === null) return "N/A";
  return `${grade.toFixed(1)}%`;
}

function formatClimbCategory(category: number | null): string {
  const info = getClimbCategoryInfo(category);
  return info?.label ?? "N/A";
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
  const [starred, setStarred] = useState(false);
  const [starLoading, setStarLoading] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [currentUserId, setCurrentUserId] = useState<string | null>(null);
  const [myEfforts, setMyEfforts] = useState<SegmentEffort[]>([]);

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

      // Check starred status, get user info, and fetch user's efforts if logged in
      if (api.getToken()) {
        setIsLoggedIn(true);
        api.isSegmentStarred(segmentId)
          .then(setStarred)
          .catch(() => setStarred(false));
        api.getMySegmentEfforts(segmentId)
          .then(setMyEfforts)
          .catch(() => setMyEfforts([]));
        api.me()
          .then((user) => setCurrentUserId(user.id))
          .catch(() => setCurrentUserId(null));
      }
    }
  }, [segmentId]);

  const handleStarToggle = async () => {
    if (!isLoggedIn || starLoading) return;
    setStarLoading(true);
    try {
      if (starred) {
        await api.unstarSegment(segmentId);
        setStarred(false);
      } else {
        await api.starSegment(segmentId);
        setStarred(true);
      }
    } catch (err) {
      console.error("Failed to toggle star:", err);
    } finally {
      setStarLoading(false);
    }
  };

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
              {getActivityTypeName(segment.activity_type_id)}
            </Badge>
            <span className="text-sm md:text-base text-muted-foreground">
              Created by{" "}
              {currentUserId === segment.creator_id ? (
                "you"
              ) : (
                <Link
                  href={`/profile/${segment.creator_id}`}
                  className="hover:underline"
                >
                  {segment.creator_name || `${segment.creator_id.slice(0, 8)}...`}
                </Link>
              )}
            </span>
            <span className="text-sm md:text-base text-muted-foreground">
              {new Date(segment.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
        <div className="flex gap-2">
          {isLoggedIn && (
            <Button
              variant={starred ? "default" : "outline"}
              onClick={handleStarToggle}
              disabled={starLoading}
            >
              {starred ? "★ Starred" : "☆ Star"}
            </Button>
          )}
          <Button variant="outline" onClick={() => router.push("/segments")}>
            Back to Segments
          </Button>
        </div>
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
            <LazyActivityMap
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
            <LazyElevationProfile
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
              label="Average Grade"
              value={formatGrade(segment.average_grade)}
            />
            <StatItem
              label="Max Grade"
              value={formatGrade(segment.max_grade)}
            />
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
            <StatItem
              label="Elevation Loss"
              value={formatElevation(segment.elevation_loss_meters)}
            />
            <StatItem
              label="Climb Category"
              value={formatClimbCategory(segment.climb_category)}
              tooltip={getClimbCategoryInfo(segment.climb_category)?.tooltip}
            />
            <StatItem
              label="Attempts"
              value={efforts.length.toString()}
            />
          </div>
        </CardContent>
      </Card>

      {isLoggedIn && myEfforts.length > 0 && efforts.length > 0 && (
        <EffortComparisonCard
          myEfforts={myEfforts}
          efforts={efforts}
          currentUserId={currentUserId}
        />
      )}

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Leaderboard</CardTitle>
          <Button
            variant="link"
            className="text-sm"
            onClick={() => router.push(`/segments/${segmentId}/leaderboard`)}
          >
            View Full Leaderboard →
          </Button>
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
              {efforts.map((effort, index) => {
                const isCurrentUser = currentUserId && effort.user_id === currentUserId;
                return (
                  <div
                    key={effort.id}
                    className={`grid grid-cols-4 py-2 text-sm border-b last:border-b-0 cursor-pointer hover:bg-muted/50 transition-colors ${
                      isCurrentUser ? "bg-primary/10 font-medium" : ""
                    }`}
                    onClick={() => router.push(`/activities/${effort.activity_id}`)}
                  >
                    <span className="font-medium">
                      {index === 0 && "🥇 "}
                      {index === 1 && "🥈 "}
                      {index === 2 && "🥉 "}
                      {index + 1}
                    </span>
                    <span className="truncate">
                      {isCurrentUser ? (
                        "You"
                      ) : (
                        <Link
                          href={`/profile/${effort.user_id}`}
                          className="hover:underline"
                          onClick={(e) => e.stopPropagation()}
                        >
                          {effort.user_name || `${effort.user_id.slice(0, 8)}...`}
                        </Link>
                      )}
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
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>

      {isLoggedIn && myEfforts.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Your Efforts ({myEfforts.length})</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="grid grid-cols-4 text-sm font-medium text-muted-foreground pb-2 border-b">
                <span>#</span>
                <span>Time</span>
                <span>Date</span>
                <span></span>
              </div>
              {myEfforts.map((effort, index) => (
                <div
                  key={effort.id}
                  className={`grid grid-cols-4 py-2 text-sm border-b last:border-b-0 cursor-pointer hover:bg-muted/50 transition-colors ${
                    effort.is_personal_record ? "bg-primary/10 font-medium" : ""
                  }`}
                  onClick={() => router.push(`/activities/${effort.activity_id}`)}
                >
                  <span>{index + 1}</span>
                  <span className="font-mono">
                    {formatTime(effort.elapsed_time_seconds)}
                  </span>
                  <span className="text-muted-foreground">
                    {new Date(effort.started_at).toLocaleDateString()}
                  </span>
                  <span>
                    {effort.is_personal_record && (
                      <Badge variant="default" className="text-xs">
                        PR
                      </Badge>
                    )}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {isLoggedIn && myEfforts.length >= 2 && (
        <Card>
          <CardHeader>
            <CardTitle>PR History</CardTitle>
          </CardHeader>
          <CardContent>
            <LazyPRHistoryChart efforts={myEfforts} />
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function StatItem({ label, value, tooltip }: { label: string; value: string; tooltip?: string }) {
  return (
    <div className="text-center p-4 bg-muted/50 rounded-lg" title={tooltip}>
      <p className="text-2xl font-bold">{value}</p>
      <p className="text-sm text-muted-foreground">{label}</p>
    </div>
  );
}

interface EffortComparisonCardProps {
  myEfforts: SegmentEffort[];
  efforts: SegmentEffort[];
  currentUserId: string | null;
}

function EffortComparisonCard({ myEfforts, efforts, currentUserId }: EffortComparisonCardProps) {
  // Find user's personal best (fastest time)
  const userBest = useMemo(() => {
    return [...myEfforts].sort((a, b) => a.elapsed_time_seconds - b.elapsed_time_seconds)[0];
  }, [myEfforts]);

  // Segment record is the first effort in the leaderboard
  const segmentRecord = efforts[0];

  // Find user's rank in the overall leaderboard
  const userRank = useMemo(() => {
    const index = efforts.findIndex((e) => e.id === userBest.id);
    return index >= 0 ? index + 1 : null;
  }, [efforts, userBest]);

  const userHoldsRecord = currentUserId && segmentRecord.user_id === currentUserId;

  const gapSeconds = userBest.elapsed_time_seconds - segmentRecord.elapsed_time_seconds;
  const gapPercent = ((userBest.elapsed_time_seconds - segmentRecord.elapsed_time_seconds) / segmentRecord.elapsed_time_seconds) * 100;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Your Best vs Record</CardTitle>
      </CardHeader>
      <CardContent>
        {userHoldsRecord ? (
          <div className="text-center py-4">
            <p className="text-2xl font-bold text-primary">You hold the segment record!</p>
            <p className="text-muted-foreground mt-2">
              Your time: {formatTime(userBest.elapsed_time_seconds)}
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex justify-between items-center py-2 border-b">
              <span className="text-muted-foreground">Your Best</span>
              <span className="font-mono font-medium">
                {formatTime(userBest.elapsed_time_seconds)}
                {userRank && (
                  <span className="text-muted-foreground ml-2">(rank #{userRank})</span>
                )}
              </span>
            </div>
            <div className="flex justify-between items-center py-2 border-b">
              <span className="text-muted-foreground">Segment Record</span>
              <span className="font-mono font-medium">
                {formatTime(segmentRecord.elapsed_time_seconds)}
                <span className="text-muted-foreground ml-2">
                  by{" "}
                  <Link
                    href={`/profile/${segmentRecord.user_id}`}
                    className="hover:underline"
                  >
                    {segmentRecord.user_name || `${segmentRecord.user_id.slice(0, 8)}...`}
                  </Link>
                </span>
              </span>
            </div>
            <div className="flex justify-between items-center py-2">
              <span className="text-muted-foreground">Gap</span>
              <span className="font-mono font-medium text-destructive">
                +{gapSeconds}s (+{gapPercent.toFixed(1)}%)
              </span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
