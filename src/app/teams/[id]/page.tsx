"use client";

import { useEffect, useState } from "react";
import { useRouter, useParams } from "next/navigation";
import Link from "next/link";
import {
  api,
  TeamWithMembership,
  TeamMember,
  FeedActivity,
  Segment,
} from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { RoleBadge } from "@/components/teams/role-badge";
import { FeedCard } from "@/components/feed/feed-card";
import { cn } from "@/lib/utils";

type TabType = "activities" | "segments" | "members";

function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(2)} km`;
  }
  return `${Math.round(meters)} m`;
}

function formatElevation(meters: number | null): string {
  if (meters === null) return "N/A";
  return `${Math.round(meters)} m`;
}

function TeamAvatar({
  name,
  avatarUrl,
  size = "lg",
}: {
  name: string;
  avatarUrl: string | null;
  size?: "md" | "lg";
}) {
  const sizeClasses = size === "lg" ? "w-16 h-16 text-xl" : "w-12 h-12 text-lg";

  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={name}
        className={cn("rounded-xl object-cover", sizeClasses)}
      />
    );
  }

  const initials = name
    .split(" ")
    .map((word) => word[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <div
      className={cn(
        "rounded-xl bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center text-primary-foreground font-bold",
        sizeClasses
      )}
    >
      {initials}
    </div>
  );
}

export default function TeamDetailPage() {
  const router = useRouter();
  const params = useParams();
  const teamId = params.id as string;
  const { user, loading: authLoading } = useAuth();

  const [team, setTeam] = useState<TeamWithMembership | null>(null);
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [activities, setActivities] = useState<FeedActivity[]>([]);
  const [segments, setSegments] = useState<Segment[]>([]);
  const [loading, setLoading] = useState(true);
  const [contentLoading, setContentLoading] = useState(false);
  const [error, setError] = useState("");
  const [activeTab, setActiveTab] = useState<TabType>("activities");

  // Fetch team data
  useEffect(() => {
    if (authLoading) return;
    if (!user) {
      router.push("/login");
      return;
    }

    setLoading(true);
    api
      .getTeam(teamId)
      .then(setTeam)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [teamId, user, authLoading, router]);

  // Fetch tab content
  useEffect(() => {
    if (!team?.is_member) return;

    setContentLoading(true);

    switch (activeTab) {
      case "activities":
        api
          .getTeamActivities(teamId)
          .then(setActivities)
          .catch(() => {})
          .finally(() => setContentLoading(false));
        break;
      case "segments":
        api
          .getTeamSegments(teamId)
          .then(setSegments)
          .catch(() => {})
          .finally(() => setContentLoading(false));
        break;
      case "members":
        api
          .listTeamMembers(teamId)
          .then(setMembers)
          .catch(() => {})
          .finally(() => setContentLoading(false));
        break;
    }
  }, [teamId, activeTab, team?.is_member]);

  if (authLoading || loading) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-64 w-full" />
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

  if (!team) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">Team not found</p>
      </div>
    );
  }

  const canManage = team.user_role === "owner" || team.user_role === "admin";

  return (
    <div className="space-y-6">
      {/* Team Header */}
      <Card>
        <CardContent className="p-6">
          <div className="flex items-start gap-4">
            <TeamAvatar name={team.name} avatarUrl={team.avatar_url} />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <h1 className="text-2xl font-bold">{team.name}</h1>
                {team.user_role && <RoleBadge role={team.user_role} />}
                {team.visibility === "private" && (
                  <Badge variant="outline">Private</Badge>
                )}
              </div>
              {team.description && (
                <p className="text-muted-foreground mt-1">{team.description}</p>
              )}
              <div className="flex gap-4 mt-3 text-sm text-muted-foreground">
                <span>{team.member_count} members</span>
                <span>{team.activity_count} activities</span>
                <span>{team.segment_count} segments</span>
              </div>
            </div>
            <div className="flex gap-2">
              {canManage && (
                <>
                  <Link href={`/teams/${teamId}/invite`}>
                    <Button variant="outline" size="sm">
                      Invite
                    </Button>
                  </Link>
                  <Link href={`/teams/${teamId}/settings`}>
                    <Button variant="outline" size="sm">
                      Settings
                    </Button>
                  </Link>
                </>
              )}
              {!team.is_member && team.join_policy !== "invitation" && (
                <JoinButton
                  teamId={teamId}
                  joinPolicy={team.join_policy}
                  onJoined={() => window.location.reload()}
                />
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Content - only show if member */}
      {team.is_member ? (
        <>
          {/* Tab Navigation */}
          <div className="flex border-b">
            <TabButton
              active={activeTab === "activities"}
              onClick={() => setActiveTab("activities")}
              count={team.activity_count}
            >
              Activities
            </TabButton>
            <TabButton
              active={activeTab === "segments"}
              onClick={() => setActiveTab("segments")}
              count={team.segment_count}
            >
              Segments
            </TabButton>
            <TabButton
              active={activeTab === "members"}
              onClick={() => setActiveTab("members")}
              count={team.member_count}
            >
              Members
            </TabButton>
          </div>

          {/* Tab Content */}
          {contentLoading ? (
            <div className="space-y-4">
              <Skeleton className="h-32 w-full" />
              <Skeleton className="h-32 w-full" />
            </div>
          ) : (
            <>
              {activeTab === "activities" && (
                <ActivitiesTab activities={activities} />
              )}
              {activeTab === "segments" && (
                <SegmentsTab segments={segments} router={router} />
              )}
              {activeTab === "members" && (
                <MembersTab
                  members={members}
                  currentUserId={user?.id}
                  canManage={canManage}
                  teamId={teamId}
                />
              )}
            </>
          )}
        </>
      ) : (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground">
              Join this team to see its activities, segments, and members.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  count,
  children,
}: {
  active: boolean;
  onClick: () => void;
  count: number;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "px-4 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2",
        active
          ? "border-primary text-primary"
          : "border-transparent text-muted-foreground hover:text-foreground"
      )}
    >
      {children}
      <Badge variant="secondary" className="text-xs">
        {count}
      </Badge>
    </button>
  );
}

function JoinButton({
  teamId,
  joinPolicy,
  onJoined,
}: {
  teamId: string;
  joinPolicy: string;
  onJoined: () => void;
}) {
  const [joining, setJoining] = useState(false);
  const [error, setError] = useState("");

  const handleJoin = async () => {
    setJoining(true);
    setError("");
    try {
      await api.joinTeam(teamId);
      onJoined();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to join");
      setJoining(false);
    }
  };

  return (
    <div>
      <Button onClick={handleJoin} disabled={joining}>
        {joining
          ? "Joining..."
          : joinPolicy === "request"
          ? "Request to Join"
          : "Join Team"}
      </Button>
      {error && <p className="text-destructive text-xs mt-1">{error}</p>}
    </div>
  );
}

function ActivitiesTab({ activities }: { activities: FeedActivity[] }) {
  if (activities.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">
            No activities shared with this team yet.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {activities.map((activity) => (
        <FeedCard key={activity.id} activity={activity} />
      ))}
    </div>
  );
}

function SegmentsTab({
  segments,
  router,
}: {
  segments: Segment[];
  router: ReturnType<typeof useRouter>;
}) {
  if (segments.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">
            No segments shared with this team yet.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {segments.map((segment) => (
        <Card
          key={segment.id}
          className="hover:bg-muted/50 cursor-pointer transition-colors"
          onClick={() => router.push(`/segments/${segment.id}`)}
        >
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg">{segment.name}</CardTitle>
              <Badge variant="secondary">{segment.activity_type}</Badge>
            </div>
            <div className="flex flex-wrap gap-4 text-sm text-muted-foreground mt-2">
              <span>Distance: {formatDistance(segment.distance_meters)}</span>
              <span>
                Elevation: {formatElevation(segment.elevation_gain_meters)}
              </span>
            </div>
          </CardHeader>
        </Card>
      ))}
    </div>
  );
}

function MembersTab({
  members,
  currentUserId,
  canManage,
  teamId,
}: {
  members: TeamMember[];
  currentUserId?: string;
  canManage: boolean;
  teamId: string;
}) {
  const router = useRouter();

  if (members.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">No members found.</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-2">
      {members.map((member) => (
        <Card key={member.user_id}>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Link href={`/profile/${member.user_id}`}>
                  <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-lg font-bold text-primary">
                    {member.user_name.charAt(0).toUpperCase()}
                  </div>
                </Link>
                <div>
                  <Link
                    href={`/profile/${member.user_id}`}
                    className="font-medium hover:underline"
                  >
                    {member.user_name}
                  </Link>
                  <div className="flex items-center gap-2">
                    <RoleBadge role={member.role} />
                    <span className="text-xs text-muted-foreground">
                      Joined {new Date(member.joined_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
              {canManage &&
                member.user_id !== currentUserId &&
                member.role !== "owner" && (
                  <MemberActions
                    teamId={teamId}
                    member={member}
                    onUpdate={() => router.refresh()}
                  />
                )}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function MemberActions({
  teamId,
  member,
  onUpdate,
}: {
  teamId: string;
  member: TeamMember;
  onUpdate: () => void;
}) {
  const [removing, setRemoving] = useState(false);

  const handleRemove = async () => {
    if (!confirm(`Remove ${member.user_name} from this team?`)) return;

    setRemoving(true);
    try {
      await api.removeTeamMember(teamId, member.user_id);
      window.location.reload();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to remove member");
      setRemoving(false);
    }
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={handleRemove}
      disabled={removing}
      className="text-destructive hover:text-destructive"
    >
      {removing ? "Removing..." : "Remove"}
    </Button>
  );
}
