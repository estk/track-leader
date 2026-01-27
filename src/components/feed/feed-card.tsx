"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { FeedActivity, api } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent } from "@/components/ui/card";
import { formatDistanceToNow } from "@/lib/utils";
import { KudosButton } from "@/components/social/kudos-button";
import { CommentsSection } from "@/components/social/comments-section";

interface FeedCardProps {
  activity: FeedActivity;
}

function formatDistance(meters: number | null): string {
  if (meters === null) return "—";
  if (meters < 1000) return `${Math.round(meters)} m`;
  return `${(meters / 1000).toFixed(1)} km`;
}

function formatDuration(seconds: number | null): string {
  if (seconds === null) return "—";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function formatElevation(meters: number | null): string {
  if (meters === null) return "—";
  return `${Math.round(meters)} m`;
}

function getActivityIcon(type: string): string {
  switch (type.toLowerCase()) {
    case "run":
      return "🏃";
    case "ride":
    case "cycling":
      return "🚴";
    case "hike":
      return "🥾";
    case "walk":
      return "🚶";
    case "swim":
      return "🏊";
    default:
      return "📍";
  }
}

export function FeedCard({ activity }: FeedCardProps) {
  const { user } = useAuth();
  const timeAgo = formatDistanceToNow(new Date(activity.submitted_at));
  const [hasGivenKudos, setHasGivenKudos] = useState(false);
  const [kudosCount, setKudosCount] = useState(activity.kudos_count);

  // Check if current user has given kudos
  useEffect(() => {
    if (user && user.id !== activity.user_id) {
      api.getKudosStatus(activity.id).then(setHasGivenKudos).catch(() => {});
    }
  }, [user, activity.id, activity.user_id]);

  const isOwnActivity = user?.id === activity.user_id;

  return (
    <Card className="hover:bg-muted/50 transition-colors">
      <CardContent className="p-4">
        <div className="flex items-start gap-3">
          {/* User avatar */}
          <Link href={`/profile/${activity.user_id}`}>
            <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-lg font-bold text-primary shrink-0">
              {activity.user_name.charAt(0).toUpperCase()}
            </div>
          </Link>

          <div className="flex-1 min-w-0">
            {/* Header */}
            <div className="flex items-center gap-2 flex-wrap">
              <Link
                href={`/profile/${activity.user_id}`}
                className="font-semibold hover:underline"
              >
                {activity.user_name}
              </Link>
              <span className="text-muted-foreground text-sm">{timeAgo}</span>
            </div>

            {/* Activity title */}
            <Link
              href={`/activities/${activity.id}`}
              className="block mt-1 hover:underline"
            >
              <span className="mr-2">{getActivityIcon(activity.activity_type)}</span>
              <span className="font-medium">{activity.name}</span>
            </Link>

            {/* Stats */}
            <div className="flex gap-4 mt-2 text-sm text-muted-foreground">
              <span>{formatDistance(activity.distance)}</span>
              <span>{formatDuration(activity.duration)}</span>
              <span>↑ {formatElevation(activity.elevation_gain)}</span>
            </div>

            {/* Kudos and comments */}
            <div className="flex items-center gap-4 mt-3 pt-3 border-t">
              {user && (
                <KudosButton
                  activityId={activity.id}
                  initialHasGiven={hasGivenKudos}
                  initialCount={kudosCount}
                  disabled={isOwnActivity}
                  onKudosChange={(hasGiven, count) => {
                    setHasGivenKudos(hasGiven);
                    setKudosCount(count);
                  }}
                />
              )}
              {!user && (
                <span className="text-sm text-muted-foreground">
                  👏 {kudosCount}
                </span>
              )}
            </div>

            {/* Comments section */}
            <div className="mt-3">
              <CommentsSection
                activityId={activity.id}
                initialCommentCount={activity.comment_count}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
