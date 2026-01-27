"use client";

import { useEffect, useState, useMemo } from "react";
import { useRouter } from "next/navigation";
import { api, Segment, StarredSegmentEffort } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { SegmentsMap } from "@/components/segments/segments-map";

type ViewMode = "list" | "map";

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

function formatClimbCategory(category: number | null): string | null {
  if (category === null) return null;
  if (category === 0) return "HC";
  return `Cat ${category}`;
}

function formatGrade(grade: number | null): string {
  if (grade === null) return "N/A";
  return `${grade.toFixed(1)}%`;
}

function formatTime(seconds: number | null): string {
  if (seconds === null) return "-";
  const mins = Math.floor(seconds / 60);
  const secs = Math.round(seconds % 60);
  if (mins >= 60) {
    const hours = Math.floor(mins / 60);
    const remainingMins = mins % 60;
    return `${hours}:${remainingMins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins}:${secs.toString().padStart(2, "0")}`;
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

type ClimbFilter = "all" | "hc" | "cat1" | "cat2" | "cat3" | "cat4" | "flat";

const CLIMB_FILTERS: { value: ClimbFilter; label: string; categories: (number | null)[] }[] = [
  { value: "all", label: "Any climb", categories: [] },
  { value: "hc", label: "HC", categories: [0] },
  { value: "cat1", label: "Cat 1", categories: [1] },
  { value: "cat2", label: "Cat 2", categories: [2] },
  { value: "cat3", label: "Cat 3", categories: [3] },
  { value: "cat4", label: "Cat 4", categories: [4] },
  { value: "flat", label: "Flat/NC", categories: [null] },
];

export default function SegmentsPage() {
  const router = useRouter();
  const [segments, setSegments] = useState<Segment[]>([]);
  const [starredEfforts, setStarredEfforts] = useState<StarredSegmentEffort[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showStarred, setShowStarred] = useState(false);
  const [showNearby, setShowNearby] = useState(false);
  const [userLocation, setUserLocation] = useState<{lat: number, lon: number} | null>(null);
  const [locationError, setLocationError] = useState("");
  const [locationLoading, setLocationLoading] = useState(false);
  const [nearbyRadius, setNearbyRadius] = useState<number>(5000);
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [activityTypeFilter, setActivityTypeFilter] = useState<string>("All");
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<SortOption>("newest");
  const [distanceFilter, setDistanceFilter] = useState<DistanceFilter>("all");
  const [climbFilter, setClimbFilter] = useState<ClimbFilter>("all");
  const [viewMode, setViewMode] = useState<ViewMode>("list");

  useEffect(() => {
    const hasToken = !!api.getToken();
    setIsLoggedIn(hasToken);
  }, []);

  useEffect(() => {
    setLoading(true);
    const typeFilter = activityTypeFilter === "All" ? undefined : activityTypeFilter;

    // Check token directly to avoid race condition with isLoggedIn state
    const hasToken = !!api.getToken();

    if (showStarred && hasToken) {
      // Fetch starred segments with effort data
      api.getStarredSegmentEfforts()
        .then((efforts) => {
          setStarredEfforts(efforts);
          setSegments([]);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    } else {
      setStarredEfforts([]);
      let fetchSegments: Promise<Segment[]>;
      if (showNearby && userLocation) {
        fetchSegments = api.getNearbySegments(userLocation.lat, userLocation.lon, nearbyRadius);
      } else {
        fetchSegments = api.listSegments(typeFilter);
      }
      fetchSegments
        .then(setSegments)
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [showStarred, showNearby, userLocation, nearbyRadius, isLoggedIn, activityTypeFilter]);

  const handleNearMeClick = () => {
    if (showNearby) {
      setShowNearby(false);
      setLocationError("");
      return;
    }

    setLocationLoading(true);
    setLocationError("");

    if (!navigator.geolocation) {
      setLocationError("Geolocation is not supported by your browser.");
      setLocationLoading(false);
      return;
    }

    navigator.geolocation.getCurrentPosition(
      (position) => {
        setUserLocation({
          lat: position.coords.latitude,
          lon: position.coords.longitude,
        });
        setShowNearby(true);
        setShowStarred(false);
        setLocationLoading(false);
      },
      (error) => {
        setLocationLoading(false);
        switch (error.code) {
          case error.PERMISSION_DENIED:
            setLocationError("Location access denied. Enable location to find nearby segments.");
            break;
          case error.POSITION_UNAVAILABLE:
            setLocationError("Location information is unavailable.");
            break;
          case error.TIMEOUT:
            setLocationError("Location request timed out.");
            break;
          default:
            setLocationError("An error occurred while getting your location.");
        }
      },
      { enableHighAccuracy: true, timeout: 10000, maximumAge: 60000 }
    );
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Segments</h1>
        {isLoggedIn && (
          <div className="flex gap-2">
            <Button
              variant={!showStarred && !showNearby ? "default" : "outline"}
              size="sm"
              onClick={() => { setShowStarred(false); setShowNearby(false); setLocationError(""); }}
            >
              All
            </Button>
            <Button
              variant={showStarred ? "default" : "outline"}
              size="sm"
              onClick={() => { setShowStarred(true); setShowNearby(false); setLocationError(""); }}
            >
              Starred
            </Button>
            <Button
              variant={showNearby ? "default" : "outline"}
              size="sm"
              onClick={handleNearMeClick}
              disabled={locationLoading}
            >
              {locationLoading ? "Locating..." : "Near Me"}
            </Button>
          </div>
        )}
      </div>

      {showNearby && userLocation && (
        <div className="flex items-center gap-4 p-3 bg-muted rounded-md">
          <span className="text-sm text-muted-foreground">
            Showing segments within:
          </span>
          <select
            value={nearbyRadius}
            onChange={(e) => setNearbyRadius(Number(e.target.value))}
            className="px-3 py-1 border rounded-md bg-background text-sm"
          >
            <option value={1000}>1 km</option>
            <option value={5000}>5 km</option>
            <option value={10000}>10 km</option>
            <option value={25000}>25 km</option>
          </select>
        </div>
      )}

      {locationError && (
        <div className="p-4 text-amber-800 bg-amber-100 dark:text-amber-200 dark:bg-amber-900/30 rounded-md">
          {locationError}
        </div>
      )}

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
        <select
          value={climbFilter}
          onChange={(e) => setClimbFilter(e.target.value as ClimbFilter)}
          className="px-3 py-2 border rounded-md bg-background text-sm"
        >
          {CLIMB_FILTERS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <div className="flex gap-1 ml-auto">
          <Button
            variant={viewMode === "list" ? "default" : "outline"}
            size="sm"
            onClick={() => setViewMode("list")}
          >
            List
          </Button>
          <Button
            variant={viewMode === "map" ? "default" : "outline"}
            size="sm"
            onClick={() => setViewMode("map")}
          >
            Map
          </Button>
        </div>
      </div>

      {!showStarred && !showNearby && (
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

      <SegmentsContent
        loading={loading}
        segments={segments}
        starredEfforts={starredEfforts}
        searchQuery={searchQuery}
        distanceFilter={distanceFilter}
        climbFilter={climbFilter}
        sortBy={sortBy}
        showStarred={showStarred}
        showNearby={showNearby}
        viewMode={viewMode}
        router={router}
      />
    </div>
  );
}

interface SegmentsContentProps {
  loading: boolean;
  segments: Segment[];
  starredEfforts: StarredSegmentEffort[];
  searchQuery: string;
  distanceFilter: DistanceFilter;
  climbFilter: ClimbFilter;
  sortBy: SortOption;
  showStarred: boolean;
  showNearby: boolean;
  viewMode: ViewMode;
  router: ReturnType<typeof useRouter>;
}

function SegmentsContent({
  loading,
  segments,
  starredEfforts,
  searchQuery,
  distanceFilter,
  climbFilter,
  sortBy,
  showStarred,
  showNearby,
  viewMode,
  router,
}: SegmentsContentProps) {
  // Filter and sort starred efforts
  const filteredStarredEfforts = useMemo(() => {
    if (!showStarred) return [];
    const distFilter = DISTANCE_FILTERS.find((f) => f.value === distanceFilter) || DISTANCE_FILTERS[0];
    return starredEfforts.filter((e) => {
      const matchesSearch = e.segment_name.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesDistance = e.distance_meters >= distFilter.min && e.distance_meters < distFilter.max;
      return matchesSearch && matchesDistance;
    });
  }, [starredEfforts, searchQuery, distanceFilter, showStarred]);

  const filteredSegments = useMemo(() => {
    if (showStarred) return [];
    const distFilter = DISTANCE_FILTERS.find((f) => f.value === distanceFilter) || DISTANCE_FILTERS[0];
    const climbFilterConfig = CLIMB_FILTERS.find((f) => f.value === climbFilter) || CLIMB_FILTERS[0];
    return sortSegments(
      segments.filter((s) => {
        const matchesSearch = s.name.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesDistance = s.distance_meters >= distFilter.min && s.distance_meters < distFilter.max;
        const matchesClimb = climbFilterConfig.categories.length === 0 ||
          climbFilterConfig.categories.includes(s.climb_category);
        return matchesSearch && matchesDistance && matchesClimb;
      }),
      sortBy
    );
  }, [segments, searchQuery, distanceFilter, climbFilter, sortBy, showStarred]);

  const hasFilters = searchQuery || distanceFilter !== "all" || climbFilter !== "all";
  const isEmpty = showStarred ? filteredStarredEfforts.length === 0 : filteredSegments.length === 0;

  if (loading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  if (isEmpty) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground mb-4">
            {hasFilters
              ? "No segments match your filters"
              : showNearby
              ? "No segments found nearby"
              : showStarred
              ? "No starred segments"
              : "No segments yet"}
          </p>
          <p className="text-sm text-muted-foreground">
            {hasFilters
              ? "Try adjusting your search or filters."
              : showNearby
              ? "Try increasing the search radius or check a different area."
              : showStarred
              ? "Star segments from their detail pages to see them here."
              : "Segments can be created from activity detail pages."}
          </p>
        </CardContent>
      </Card>
    );
  }

  // Starred segments with effort data
  if (showStarred) {
    // Map view requires Segment[] - convert starred efforts to segment-like objects
    if (viewMode === "map") {
      const mapSegments: Segment[] = filteredStarredEfforts.map((e) => ({
        id: e.segment_id,
        creator_id: "",
        name: e.segment_name,
        description: null,
        activity_type: e.activity_type,
        distance_meters: e.distance_meters,
        elevation_gain_meters: e.elevation_gain_meters,
        elevation_loss_meters: null,
        average_grade: null,
        max_grade: null,
        climb_category: null,
        visibility: "public" as const,
        created_at: "",
      }));
      return <SegmentsMap segments={mapSegments} />;
    }

    return (
      <div className="space-y-4">
        {filteredStarredEfforts.map((effort) => (
          <Card
            key={effort.segment_id}
            className="hover:bg-muted/50 cursor-pointer transition-colors"
            onClick={() => router.push(`/segments/${effort.segment_id}`)}
          >
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">{effort.segment_name}</CardTitle>
                <Badge variant="secondary">
                  {ACTIVITY_TYPE_LABELS[effort.activity_type] || effort.activity_type}
                </Badge>
              </div>
              <div className="flex flex-wrap gap-4 text-sm text-muted-foreground mt-2">
                <span>Distance: {formatDistance(effort.distance_meters)}</span>
                <span>Elevation: {formatElevation(effort.elevation_gain_meters)}</span>
              </div>
              {/* Effort Stats Section */}
              <div className="mt-4 pt-3 border-t">
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                  <div>
                    <p className="text-xs text-muted-foreground uppercase tracking-wide">Your Best</p>
                    <p className="text-sm font-medium">
                      {effort.best_time_seconds !== null ? (
                        <>
                          {formatTime(effort.best_time_seconds)}
                          {effort.best_effort_rank !== null && (
                            <span className="text-muted-foreground ml-1">(#{effort.best_effort_rank})</span>
                          )}
                        </>
                      ) : (
                        <span className="text-muted-foreground">No efforts</span>
                      )}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs text-muted-foreground uppercase tracking-wide">Your Efforts</p>
                    <p className="text-sm font-medium">{effort.user_effort_count}</p>
                  </div>
                  <div>
                    <p className="text-xs text-muted-foreground uppercase tracking-wide">Leader</p>
                    <p className="text-sm font-medium">
                      {effort.leader_time_seconds !== null ? (
                        formatTime(effort.leader_time_seconds)
                      ) : (
                        <span className="text-muted-foreground">-</span>
                      )}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs text-muted-foreground uppercase tracking-wide">Gap to Leader</p>
                    <p className="text-sm font-medium">
                      {effort.best_time_seconds !== null && effort.leader_time_seconds !== null ? (
                        effort.best_time_seconds === effort.leader_time_seconds ? (
                          <span className="text-green-600 dark:text-green-400">You lead!</span>
                        ) : (
                          `+${formatTime(effort.best_time_seconds - effort.leader_time_seconds)}`
                        )
                      ) : (
                        <span className="text-muted-foreground">-</span>
                      )}
                    </p>
                  </div>
                </div>
              </div>
            </CardHeader>
          </Card>
        ))}
      </div>
    );
  }

  if (viewMode === "map") {
    return <SegmentsMap segments={filteredSegments} />;
  }

  return (
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
              <div className="flex gap-2">
                {formatClimbCategory(segment.climb_category) && (
                  <Badge variant="outline">
                    {formatClimbCategory(segment.climb_category)}
                  </Badge>
                )}
                <Badge variant="secondary">
                  {ACTIVITY_TYPE_LABELS[segment.activity_type] || segment.activity_type}
                </Badge>
              </div>
            </div>
            <div className="flex flex-wrap gap-4 text-sm text-muted-foreground mt-2">
              <span>Distance: {formatDistance(segment.distance_meters)}</span>
              <span>Elevation: {formatElevation(segment.elevation_gain_meters)}</span>
              {segment.average_grade !== null && (
                <span>Grade: {formatGrade(segment.average_grade)}</span>
              )}
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
}
