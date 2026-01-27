"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/lib/auth-context";
import { api, AchievementWithSegment, AchievementType } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { CrownBadge } from "@/components/leaderboard/crown-badge";
import { Crown, Star, Trophy } from "lucide-react";

type FilterType = "all" | "kom" | "qom" | "local_legend";

function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${minutes}:${secs.toString().padStart(2, "0")}`;
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(2)} km`;
  }
  return `${Math.round(meters)} m`;
}

export default function AchievementsPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [achievements, setAchievements] = useState<AchievementWithSegment[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<FilterType>("all");
  const [showLost, setShowLost] = useState(false);

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user) {
      setLoading(true);
      api
        .getMyAchievements(showLost)
        .then(setAchievements)
        .catch(() => {})
        .finally(() => setLoading(false));
    }
  }, [user, authLoading, router, showLost]);

  if (authLoading) {
    return <LoadingSkeleton />;
  }

  if (!user) {
    return null;
  }

  // Filter achievements based on current filter and lost status
  const filteredAchievements = achievements.filter((a) => {
    if (filter !== "all" && a.achievement_type !== filter) {
      return false;
    }
    return true;
  });

  // Separate current and lost achievements for display
  const currentAchievements = filteredAchievements.filter((a) => !a.lost_at);
  const lostAchievements = filteredAchievements.filter((a) => a.lost_at);

  // Calculate stats (only for current/active achievements)
  const allCurrentAchievements = achievements.filter((a) => !a.lost_at);
  const komCount = allCurrentAchievements.filter((a) => a.achievement_type === "kom").length;
  const qomCount = allCurrentAchievements.filter((a) => a.achievement_type === "qom").length;
  const localLegendCount = allCurrentAchievements.filter(
    (a) => a.achievement_type === "local_legend"
  ).length;
  const totalCrowns = allCurrentAchievements.length;

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Achievements</h1>
        <Button variant="outline" onClick={() => router.push("/profile")}>
          Back to Profile
        </Button>
      </div>

      {/* Summary Stats */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Trophy className="h-5 w-5" />
            Crown Summary
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="grid grid-cols-4 gap-4">
              {[...Array(4)].map((_, i) => (
                <Skeleton key={i} className="h-20 w-full" />
              ))}
            </div>
          ) : (
            <div className="grid grid-cols-4 gap-4 text-center">
              <div className="p-4 bg-muted/50 rounded-lg">
                <p className="text-3xl font-bold">{totalCrowns}</p>
                <p className="text-sm text-muted-foreground">Total Crowns</p>
              </div>
              <div className="p-4 bg-amber-50 dark:bg-amber-950/20 rounded-lg border border-amber-200 dark:border-amber-800">
                <div className="flex items-center justify-center gap-1">
                  <Crown className="h-5 w-5 text-amber-600" />
                  <p className="text-3xl font-bold text-amber-700 dark:text-amber-400">
                    {komCount}
                  </p>
                </div>
                <p className="text-sm text-amber-600 dark:text-amber-500">KOMs</p>
              </div>
              <div className="p-4 bg-amber-50 dark:bg-amber-950/20 rounded-lg border border-amber-200 dark:border-amber-800">
                <div className="flex items-center justify-center gap-1">
                  <Crown className="h-5 w-5 text-amber-600" />
                  <p className="text-3xl font-bold text-amber-700 dark:text-amber-400">
                    {qomCount}
                  </p>
                </div>
                <p className="text-sm text-amber-600 dark:text-amber-500">QOMs</p>
              </div>
              <div className="p-4 bg-purple-50 dark:bg-purple-950/20 rounded-lg border border-purple-200 dark:border-purple-800">
                <div className="flex items-center justify-center gap-1">
                  <Star className="h-5 w-5 text-purple-600" />
                  <p className="text-3xl font-bold text-purple-700 dark:text-purple-400">
                    {localLegendCount}
                  </p>
                </div>
                <p className="text-sm text-purple-600 dark:text-purple-500">Local Legends</p>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Filter Controls */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div className="flex flex-wrap gap-2">
              <Button
                variant={filter === "all" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("all")}
              >
                All
              </Button>
              <Button
                variant={filter === "kom" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("kom")}
                className={filter === "kom" ? "" : "hover:bg-amber-50 hover:text-amber-700"}
              >
                <Crown className="h-4 w-4 mr-1" />
                KOM
              </Button>
              <Button
                variant={filter === "qom" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("qom")}
                className={filter === "qom" ? "" : "hover:bg-amber-50 hover:text-amber-700"}
              >
                <Crown className="h-4 w-4 mr-1" />
                QOM
              </Button>
              <Button
                variant={filter === "local_legend" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("local_legend")}
                className={
                  filter === "local_legend" ? "" : "hover:bg-purple-50 hover:text-purple-700"
                }
              >
                <Star className="h-4 w-4 mr-1" />
                Local Legend
              </Button>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowLost(!showLost)}
              className={showLost ? "text-muted-foreground" : ""}
            >
              {showLost ? "Hide" : "Show"} Lost Crowns
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Achievement Grid */}
      {loading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {[...Array(4)].map((_, i) => (
            <Card key={i}>
              <CardContent className="pt-6">
                <div className="flex items-start gap-4">
                  <Skeleton className="h-12 w-12 rounded-full" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-5 w-3/4" />
                    <Skeleton className="h-4 w-1/2" />
                    <Skeleton className="h-4 w-1/4" />
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : filteredAchievements.length === 0 ? (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-12">
              <Trophy className="h-16 w-16 mx-auto text-muted-foreground/50 mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Achievements Yet</h3>
              <p className="text-muted-foreground mb-4">
                {filter === "all"
                  ? "Start riding segments to earn KOMs, QOMs, and Local Legend crowns!"
                  : `You haven't earned any ${filter === "local_legend" ? "Local Legend" : filter.toUpperCase()} achievements yet.`}
              </p>
              <Button onClick={() => router.push("/segments")}>Explore Segments</Button>
            </div>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-6">
          {/* Current Achievements */}
          {currentAchievements.length > 0 && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {currentAchievements.map((achievement) => (
                <AchievementCard key={achievement.id} achievement={achievement} />
              ))}
            </div>
          )}

          {/* Lost Achievements */}
          {showLost && lostAchievements.length > 0 && (
            <>
              <div className="flex items-center gap-2 text-muted-foreground">
                <div className="h-px flex-1 bg-border" />
                <span className="text-sm">Lost Crowns</span>
                <div className="h-px flex-1 bg-border" />
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {lostAchievements.map((achievement) => (
                  <AchievementCard key={achievement.id} achievement={achievement} isLost />
                ))}
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function AchievementCard({
  achievement,
  isLost = false,
}: {
  achievement: AchievementWithSegment;
  isLost?: boolean;
}) {
  return (
    <Card className={isLost ? "opacity-60" : ""}>
      <CardContent className="pt-6">
        <div className="flex items-start gap-4">
          <div className={isLost ? "grayscale" : ""}>
            <CrownBadge type={achievement.achievement_type} size="lg" showLabel={false} />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <CrownBadge type={achievement.achievement_type} size="sm" />
              {isLost && (
                <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                  Lost
                </span>
              )}
            </div>
            <Link
              href={`/segments/${achievement.segment_id}`}
              className="font-semibold hover:underline line-clamp-1"
            >
              {achievement.segment_name}
            </Link>
            <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
              <p>{formatDistance(achievement.segment_distance_meters)}</p>
              <p>Earned {formatDate(achievement.earned_at)}</p>
              {isLost && achievement.lost_at && (
                <p className="text-red-500">Lost {formatDate(achievement.lost_at)}</p>
              )}
              {achievement.effort_count && (
                <p>{achievement.effort_count} efforts</p>
              )}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function LoadingSkeleton() {
  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <Skeleton className="h-10 w-48" />
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-40" />
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-4 gap-4">
            {[...Array(4)].map((_, i) => (
              <Skeleton key={i} className="h-20 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <div className="flex gap-2">
            {[...Array(4)].map((_, i) => (
              <Skeleton key={i} className="h-8 w-20" />
            ))}
          </div>
        </CardContent>
      </Card>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {[...Array(4)].map((_, i) => (
          <Card key={i}>
            <CardContent className="pt-6">
              <div className="flex items-start gap-4">
                <Skeleton className="h-12 w-12 rounded-full" />
                <div className="flex-1 space-y-2">
                  <Skeleton className="h-5 w-3/4" />
                  <Skeleton className="h-4 w-1/2" />
                  <Skeleton className="h-4 w-1/4" />
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
