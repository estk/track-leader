"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { api, Segment } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
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

export default function SegmentsPage() {
  const router = useRouter();
  const [segments, setSegments] = useState<Segment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showStarred, setShowStarred] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [activityTypeFilter, setActivityTypeFilter] = useState<string>("All");

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
      ) : segments.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground mb-4">
              {showStarred ? "No starred segments" : "No segments yet"}
            </p>
            <p className="text-sm text-muted-foreground">
              {showStarred
                ? "Star segments from their detail pages to see them here."
                : "Segments can be created from activity detail pages."}
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {segments.map((segment) => (
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
      )}
    </div>
  );
}
