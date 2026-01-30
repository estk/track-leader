"use client";

import { useEffect, useState, useMemo } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/lib/auth-context";
import {
  api,
  StarredSegmentEffort,
  AchievementWithSegment,
} from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { CrownBadge } from "@/components/leaderboard/crown-badge";
import { cn } from "@/lib/utils";

type SortOption = "rank" | "gap" | "name";

function formatTime(seconds: number | null): string {
  if (seconds === null) return "-";
  const mins = Math.floor(seconds / 60);
  const secs = Math.round(seconds % 60);
  if (mins >= 60) {
    const hours = Math.floor(mins / 60);
    const remainingMins = mins % 60;
    return `${hours}:${remainingMins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(2)} km`;
  }
  return `${Math.round(meters)} m`;
}

function formatGap(userTime: number | null, leaderTime: number | null): string {
  if (userTime === null || leaderTime === null) return "-";
  if (userTime === leaderTime) return "Leader";
  const gap = userTime - leaderTime;
  return `+${formatTime(gap)}`;
}

function getGapSeconds(effort: StarredSegmentEffort): number {
  if (effort.best_time_seconds === null || effort.leader_time_seconds === null) {
    return Infinity;
  }
  return effort.best_time_seconds - effort.leader_time_seconds;
}

interface RankingsData {
  efforts: StarredSegmentEffort[];
  achievements: AchievementWithSegment[];
}

export default function RankingsPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [data, setData] = useState<RankingsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [sortBy, setSortBy] = useState<SortOption>("rank");

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user) {
      Promise.all([
        api.getStarredSegmentEfforts(),
        api.getMyAchievements(),
      ])
        .then(([efforts, achievements]) => {
          setData({ efforts, achievements });
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [user, authLoading, router]);

  // Build a map of segment_id -> achievement types for quick lookup
  const achievementsBySegment = useMemo(() => {
    if (!data) return new Map<string, Set<string>>();
    const map = new Map<string, Set<string>>();
    for (const a of data.achievements) {
      if (a.lost_at === null) {
        // Only active achievements
        if (!map.has(a.segment_id)) {
          map.set(a.segment_id, new Set());
        }
        map.get(a.segment_id)!.add(a.achievement_type);
      }
    }
    return map;
  }, [data]);

  // Filter to only efforts where user has attempts, then sort
  const sortedEfforts = useMemo(() => {
    if (!data) return [];
    const withEfforts = data.efforts.filter(
      (e) => e.user_effort_count > 0 && e.best_time_seconds !== null
    );

    return [...withEfforts].sort((a, b) => {
      switch (sortBy) {
        case "rank":
          // Sort by rank ascending (best first), null ranks go to end
          const rankA = a.best_effort_rank ?? Infinity;
          const rankB = b.best_effort_rank ?? Infinity;
          return rankA - rankB;
        case "gap":
          // Sort by gap descending (biggest improvement potential first)
          return getGapSeconds(b) - getGapSeconds(a);
        case "name":
          return a.segment_name.localeCompare(b.segment_name);
        default:
          return 0;
      }
    });
  }, [data, sortBy]);

  // Calculate summary stats
  const summaryStats = useMemo(() => {
    if (!sortedEfforts.length) return null;
    const top10Count = sortedEfforts.filter(
      (e) => e.best_effort_rank !== null && e.best_effort_rank <= 10
    ).length;
    const top3Count = sortedEfforts.filter(
      (e) => e.best_effort_rank !== null && e.best_effort_rank <= 3
    ).length;
    const leaderCount = sortedEfforts.filter(
      (e) => e.best_effort_rank === 1
    ).length;
    return { top10Count, top3Count, leaderCount, totalSegments: sortedEfforts.length };
  }, [sortedEfforts]);

  if (authLoading || loading) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (!user) {
    return null;
  }

  if (error) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold">My Rankings</h1>
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-destructive">{error}</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!data || sortedEfforts.length === 0) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold">My Rankings</h1>
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground mb-4">
              No segment efforts found
            </p>
            <p className="text-sm text-muted-foreground mb-4">
              Star some segments and complete activities that pass through them
              to see your rankings here.
            </p>
            <Button onClick={() => router.push("/segments")}>
              Browse Segments
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">My Rankings</h1>
      </div>

      {/* Summary Stats */}
      {summaryStats && (
        <Card>
          <CardContent className="py-4">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 text-center">
              <div className="p-3 bg-muted/50 rounded-lg">
                <p className="text-2xl font-bold">{summaryStats.totalSegments}</p>
                <p className="text-xs text-muted-foreground">Segments with Efforts</p>
              </div>
              <div className="p-3 bg-amber-50 dark:bg-amber-900/20 rounded-lg">
                <p className="text-2xl font-bold text-amber-700 dark:text-amber-400">
                  {summaryStats.leaderCount}
                </p>
                <p className="text-xs text-muted-foreground">Leader</p>
              </div>
              <div className="p-3 bg-muted/50 rounded-lg">
                <p className="text-2xl font-bold">{summaryStats.top3Count}</p>
                <p className="text-xs text-muted-foreground">Top 3</p>
              </div>
              <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <p className="text-2xl font-bold text-green-700 dark:text-green-400">
                  {summaryStats.top10Count}
                </p>
                <p className="text-xs text-muted-foreground">Top 10</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Sort Controls */}
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">Sort by:</span>
        <Button
          variant={sortBy === "rank" ? "default" : "outline"}
          size="sm"
          onClick={() => setSortBy("rank")}
        >
          Rank
        </Button>
        <Button
          variant={sortBy === "gap" ? "default" : "outline"}
          size="sm"
          onClick={() => setSortBy("gap")}
        >
          Improvement Potential
        </Button>
        <Button
          variant={sortBy === "name" ? "default" : "outline"}
          size="sm"
          onClick={() => setSortBy("name")}
        >
          Name
        </Button>
      </div>

      {/* Rankings Table */}
      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                    Segment
                  </th>
                  <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                    Your Best
                  </th>
                  <th className="px-4 py-3 text-center text-sm font-medium text-muted-foreground">
                    Rank
                  </th>
                  <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                    Leader
                  </th>
                  <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                    Gap
                  </th>
                </tr>
              </thead>
              <tbody>
                {sortedEfforts.map((effort) => {
                  const segmentAchievements = achievementsBySegment.get(effort.segment_id);
                  const hasKOM = segmentAchievements?.has("kom");
                  const hasQOM = segmentAchievements?.has("qom");
                  const isLeader = effort.best_effort_rank === 1;
                  const isTop3 = effort.best_effort_rank !== null && effort.best_effort_rank <= 3;
                  const isTop10 = effort.best_effort_rank !== null && effort.best_effort_rank <= 10;

                  return (
                    <tr
                      key={effort.segment_id}
                      className={cn(
                        "border-b transition-colors hover:bg-muted/50",
                        isLeader && "bg-amber-50/50 dark:bg-amber-900/10"
                      )}
                    >
                      <td className="px-4 py-3">
                        <div className="flex flex-col gap-1">
                          <Link
                            href={`/segments/${effort.segment_id}`}
                            className="font-medium hover:underline text-primary"
                          >
                            {effort.segment_name}
                          </Link>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <span>{formatDistance(effort.distance_meters)}</span>
                            {effort.elevation_gain_meters !== null && (
                              <span>{Math.round(effort.elevation_gain_meters)}m gain</span>
                            )}
                          </div>
                          {/* Achievement badges */}
                          <div className="flex items-center gap-1 flex-wrap">
                            {hasKOM && <CrownBadge type="kom" size="sm" />}
                            {hasQOM && <CrownBadge type="qom" size="sm" />}
                          </div>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-right font-mono">
                        {formatTime(effort.best_time_seconds)}
                      </td>
                      <td className="px-4 py-3 text-center">
                        {effort.best_effort_rank !== null ? (
                          <span
                            className={cn(
                              "inline-flex items-center justify-center min-w-[2rem] px-2 py-0.5 rounded-full text-sm font-medium",
                              isLeader && "bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-300",
                              !isLeader && isTop3 && "bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-300",
                              !isTop3 && isTop10 && "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-300",
                              !isTop10 && "bg-muted text-muted-foreground"
                            )}
                          >
                            #{effort.best_effort_rank}
                          </span>
                        ) : (
                          <span className="text-muted-foreground">-</span>
                        )}
                      </td>
                      <td className="px-4 py-3 text-right font-mono text-muted-foreground">
                        {formatTime(effort.leader_time_seconds)}
                      </td>
                      <td className="px-4 py-3 text-right">
                        {isLeader ? (
                          <Badge
                            variant="secondary"
                            className="bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-300"
                          >
                            Leader
                          </Badge>
                        ) : (
                          <span className="font-mono text-muted-foreground">
                            {formatGap(effort.best_time_seconds, effort.leader_time_seconds)}
                          </span>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
