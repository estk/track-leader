"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, Activity, TrackData, TrackPoint, ActivitySegmentEffort, PreviewSegmentResponse, ActivityVisibility, ACTIVITY_TYPE_OPTIONS, getActivityTypeName, DigTimeSummary, DigSegment } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { LazyActivityMap } from "@/components/activity/lazy-activity-map";
import { LazyElevationProfile } from "@/components/activity/lazy-elevation-profile";
import { Textarea } from "@/components/ui/textarea";

interface ClimbCategoryInfo {
  label: string;
  tooltip: string;
}

function getClimbCategoryInfo(category: number | null): ClimbCategoryInfo | null {
  if (category === null) return null;
  switch (category) {
    case 0:
      return {
        label: "HC",
        tooltip: "Hors Categorie: The most difficult climbs, typically 800m+ elevation gain. Beyond normal categorization.",
      };
    case 1:
      return {
        label: "Cat 1",
        tooltip: "Category 1: Very difficult climbs, typically 640-800m gain over 10+ km at 7-9% gradient.",
      };
    case 2:
      return {
        label: "Cat 2",
        tooltip: "Category 2: Difficult climbs, typically 320-640m gain over 5-10 km at 6-9% gradient.",
      };
    case 3:
      return {
        label: "Cat 3",
        tooltip: "Category 3: Moderate climbs, typically 160-320m gain over 4-5 km at 6-8% gradient.",
      };
    case 4:
      return {
        label: "Cat 4",
        tooltip: "Category 4: Easy climbs, typically 80-160m gain over 1-3 km at 3-6% gradient.",
      };
    default:
      return null;
  }
}

// Convert climb category number to display string
function formatClimbCategory(category: number | null): string | null {
  const info = getClimbCategoryInfo(category);
  return info?.label ?? null;
}

export default function ActivityDetailPage() {
  const params = useParams();
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [activity, setActivity] = useState<Activity | null>(null);
  const [trackData, setTrackData] = useState<TrackData | null>(null);
  const [segmentEfforts, setSegmentEfforts] = useState<ActivitySegmentEffort[]>([]);
  const [digTimeSummary, setDigTimeSummary] = useState<DigTimeSummary | null>(null);
  const [digSegments, setDigSegments] = useState<DigSegment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [highlightIndex, setHighlightIndex] = useState<number | null>(null);
  const [highlightedSegment, setHighlightedSegment] = useState<{start: number, end: number} | null>(null);

  // Edit modal state
  const [editOpen, setEditOpen] = useState(false);
  const [editName, setEditName] = useState("");
  const [editType, setEditType] = useState("");
  const [editVisibility, setEditVisibility] = useState<ActivityVisibility>("public");
  const [saving, setSaving] = useState(false);

  // Delete modal state
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // Segment creation state
  const [segmentMode, setSegmentMode] = useState(false);
  const [segmentStart, setSegmentStart] = useState<number | null>(null);
  const [segmentEnd, setSegmentEnd] = useState<number | null>(null);
  const [segmentModalOpen, setSegmentModalOpen] = useState(false);
  const [segmentName, setSegmentName] = useState("");
  const [segmentDescription, setSegmentDescription] = useState("");
  const [creatingSegment, setCreatingSegment] = useState(false);
  const [segmentPreview, setSegmentPreview] = useState<PreviewSegmentResponse | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);

  const activityId = params.id as string;

  useEffect(() => {
    // Wait for auth to finish loading before making requests
    if (authLoading) return;

    if (activityId) {
      Promise.all([
        api.getActivity(activityId),
        api.getActivityTrack(activityId),
        api.getActivitySegments(activityId),
        api.getDigTime(activityId).catch(() => null),
        api.getDigSegments(activityId).catch(() => []),
      ])
        .then(([act, track, segments, digTime, digs]) => {
          setActivity(act);
          setTrackData(track);
          setSegmentEfforts(segments);
          setDigTimeSummary(digTime);
          setDigSegments(digs);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [authLoading, activityId]);

  // Fetch segment preview when selection changes
  useEffect(() => {
    if (!trackData || segmentStart === null || segmentEnd === null) {
      setSegmentPreview(null);
      return;
    }

    const startIdx = Math.min(segmentStart, segmentEnd);
    const endIdx = Math.max(segmentStart, segmentEnd);
    const selectedPoints = trackData.points.slice(startIdx, endIdx + 1);

    if (selectedPoints.length < 2) {
      setSegmentPreview(null);
      return;
    }

    setPreviewLoading(true);
    const points = selectedPoints.map(p => ({
      lat: p.lat,
      lon: p.lon,
      ele: p.ele ?? undefined,
    }));

    api.previewSegment(points)
      .then(setSegmentPreview)
      .catch(() => setSegmentPreview(null))
      .finally(() => setPreviewLoading(false));
  }, [trackData, segmentStart, segmentEnd]);

  const handleEdit = () => {
    if (activity) {
      setEditName(activity.name);
      setEditType(activity.activity_type_id);
      setEditVisibility(activity.visibility);
      setEditOpen(true);
    }
  };

  const handleSaveEdit = async () => {
    if (!activity) return;

    setSaving(true);
    try {
      const updated = await api.updateActivity(activity.id, {
        name: editName !== activity.name ? editName : undefined,
        activity_type_id: editType !== activity.activity_type_id ? editType : undefined,
        visibility: editVisibility !== activity.visibility ? editVisibility : undefined,
      });
      setActivity(updated);
      setEditOpen(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save");
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!activity) return;

    setDeleting(true);
    try {
      await api.deleteActivity(activity.id);
      router.push("/activities");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete");
      setDeleting(false);
    }
  };

  // Check if current user is the owner of this activity
  const isOwner = user && activity && user.id === activity.user_id;

  const handleSegmentPointClick = (index: number) => {
    if (!segmentMode) return;

    if (segmentStart === null) {
      setSegmentStart(index);
    } else if (segmentEnd === null) {
      // Ensure start < end
      if (index < segmentStart) {
        setSegmentEnd(segmentStart);
        setSegmentStart(index);
      } else {
        setSegmentEnd(index);
      }
    } else {
      // Reset and start over
      setSegmentStart(index);
      setSegmentEnd(null);
    }
  };

  const handleCreateSegment = async () => {
    if (!activity || !trackData || segmentStart === null || segmentEnd === null) return;

    const startIdx = Math.min(segmentStart, segmentEnd);
    const endIdx = Math.max(segmentStart, segmentEnd);
    const segmentPoints = trackData.points.slice(startIdx, endIdx + 1);

    if (segmentPoints.length < 2) {
      setError("Segment must have at least 2 points");
      return;
    }

    setCreatingSegment(true);
    try {
      const segment = await api.createSegment({
        name: segmentName || `${activity.name} segment`,
        description: segmentDescription || undefined,
        // activity_type is inherited from source_activity_id
        points: segmentPoints.map((p) => ({
          lat: p.lat,
          lon: p.lon,
          ele: p.ele ?? undefined,
        })),
        visibility: "public",
        source_activity_id: activity.id,
      });
      router.push(`/segments/${segment.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create segment");
      setCreatingSegment(false);
    }
  };

  const cancelSegmentMode = () => {
    setSegmentMode(false);
    setSegmentStart(null);
    setSegmentEnd(null);
    setSegmentName("");
    setSegmentDescription("");
  };

  if (authLoading || loading) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-10 w-64" />
        <Skeleton className="h-[400px] w-full" />
        <Skeleton className="h-[200px] w-full" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-destructive bg-destructive/10 rounded-md">
        {error}
      </div>
    );
  }

  if (!activity || !trackData) {
    return (
      <div className="p-4 text-muted-foreground">Activity not found</div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Edit Modal */}
      {editOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-md">
            <CardHeader>
              <CardTitle>Edit Activity</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="edit-name">Name</Label>
                <Input
                  id="edit-name"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="edit-type">Activity Type</Label>
                <select
                  id="edit-type"
                  value={editType}
                  onChange={(e) => setEditType(e.target.value)}
                  className="w-full h-10 px-3 py-2 border rounded-md bg-background"
                >
                  {ACTIVITY_TYPE_OPTIONS.map((type) => (
                    <option key={type.id} value={type.id}>
                      {type.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <Label>Visibility</Label>
                <div className="flex flex-wrap gap-4">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="edit-visibility"
                      value="public"
                      checked={editVisibility === "public"}
                      onChange={() => setEditVisibility("public")}
                      className="w-4 h-4"
                    />
                    <span>Public</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="edit-visibility"
                      value="private"
                      checked={editVisibility === "private"}
                      onChange={() => setEditVisibility("private")}
                      className="w-4 h-4"
                    />
                    <span>Private</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="edit-visibility"
                      value="teams_only"
                      checked={editVisibility === "teams_only"}
                      onChange={() => setEditVisibility("teams_only")}
                      className="w-4 h-4"
                    />
                    <span>Teams Only</span>
                  </label>
                </div>
                {editVisibility === "teams_only" && (
                  <p className="text-xs text-muted-foreground">
                    To change which teams can see this activity, use the activity sharing settings.
                  </p>
                )}
              </div>
              <div className="flex gap-2 pt-4">
                <Button
                  variant="outline"
                  className="flex-1"
                  onClick={() => setEditOpen(false)}
                  disabled={saving}
                >
                  Cancel
                </Button>
                <Button
                  className="flex-1"
                  onClick={handleSaveEdit}
                  disabled={saving}
                >
                  {saving ? "Saving..." : "Save"}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {deleteOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-md">
            <CardHeader>
              <CardTitle>Delete Activity</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p>
                Are you sure you want to delete <strong>{activity.name}</strong>?
                This action cannot be undone.
              </p>
              <div className="flex gap-2 pt-4">
                <Button
                  variant="outline"
                  className="flex-1"
                  onClick={() => setDeleteOpen(false)}
                  disabled={deleting}
                >
                  Cancel
                </Button>
                <Button
                  variant="destructive"
                  className="flex-1"
                  onClick={handleDelete}
                  disabled={deleting}
                >
                  {deleting ? "Deleting..." : "Delete"}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Segment Creation Modal */}
      {segmentModalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <Card className="w-full max-w-md">
            <CardHeader>
              <CardTitle>Create Segment</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="segment-name">Name</Label>
                <Input
                  id="segment-name"
                  placeholder="Segment name"
                  value={segmentName}
                  onChange={(e) => setSegmentName(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="segment-description">Description (optional)</Label>
                <Textarea
                  id="segment-description"
                  placeholder="Describe this segment..."
                  value={segmentDescription}
                  onChange={(e) => setSegmentDescription(e.target.value)}
                  rows={3}
                />
              </div>
              {segmentStart !== null && segmentEnd !== null && (
                previewLoading ? (
                  <div className="bg-muted/50 rounded-lg p-3">
                    <p className="text-sm text-muted-foreground">Calculating preview...</p>
                  </div>
                ) : segmentPreview && (
                  <div className="space-y-2">
                    <div className="bg-muted/50 rounded-lg p-3 space-y-2">
                      <p className="text-sm font-medium">Segment Preview</p>
                      <div className="grid grid-cols-2 gap-2 text-sm text-muted-foreground">
                        <div>
                          <span className="font-medium">Distance:</span>{" "}
                          {segmentPreview.distance_meters >= 1000
                            ? `${(segmentPreview.distance_meters / 1000).toFixed(2)} km`
                            : `${Math.round(segmentPreview.distance_meters)} m`}
                        </div>
                        <div>
                          <span className="font-medium">Elevation Gain:</span>{" "}
                          {segmentPreview.elevation_gain_meters !== null
                            ? `${Math.round(segmentPreview.elevation_gain_meters)} m`
                            : "N/A"}
                        </div>
                        {segmentPreview.average_grade !== null && (
                          <div>
                            <span className="font-medium">Grade:</span>{" "}
                            {segmentPreview.average_grade.toFixed(1)}%
                          </div>
                        )}
                        {getClimbCategoryInfo(segmentPreview.climb_category) && (
                          <div title={getClimbCategoryInfo(segmentPreview.climb_category)?.tooltip}>
                            <span className="font-medium">Category:</span>{" "}
                            {getClimbCategoryInfo(segmentPreview.climb_category)?.label}
                          </div>
                        )}
                      </div>
                    </div>
                    {!segmentPreview.validation.is_valid && (
                      <div className="bg-destructive/10 text-destructive rounded-lg p-3">
                        <p className="text-sm font-medium mb-1">Cannot create segment:</p>
                        <ul className="text-sm list-disc list-inside">
                          {segmentPreview.validation.errors.map((err, i) => (
                            <li key={i}>{err}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                )
              )}
              <div className="flex gap-2 pt-4">
                <Button
                  variant="outline"
                  className="flex-1"
                  onClick={() => setSegmentModalOpen(false)}
                  disabled={creatingSegment}
                >
                  Cancel
                </Button>
                <Button
                  className="flex-1"
                  onClick={handleCreateSegment}
                  disabled={
                    creatingSegment ||
                    !segmentName.trim() ||
                    previewLoading ||
                    !segmentPreview ||
                    !segmentPreview.validation.is_valid
                  }
                >
                  {creatingSegment ? "Creating..." : "Create Segment"}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Segment Mode Banner */}
      {segmentMode && (
        <Card className="bg-blue-50 dark:bg-blue-950 border-blue-200 dark:border-blue-800">
          <CardContent className="py-4">
            <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
              <div>
                <p className="font-medium text-blue-900 dark:text-blue-100">
                  Segment Creation Mode
                </p>
                <p className="text-sm text-blue-700 dark:text-blue-300">
                  {segmentStart === null
                    ? "Click on the elevation profile to select the start point"
                    : segmentEnd === null
                    ? "Click to select the end point"
                    : "Segment selected! Click 'Create' to continue or click again to reset"}
                </p>
              </div>
              <div className="flex gap-2">
                {segmentStart !== null && segmentEnd !== null && (
                  <Button
                    size="sm"
                    onClick={() => setSegmentModalOpen(true)}
                  >
                    Create
                  </Button>
                )}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={cancelSegmentMode}
                >
                  Cancel
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-2xl md:text-3xl font-bold">{activity.name}</h1>
          <div className="flex flex-wrap items-center gap-2 md:gap-4 mt-2">
            <Badge variant="secondary">{getActivityTypeName(activity.activity_type_id)}</Badge>
            <Badge variant={activity.visibility === "public" ? "default" : "outline"}>
              {activity.visibility === "public" ? "Public" : activity.visibility === "private" ? "Private" : "Teams Only"}
            </Badge>
            <span className="text-sm md:text-base text-muted-foreground">
              {new Date(activity.submitted_at).toLocaleDateString(undefined, {
                weekday: "long",
                year: "numeric",
                month: "long",
                day: "numeric",
              })}
            </span>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => router.push("/activities")}
          >
            Back
          </Button>
          {isOwner && (
            <Button variant="outline" size="sm" onClick={handleEdit}>
              Edit
            </Button>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() =>
              window.open(`/api/activities/${activityId}/download`, "_blank")
            }
          >
            Download
          </Button>
          {isOwner && (
            <>
              {!segmentMode ? (
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => setSegmentMode(true)}
                >
                  Create Segment
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={cancelSegmentMode}
                >
                  Cancel Segment
                </Button>
              )}
              <Button
                variant="destructive"
                size="sm"
                onClick={() => setDeleteOpen(true)}
              >
                Delete
              </Button>
            </>
          )}
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Route</CardTitle>
        </CardHeader>
        <CardContent>
          <LazyActivityMap
            trackData={trackData}
            highlightIndex={highlightIndex ?? undefined}
            selectionStart={segmentMode ? segmentStart : highlightedSegment?.start ?? null}
            selectionEnd={segmentMode ? segmentEnd : highlightedSegment?.end ?? null}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Elevation Profile</CardTitle>
        </CardHeader>
        <CardContent>
          <LazyElevationProfile
            points={trackData.points}
            onHover={setHighlightIndex}
            selectionMode={segmentMode}
            selectionStart={segmentStart}
            selectionEnd={segmentEnd}
            onPointClick={handleSegmentPointClick}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Statistics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatItem
              label="Points"
              value={trackData.points.length.toLocaleString()}
            />
            <StatItem
              label="Start Elevation"
              value={`${trackData.points[0]?.ele?.toFixed(0) ?? "N/A"}m`}
            />
            <StatItem
              label="End Elevation"
              value={`${trackData.points[trackData.points.length - 1]?.ele?.toFixed(0) ?? "N/A"}m`}
            />
            <StatItem
              label="Bounds"
              value={`${trackData.bounds.min_lat.toFixed(3)}°, ${trackData.bounds.min_lon.toFixed(3)}°`}
            />
          </div>
        </CardContent>
      </Card>

      {/* Dig Time */}
      {digTimeSummary && digTimeSummary.dig_segment_count > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
                className="w-5 h-5 text-amber-600"
              >
                <path
                  fillRule="evenodd"
                  d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25zM12.75 6a.75.75 0 00-1.5 0v6c0 .414.336.75.75.75h4.5a.75.75 0 000-1.5h-3.75V6z"
                  clipRule="evenodd"
                />
              </svg>
              Trail Maintenance
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4">
              <StatItem
                label="Dig Time"
                value={formatDigDuration(digTimeSummary.total_dig_time_seconds)}
              />
              <StatItem
                label="Dig Sessions"
                value={digTimeSummary.dig_segment_count.toString()}
              />
            </div>
            {digSegments.length > 0 && (
              <div className="mt-4 space-y-2">
                <p className="text-sm font-medium text-muted-foreground">Sessions</p>
                {digSegments.map((seg) => (
                  <div
                    key={seg.id}
                    className="flex items-center justify-between p-2 bg-amber-50 dark:bg-amber-950/30 rounded-md text-sm"
                  >
                    <span>
                      {new Date(seg.start_time).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                      {" - "}
                      {new Date(seg.end_time).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </span>
                    <span className="font-medium text-amber-700 dark:text-amber-400">
                      {formatDigDuration(seg.duration_seconds)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Matched Segments */}
      {segmentEfforts.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Segments ({segmentEfforts.length})</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {segmentEfforts.map((effort) => {
                const handleMouseEnter = () => {
                  if (effort.start_fraction !== null && effort.end_fraction !== null && trackData) {
                    const startIdx = Math.round(effort.start_fraction * (trackData.points.length - 1));
                    const endIdx = Math.round(effort.end_fraction * (trackData.points.length - 1));
                    setHighlightedSegment({ start: startIdx, end: endIdx });
                  }
                };
                const handleMouseLeave = () => {
                  setHighlightedSegment(null);
                };
                return (
                  <div
                    key={effort.effort_id}
                    className="flex items-center justify-between p-3 bg-muted/50 rounded-lg hover:bg-muted cursor-pointer"
                    onClick={() => router.push(`/segments/${effort.segment_id}`)}
                    onMouseEnter={handleMouseEnter}
                    onMouseLeave={handleMouseLeave}
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium truncate">{effort.segment_name}</span>
                        {effort.is_personal_record && (
                          <Badge variant="secondary" className="text-xs">PR</Badge>
                        )}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {(effort.segment_distance / 1000).toFixed(2)} km
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="font-mono font-medium">
                        {formatTime(effort.elapsed_time_seconds)}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        #{effort.rank}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="text-center p-4 bg-muted/50 rounded-lg">
      <p className="text-2xl font-bold">{value}</p>
      <p className="text-sm text-muted-foreground">{label}</p>
    </div>
  );
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  if (mins >= 60) {
    const hours = Math.floor(mins / 60);
    const remainingMins = mins % 60;
    return `${hours}:${remainingMins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function formatDigDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  if (minutes === 0) {
    return `${remainingSeconds}s`;
  }
  if (remainingSeconds === 0) {
    return `${minutes}m`;
  }
  return `${minutes}m ${remainingSeconds}s`;
}
