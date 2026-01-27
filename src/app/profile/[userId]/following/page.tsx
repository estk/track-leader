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

export default function FollowingPage() {
  const params = useParams();
  const router = useRouter();
  const userId = params.userId as string;
  const { user: currentUser } = useAuth();

  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [following, setFollowing] = useState<UserSummary[]>([]);
  const [followingStatus, setFollowingStatus] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);
  const [totalCount, setTotalCount] = useState(0);

  useEffect(() => {
    if (!userId) return;

    const loadData = async () => {
      try {
        const [profileData, followingData] = await Promise.all([
          api.getUserProfile(userId),
          api.getFollowing(userId),
        ]);
        setProfile(profileData);
        setFollowing(followingData.users);
        setTotalCount(followingData.total_count);

        // Check which users the current user is following
        if (currentUser) {
          const statusPromises = followingData.users.map(async (user) => {
            if (user.id === currentUser.id) return { id: user.id, following: false };
            const isFollowing = await api.getFollowStatus(user.id);
            return { id: user.id, following: isFollowing };
          });
          const statuses = await Promise.all(statusPromises);
          const statusMap: Record<string, boolean> = {};
          statuses.forEach(({ id, following: isFollowing }) => {
            statusMap[id] = isFollowing;
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
          {profile?.name} is Following
        </h1>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{totalCount} Following</CardTitle>
        </CardHeader>
        <CardContent>
          {following.length === 0 ? (
            <p className="text-muted-foreground text-center py-8">
              Not following anyone yet
            </p>
          ) : (
            <div className="space-y-4">
              {following.map((user) => (
                <div
                  key={user.id}
                  className="flex items-center justify-between p-3 rounded-lg hover:bg-muted/50"
                >
                  <Link
                    href={`/profile/${user.id}`}
                    className="flex items-center gap-3 flex-1"
                  >
                    <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-lg font-bold text-primary">
                      {user.name.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <p className="font-medium">{user.name}</p>
                      <p className="text-sm text-muted-foreground">
                        {user.follower_count} followers
                      </p>
                    </div>
                  </Link>
                  {currentUser && user.id !== currentUser.id && (
                    <FollowButton
                      userId={user.id}
                      initialIsFollowing={followingStatus[user.id] || false}
                      onFollowChange={(isFollowing) =>
                        handleFollowChange(user.id, isFollowing)
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
