"use client";

import { useEffect, useState, useMemo, Suspense } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import {
  api,
  Activity,
  getActivityTypeName,
  ACTIVITY_TYPE_OPTIONS,
  UserActivitiesFilters,
  DateRangeFilter,
  VisibilityFilter,
  ActivitySortBy,
} from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Input } from "@/components/ui/input";
import { useUrlFilters } from "@/hooks/use-url-filters";
import { Globe, Lock, Users } from "lucide-react";

// Filter options
const DATE_RANGE_OPTIONS: { value: DateRangeFilter | "all"; label: string }[] = [
  { value: "all", label: "All Time" },
  { value: "week", label: "This Week" },
  { value: "month", label: "This Month" },
  { value: "year", label: "This Year" },
];

const SORT_OPTIONS: { value: ActivitySortBy; label: string }[] = [
  { value: "recent", label: "Recent" },
  { value: "oldest", label: "Oldest" },
  { value: "distance", label: "Distance" },
  { value: "duration", label: "Duration" },
];

const VISIBILITY_OPTIONS: { value: VisibilityFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "public", label: "Public" },
  { value: "private", label: "Private" },
  { value: "teams_only", label: "Teams Only" },
];

const ACTIVITY_TYPE_FILTERS = [
  { id: "all", name: "All Types" },
  ...ACTIVITY_TYPE_OPTIONS,
];

const DEFAULT_FILTERS = {
  activityType: undefined as string | undefined,
  dateRange: undefined as string | undefined,
  sortBy: undefined as string | undefined,
  visibility: undefined as string | undefined,
  search: undefined as string | undefined,
};

function ActivitiesPageContent() {
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [activities, setActivities] = useState<Activity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const [filters, setFilters, resetFilters] = useUrlFilters(DEFAULT_FILTERS);

  // Convert URL filters to API filters
  const apiFilters = useMemo((): UserActivitiesFilters => {
    const result: UserActivitiesFilters = {};
    if (filters.activityType && filters.activityType !== "all") {
      result.activityTypeId = filters.activityType;
    }
    if (filters.dateRange && filters.dateRange !== "all") {
      result.dateRange = filters.dateRange as DateRangeFilter;
    }
    if (filters.sortBy) {
      result.sortBy = filters.sortBy as ActivitySortBy;
    }
    if (filters.visibility && filters.visibility !== "all") {
      result.visibility = filters.visibility as VisibilityFilter;
    }
    if (filters.search) {
      result.search = filters.search;
    }
    return result;
  }, [filters]);

  // Check if any filters are active
  const hasActiveFilters = useMemo(() => {
    return (
      (filters.activityType && filters.activityType !== "all") ||
      (filters.dateRange && filters.dateRange !== "all") ||
      filters.sortBy ||
      (filters.visibility && filters.visibility !== "all") ||
      filters.search
    );
  }, [filters]);

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user) {
      setLoading(true);
      api
        .getUserActivities(user.id, apiFilters)
        .then(setActivities)
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [user, authLoading, router, apiFilters]);

  if (authLoading || (!user && !error)) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Activities</h1>
        <Button onClick={() => router.push("/activities/upload")}>
          Upload Activity
        </Button>
      </div>

      {/* Filter Bar */}
      <div className="flex flex-col gap-4">
        {/* Search and Sort Row */}
        <div className="flex flex-col sm:flex-row gap-4">
          <Input
            type="text"
            placeholder="Search activities..."
            value={filters.search || ""}
            onChange={(e) => setFilters({ search: e.target.value || undefined })}
            className="max-w-md"
          />
          <select
            value={filters.sortBy || "recent"}
            onChange={(e) =>
              setFilters({ sortBy: e.target.value === "recent" ? undefined : e.target.value })
            }
            className="px-3 py-2 border rounded-md bg-background text-sm"
          >
            {SORT_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
          <select
            value={filters.dateRange || "all"}
            onChange={(e) =>
              setFilters({ dateRange: e.target.value === "all" ? undefined : e.target.value })
            }
            className="px-3 py-2 border rounded-md bg-background text-sm"
          >
            {DATE_RANGE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
          <select
            value={filters.visibility || "all"}
            onChange={(e) =>
              setFilters({ visibility: e.target.value === "all" ? undefined : e.target.value })
            }
            className="px-3 py-2 border rounded-md bg-background text-sm"
          >
            {VISIBILITY_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* Activity Type Filter Row */}
        <div className="flex flex-wrap items-center gap-2">
          {ACTIVITY_TYPE_FILTERS.map((type) => (
            <Button
              key={type.id}
              variant={
                (filters.activityType || "all") === type.id ? "default" : "outline"
              }
              size="sm"
              onClick={() =>
                setFilters({ activityType: type.id === "all" ? undefined : type.id })
              }
            >
              {type.name}
            </Button>
          ))}
          {hasActiveFilters && (
            <Button variant="ghost" size="sm" onClick={resetFilters} className="ml-auto">
              Clear filters
            </Button>
          )}
        </div>
      </div>

      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {loading ? (
        <div className="space-y-4">
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
        </div>
      ) : activities.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground mb-4">
              {hasActiveFilters ? "No activities match your filters" : "No activities yet"}
            </p>
            {hasActiveFilters ? (
              <Button variant="outline" onClick={resetFilters}>
                Clear filters
              </Button>
            ) : (
              <Button onClick={() => router.push("/activities/upload")}>
                Upload your first activity
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {activities.map((activity) => (
            <Card
              key={activity.id}
              className="hover:bg-muted/50 cursor-pointer transition-colors"
              onClick={() => router.push(`/activities/${activity.id}`)}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 flex-wrap">
                    <CardTitle className="text-lg">{activity.name}</CardTitle>
                    {activity.visibility === "public" && (
                      <Badge variant="secondary" className="gap-1">
                        <Globe className="h-3 w-3" />
                        Public
                      </Badge>
                    )}
                    {activity.visibility === "private" && (
                      <Badge variant="outline" className="gap-1">
                        <Lock className="h-3 w-3" />
                        Private
                      </Badge>
                    )}
                    {activity.visibility === "teams_only" && (
                      <Badge variant="default" className="gap-1 bg-blue-600 hover:bg-blue-700">
                        <Users className="h-3 w-3" />
                        Teams
                      </Badge>
                    )}
                  </div>
                  <Badge variant="secondary">
                    {getActivityTypeName(activity.activity_type_id)}
                  </Badge>
                </div>
                <p className="text-sm text-muted-foreground">
                  {new Date(activity.submitted_at).toLocaleDateString(undefined, {
                    weekday: "long",
                    year: "numeric",
                    month: "long",
                    day: "numeric",
                  })}
                </p>
              </CardHeader>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}

export default function ActivitiesPage() {
  return (
    <Suspense
      fallback={
        <div className="space-y-4">
          <Skeleton className="h-8 w-48" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
        </div>
      }
    >
      <ActivitiesPageContent />
    </Suspense>
  );
}
