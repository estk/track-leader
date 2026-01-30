"use client";

import { useEffect, useState, useCallback } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, FeedActivity } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { FeedCard } from "@/components/feed/feed-card";
import { LazyDailyActivitiesMap } from "@/components/activity/lazy-daily-activities-map";

function getTodayDateString(): string {
  const today = new Date();
  return today.toISOString().split("T")[0];
}

export default function DailyActivitiesPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { user, loading: authLoading } = useAuth();

  // Get date from URL or default to today
  const dateParam = searchParams.get("date");
  const mineParam = searchParams.get("mine");

  const [date, setDate] = useState(dateParam || getTodayDateString());
  const [mineOnly, setMineOnly] = useState(mineParam === "true");
  const [activities, setActivities] = useState<FeedActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Update URL when date or mine filter changes
  const updateUrl = useCallback(
    (newDate: string, newMineOnly: boolean) => {
      const params = new URLSearchParams();
      params.set("date", newDate);
      if (newMineOnly) {
        params.set("mine", "true");
      }
      router.replace(`/activities/daily?${params.toString()}`, { scroll: false });
    },
    [router]
  );

  // Load activities when date or filter changes
  const loadActivities = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      // Only pass mineOnly if user is logged in and filter is enabled
      const mineOnlyParam = user && mineOnly ? true : undefined;
      const data = await api.getActivitiesByDate(date, mineOnlyParam);
      setActivities(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load activities");
    } finally {
      setLoading(false);
    }
  }, [date, mineOnly, user]);

  // Load activities when parameters change
  useEffect(() => {
    // Wait for auth to finish loading before making the request
    if (authLoading) return;
    loadActivities();
  }, [authLoading, loadActivities]);

  // Sync state with URL params when they change
  useEffect(() => {
    const urlDate = searchParams.get("date");
    const urlMine = searchParams.get("mine");
    if (urlDate && urlDate !== date) {
      setDate(urlDate);
    }
    if (urlMine === "true" !== mineOnly) {
      setMineOnly(urlMine === "true");
    }
  }, [searchParams, date, mineOnly]);

  const handleDateChange = (newDate: string) => {
    setDate(newDate);
    updateUrl(newDate, mineOnly);
  };

  const handleMineOnlyToggle = () => {
    const newMineOnly = !mineOnly;
    setMineOnly(newMineOnly);
    updateUrl(date, newMineOnly);
  };

  const handlePreviousDay = () => {
    const currentDate = new Date(date);
    currentDate.setDate(currentDate.getDate() - 1);
    const newDate = currentDate.toISOString().split("T")[0];
    handleDateChange(newDate);
  };

  const handleNextDay = () => {
    const currentDate = new Date(date);
    currentDate.setDate(currentDate.getDate() + 1);
    const newDate = currentDate.toISOString().split("T")[0];
    handleDateChange(newDate);
  };

  const handleToday = () => {
    handleDateChange(getTodayDateString());
  };

  const isToday = date === getTodayDateString();

  // Format the date for display
  const displayDate = new Date(date + "T00:00:00").toLocaleDateString(undefined, {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-3xl font-bold">Daily Activities</h1>
      </div>

      {/* Date and filter controls */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
            {/* Date picker section */}
            <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:gap-4">
              <div className="space-y-1">
                <Label htmlFor="date-picker">Date</Label>
                <Input
                  id="date-picker"
                  type="date"
                  value={date}
                  onChange={(e) => handleDateChange(e.target.value)}
                  className="w-full sm:w-auto"
                />
              </div>

              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handlePreviousDay}
                  title="Previous day"
                >
                  Previous
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleNextDay}
                  title="Next day"
                >
                  Next
                </Button>
                {!isToday && (
                  <Button variant="outline" size="sm" onClick={handleToday}>
                    Today
                  </Button>
                )}
              </div>
            </div>

            {/* Mine only toggle - only show when logged in */}
            {user && (
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="mine-only"
                  checked={mineOnly}
                  onChange={handleMineOnlyToggle}
                  className="h-4 w-4 rounded border-gray-300"
                />
                <Label htmlFor="mine-only" className="cursor-pointer">
                  My activities only
                </Label>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Display selected date */}
      <p className="text-muted-foreground">{displayDate}</p>

      {/* Error state */}
      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {/* Loading state */}
      {loading || authLoading ? (
        <div className="space-y-4">
          <Skeleton className="h-[400px] w-full rounded-lg" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
        </div>
      ) : activities.length === 0 ? (
        /* Empty state */
        <Card>
          <CardHeader>
            <CardTitle>No Activities</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">
              {mineOnly
                ? "You don't have any activities on this date."
                : "No public activities found for this date."}
            </p>
          </CardContent>
        </Card>
      ) : (
        /* Activities map and list */
        <div className="space-y-6">
          {/* Map showing all activities */}
          <LazyDailyActivitiesMap activities={activities} />

          {/* Activity list */}
          <div className="space-y-4">
            <h2 className="text-xl font-semibold">
              {activities.length} {activities.length === 1 ? "Activity" : "Activities"}
            </h2>
            {activities.map((activity) => (
              <FeedCard key={activity.id} activity={activity} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
