"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { api, UserSummary, UserProfile } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { FollowButton } from "@/components/social/follow-button";
import Link from "next/link";

export default function FollowersPage() {
  const params = useParams();
  const router = useRouter();
  const userId = params.userId as string;
  const { user: currentUser } = useAuth();

  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [followers, setFollowers] = useState<UserSummary[]>([]);
  const [followingStatus, setFollowingStatus] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);
  const [totalCount, setTotalCount] = useState(0);

  useEffect(() => {
    if (!userId) return;

    const loadData = async () => {
      try {
        const [profileData, followersData] = await Promise.all([
          api.getUserProfile(userId),
          api.getFollowers(userId),
        ]);
        setProfile(profileData);
        setFollowers(followersData.users);
        setTotalCount(followersData.total_count);

        // Check which followers the current user is following
        if (currentUser) {
          const statusPromises = followersData.users.map(async (follower) => {
            if (follower.id === currentUser.id) return { id: follower.id, following: false };
            const following = await api.getFollowStatus(follower.id);
            return { id: follower.id, following };
          });
          const statuses = await Promise.all(statusPromises);
          const statusMap: Record<string, boolean> = {};
          statuses.forEach(({ id, following }) => {
            statusMap[id] = following;
          });
          setFollowingStatus(statusMap);
        }
      } catch {
        // Error loading
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [userId, currentUser]);

  const handleFollowChange = (targetUserId: string, isFollowing: boolean) => {
    setFollowingStatus((prev) => ({ ...prev, [targetUserId]: isFollowing }));
  };

  if (loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-6">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-48 w-full" />
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
          {profile?.name}&apos;s Followers
        </h1>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{totalCount} Followers</CardTitle>
        </CardHeader>
        <CardContent>
          {followers.length === 0 ? (
            <p className="text-muted-foreground text-center py-8">
              No followers yet
            </p>
          ) : (
            <div className="space-y-4">
              {followers.map((follower) => (
                <div
                  key={follower.id}
                  className="flex items-center justify-between p-3 rounded-lg hover:bg-muted/50"
                >
                  <Link
                    href={`/profile/${follower.id}`}
                    className="flex items-center gap-3 flex-1"
                  >
                    <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-lg font-bold text-primary">
                      {follower.name.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <p className="font-medium">{follower.name}</p>
                      <p className="text-sm text-muted-foreground">
                        {follower.follower_count} followers
                      </p>
                    </div>
                  </Link>
                  {currentUser && follower.id !== currentUser.id && (
                    <FollowButton
                      userId={follower.id}
                      initialIsFollowing={followingStatus[follower.id] || false}
                      onFollowChange={(isFollowing) =>
                        handleFollowChange(follower.id, isFollowing)
                      }
                    />
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
