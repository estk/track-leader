"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, UserProfile, Activity } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { FollowButton } from "@/components/social/follow-button";
import { FollowStats } from "@/components/social/follow-stats";

export default function UserProfilePage() {
  const params = useParams();
  const router = useRouter();
  const userId = params.userId as string;
  const { user: currentUser } = useAuth();

  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [isFollowing, setIsFollowing] = useState(false);
  const [followStatusLoaded, setFollowStatusLoaded] = useState(false);
  const [loading, setLoading] = useState(true);

  const isOwnProfile = currentUser?.id === userId;

  useEffect(() => {
    if (!userId) return;

    // Redirect to own profile page if viewing self
    if (isOwnProfile) {
      router.replace("/profile");
      return;
    }

    const loadProfile = async () => {
      try {
        // Fetch profile first - this is the critical data
        const profileData = await api.getUserProfile(userId);
        setProfile(profileData);

        // Fetch activities separately - don't fail the whole page if this fails
        try {
          const activitiesData = await api.getUserActivities(userId);
          setActivities(activitiesData.filter((a) => a.visibility === "public"));
        } catch {
          // Activities failed to load - show empty list
          setActivities([]);
        }

        // Check if current user is following this user
        if (currentUser) {
          try {
            const following = await api.getFollowStatus(userId);
            setIsFollowing(following);
          } catch {
            // Follow status failed - default to not following
          }
          setFollowStatusLoaded(true);
        }
      } catch {
        // User not found - profile is null
      } finally {
        setLoading(false);
      }
    };

    loadProfile();
  }, [userId, currentUser, isOwnProfile, router]);

  const handleFollowChange = (newIsFollowing: boolean) => {
    setIsFollowing(newIsFollowing);
    if (profile) {
      setProfile({
        ...profile,
        follower_count: profile.follower_count + (newIsFollowing ? 1 : -1),
      });
    }
  };

  if (loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-6">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-48 w-full" />
      </div>
    );
  }

  if (!profile) {
    return (
      <div className="max-w-2xl mx-auto text-center py-12">
        <h1 className="text-2xl font-bold">User Not Found</h1>
        <p className="text-muted-foreground mt-2">
          The user you&apos;re looking for doesn&apos;t exist.
        </p>
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Profile</CardTitle>
            {currentUser && !isOwnProfile && followStatusLoaded && (
              <FollowButton
                userId={userId}
                initialIsFollowing={isFollowing}
                onFollowChange={handleFollowChange}
              />
            )}
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center text-2xl font-bold text-primary">
              {profile.name.charAt(0).toUpperCase()}
            </div>
            <div>
              <h2 className="text-xl font-semibold">{profile.name}</h2>
              <FollowStats
                userId={userId}
                followerCount={profile.follower_count}
                followingCount={profile.following_count}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Public Activities</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4 text-center mb-4">
            <div className="p-4 bg-muted/50 rounded-lg">
              <p className="text-3xl font-bold">{activities.length}</p>
              <p className="text-sm text-muted-foreground">Public Activities</p>
            </div>
            <div className="p-4 bg-muted/50 rounded-lg">
              <p className="text-3xl font-bold">
                {Math.round(
                  activities.reduce((sum) => sum, 0) / 1000
                )}
              </p>
              <p className="text-sm text-muted-foreground">Kilometers</p>
            </div>
          </div>
          {activities.length > 0 && (
            <Button
              variant="outline"
              className="w-full"
              onClick={() => router.push(`/profile/${userId}/activities`)}
            >
              View All Activities
            </Button>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Achievements</CardTitle>
        </CardHeader>
        <CardContent>
          <Button
            variant="outline"
            className="w-full"
            onClick={() => router.push(`/profile/${userId}/achievements`)}
          >
            View Achievements
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
