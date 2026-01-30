"use client";

import { useEffect, useState } from "react";
import { api, CrownCountEntry, DistanceLeaderEntry } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { RankBadge, CrownBadge } from "@/components/leaderboard/crown-badge";
import { Crown, MapPin } from "lucide-react";

type LeaderboardTab = "crowns" | "distance";

const PAGE_SIZE = 20;

function formatDistance(meters: number): string {
  const km = meters / 1000;
  if (km >= 1000) {
    return `${(km / 1000).toFixed(1)}k km`;
  }
  return `${km.toFixed(1)} km`;
}

export default function LeaderboardsPage() {
  const { user } = useAuth();
  const [activeTab, setActiveTab] = useState<LeaderboardTab>("crowns");
  const [crownEntries, setCrownEntries] = useState<CrownCountEntry[]>([]);
  const [distanceEntries, setDistanceEntries] = useState<DistanceLeaderEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState("");
  const [hasMoreCrowns, setHasMoreCrowns] = useState(true);
  const [hasMoreDistance, setHasMoreDistance] = useState(true);

  // Load initial data when tab changes
  useEffect(() => {
    setLoading(true);
    setError("");

    if (activeTab === "crowns") {
      api
        .getCrownLeaderboard(PAGE_SIZE, 0)
        .then((entries) => {
          setCrownEntries(entries);
          setHasMoreCrowns(entries.length === PAGE_SIZE);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    } else {
      api
        .getDistanceLeaderboard(PAGE_SIZE, 0)
        .then((entries) => {
          setDistanceEntries(entries);
          setHasMoreDistance(entries.length === PAGE_SIZE);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [activeTab]);

  const loadMoreCrowns = async () => {
    setLoadingMore(true);
    try {
      const newEntries = await api.getCrownLeaderboard(PAGE_SIZE, crownEntries.length);
      setCrownEntries((prev) => [...prev, ...newEntries]);
      setHasMoreCrowns(newEntries.length === PAGE_SIZE);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load more");
    } finally {
      setLoadingMore(false);
    }
  };

  const loadMoreDistance = async () => {
    setLoadingMore(true);
    try {
      const newEntries = await api.getDistanceLeaderboard(PAGE_SIZE, distanceEntries.length);
      setDistanceEntries((prev) => [...prev, ...newEntries]);
      setHasMoreDistance(newEntries.length === PAGE_SIZE);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load more");
    } finally {
      setLoadingMore(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Leaderboards</h1>
      </div>

      {/* Tab buttons */}
      <div className="flex gap-2">
        <Button
          variant={activeTab === "crowns" ? "default" : "outline"}
          onClick={() => setActiveTab("crowns")}
          className="gap-2"
        >
          <Crown className="h-4 w-4" />
          Crowns
        </Button>
        <Button
          variant={activeTab === "distance" ? "default" : "outline"}
          onClick={() => setActiveTab("distance")}
          className="gap-2"
        >
          <MapPin className="h-4 w-4" />
          Distance
        </Button>
      </div>

      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {loading ? (
        <LeaderboardSkeleton />
      ) : activeTab === "crowns" ? (
        <CrownLeaderboard
          entries={crownEntries}
          currentUserId={user?.id}
          hasMore={hasMoreCrowns}
          loadingMore={loadingMore}
          onLoadMore={loadMoreCrowns}
        />
      ) : (
        <DistanceLeaderboard
          entries={distanceEntries}
          currentUserId={user?.id}
          hasMore={hasMoreDistance}
          loadingMore={loadingMore}
          onLoadMore={loadMoreDistance}
        />
      )}
    </div>
  );
}

function LeaderboardSkeleton() {
  return (
    <Card>
      <CardContent className="p-0">
        <div className="divide-y">
          {Array.from({ length: 10 }).map((_, i) => (
            <div key={i} className="flex items-center gap-4 p-4">
              <Skeleton className="h-8 w-8 rounded-full" />
              <Skeleton className="h-5 w-32" />
              <Skeleton className="h-5 w-20 ml-auto" />
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

interface CrownLeaderboardProps {
  entries: CrownCountEntry[];
  currentUserId: string | undefined;
  hasMore: boolean;
  loadingMore: boolean;
  onLoadMore: () => void;
}

function CrownLeaderboard({
  entries,
  currentUserId,
  hasMore,
  loadingMore,
  onLoadMore,
}: CrownLeaderboardProps) {
  if (entries.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <Crown className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">No crown holders yet</p>
          <p className="text-sm text-muted-foreground mt-2">
            Be the first to claim a KOM or QOM!
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-lg">Crown Leaderboard</CardTitle>
        <p className="text-sm text-muted-foreground">
          Athletes ranked by total crowns (KOMs and QOMs)
        </p>
      </CardHeader>
      <CardContent className="p-0">
        {/* Header row */}
        <div className="hidden sm:grid sm:grid-cols-[3rem_1fr_5rem_5rem_5rem] gap-4 px-4 py-2 bg-muted/50 text-xs font-medium text-muted-foreground uppercase tracking-wide border-b">
          <div>Rank</div>
          <div>Athlete</div>
          <div className="text-center">KOM</div>
          <div className="text-center">QOM</div>
          <div className="text-center">Total</div>
        </div>

        <div className="divide-y">
          {entries.map((entry) => {
            const isCurrentUser = currentUserId === entry.user_id;
            return (
              <div
                key={entry.user_id}
                className={`grid grid-cols-[3rem_1fr] sm:grid-cols-[3rem_1fr_5rem_5rem_5rem] gap-4 px-4 py-3 items-center ${
                  isCurrentUser ? "bg-primary/5 border-l-2 border-l-primary" : ""
                }`}
              >
                {/* Rank */}
                <div className="flex justify-center">
                  <RankBadge rank={entry.rank} size="sm" />
                </div>

                {/* User name */}
                <div className="font-medium truncate">
                  {entry.user_name}
                  {isCurrentUser && (
                    <span className="text-xs text-muted-foreground ml-2">(you)</span>
                  )}
                </div>

                {/* KOM count */}
                <div className="hidden sm:flex items-center justify-center gap-1">
                  <Crown className="h-4 w-4 text-amber-500" />
                  <span className="font-medium">{entry.kom_count}</span>
                </div>

                {/* QOM count */}
                <div className="hidden sm:flex items-center justify-center gap-1">
                  <Crown className="h-4 w-4 text-amber-500" />
                  <span className="font-medium">{entry.qom_count}</span>
                </div>

                {/* Total */}
                <div className="hidden sm:flex items-center justify-center">
                  <span className="font-bold text-lg">{entry.total_crowns}</span>
                </div>

                {/* Mobile-only stats row */}
                <div className="sm:hidden col-span-2 flex items-center gap-4 text-sm text-muted-foreground">
                  <span className="flex items-center gap-1">
                    <Crown className="h-3 w-3 text-amber-500" />
                    {entry.kom_count} KOM
                  </span>
                  <span className="flex items-center gap-1">
                    <Crown className="h-3 w-3 text-amber-500" />
                    {entry.qom_count} QOM
                  </span>
                  <span className="ml-auto font-bold">{entry.total_crowns} total</span>
                </div>
              </div>
            );
          })}
        </div>

        {hasMore && (
          <div className="p-4 border-t">
            <Button
              variant="outline"
              className="w-full"
              onClick={onLoadMore}
              disabled={loadingMore}
            >
              {loadingMore ? "Loading..." : "Load more"}
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

interface DistanceLeaderboardProps {
  entries: DistanceLeaderEntry[];
  currentUserId: string | undefined;
  hasMore: boolean;
  loadingMore: boolean;
  onLoadMore: () => void;
}

function DistanceLeaderboard({
  entries,
  currentUserId,
  hasMore,
  loadingMore,
  onLoadMore,
}: DistanceLeaderboardProps) {
  if (entries.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <MapPin className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">No distance data yet</p>
          <p className="text-sm text-muted-foreground mt-2">
            Upload activities to start climbing the leaderboard!
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-lg">Distance Leaderboard</CardTitle>
        <p className="text-sm text-muted-foreground">
          Athletes ranked by total distance covered
        </p>
      </CardHeader>
      <CardContent className="p-0">
        {/* Header row */}
        <div className="hidden sm:grid sm:grid-cols-[3rem_1fr_7rem_6rem] gap-4 px-4 py-2 bg-muted/50 text-xs font-medium text-muted-foreground uppercase tracking-wide border-b">
          <div>Rank</div>
          <div>Athlete</div>
          <div className="text-right">Distance</div>
          <div className="text-right">Activities</div>
        </div>

        <div className="divide-y">
          {entries.map((entry) => {
            const isCurrentUser = currentUserId === entry.user_id;
            return (
              <div
                key={entry.user_id}
                className={`grid grid-cols-[3rem_1fr] sm:grid-cols-[3rem_1fr_7rem_6rem] gap-4 px-4 py-3 items-center ${
                  isCurrentUser ? "bg-primary/5 border-l-2 border-l-primary" : ""
                }`}
              >
                {/* Rank */}
                <div className="flex justify-center">
                  <RankBadge rank={entry.rank} size="sm" />
                </div>

                {/* User name */}
                <div className="font-medium truncate">
                  {entry.user_name}
                  {isCurrentUser && (
                    <span className="text-xs text-muted-foreground ml-2">(you)</span>
                  )}
                </div>

                {/* Distance */}
                <div className="hidden sm:block text-right font-medium">
                  {formatDistance(entry.total_distance_meters)}
                </div>

                {/* Activity count */}
                <div className="hidden sm:block text-right text-muted-foreground">
                  {entry.activity_count} {entry.activity_count === 1 ? "activity" : "activities"}
                </div>

                {/* Mobile-only stats row */}
                <div className="sm:hidden col-span-2 flex items-center justify-between text-sm">
                  <span className="font-medium">
                    {formatDistance(entry.total_distance_meters)}
                  </span>
                  <span className="text-muted-foreground">
                    {entry.activity_count} {entry.activity_count === 1 ? "activity" : "activities"}
                  </span>
                </div>
              </div>
            );
          })}
        </div>

        {hasMore && (
          <div className="p-4 border-t">
            <Button
              variant="outline"
              className="w-full"
              onClick={onLoadMore}
              disabled={loadingMore}
            >
              {loadingMore ? "Loading..." : "Load more"}
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
