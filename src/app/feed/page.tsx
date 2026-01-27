"use client";

import { useEffect, useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, FeedActivity } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { FeedCard } from "@/components/feed/feed-card";

const PAGE_SIZE = 20;

export default function FeedPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [activities, setActivities] = useState<FeedActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  const loadFeed = useCallback(async (offset: number = 0) => {
    try {
      const data = await api.getFeed(PAGE_SIZE, offset);
      if (offset === 0) {
        setActivities(data);
      } else {
        setActivities((prev) => [...prev, ...data]);
      }
      setHasMore(data.length === PAGE_SIZE);
    } catch {
      // Error loading feed
    }
  }, []);

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user) {
      loadFeed().finally(() => setLoading(false));
    }
  }, [user, authLoading, router, loadFeed]);

  const handleLoadMore = async () => {
    setLoadingMore(true);
    await loadFeed(activities.length);
    setLoadingMore(false);
  };

  if (authLoading || loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-4">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  if (!user) {
    return null;
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <h1 className="text-3xl font-bold">Activity Feed</h1>

      {activities.length === 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>No Activities Yet</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground mb-4">
              Follow other users to see their activities in your feed.
            </p>
            <Button onClick={() => router.push("/leaderboards")}>
              Find People to Follow
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          <div className="space-y-4">
            {activities.map((activity) => (
              <FeedCard key={activity.id} activity={activity} />
            ))}
          </div>

          {hasMore && (
            <div className="flex justify-center">
              <Button
                variant="outline"
                onClick={handleLoadMore}
                disabled={loadingMore}
              >
                {loadingMore ? "Loading..." : "Load More"}
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
