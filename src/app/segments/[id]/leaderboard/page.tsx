"use client";

import { useEffect, useState, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import {
  api,
  Segment,
  LeaderboardEntry,
  LeaderboardPosition,
  SegmentAchievements,
} from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { LeaderboardTable } from "@/components/leaderboard/leaderboard-table";
import {
  LeaderboardFiltersComponent,
  useLeaderboardFilters,
} from "@/components/leaderboard/leaderboard-filters";
import { CrownBadge } from "@/components/leaderboard/crown-badge";
import { ArrowLeft } from "lucide-react";

const PAGE_SIZE = 20;

function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${minutes}:${secs.toString().padStart(2, "0")}`;
}

export default function SegmentLeaderboardPage() {
  const params = useParams();
  const router = useRouter();
  const segmentId = params.id as string;

  const { filters, setFilters } = useLeaderboardFilters();

  const [segment, setSegment] = useState<Segment | null>(null);
  const [entries, setEntries] = useState<LeaderboardEntry[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [achievements, setAchievements] = useState<SegmentAchievements | null>(null);
  const [position, setPosition] = useState<LeaderboardPosition | null>(null);
  const [currentUserId, setCurrentUserId] = useState<string | null>(null);
  const [isLoggedIn, setIsLoggedIn] = useState(false);

  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState("");

  // Initial load
  useEffect(() => {
    if (!segmentId) return;

    const fetchInitialData = async () => {
      setLoading(true);
      setError("");

      try {
        // Fetch segment info and achievements in parallel
        const [seg, ach] = await Promise.all([
          api.getSegment(segmentId),
          api.getSegmentAchievements(segmentId),
        ]);
        setSegment(seg);
        setAchievements(ach);

        // Check if logged in and fetch user data
        if (api.getToken()) {
          setIsLoggedIn(true);
          try {
            const user = await api.me();
            setCurrentUserId(user.id);
          } catch {
            setCurrentUserId(null);
          }
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load segment");
      } finally {
        setLoading(false);
      }
    };

    fetchInitialData();
  }, [segmentId]);

  // Fetch leaderboard when filters change
  useEffect(() => {
    if (!segmentId) return;

    const fetchLeaderboard = async () => {
      setLoading(true);
      setError("");

      try {
        const response = await api.getFilteredLeaderboard(segmentId, {
          ...filters,
          limit: PAGE_SIZE,
          offset: 0,
        });
        setEntries(response.entries);
        setTotalCount(response.total_count);

        // Fetch user's position if logged in
        if (isLoggedIn) {
          try {
            const pos = await api.getLeaderboardPosition(segmentId, {
              scope: filters.scope,
              gender: filters.gender,
              age_group: filters.age_group,
              weight_class: filters.weight_class,
              country: filters.country,
            });
            setPosition(pos);
          } catch {
            setPosition(null);
          }
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load leaderboard");
      } finally {
        setLoading(false);
      }
    };

    fetchLeaderboard();
  }, [segmentId, filters, isLoggedIn]);

  const handleLoadMore = useCallback(async () => {
    if (loadingMore || entries.length >= totalCount) return;

    setLoadingMore(true);
    try {
      const response = await api.getFilteredLeaderboard(segmentId, {
        ...filters,
        limit: PAGE_SIZE,
        offset: entries.length,
      });
      setEntries((prev) => [...prev, ...response.entries]);
    } catch (err) {
      console.error("Failed to load more entries:", err);
    } finally {
      setLoadingMore(false);
    }
  }, [segmentId, filters, entries.length, totalCount, loadingMore]);

  if (loading && !segment) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-10 w-64" />
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (error && !segment) {
    return (
      <div className="space-y-4">
        <Link
          href={`/segments/${segmentId}`}
          className="inline-flex items-center text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to segment
        </Link>
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      </div>
    );
  }

  if (!segment) {
    return (
      <div className="space-y-4">
        <Link
          href="/segments"
          className="inline-flex items-center text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to segments
        </Link>
        <div className="p-4 text-muted-foreground">Segment not found</div>
      </div>
    );
  }

  const hasMore = entries.length < totalCount;

  return (
    <div className="space-y-6">
      {/* Back link and header */}
      <div className="space-y-4">
        <Link
          href={`/segments/${segmentId}`}
          className="inline-flex items-center text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to segment
        </Link>
        <div>
          <h1 className="text-2xl md:text-3xl font-bold">{segment.name}</h1>
          <p className="text-muted-foreground mt-1">Full Leaderboard</p>
        </div>
      </div>

      {/* Achievements section */}
      {achievements && (achievements.kom || achievements.qom) && (
        <Card>
          <CardHeader>
            <CardTitle>Segment Records</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {achievements.kom && (
                <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
                  <CrownBadge type="kom" size="lg" />
                  <div className="min-w-0">
                    <p className="font-medium truncate">{achievements.kom.user_name}</p>
                    {achievements.kom.elapsed_time_seconds && (
                      <p className="text-sm text-muted-foreground font-mono">
                        {formatTime(achievements.kom.elapsed_time_seconds)}
                      </p>
                    )}
                  </div>
                </div>
              )}
              {achievements.qom && (
                <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
                  <CrownBadge type="qom" size="lg" />
                  <div className="min-w-0">
                    <p className="font-medium truncate">{achievements.qom.user_name}</p>
                    {achievements.qom.elapsed_time_seconds && (
                      <p className="text-sm text-muted-foreground font-mono">
                        {formatTime(achievements.qom.elapsed_time_seconds)}
                      </p>
                    )}
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* User position summary */}
      {isLoggedIn && position && position.user_entry && (
        <Card>
          <CardHeader>
            <CardTitle>Your Position</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
              <div className="flex items-center gap-4">
                <div className="text-center">
                  <p className="text-3xl font-bold text-primary">
                    #{position.user_rank}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    of {position.total_count}
                  </p>
                </div>
                <div>
                  <p className="font-medium">Your best time</p>
                  <p className="text-lg font-mono">
                    {formatTime(position.user_entry.elapsed_time_seconds)}
                  </p>
                </div>
              </div>
              {position.user_entry.gap_seconds !== null && position.user_entry.gap_seconds > 0 && (
                <div className="text-sm text-muted-foreground">
                  <span className="font-mono">+{formatTime(position.user_entry.gap_seconds)}</span>
                  {" "}behind leader
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Filters</CardTitle>
        </CardHeader>
        <CardContent>
          <LeaderboardFiltersComponent
            filters={filters}
            onChange={setFilters}
          />
        </CardContent>
      </Card>

      {/* Error state for leaderboard */}
      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {/* Leaderboard table */}
      <LeaderboardTable
        entries={entries}
        currentUserId={currentUserId}
        loading={loading || loadingMore}
        onLoadMore={hasMore ? handleLoadMore : undefined}
      />

      {/* Total count summary */}
      {!loading && entries.length > 0 && (
        <p className="text-center text-sm text-muted-foreground">
          Showing {entries.length} of {totalCount} entries
        </p>
      )}
    </div>
  );
}
