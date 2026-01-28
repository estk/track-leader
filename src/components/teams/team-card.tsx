"use client";

import Link from "next/link";
import { TeamWithMembership, TeamRole } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface TeamCardProps {
  team: TeamWithMembership;
}

function getRoleBadgeClass(role: TeamRole): string {
  switch (role) {
    case "owner":
      return "bg-amber-500/15 text-amber-600 border-amber-500/20";
    case "admin":
      return "bg-blue-500/15 text-blue-600 border-blue-500/20";
    case "member":
    default:
      return "bg-secondary text-secondary-foreground";
  }
}

function getRoleLabel(role: TeamRole): string {
  switch (role) {
    case "owner":
      return "Owner";
    case "admin":
      return "Admin";
    case "member":
      return "Member";
    default:
      return role;
  }
}

function TeamAvatar({ name, avatarUrl }: { name: string; avatarUrl: string | null }) {
  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={name}
        className="w-12 h-12 rounded-lg object-cover"
      />
    );
  }

  // Generate initials from team name
  const initials = name
    .split(" ")
    .map((word) => word[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center text-primary-foreground font-bold text-lg">
      {initials}
    </div>
  );
}

export function TeamCard({ team }: TeamCardProps) {
  return (
    <Card className="hover:shadow-md transition-shadow group">
      <CardHeader className="pb-2">
        <div className="flex items-start gap-3">
          <TeamAvatar name={team.name} avatarUrl={team.avatar_url} />
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <CardTitle className="text-lg truncate">{team.name}</CardTitle>
              {team.user_role && (
                <Badge
                  variant="outline"
                  className={getRoleBadgeClass(team.user_role)}
                >
                  {getRoleLabel(team.user_role)}
                </Badge>
              )}
            </div>
            {team.description && (
              <p className="text-sm text-muted-foreground line-clamp-2 mt-1">
                {team.description}
              </p>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="pt-0">
        <div className="flex items-center justify-between">
          <div className="flex gap-4 text-sm text-muted-foreground">
            <span>{team.member_count} members</span>
            <span>{team.activity_count} activities</span>
            <span>{team.segment_count} segments</span>
          </div>
          <Link href={`/teams/${team.id}`}>
            <Button
              variant="ghost"
              size="sm"
              className="opacity-0 group-hover:opacity-100 transition-opacity"
            >
              View
            </Button>
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
