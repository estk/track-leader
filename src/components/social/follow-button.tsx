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
    try {
      if (isFollowing) {
        await api.unfollowUser(userId);
        setIsFollowing(false);
        onFollowChange?.(false);
      } else {
        await api.followUser(userId);
        setIsFollowing(true);
        onFollowChange?.(true);
      }
    } catch {
      // Revert on error
      setIsFollowing(isFollowing);
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
