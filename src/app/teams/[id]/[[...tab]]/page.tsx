"use client";

import { useEffect, useState, useCallback } from "react";
import { useRouter, useParams } from "next/navigation";
import Link from "next/link";
import {
  api,
  TeamWithMembership,
  TeamMember,
  FeedActivity,
  Segment,
  getActivityTypeName,
  LeaderboardType,
  CrownCountEntry,
  DistanceLeaderEntry,
  DigTimeLeaderEntry,
  DigPercentageLeaderEntry,
  AverageSpeedLeaderEntry,
} from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RoleBadge } from "@/components/teams/role-badge";
import { FeedCard } from "@/components/feed/feed-card";
import { LazyDailyActivitiesMap } from "@/components/activity/lazy-daily-activities-map";
import { LazyTeamHeatmap } from "@/components/maps/lazy-team-heatmap";
import { LazyDigHeatmap } from "@/components/maps/lazy-dig-heatmap";
import { RankBadge } from "@/components/leaderboard/crown-badge";
import { cn } from "@/lib/utils";
import { Crown, MapPin, Shovel, Percent, Gauge } from "lucide-react";

type TabType = "daily-map" | "heat-map" | "dig-heatmap" | "activities" | "segments" | "members" | "leaderboard";

const VALID_TABS: TabType[] = ["daily-map", "heat-map", "dig-heatmap", "activities", "segments", "members", "leaderboard"];

function parseTab(tabParam: string[] | undefined): TabType {
  if (!tabParam || tabParam.length === 0) return "daily-map";
  const tab = tabParam[0];
  if (VALID_TABS.includes(tab as TabType)) return tab as TabType;
  return "daily-map";
}

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
  const activeTab = parseTab(params.tab as string[] | undefined);
  const { user, loading: authLoading } = useAuth();

  const [team, setTeam] = useState<TeamWithMembership | null>(null);
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [activities, setActivities] = useState<FeedActivity[]>([]);
  const [segments, setSegments] = useState<Segment[]>([]);
  const [loading, setLoading] = useState(true);
  const [contentLoading, setContentLoading] = useState(false);
  const [error, setError] = useState("");

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

    // These tabs handle their own loading
    if (activeTab === "daily-map" || activeTab === "heat-map" || activeTab === "dig-heatmap" || activeTab === "leaderboard") {
      setContentLoading(false);
      return;
    }

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

      {/* Featured Leaderboard - show if configured and user is member */}
      {team.is_member && team.featured_leaderboard && (
        <FeaturedLeaderboard
          teamId={teamId}
          leaderboardType={team.featured_leaderboard}
          currentUserId={user?.id}
        />
      )}

      {/* Content - only show if member */}
      {team.is_member ? (
        <>
          {/* Tab Navigation */}
          <div className="flex border-b overflow-x-auto">
            <TabLink
              href={`/teams/${teamId}`}
              active={activeTab === "daily-map"}
            >
              Daily Map
            </TabLink>
            <TabLink
              href={`/teams/${teamId}/heat-map`}
              active={activeTab === "heat-map"}
            >
              Heat Map
            </TabLink>
            <TabLink
              href={`/teams/${teamId}/dig-heatmap`}
              active={activeTab === "dig-heatmap"}
            >
              <Shovel className="h-4 w-4" />
              Dig Map
            </TabLink>
            <TabLink
              href={`/teams/${teamId}/activities`}
              active={activeTab === "activities"}
              count={team.activity_count}
            >
              Activities
            </TabLink>
            <TabLink
              href={`/teams/${teamId}/segments`}
              active={activeTab === "segments"}
              count={team.segment_count}
            >
              Segments
            </TabLink>
            <TabLink
              href={`/teams/${teamId}/members`}
              active={activeTab === "members"}
              count={team.member_count}
            >
              Members
            </TabLink>
            <TabLink
              href={`/teams/${teamId}/leaderboard`}
              active={activeTab === "leaderboard"}
            >
              Leaderboard
            </TabLink>
          </div>

          {/* Tab Content */}
          {contentLoading ? (
            <div className="space-y-4">
              <Skeleton className="h-32 w-full" />
              <Skeleton className="h-32 w-full" />
            </div>
          ) : (
            <>
              {activeTab === "daily-map" && <DailyMapTab teamId={teamId} />}
              {activeTab === "heat-map" && <HeatMapTab teamId={teamId} />}
              {activeTab === "dig-heatmap" && <DigHeatMapTab teamId={teamId} />}
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
              {activeTab === "leaderboard" && (
                <LeaderboardTab teamId={teamId} currentUserId={user?.id} />
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

function TabLink({
  href,
  active,
  count,
  children,
}: {
  href: string;
  active: boolean;
  count?: number;
  children: React.ReactNode;
}) {
  return (
    <Link
      href={href}
      className={cn(
        "px-4 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 whitespace-nowrap",
        active
          ? "border-primary text-primary"
          : "border-transparent text-muted-foreground hover:text-foreground"
      )}
    >
      {children}
      {count !== undefined && (
        <Badge variant="secondary" className="text-xs">
          {count}
        </Badge>
      )}
    </Link>
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

function getTodayDateString(): string {
  const today = new Date();
  return today.toISOString().split("T")[0];
}

function DailyMapTab({ teamId }: { teamId: string }) {
  const [dailyDate, setDailyDate] = useState(getTodayDateString());
  const [dailyActivities, setDailyActivities] = useState<FeedActivity[]>([]);
  const [dailyLoading, setDailyLoading] = useState(true);

  // Load daily activities for the map
  const loadDailyActivities = useCallback(async () => {
    setDailyLoading(true);
    try {
      const data = await api.getTeamActivitiesByDate(teamId, dailyDate);
      setDailyActivities(data);
    } catch {
      setDailyActivities([]);
    } finally {
      setDailyLoading(false);
    }
  }, [teamId, dailyDate]);

  useEffect(() => {
    loadDailyActivities();
  }, [loadDailyActivities]);

  const handlePreviousDay = () => {
    const currentDate = new Date(dailyDate);
    currentDate.setDate(currentDate.getDate() - 1);
    setDailyDate(currentDate.toISOString().split("T")[0]);
  };

  const handleNextDay = () => {
    const currentDate = new Date(dailyDate);
    currentDate.setDate(currentDate.getDate() + 1);
    setDailyDate(currentDate.toISOString().split("T")[0]);
  };

  const handleToday = () => {
    setDailyDate(getTodayDateString());
  };

  const isToday = dailyDate === getTodayDateString();

  // Format the date for display
  const displayDate = new Date(dailyDate + "T00:00:00").toLocaleDateString(
    undefined,
    {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    }
  );

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Daily Team Map</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Date picker controls */}
        <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:gap-4">
          <div className="space-y-1">
            <Label htmlFor="daily-date-picker">Date</Label>
            <Input
              id="daily-date-picker"
              type="date"
              value={dailyDate}
              onChange={(e) => setDailyDate(e.target.value)}
              className="w-full sm:w-auto"
            />
          </div>

          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handlePreviousDay}
              title="Previous day"
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleNextDay}
              title="Next day"
            >
              Next
            </Button>
            {!isToday && (
              <Button variant="outline" size="sm" onClick={handleToday}>
                Today
              </Button>
            )}
          </div>
        </div>

        <p className="text-sm text-muted-foreground">{displayDate}</p>

        {/* Map */}
        {dailyLoading ? (
          <Skeleton className="h-[750px] w-full rounded-lg" />
        ) : dailyActivities.length === 0 ? (
          <div className="p-8 text-center text-muted-foreground border rounded-lg">
            No team activities on this date.
          </div>
        ) : (
          <div>
            <LazyDailyActivitiesMap activities={dailyActivities} />
            <p className="text-sm text-muted-foreground mt-2">
              {dailyActivities.length}{" "}
              {dailyActivities.length === 1 ? "activity" : "activities"} on this
              date
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function HeatMapTab({ teamId }: { teamId: string }) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Team Heat Map</CardTitle>
      </CardHeader>
      <CardContent>
        <LazyTeamHeatmap teamId={teamId} />
      </CardContent>
    </Card>
  );
}

function DigHeatMapTab({ teamId }: { teamId: string }) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center gap-2">
          <Shovel className="h-5 w-5" />
          Team Dig Heat Map
        </CardTitle>
      </CardHeader>
      <CardContent>
        <LazyDigHeatmap teamId={teamId} />
      </CardContent>
    </Card>
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
              <Badge variant="secondary">{getActivityTypeName(segment.activity_type_id)}</Badge>
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

type TeamLeaderboardType = "crowns" | "distance" | "dig_time" | "dig_percentage" | "average_speed";

const LEADERBOARD_OPTIONS: { value: TeamLeaderboardType; label: string; icon: React.ReactNode }[] = [
  { value: "crowns", label: "Crowns", icon: <Crown className="h-4 w-4" /> },
  { value: "distance", label: "Distance", icon: <MapPin className="h-4 w-4" /> },
  { value: "dig_time", label: "Dig Time", icon: <Shovel className="h-4 w-4" /> },
  { value: "dig_percentage", label: "Dig %", icon: <Percent className="h-4 w-4" /> },
  { value: "average_speed", label: "Avg Speed", icon: <Gauge className="h-4 w-4" /> },
];

type TeamLeaderboardEntry =
  | CrownCountEntry
  | DistanceLeaderEntry
  | DigTimeLeaderEntry
  | DigPercentageLeaderEntry
  | AverageSpeedLeaderEntry;

function LeaderboardTab({
  teamId,
  currentUserId,
}: {
  teamId: string;
  currentUserId?: string;
}) {
  const [selectedType, setSelectedType] = useState<TeamLeaderboardType>("crowns");
  const [entries, setEntries] = useState<TeamLeaderboardEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    setLoading(true);
    setError("");
    api
      .getTeamLeaderboard(teamId, selectedType)
      .then(setEntries)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [teamId, selectedType]);

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-lg">Team Leaderboard</CardTitle>
        <p className="text-sm text-muted-foreground">
          Rankings among team members only
        </p>
      </CardHeader>
      <CardContent>
        {/* Leaderboard type selector */}
        <div className="flex flex-wrap gap-2 mb-4">
          {LEADERBOARD_OPTIONS.map((opt) => (
            <Button
              key={opt.value}
              variant={selectedType === opt.value ? "default" : "outline"}
              size="sm"
              onClick={() => setSelectedType(opt.value)}
              className="gap-2"
            >
              {opt.icon}
              {opt.label}
            </Button>
          ))}
        </div>

        {error && (
          <div className="p-4 text-destructive bg-destructive/10 rounded-md mb-4">
            {error}
          </div>
        )}

        {loading ? (
          <div className="space-y-2">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : entries.length === 0 ? (
          <div className="py-8 text-center text-muted-foreground">
            No data available for this leaderboard.
          </div>
        ) : (
          <TeamLeaderboardList
            entries={entries}
            type={selectedType}
            currentUserId={currentUserId}
          />
        )}
      </CardContent>
    </Card>
  );
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function formatSpeed(mps: number): string {
  const kph = mps * 3.6;
  return `${kph.toFixed(1)} km/h`;
}

function TeamLeaderboardList({
  entries,
  type,
  currentUserId,
}: {
  entries: TeamLeaderboardEntry[];
  type: TeamLeaderboardType;
  currentUserId?: string;
}) {
  return (
    <div className="divide-y border rounded-md">
      {entries.map((entry) => {
        const isCurrentUser = currentUserId === entry.user_id;
        return (
          <div
            key={entry.user_id}
            className={cn(
              "flex items-center gap-4 p-3",
              isCurrentUser && "bg-primary/5"
            )}
          >
            <div className="w-8 flex justify-center">
              <RankBadge rank={entry.rank} size="sm" />
            </div>
            <div className="flex-1 min-w-0">
              <Link
                href={`/profile/${entry.user_id}`}
                className="font-medium hover:underline truncate block"
              >
                {entry.user_name}
              </Link>
            </div>
            <div className="text-right font-medium">
              {type === "crowns" && (
                <span className="flex items-center gap-1">
                  <Crown className="h-4 w-4 text-amber-500" />
                  {(entry as CrownCountEntry).total_crowns}
                </span>
              )}
              {type === "distance" && (
                <span>{formatDistance((entry as DistanceLeaderEntry).total_distance_meters)}</span>
              )}
              {type === "dig_time" && (
                <span>{formatDuration((entry as DigTimeLeaderEntry).total_dig_time_seconds)}</span>
              )}
              {type === "dig_percentage" && (
                <span>{((entry as DigPercentageLeaderEntry).dig_percentage * 100).toFixed(1)}%</span>
              )}
              {type === "average_speed" && (
                <span>{formatSpeed((entry as AverageSpeedLeaderEntry).average_speed_mps)}</span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

const LEADERBOARD_TITLES: Record<string, string> = {
  crowns: "Crown Leaders",
  distance: "Distance Leaders",
  dig_time: "Dig Time Leaders (Weekly)",
  dig_percentage: "Dig Percentage Leaders",
  average_speed: "Speed Leaders",
};

const LEADERBOARD_ICONS: Record<string, React.ReactNode> = {
  crowns: <Crown className="h-5 w-5 text-amber-500" />,
  distance: <MapPin className="h-5 w-5 text-blue-500" />,
  dig_time: <Shovel className="h-5 w-5 text-orange-500" />,
  dig_percentage: <Percent className="h-5 w-5 text-green-500" />,
  average_speed: <Gauge className="h-5 w-5 text-purple-500" />,
};

function FeaturedLeaderboard({
  teamId,
  leaderboardType,
  currentUserId,
}: {
  teamId: string;
  leaderboardType: LeaderboardType;
  currentUserId?: string;
}) {
  const [entries, setEntries] = useState<TeamLeaderboardEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    setLoading(true);
    setError("");
    api
      .getTeamLeaderboard(teamId, leaderboardType)
      .then(setEntries)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [teamId, leaderboardType]);

  const title = LEADERBOARD_TITLES[leaderboardType] || "Leaderboard";
  const icon = LEADERBOARD_ICONS[leaderboardType];

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center gap-2">
          {icon}
          <CardTitle className="text-lg">{title}</CardTitle>
        </div>
        <p className="text-sm text-muted-foreground">
          Top performers in your team
        </p>
      </CardHeader>
      <CardContent>
        {error && (
          <div className="p-4 text-destructive bg-destructive/10 rounded-md">
            {error}
          </div>
        )}

        {loading ? (
          <div className="space-y-2">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
          </div>
        ) : entries.length === 0 ? (
          <div className="py-6 text-center text-muted-foreground">
            No data available yet.
          </div>
        ) : (
          <div className="divide-y border rounded-md">
            {entries.slice(0, 5).map((entry) => {
              const isCurrentUser = currentUserId === entry.user_id;
              return (
                <div
                  key={entry.user_id}
                  className={cn(
                    "flex items-center gap-4 p-3",
                    isCurrentUser && "bg-primary/5"
                  )}
                >
                  <div className="w-8 flex justify-center">
                    <RankBadge rank={entry.rank} size="sm" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <Link
                      href={`/profile/${entry.user_id}`}
                      className="font-medium hover:underline truncate block"
                    >
                      {entry.user_name}
                    </Link>
                  </div>
                  <div className="text-right font-medium">
                    {leaderboardType === "crowns" && (
                      <span className="flex items-center gap-1">
                        <Crown className="h-4 w-4 text-amber-500" />
                        {(entry as CrownCountEntry).total_crowns}
                      </span>
                    )}
                    {leaderboardType === "distance" && (
                      <span>{formatDistance((entry as DistanceLeaderEntry).total_distance_meters)}</span>
                    )}
                    {leaderboardType === "dig_time" && (
                      <span>{formatDuration((entry as DigTimeLeaderEntry).total_dig_time_seconds)}</span>
                    )}
                    {leaderboardType === "dig_percentage" && (
                      <span>{((entry as DigPercentageLeaderEntry).dig_percentage * 100).toFixed(1)}%</span>
                    )}
                    {leaderboardType === "average_speed" && (
                      <span>{formatSpeed((entry as AverageSpeedLeaderEntry).average_speed_mps)}</span>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {entries.length > 5 && (
          <div className="mt-3 text-center">
            <Link
              href={`/teams/${teamId}/leaderboard`}
              className="text-sm text-primary hover:underline"
            >
              View full leaderboard →
            </Link>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
