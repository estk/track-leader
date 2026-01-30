"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { api, AchievementWithSegment, UserProfile } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { CrownBadge } from "@/components/leaderboard/crown-badge";
import { Crown, Trophy } from "lucide-react";

type FilterType = "all" | "kom" | "qom";

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

export default function UserAchievementsPage() {
  const params = useParams();
  const router = useRouter();
  const userId = params.userId as string;

  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [achievements, setAchievements] = useState<AchievementWithSegment[]>(
    [],
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [filter, setFilter] = useState<FilterType>("all");

  useEffect(() => {
    if (!userId) return;

    const loadData = async () => {
      try {
        const [profileData, achievementsData] = await Promise.all([
          api.getUserProfile(userId),
          api.getUserAchievements(userId),
        ]);
        setProfile(profileData);
        // Only show current achievements (not lost)
        setAchievements(achievementsData.filter((a) => !a.lost_at));
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to load achievements",
        );
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [userId]);

  // Filter achievements based on type
  const filteredAchievements = achievements.filter((a) => {
    if (filter !== "all" && a.achievement_type !== filter) {
      return false;
    }
    return true;
  });

  // Calculate stats
  const komCount = achievements.filter(
    (a) => a.achievement_type === "kom",
  ).length;
  const qomCount = achievements.filter(
    (a) => a.achievement_type === "qom",
  ).length;
  const totalCrowns = achievements.length;

  if (loading) {
    return <LoadingSkeleton />;
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm" onClick={() => router.back()}>
          &larr;
        </Button>
        <h1 className="text-2xl font-bold">
          {profile?.name}&apos;s Achievements
        </h1>
      </div>

      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {/* Summary Stats */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Trophy className="h-5 w-5" />
            Crown Summary
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4 text-center">
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
          </div>
        </CardContent>
      </Card>

      {/* Filter Controls */}
      <Card>
        <CardContent className="pt-6">
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
              className={
                filter === "kom" ? "" : "hover:bg-amber-50 hover:text-amber-700"
              }
            >
              <Crown className="h-4 w-4 mr-1" />
              KOM
            </Button>
            <Button
              variant={filter === "qom" ? "default" : "outline"}
              size="sm"
              onClick={() => setFilter("qom")}
              className={
                filter === "qom" ? "" : "hover:bg-amber-50 hover:text-amber-700"
              }
            >
              <Crown className="h-4 w-4 mr-1" />
              QOM
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Achievement Grid */}
      {filteredAchievements.length === 0 ? (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-12">
              <Trophy className="h-16 w-16 mx-auto text-muted-foreground/50 mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Achievements</h3>
              <p className="text-muted-foreground">
                {filter === "all"
                  ? `${profile?.name} hasn't earned any crowns yet.`
                  : `${profile?.name} hasn't earned any ${filter.toUpperCase()} achievements yet.`}
              </p>
            </div>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {filteredAchievements.map((achievement) => (
            <AchievementCard key={achievement.id} achievement={achievement} />
          ))}
        </div>
      )}
    </div>
  );
}

function AchievementCard({
  achievement,
}: {
  achievement: AchievementWithSegment;
}) {
  return (
    <Card>
      <CardContent className="pt-6">
        <div className="flex items-start gap-4">
          <CrownBadge
            type={achievement.achievement_type}
            size="lg"
            showLabel={false}
          />
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <CrownBadge type={achievement.achievement_type} size="sm" />
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
          <div className="grid grid-cols-3 gap-4">
            {[...Array(3)].map((_, i) => (
              <Skeleton key={i} className="h-20 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <div className="flex gap-2">
            {[...Array(3)].map((_, i) => (
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
