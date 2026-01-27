"use client";

import Link from "next/link";
import { FeedActivity } from "@/lib/api";
import { Card, CardContent } from "@/components/ui/card";
import { formatDistanceToNow } from "@/lib/utils";

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
  const timeAgo = formatDistanceToNow(new Date(activity.submitted_at));

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

            {/* Social stats */}
            <div className="flex gap-4 mt-2 text-sm">
              <span className="text-muted-foreground">
                {activity.kudos_count} kudos
              </span>
              <span className="text-muted-foreground">
                {activity.comment_count} comments
              </span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
