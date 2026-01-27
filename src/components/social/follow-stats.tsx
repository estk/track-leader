"use client";

import Link from "next/link";

interface FollowStatsProps {
  userId: string;
  followerCount: number;
  followingCount: number;
}

export function FollowStats({
  userId,
  followerCount,
  followingCount,
}: FollowStatsProps) {
  return (
    <div className="flex gap-4 text-sm">
      <Link
        href={`/profile/${userId}/followers`}
        className="hover:underline"
      >
        <span className="font-semibold">{followerCount}</span>{" "}
        <span className="text-muted-foreground">followers</span>
      </Link>
      <Link
        href={`/profile/${userId}/following`}
        className="hover:underline"
      >
        <span className="font-semibold">{followingCount}</span>{" "}
        <span className="text-muted-foreground">following</span>
      </Link>
    </div>
  );
}
