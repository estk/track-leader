"use client";

import { useEffect, useState, useMemo } from "react";
import { useRouter } from "next/navigation";
import { api, Segment, StarredSegmentEffort, ListSegmentsOptions, SegmentSortBy, SortOrder, ClimbCategoryFilter, getActivityTypeName, ACTIVITY_TYPE_OPTIONS } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { LazySegmentsMap } from "@/components/segments/lazy-segments-map";

type ViewMode = "list" | "map";

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

const ACTIVITY_TYPE_FILTERS = [
  { id: "All", name: "All Types" },
  ...ACTIVITY_TYPE_OPTIONS,
];

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

type DistanceFilter = "all" | "under1k" | "1k-5k" | "5k-10k" | "10k-20k" | "over20k";

// Distance filter options with min/max values for API calls
const DISTANCE_FILTERS: { value: DistanceFilter; label: string; min?: number; max?: number }[] = [
  { value: "all", label: "Any distance" },
  { value: "under1k", label: "< 1 km", max: 1000 },
  { value: "1k-5k", label: "1-5 km", min: 1000, max: 5000 },
  { value: "5k-10k", label: "5-10 km", min: 5000, max: 10000 },
  { value: "10k-20k", label: "10-20 km", min: 10000, max: 20000 },
  { value: "over20k", label: "> 20 km", min: 20000 },
];

type ClimbFilter = "all" | "hc" | "cat1" | "cat2" | "cat3" | "cat4" | "flat";

const CLIMB_FILTERS: { value: ClimbFilter; label: string; apiValue?: ClimbCategoryFilter }[] = [
  { value: "all", label: "Any climb" },
  { value: "hc", label: "HC", apiValue: "hc" },
  { value: "cat1", label: "Cat 1", apiValue: "cat1" },
  { value: "cat2", label: "Cat 2", apiValue: "cat2" },
  { value: "cat3", label: "Cat 3", apiValue: "cat3" },
  { value: "cat4", label: "Cat 4", apiValue: "cat4" },
  { value: "flat", label: "Flat/NC", apiValue: "flat" },
];

// Helper to convert UI sort option to API parameters
function sortOptionToApi(sortBy: SortOption): { sortBy: SegmentSortBy; sortOrder: SortOrder } {
  switch (sortBy) {
    case "newest":
      return { sortBy: "created_at", sortOrder: "desc" };
    case "oldest":
      return { sortBy: "created_at", sortOrder: "asc" };
    case "name-asc":
      return { sortBy: "name", sortOrder: "asc" };
    case "name-desc":
      return { sortBy: "name", sortOrder: "desc" };
    case "distance-asc":
      return { sortBy: "distance", sortOrder: "asc" };
    case "distance-desc":
      return { sortBy: "distance", sortOrder: "desc" };
    case "elevation-desc":
      return { sortBy: "elevation_gain", sortOrder: "desc" };
    case "elevation-asc":
      return { sortBy: "elevation_gain", sortOrder: "asc" };
  }
}

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

    // Check token directly to avoid race condition with isLoggedIn state
    const hasToken = !!api.getToken();

    if (showStarred && hasToken) {
      // Fetch starred segments with effort data (client-side filtering still applies)
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
        // Nearby segments use a different endpoint without server-side filtering
        fetchSegments = api.getNearbySegments(userLocation.lat, userLocation.lon, nearbyRadius);
      } else {
        // Build API options with server-side filtering and sorting
        const distFilter = DISTANCE_FILTERS.find((f) => f.value === distanceFilter);
        const climbFilterConfig = CLIMB_FILTERS.find((f) => f.value === climbFilter);
        const { sortBy: apiSortBy, sortOrder: apiSortOrder } = sortOptionToApi(sortBy);

        const options: ListSegmentsOptions = {
          activityTypeId: activityTypeFilter === "All" ? undefined : activityTypeFilter,
          search: searchQuery || undefined,
          sortBy: apiSortBy,
          sortOrder: apiSortOrder,
          minDistanceMeters: distFilter?.min,
          maxDistanceMeters: distFilter?.max,
          climbCategory: climbFilterConfig?.apiValue,
        };
        fetchSegments = api.listSegments(options);
      }
      fetchSegments
        .then(setSegments)
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [showStarred, showNearby, userLocation, nearbyRadius, isLoggedIn, activityTypeFilter, searchQuery, sortBy, distanceFilter, climbFilter]);

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
          {ACTIVITY_TYPE_FILTERS.map((type) => (
            <Button
              key={type.id}
              variant={activityTypeFilter === type.id ? "default" : "outline"}
              size="sm"
              onClick={() => setActivityTypeFilter(type.id)}
            >
              {type.name}
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
  showStarred,
  showNearby,
  viewMode,
  router,
}: SegmentsContentProps) {
  // Filter starred efforts client-side (starred endpoint doesn't support server-side filtering)
  const filteredStarredEfforts = useMemo(() => {
    if (!showStarred) return [];
    const distFilter = DISTANCE_FILTERS.find((f) => f.value === distanceFilter);
    return starredEfforts.filter((e) => {
      const matchesSearch = searchQuery
        ? e.segment_name.toLowerCase().includes(searchQuery.toLowerCase())
        : true;
      const matchesMinDistance = distFilter?.min === undefined || e.distance_meters >= distFilter.min;
      const matchesMaxDistance = distFilter?.max === undefined || e.distance_meters < distFilter.max;
      return matchesSearch && matchesMinDistance && matchesMaxDistance;
    });
  }, [starredEfforts, searchQuery, distanceFilter, showStarred]);

  // Regular segments are now filtered server-side, so just use them directly
  const filteredSegments = showStarred ? [] : segments;

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
        activity_type_id: e.activity_type_id,
        distance_meters: e.distance_meters,
        elevation_gain_meters: e.elevation_gain_meters,
        elevation_loss_meters: null,
        average_grade: null,
        max_grade: null,
        climb_category: null,
        visibility: "public" as const,
        created_at: "",
      }));
      return <LazySegmentsMap segments={mapSegments} />;
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
                  {getActivityTypeName(effort.activity_type_id)}
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
    return <LazySegmentsMap segments={filteredSegments} />;
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
                  {getActivityTypeName(segment.activity_type_id)}
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
