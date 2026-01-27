"use client";

import { cn } from "@/lib/utils";
import { LeaderboardEntry } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

interface LeaderboardTableProps {
  entries: LeaderboardEntry[];
  currentUserId: string | null;
  loading: boolean;
  onLoadMore?: () => void;
}

/**
 * Format seconds to "MM:SS" or "HH:MM:SS" depending on duration.
 */
function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${minutes}:${secs.toString().padStart(2, "0")}`;
}

/**
 * Format speed from m/s to km/h with unit.
 */
function formatSpeed(mps: number | null): string {
  if (mps === null) return "-";
  const kmh = mps * 3.6;
  return `${kmh.toFixed(1)} km/h`;
}

/**
 * Format ISO date string to readable date (e.g., "Jan 15, 2024").
 */
function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

/**
 * Get medal emoji for top 3 ranks.
 */
function getRankDisplay(rank: number): React.ReactNode {
  switch (rank) {
    case 1:
      return <span className="text-lg">1</span>;
    case 2:
      return <span className="text-lg">2</span>;
    case 3:
      return <span className="text-lg">3</span>;
    default:
      return <span>{rank}</span>;
  }
}

/**
 * Get medal icon for top 3 ranks.
 */
function getMedalIcon(rank: number): string | null {
  switch (rank) {
    case 1:
      return "\u{1F947}"; // Gold medal
    case 2:
      return "\u{1F948}"; // Silver medal
    case 3:
      return "\u{1F949}"; // Bronze medal
    default:
      return null;
  }
}

function LoadingSkeleton() {
  return (
    <div className="space-y-2">
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="flex items-center space-x-4 p-3">
          <Skeleton className="h-8 w-8 rounded-full" />
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-4 w-16 ml-auto" />
          <Skeleton className="h-4 w-20" />
          <Skeleton className="h-4 w-24" />
        </div>
      ))}
    </div>
  );
}

export function LeaderboardTable({
  entries,
  currentUserId,
  loading,
  onLoadMore,
}: LeaderboardTableProps) {
  if (loading && entries.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Leaderboard</CardTitle>
        </CardHeader>
        <CardContent>
          <LoadingSkeleton />
        </CardContent>
      </Card>
    );
  }

  if (!loading && entries.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Leaderboard</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-center py-8">
            No entries yet. Be the first to complete this segment!
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Leaderboard</CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground w-16">
                  Rank
                </th>
                <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                  User
                </th>
                <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                  Time
                </th>
                <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                  Speed
                </th>
                <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                  Date
                </th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => {
                const isCurrentUser = currentUserId === entry.user_id;
                const medal = getMedalIcon(entry.rank);

                return (
                  <tr
                    key={entry.effort_id}
                    className={cn(
                      "border-b transition-colors hover:bg-muted/50",
                      isCurrentUser && "bg-primary/10 hover:bg-primary/15"
                    )}
                  >
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-1">
                        {medal && (
                          <span className="text-lg" aria-label={`Rank ${entry.rank}`}>
                            {medal}
                          </span>
                        )}
                        <span
                          className={cn(
                            "font-medium",
                            entry.rank <= 3 && "font-bold"
                          )}
                        >
                          {entry.rank}
                        </span>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <span
                          className={cn(
                            "font-medium",
                            isCurrentUser && "text-primary"
                          )}
                        >
                          {entry.user_name}
                        </span>
                        {entry.is_personal_record && (
                          <Badge variant="secondary" className="text-xs">
                            PR
                          </Badge>
                        )}
                        {isCurrentUser && (
                          <Badge variant="outline" className="text-xs">
                            You
                          </Badge>
                        )}
                      </div>
                    </td>
                    <td className="px-4 py-3 text-right font-mono">
                      <span className={cn(entry.rank === 1 && "font-bold")}>
                        {formatTime(entry.elapsed_time_seconds)}
                      </span>
                      {entry.gap_seconds !== null && entry.gap_seconds > 0 && (
                        <span className="text-muted-foreground text-sm ml-2">
                          +{formatTime(entry.gap_seconds)}
                        </span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-right text-muted-foreground">
                      {formatSpeed(entry.average_speed_mps)}
                    </td>
                    <td className="px-4 py-3 text-right text-muted-foreground text-sm">
                      {formatDate(entry.started_at)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {onLoadMore && (
          <div className="p-4 text-center border-t">
            <button
              onClick={onLoadMore}
              disabled={loading}
              className={cn(
                "text-sm text-primary hover:underline",
                loading && "opacity-50 cursor-not-allowed"
              )}
            >
              {loading ? "Loading..." : "Load more"}
            </button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
