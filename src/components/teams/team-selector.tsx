"use client";

import { useState, useEffect } from "react";
import { TeamWithMembership, api } from "@/lib/api";
import { cn } from "@/lib/utils";

interface TeamSelectorProps {
  selectedTeamIds: string[];
  onSelectionChange: (teamIds: string[]) => void;
  disabled?: boolean;
}

function TeamAvatar({ name, avatarUrl }: { name: string; avatarUrl: string | null }) {
  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={name}
        className="w-10 h-10 rounded-lg object-cover"
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
    <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center text-primary-foreground font-bold text-sm">
      {initials}
    </div>
  );
}

export function TeamSelector({
  selectedTeamIds,
  onSelectionChange,
  disabled = false,
}: TeamSelectorProps) {
  const [teams, setTeams] = useState<TeamWithMembership[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    api
      .listMyTeams()
      .then(setTeams)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, []);

  const toggleTeam = (teamId: string) => {
    if (disabled) return;

    if (selectedTeamIds.includes(teamId)) {
      onSelectionChange(selectedTeamIds.filter((id) => id !== teamId));
    } else {
      onSelectionChange([...selectedTeamIds, teamId]);
    }
  };

  if (loading) {
    return (
      <div className="space-y-2">
        {[1, 2, 3].map((i) => (
          <div
            key={i}
            className="h-16 bg-muted/50 rounded-lg animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-destructive bg-destructive/10 rounded-md text-sm">
        Failed to load teams: {error}
      </div>
    );
  }

  if (teams.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        <p className="mb-2">You&apos;re not a member of any teams yet.</p>
        <a
          href="/teams/new"
          className="text-primary hover:underline text-sm"
        >
          Create your first team
        </a>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {teams.map((team) => {
        const isSelected = selectedTeamIds.includes(team.id);
        return (
          <button
            key={team.id}
            type="button"
            onClick={() => toggleTeam(team.id)}
            disabled={disabled}
            className={cn(
              "w-full flex items-center gap-3 p-3 rounded-lg border-2 transition-all text-left",
              isSelected
                ? "border-primary bg-primary/5"
                : "border-muted hover:border-muted-foreground/30",
              disabled && "opacity-50 cursor-not-allowed"
            )}
          >
            <TeamAvatar name={team.name} avatarUrl={team.avatar_url} />
            <div className="flex-1 min-w-0">
              <div className="font-medium truncate">{team.name}</div>
              <div className="text-sm text-muted-foreground">
                {team.member_count} members
              </div>
            </div>
            <div
              className={cn(
                "w-5 h-5 rounded-md border-2 flex items-center justify-center transition-colors",
                isSelected
                  ? "border-primary bg-primary text-primary-foreground"
                  : "border-muted-foreground/30"
              )}
            >
              {isSelected && (
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  className="w-3.5 h-3.5"
                >
                  <path
                    fillRule="evenodd"
                    d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </div>
          </button>
        );
      })}
    </div>
  );
}
