"use client";

import { useEffect, useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import {
  api,
  FeedActivity,
  FeedFilters,
  DateRangeFilter,
  ACTIVITY_TYPE_OPTIONS,
  DATE_RANGE_OPTIONS,
} from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { FeedCard } from "@/components/feed/feed-card";

const PAGE_SIZE = 20;

export default function FeedPage() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [activities, setActivities] = useState<FeedActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  // Filter state
  const [activityTypeId, setActivityTypeId] = useState<string | undefined>(
    undefined
  );
  const [dateRange, setDateRange] = useState<DateRangeFilter>("all");

  const hasActiveFilters = activityTypeId !== undefined || dateRange !== "all";

  const loadFeed = useCallback(
    async (offset: number = 0) => {
      try {
        const filters: FeedFilters = {
          limit: PAGE_SIZE,
          offset,
        };
        if (activityTypeId) {
          filters.activityTypeId = activityTypeId;
        }
        if (dateRange !== "all") {
          filters.dateRange = dateRange;
        }

        const data = await api.getFeed(filters);
        if (offset === 0) {
          setActivities(data);
        } else {
          setActivities((prev) => [...prev, ...data]);
        }
        setHasMore(data.length === PAGE_SIZE);
      } catch {
        // Error loading feed
      }
    },
    [activityTypeId, dateRange]
  );

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user) {
      setLoading(true);
      loadFeed().finally(() => setLoading(false));
    }
  }, [user, authLoading, router, loadFeed]);

  const handleLoadMore = async () => {
    setLoadingMore(true);
    await loadFeed(activities.length);
    setLoadingMore(false);
  };

  const handleClearFilters = () => {
    setActivityTypeId(undefined);
    setDateRange("all");
  };

  const handleActivityTypeChange = (value: string) => {
    setActivityTypeId(value === "all" ? undefined : value);
  };

  const handleDateRangeChange = (value: string) => {
    setDateRange(value as DateRangeFilter);
  };

  if (authLoading || loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-4">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-10 w-full" />
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

      {/* Filter Bar */}
      <div className="flex flex-wrap items-center gap-3">
        <Select
          value={activityTypeId ?? "all"}
          onValueChange={handleActivityTypeChange}
        >
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Activity Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Types</SelectItem>
            {ACTIVITY_TYPE_OPTIONS.map((option) => (
              <SelectItem key={option.id} value={option.id}>
                {option.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Select value={dateRange} onValueChange={handleDateRangeChange}>
          <SelectTrigger className="w-[150px]">
            <SelectValue placeholder="Date Range" />
          </SelectTrigger>
          <SelectContent>
            {DATE_RANGE_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {hasActiveFilters && (
          <Button variant="ghost" size="sm" onClick={handleClearFilters}>
            Clear filters
          </Button>
        )}
      </div>

      {activities.length === 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>
              {hasActiveFilters ? "No Matching Activities" : "No Activities Yet"}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground mb-4">
              {hasActiveFilters
                ? "Try adjusting your filters to see more activities."
                : "Follow other users to see their activities in your feed."}
            </p>
            {hasActiveFilters ? (
              <Button onClick={handleClearFilters}>Clear Filters</Button>
            ) : (
              <Button onClick={() => router.push("/leaderboards")}>
                Find People to Follow
              </Button>
            )}
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
