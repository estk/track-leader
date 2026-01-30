"use client";

import { useEffect, useState, useMemo, useCallback } from "react";
import Link from "next/link";
import {
  api,
  CrownCountEntry,
  DistanceLeaderEntry,
  GlobalLeaderboardFilters,
  LeaderboardScope,
  GenderFilter,
  AgeGroup,
  WeightClass,
  CountryStats,
  ACTIVITY_TYPE_OPTIONS,
  getActivityTypeName,
} from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { RankBadge } from "@/components/leaderboard/crown-badge";
import { Crown, MapPin, Filter, ChevronDown, ChevronUp, X } from "lucide-react";

type LeaderboardTab = "crowns" | "distance";

const PAGE_SIZE = 20;

// Filter options
const SCOPE_OPTIONS: { value: LeaderboardScope; label: string }[] = [
  { value: "all_time", label: "All Time" },
  { value: "year", label: "This Year" },
  { value: "month", label: "This Month" },
  { value: "week", label: "This Week" },
];

const GENDER_OPTIONS: { value: GenderFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "male", label: "Male" },
  { value: "female", label: "Female" },
];

const AGE_GROUP_OPTIONS: { value: AgeGroup; label: string }[] = [
  { value: "all", label: "All" },
  { value: "18-24", label: "18-24" },
  { value: "25-29", label: "25-29" },
  { value: "30-34", label: "30-34" },
  { value: "35-39", label: "35-39" },
  { value: "40-49", label: "40-49" },
  { value: "50-59", label: "50-59" },
  { value: "60+", label: "60+" },
];

const WEIGHT_CLASS_OPTIONS: { value: WeightClass; label: string }[] = [
  { value: "all", label: "All" },
  { value: "featherweight", label: "Featherweight (<55 kg)" },
  { value: "lightweight", label: "Lightweight (55-64 kg)" },
  { value: "welterweight", label: "Welterweight (65-74 kg)" },
  { value: "middleweight", label: "Middleweight (75-84 kg)" },
  { value: "cruiserweight", label: "Cruiserweight (85-94 kg)" },
  { value: "heavyweight", label: "Heavyweight (95+ kg)" },
];

interface FilterState {
  scope: LeaderboardScope;
  gender: GenderFilter;
  ageGroup: AgeGroup;
  weightClass: WeightClass;
  country: string | null;
  activityTypeId: string | null;  // For crown leaderboard only
}

const DEFAULT_FILTERS: FilterState = {
  scope: "all_time",
  gender: "all",
  ageGroup: "all",
  weightClass: "all",
  country: null,
  activityTypeId: null,
};

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

  // Filter state
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS);
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [countries, setCountries] = useState<CountryStats[]>([]);
  const [countriesLoading, setCountriesLoading] = useState(true);

  // Count active filters (excluding defaults)
  const activeFilterCount = useMemo(() => {
    let count = 0;
    if (filters.scope !== "all_time") count++;
    if (filters.gender !== "all") count++;
    if (filters.ageGroup !== "all") count++;
    if (filters.weightClass !== "all") count++;
    if (filters.country) count++;
    if (filters.activityTypeId && activeTab === "crowns") count++;
    return count;
  }, [filters, activeTab]);

  // Load countries on mount
  useEffect(() => {
    api.getCountries()
      .then(setCountries)
      .catch(() => setCountries([]))
      .finally(() => setCountriesLoading(false));
  }, []);

  // Convert filter state to API filters
  const buildApiFilters = useCallback((offset: number): GlobalLeaderboardFilters => {
    const apiFilters: GlobalLeaderboardFilters = {
      limit: PAGE_SIZE,
      offset,
    };
    if (filters.scope !== "all_time") apiFilters.scope = filters.scope;
    if (filters.gender !== "all") apiFilters.gender = filters.gender;
    if (filters.ageGroup !== "all") apiFilters.ageGroup = filters.ageGroup;
    if (filters.weightClass !== "all") apiFilters.weightClass = filters.weightClass;
    if (filters.country) apiFilters.country = filters.country;
    if (filters.activityTypeId && activeTab === "crowns") {
      apiFilters.activityTypeId = filters.activityTypeId;
    }
    return apiFilters;
  }, [filters, activeTab]);

  // Load initial data when tab or filters change
  useEffect(() => {
    setLoading(true);
    setError("");

    const apiFilters = buildApiFilters(0);

    if (activeTab === "crowns") {
      api
        .getCrownLeaderboard(apiFilters)
        .then((entries) => {
          setCrownEntries(entries);
          setHasMoreCrowns(entries.length === PAGE_SIZE);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    } else {
      api
        .getDistanceLeaderboard(apiFilters)
        .then((entries) => {
          setDistanceEntries(entries);
          setHasMoreDistance(entries.length === PAGE_SIZE);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [activeTab, filters, buildApiFilters]);

  const loadMoreCrowns = async () => {
    setLoadingMore(true);
    try {
      const apiFilters = buildApiFilters(crownEntries.length);
      const newEntries = await api.getCrownLeaderboard(apiFilters);
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
      const apiFilters = buildApiFilters(distanceEntries.length);
      const newEntries = await api.getDistanceLeaderboard(apiFilters);
      setDistanceEntries((prev) => [...prev, ...newEntries]);
      setHasMoreDistance(newEntries.length === PAGE_SIZE);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load more");
    } finally {
      setLoadingMore(false);
    }
  };

  const updateFilter = (key: keyof FilterState, value: string | null) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
  };

  const clearFilters = () => {
    setFilters(DEFAULT_FILTERS);
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

      {/* Filter toggle button */}
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          onClick={() => setFiltersOpen(!filtersOpen)}
          className="gap-2"
        >
          <Filter className="h-4 w-4" />
          Filters
          {activeFilterCount > 0 && (
            <span className="ml-1 px-2 py-0.5 bg-primary text-primary-foreground rounded-full text-xs">
              {activeFilterCount}
            </span>
          )}
          {filtersOpen ? (
            <ChevronUp className="h-4 w-4" />
          ) : (
            <ChevronDown className="h-4 w-4" />
          )}
        </Button>
        {activeFilterCount > 0 && (
          <Button variant="ghost" size="sm" onClick={clearFilters} className="gap-1">
            <X className="h-4 w-4" />
            Clear all
          </Button>
        )}
      </div>

      {/* Collapsible filter section */}
      {filtersOpen && (
        <Card>
          <CardContent className="pt-6">
            <div className="flex flex-col gap-4">
              {/* Primary row: Time, Gender, Age */}
              <div className="flex flex-col sm:flex-row gap-4">
                {/* Time scope filter */}
                <div className="flex flex-col gap-1">
                  <label htmlFor="scope-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
                    Time Period
                  </label>
                  <select
                    id="scope-filter"
                    value={filters.scope}
                    onChange={(e) => updateFilter("scope", e.target.value as LeaderboardScope)}
                    className="px-3 py-2 border rounded-md bg-background text-sm min-w-[140px]"
                  >
                    {SCOPE_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Gender filter */}
                <div className="flex flex-col gap-1">
                  <label htmlFor="gender-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
                    Gender
                  </label>
                  <select
                    id="gender-filter"
                    value={filters.gender}
                    onChange={(e) => updateFilter("gender", e.target.value as GenderFilter)}
                    className="px-3 py-2 border rounded-md bg-background text-sm min-w-[100px]"
                  >
                    {GENDER_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Age group filter */}
                <div className="flex flex-col gap-1">
                  <label htmlFor="age-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
                    Age Group
                  </label>
                  <select
                    id="age-filter"
                    value={filters.ageGroup}
                    onChange={(e) => updateFilter("ageGroup", e.target.value as AgeGroup)}
                    className="px-3 py-2 border rounded-md bg-background text-sm min-w-[100px]"
                  >
                    {AGE_GROUP_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Secondary row: Weight, Country, Activity Type (crowns only) */}
              <div className="flex flex-col sm:flex-row gap-4">
                {/* Weight class filter */}
                <div className="flex flex-col gap-1">
                  <label htmlFor="weight-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
                    Weight Class
                  </label>
                  <select
                    id="weight-filter"
                    value={filters.weightClass}
                    onChange={(e) => updateFilter("weightClass", e.target.value as WeightClass)}
                    className="px-3 py-2 border rounded-md bg-background text-sm min-w-[180px]"
                  >
                    {WEIGHT_CLASS_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Country filter */}
                <div className="flex flex-col gap-1">
                  <label htmlFor="country-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
                    Country
                  </label>
                  <select
                    id="country-filter"
                    value={filters.country || ""}
                    onChange={(e) => updateFilter("country", e.target.value || null)}
                    className="px-3 py-2 border rounded-md bg-background text-sm min-w-[200px]"
                    disabled={countriesLoading}
                  >
                    <option value="">All Countries</option>
                    {countries.map((c) => (
                      <option key={c.country} value={c.country}>
                        {c.country} ({c.user_count.toLocaleString()})
                      </option>
                    ))}
                  </select>
                </div>

                {/* Activity type filter (crowns tab only) */}
                {activeTab === "crowns" && (
                  <div className="flex flex-col gap-1">
                    <label htmlFor="activity-type-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
                      Activity Type
                    </label>
                    <select
                      id="activity-type-filter"
                      value={filters.activityTypeId || ""}
                      onChange={(e) => updateFilter("activityTypeId", e.target.value || null)}
                      className="px-3 py-2 border rounded-md bg-background text-sm min-w-[160px]"
                    >
                      <option value="">All Types</option>
                      {ACTIVITY_TYPE_OPTIONS.map((opt) => (
                        <option key={opt.id} value={opt.id}>
                          {opt.name}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      )}

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
          activityTypeFilter={filters.activityTypeId}
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
  activityTypeFilter: string | null;
}

function CrownLeaderboard({
  entries,
  currentUserId,
  hasMore,
  loadingMore,
  onLoadMore,
  activityTypeFilter,
}: CrownLeaderboardProps) {
  if (entries.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <Crown className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">No crown holders found</p>
          <p className="text-sm text-muted-foreground mt-2">
            {activityTypeFilter
              ? `No athletes have crowns for ${getActivityTypeName(activityTypeFilter)} with the current filters.`
              : "Be the first to claim a KOM or QOM!"}
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
          Athletes ranked by total crowns
          {activityTypeFilter && (
            <span className="ml-1">
              for {getActivityTypeName(activityTypeFilter)}
            </span>
          )}
        </p>
      </CardHeader>
      <CardContent className="p-0">
        {/* Header row */}
        <div className="hidden sm:grid sm:grid-cols-[3rem_1fr_6rem] gap-4 px-4 py-2 bg-muted/50 text-xs font-medium text-muted-foreground uppercase tracking-wide border-b">
          <div>Rank</div>
          <div>Athlete</div>
          <div className="text-center">Crowns</div>
        </div>

        <div className="divide-y">
          {entries.map((entry) => {
            const isCurrentUser = currentUserId === entry.user_id;
            return (
              <div
                key={entry.user_id}
                className={`grid grid-cols-[3rem_1fr_4rem] sm:grid-cols-[3rem_1fr_6rem] gap-4 px-4 py-3 items-center ${
                  isCurrentUser ? "bg-primary/5 border-l-2 border-l-primary" : ""
                }`}
              >
                {/* Rank */}
                <div className="flex justify-center">
                  <RankBadge rank={entry.rank} size="sm" />
                </div>

                {/* User name */}
                <div className="truncate">
                  <Link
                    href={`/profile/${entry.user_id}`}
                    className="font-medium truncate hover:underline"
                  >
                    {entry.user_name}
                  </Link>
                  {isCurrentUser && (
                    <span className="text-xs text-muted-foreground ml-2">(you)</span>
                  )}
                </div>

                {/* Crowns */}
                <div className="flex items-center justify-center gap-1">
                  <Crown className="h-4 w-4 text-amber-500" />
                  <span className="font-bold text-lg">{entry.total_crowns}</span>
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
          <p className="text-muted-foreground">No distance data found</p>
          <p className="text-sm text-muted-foreground mt-2">
            No athletes match the current filters, or no activities have been uploaded yet.
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
                <div className="truncate">
                  <Link
                    href={`/profile/${entry.user_id}`}
                    className="font-medium truncate hover:underline"
                  >
                    {entry.user_name}
                  </Link>
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
