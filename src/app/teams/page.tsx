"use client";

import { Suspense, useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import Link from "next/link";
import { api, TeamWithMembership, TeamSummary } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { TeamCard } from "@/components/teams/team-card";

type ViewMode = "my-teams" | "discover";

function TeamsPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { user, loading: authLoading } = useAuth();
  const [myTeams, setMyTeams] = useState<TeamWithMembership[]>([]);
  const [discoverableTeams, setDiscoverableTeams] = useState<TeamSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Initialize from URL param, default to "my-teams"
  const viewParam = searchParams.get("view");
  const [viewMode, setViewMode] = useState<ViewMode>(
    viewParam === "discover" ? "discover" : "my-teams"
  );

  // Sync viewMode with URL param changes
  useEffect(() => {
    const newMode = viewParam === "discover" ? "discover" : "my-teams";
    setViewMode(newMode);
  }, [viewParam]);

  useEffect(() => {
    if (authLoading) return;

    if (!user) {
      router.push("/login");
      return;
    }

    setLoading(true);
    setError("");

    if (viewMode === "my-teams") {
      api
        .listMyTeams()
        .then(setMyTeams)
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    } else {
      api
        .discoverTeams(50)
        .then(setDiscoverableTeams)
        .catch((err) => setError(err.message))
        .finally(() => setLoading(false));
    }
  }, [user, authLoading, router, viewMode]);

  if (authLoading || !user) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Teams</h1>
        <Link href="/teams/new">
          <Button>Create Team</Button>
        </Link>
      </div>

      <div className="flex gap-2">
        <Button
          variant={viewMode === "my-teams" ? "default" : "outline"}
          size="sm"
          onClick={() => {
            setViewMode("my-teams");
            router.push("/teams");
          }}
        >
          My Teams
        </Button>
        <Button
          variant={viewMode === "discover" ? "default" : "outline"}
          size="sm"
          onClick={() => {
            setViewMode("discover");
            router.push("/teams?view=discover");
          }}
        >
          Discover
        </Button>
      </div>

      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md">
          {error}
        </div>
      )}

      {loading ? (
        <div className="space-y-4">
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
        </div>
      ) : viewMode === "my-teams" ? (
        <MyTeamsContent teams={myTeams} />
      ) : (
        <DiscoverContent teams={discoverableTeams} router={router} />
      )}
    </div>
  );
}

export default function TeamsPage() {
  return (
    <Suspense
      fallback={
        <div className="space-y-4">
          <Skeleton className="h-10 w-48" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-32 w-full" />
        </div>
      }
    >
      <TeamsPageContent />
    </Suspense>
  );
}

function MyTeamsContent({ teams }: { teams: TeamWithMembership[] }) {
  if (teams.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground mb-4">
            You&apos;re not a member of any teams yet.
          </p>
          <div className="flex gap-4 justify-center">
            <Link href="/teams/new">
              <Button>Create Your First Team</Button>
            </Link>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2">
      {teams.map((team) => (
        <TeamCard key={team.id} team={team} />
      ))}
    </div>
  );
}

function DiscoverContent({
  teams,
  router,
}: {
  teams: TeamSummary[];
  router: ReturnType<typeof useRouter>;
}) {
  if (teams.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">
            No discoverable teams available.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2">
      {teams.map((team) => (
        <Card
          key={team.id}
          className="hover:shadow-md transition-shadow cursor-pointer"
          onClick={() => router.push(`/teams/${team.id}`)}
        >
          <CardContent className="p-4">
            <div className="flex items-start gap-3">
              <TeamAvatar name={team.name} avatarUrl={team.avatar_url} />
              <div className="flex-1 min-w-0">
                <h3 className="font-semibold truncate">{team.name}</h3>
                {team.description && (
                  <p className="text-sm text-muted-foreground line-clamp-2 mt-1">
                    {team.description}
                  </p>
                )}
                <div className="flex gap-4 text-sm text-muted-foreground mt-2">
                  <span>{team.member_count} members</span>
                  <span>{team.activity_count} activities</span>
                  <span>{team.segment_count} segments</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function TeamAvatar({
  name,
  avatarUrl,
}: {
  name: string;
  avatarUrl: string | null;
}) {
  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={name}
        className="w-12 h-12 rounded-lg object-cover"
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
    <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center text-primary-foreground font-bold text-lg">
      {initials}
    </div>
  );
}
