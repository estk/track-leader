"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, Activity, TrackData } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { ActivityMap } from "@/components/activity/activity-map";
import { ElevationProfile } from "@/components/activity/elevation-profile";

const ACTIVITY_TYPES = [
  { value: "Running", label: "Run" },
  { value: "RoadCycling", label: "Road Cycling" },
  { value: "MountainBiking", label: "Mountain Biking" },
  { value: "Hiking", label: "Hike" },
  { value: "Walking", label: "Walk" },
  { value: "Unknown", label: "Other" },
];

export default function ActivityDetailPage() {
  const params = useParams();
  const router = useRouter();
  const { user, loading: authLoading } = useAuth();
  const [activity, setActivity] = useState<Activity | null>(null);
  const [trackData, setTrackData] = useState<TrackData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [highlightIndex, setHighlightIndex] = useState<number | null>(null);

  // Edit modal state
  const [editOpen, setEditOpen] = useState(false);
  const [editName, setEditName] = useState("");
  const [editType, setEditType] = useState("");
  const [saving, setSaving] = useState(false);

  // Delete modal state
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const activityId = params.id as string;

  useEffect(() => {
    if (!authLoading && !user) {
      router.push("/login");
      return;
    }

    if (user && activityId) {
      Promise.all([
        api.getActivity(activityId),
        api.getActivityTrack(activityId),
      ])
        .then(([act, track]) => {
          setActivity(act);
          setTrackData(track);
        })
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [user, authLoading, activityId, router]);

  const handleEdit = () => {
    if (activity) {
      setEditName(activity.name);
      setEditType(activity.activity_type);
      setEditOpen(true);
    }
  };

  const handleSaveEdit = async () => {
    if (!activity) return;

    setSaving(true);
    try {
      const updated = await api.updateActivity(activity.id, {
        name: editName !== activity.name ? editName : undefined,
        activity_type: editType !== activity.activity_type ? editType : undefined,
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
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
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
                  {ACTIVITY_TYPES.map((type) => (
                    <option key={type.value} value={type.value}>
                      {type.label}
                    </option>
                  ))}
                </select>
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
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
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

      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">{activity.name}</h1>
          <div className="flex items-center gap-4 mt-2">
            <Badge variant="secondary">{activity.activity_type}</Badge>
            <span className="text-muted-foreground">
              {new Date(activity.submitted_at).toLocaleDateString(undefined, {
                weekday: "long",
                year: "numeric",
                month: "long",
                day: "numeric",
              })}
            </span>
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => router.push("/activities")}
          >
            Back
          </Button>
          <Button variant="outline" onClick={handleEdit}>
            Edit
          </Button>
          <Button
            variant="outline"
            onClick={() =>
              window.open(`/api/activities/${activityId}/download`, "_blank")
            }
          >
            Download
          </Button>
          <Button
            variant="destructive"
            onClick={() => setDeleteOpen(true)}
          >
            Delete
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Route</CardTitle>
        </CardHeader>
        <CardContent>
          <ActivityMap
            trackData={trackData}
            highlightIndex={highlightIndex ?? undefined}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Elevation Profile</CardTitle>
        </CardHeader>
        <CardContent>
          <ElevationProfile
            points={trackData.points}
            onHover={setHighlightIndex}
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
