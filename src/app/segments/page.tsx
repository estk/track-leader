"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { api, Segment } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";

const ACTIVITY_TYPE_LABELS: Record<string, string> = {
  Running: "Run",
  RoadCycling: "Road Cycling",
  MountainBiking: "Mountain Biking",
  Hiking: "Hike",
  Walking: "Walk",
  Unknown: "Other",
};

function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(2)} km`;
  }
  return `${Math.round(meters)} m`;
}

function formatElevation(meters: number | null): string {
  if (meters === null) return "N/A";
  return `${Math.round(meters)} m`;
}

const ACTIVITY_TYPES = ["All", "Running", "RoadCycling", "MountainBiking", "Hiking", "Walking"];

type SortOption = "newest" | "oldest" | "name-asc" | "name-desc" | "distance-asc" | "distance-desc" | "elevation-asc" | "elevation-desc";

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
  { value: "newest", label: "Newest" },
  { value: "oldest", label: "Oldest" },
  { value: "name-asc", label: "Name (A-Z)" },
  { value: "name-desc", label: "Name (Z-A)" },
  { value: "distance-asc", label: "Distance (shortest)" },
  { value: "distance-desc", label: "Distance (longest)" },
  { value: "elevation-desc", label: "Elevation (highest)" },
  { value: "elevation-asc", label: "Elevation (lowest)" },
];

function sortSegments(segments: Segment[], sortBy: SortOption): Segment[] {
  const sorted = [...segments];
  switch (sortBy) {
    case "newest":
      return sorted.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
    case "oldest":
      return sorted.sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime());
    case "name-asc":
      return sorted.sort((a, b) => a.name.localeCompare(b.name));
    case "name-desc":
      return sorted.sort((a, b) => b.name.localeCompare(a.name));
    case "distance-asc":
      return sorted.sort((a, b) => a.distance_meters - b.distance_meters);
    case "distance-desc":
      return sorted.sort((a, b) => b.distance_meters - a.distance_meters);
    case "elevation-desc":
      return sorted.sort((a, b) => (b.elevation_gain_meters ?? 0) - (a.elevation_gain_meters ?? 0));
    case "elevation-asc":
      return sorted.sort((a, b) => (a.elevation_gain_meters ?? 0) - (b.elevation_gain_meters ?? 0));
    default:
      return sorted;
  }
}

type DistanceFilter = "all" | "under1k" | "1k-5k" | "5k-10k" | "10k-20k" | "over20k";

const DISTANCE_FILTERS: { value: DistanceFilter; label: string; min: number; max: number }[] = [
  { value: "all", label: "Any distance", min: 0, max: Infinity },
  { value: "under1k", label: "< 1 km", min: 0, max: 1000 },
  { value: "1k-5k", label: "1-5 km", min: 1000, max: 5000 },
  { value: "5k-10k", label: "5-10 km", min: 5000, max: 10000 },
  { value: "10k-20k", label: "10-20 km", min: 10000, max: 20000 },
  { value: "over20k", label: "> 20 km", min: 20000, max: Infinity },
];

export default function SegmentsPage() {
  const router = useRouter();
  const [segments, setSegments] = useState<Segment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showStarred, setShowStarred] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [activityTypeFilter, setActivityTypeFilter] = useState<string>("All");
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<SortOption>("newest");
  const [distanceFilter, setDistanceFilter] = useState<DistanceFilter>("all");

  useEffect(() => {
    const hasToken = !!api.getToken();
    setIsLoggedIn(hasToken);
  }, []);

  useEffect(() => {
    setLoading(true);
    const typeFilter = activityTypeFilter === "All" ? undefined : activityTypeFilter;

    const fetchSegments = showStarred && isLoggedIn
      ? api.getStarredSegments()
      : api.listSegments(typeFilter);

    fetchSegments
      .then(setSegments)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [showStarred, isLoggedIn, activityTypeFilter]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Segments</h1>
        {isLoggedIn && (
          <div className="flex gap-2">
            <Button
              variant={showStarred ? "outline" : "default"}
              size="sm"
              onClick={() => setShowStarred(false)}
            >
              All
            </Button>
            <Button
              variant={showStarred ? "default" : "outline"}
              size="sm"
              onClick={() => setShowStarred(true)}
            >
              ★ Starred
            </Button>
          </div>
        )}
      </div>

      <div className="flex flex-col sm:flex-row gap-4">
        <Input
          type="text"
          placeholder="Search segments by name..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="max-w-md"
        />
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as SortOption)}
          className="px-3 py-2 border rounded-md bg-background text-sm"
        >
          {SORT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <select
          value={distanceFilter}
          onChange={(e) => setDistanceFilter(e.target.value as DistanceFilter)}
          className="px-3 py-2 border rounded-md bg-background text-sm"
        >
          {DISTANCE_FILTERS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      {!showStarred && (
        <div className="flex flex-wrap gap-2">
          {ACTIVITY_TYPES.map((type) => (
            <Button
              key={type}
              variant={activityTypeFilter === type ? "default" : "outline"}
              size="sm"
              onClick={() => setActivityTypeFilter(type)}
            >
              {type === "All" ? "All Types" : ACTIVITY_TYPE_LABELS[type] || type}
            </Button>
          ))}
        </div>
      )}

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
      ) : (() => {
        const distFilter = DISTANCE_FILTERS.find((f) => f.value === distanceFilter) || DISTANCE_FILTERS[0];
        const filteredSegments = sortSegments(
          segments.filter((s) => {
            const matchesSearch = s.name.toLowerCase().includes(searchQuery.toLowerCase());
            const matchesDistance = s.distance_meters >= distFilter.min && s.distance_meters < distFilter.max;
            return matchesSearch && matchesDistance;
          }),
          sortBy
        );
        const hasFilters = searchQuery || distanceFilter !== "all";
        return filteredSegments.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground mb-4">
              {hasFilters
                ? "No segments match your filters"
                : showStarred
                ? "No starred segments"
                : "No segments yet"}
            </p>
            <p className="text-sm text-muted-foreground">
              {hasFilters
                ? "Try adjusting your search or filters."
                : showStarred
                ? "Star segments from their detail pages to see them here."
                : "Segments can be created from activity detail pages."}
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {filteredSegments.map((segment) => (
            <Card
              key={segment.id}
              className="hover:bg-muted/50 cursor-pointer transition-colors"
              onClick={() => router.push(`/segments/${segment.id}`)}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{segment.name}</CardTitle>
                  <Badge variant="secondary">
                    {ACTIVITY_TYPE_LABELS[segment.activity_type] || segment.activity_type}
                  </Badge>
                </div>
                <div className="flex gap-4 text-sm text-muted-foreground mt-2">
                  <span>Distance: {formatDistance(segment.distance_meters)}</span>
                  <span>Elevation Gain: {formatElevation(segment.elevation_gain_meters)}</span>
                </div>
                {segment.description && (
                  <p className="text-sm text-muted-foreground mt-2">
                    {segment.description}
                  </p>
                )}
              </CardHeader>
            </Card>
          ))}
        </div>
      );
      })()}
    </div>
  );
}
