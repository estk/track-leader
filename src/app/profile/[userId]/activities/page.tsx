"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { api, Activity, UserProfile, getActivityTypeName } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

export default function UserActivitiesPage() {
  const params = useParams();
  const router = useRouter();
  const userId = params.userId as string;

  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!userId) return;

    const loadData = async () => {
      try {
        const [profileData, activitiesData] = await Promise.all([
          api.getUserProfile(userId),
          api.getUserActivities(userId),
        ]);
        setProfile(profileData);
        // Only show public activities
        setActivities(activitiesData.filter((a) => a.visibility === "public"));
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to load activities",
        );
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [userId]);

  if (loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-6">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm" onClick={() => router.back()}>
          &larr;
        </Button>
        <h1 className="text-2xl font-bold">
          {profile?.name}&apos;s Activities
        </h1>
      </div>

      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {activities.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground">No public activities</p>
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
                  <CardTitle className="text-lg">{activity.name}</CardTitle>
                  <Badge variant="secondary">
                    {getActivityTypeName(activity.activity_type_id)}
                  </Badge>
                </div>
                <p className="text-sm text-muted-foreground">
                  {new Date(activity.submitted_at).toLocaleDateString(
                    undefined,
                    {
                      weekday: "long",
                      year: "numeric",
                      month: "long",
                      day: "numeric",
                    },
                  )}
                </p>
              </CardHeader>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
