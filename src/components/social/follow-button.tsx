"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";

interface FollowButtonProps {
  userId: string;
  initialIsFollowing: boolean;
  onFollowChange?: (isFollowing: boolean) => void;
}

export function FollowButton({
  userId,
  initialIsFollowing,
  onFollowChange,
}: FollowButtonProps) {
  const [isFollowing, setIsFollowing] = useState(initialIsFollowing);
  const [loading, setLoading] = useState(false);

  const handleToggleFollow = async () => {
    setLoading(true);
    const previousState = isFollowing;

    try {
      if (isFollowing) {
        setIsFollowing(false);
        await api.unfollowUser(userId);
        onFollowChange?.(false);
      } else {
        setIsFollowing(true);
        await api.followUser(userId);
        onFollowChange?.(true);
      }
    } catch (error) {
      setIsFollowing(previousState);
      console.error("Failed to update follow status:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      variant={isFollowing ? "outline" : "default"}
      onClick={handleToggleFollow}
      disabled={loading}
      className="min-w-[100px]"
    >
      {loading ? "..." : isFollowing ? "Following" : "Follow"}
    </Button>
  );
}
