"use client";

import { useEffect, useState } from "react";
import { useRouter, useParams } from "next/navigation";
import { api, TeamWithMembership, TeamVisibility, TeamJoinPolicy, LeaderboardType } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

interface RadioCardProps {
  selected: boolean;
  onClick: () => void;
  disabled?: boolean;
  icon: React.ReactNode;
  label: string;
  description: string;
}

function RadioCard({
  selected,
  onClick,
  disabled,
  icon,
  label,
  description,
}: RadioCardProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "w-full flex items-start gap-3 p-4 rounded-lg border-2 transition-all text-left",
        selected
          ? "border-primary bg-primary/5"
          : "border-muted hover:border-muted-foreground/30",
        disabled && "opacity-50 cursor-not-allowed"
      )}
    >
      <div
        className={cn(
          "w-10 h-10 rounded-lg flex items-center justify-center text-lg",
          selected ? "bg-primary text-primary-foreground" : "bg-muted"
        )}
      >
        {icon}
      </div>
      <div className="flex-1">
        <div className="font-medium">{label}</div>
        <div className="text-sm text-muted-foreground">{description}</div>
      </div>
      <div
        className={cn(
          "w-5 h-5 rounded-full border-2 flex items-center justify-center mt-1 shrink-0",
          selected ? "border-primary" : "border-muted-foreground/30"
        )}
      >
        {selected && <div className="w-2.5 h-2.5 rounded-full bg-primary" />}
      </div>
    </button>
  );
}

export default function TeamSettingsPage() {
  const router = useRouter();
  const params = useParams();
  const teamId = params.id as string;
  const { user, loading: authLoading } = useAuth();

  const [team, setTeam] = useState<TeamWithMembership | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<TeamVisibility>("private");
  const [joinPolicy, setJoinPolicy] = useState<TeamJoinPolicy>("invitation");
  const [featuredLeaderboard, setFeaturedLeaderboard] = useState<LeaderboardType | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  useEffect(() => {
    if (authLoading) return;
    if (!user) {
      router.push("/login");
      return;
    }

    api
      .getTeam(teamId)
      .then((t) => {
        setTeam(t);
        setName(t.name);
        setDescription(t.description || "");
        setVisibility(t.visibility);
        setJoinPolicy(t.join_policy);
        setFeaturedLeaderboard(t.featured_leaderboard || null);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [teamId, user, authLoading, router]);

  if (authLoading || loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-4">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-96 w-full" />
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

  const canModify = team.user_role === "owner" || team.user_role === "admin";
  const canDelete = team.user_role === "owner";

  if (!canModify) {
    router.push(`/teams/${teamId}`);
    return null;
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!name.trim()) {
      setError("Team name is required");
      return;
    }

    setSaving(true);
    setError("");
    setSuccess("");

    try {
      await api.updateTeam(teamId, {
        name: name.trim(),
        description: description.trim() || undefined,
        visibility,
        join_policy: joinPolicy,
        featured_leaderboard: featuredLeaderboard || undefined,
      });
      setSuccess("Team settings saved successfully");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save");
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (
      !confirm(
        "Are you sure you want to delete this team? This action cannot be undone."
      )
    ) {
      return;
    }

    setDeleting(true);
    setError("");

    try {
      await api.deleteTeam(teamId);
      router.push("/teams");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete team");
      setDeleting(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-6">Team Settings</h1>

      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>General Settings</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSave} className="space-y-6">
              {error && (
                <div className="p-4 text-destructive bg-destructive/10 rounded-md">
                  {error}
                </div>
              )}

              {success && (
                <div className="p-4 text-green-800 bg-green-100 dark:text-green-400 dark:bg-green-900/30 rounded-md">
                  {success}
                </div>
              )}

              <div className="space-y-2">
                <Label htmlFor="name">Team Name *</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  disabled={saving}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  rows={3}
                  disabled={saving}
                />
              </div>

              {canDelete && (
                <div className="space-y-2">
                  <Label htmlFor="featured-leaderboard">Featured Leaderboard</Label>
                  <p className="text-sm text-muted-foreground">
                    Choose a leaderboard to display prominently on the team page
                  </p>
                  <select
                    id="featured-leaderboard"
                    value={featuredLeaderboard || ""}
                    onChange={(e) =>
                      setFeaturedLeaderboard(
                        e.target.value ? (e.target.value as LeaderboardType) : null
                      )
                    }
                    disabled={saving}
                    className="w-full px-3 py-2 border rounded-md bg-background text-sm"
                  >
                    <option value="">None</option>
                    <option value="crowns">Crowns</option>
                    <option value="distance">Distance</option>
                    <option value="dig_time">Dig Time</option>
                    <option value="dig_percentage">Dig Percentage</option>
                    <option value="average_speed">Average Speed</option>
                  </select>
                </div>
              )}

              <div className="space-y-3">
                <Label>Visibility</Label>
                <div className="space-y-2">
                  <RadioCard
                    selected={visibility === "private"}
                    onClick={() => setVisibility("private")}
                    disabled={saving}
                    icon={
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        className="w-5 h-5"
                      >
                        <path
                          fillRule="evenodd"
                          d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z"
                          clipRule="evenodd"
                        />
                      </svg>
                    }
                    label="Private"
                    description="Only members can find and view this team"
                  />
                  <RadioCard
                    selected={visibility === "public"}
                    onClick={() => setVisibility("public")}
                    disabled={saving}
                    icon={
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        className="w-5 h-5"
                      >
                        <path
                          fillRule="evenodd"
                          d="M10 18a8 8 0 100-16 8 8 0 000 16zM4.332 8.027a6.012 6.012 0 011.912-2.706C6.512 5.73 6.974 6 7.5 6A1.5 1.5 0 019 7.5V8a2 2 0 004 0 2 2 0 011.523-1.943 5.977 5.977 0 01.585 3.566A1.5 1.5 0 0114 11v.028a5.98 5.98 0 01-1.858 3.631L11 14a1 1 0 01-1-1v-1a2 2 0 00-2-2H6.172a3 3 0 01-1.805-.613 5.994 5.994 0 01-.035-1.36z"
                          clipRule="evenodd"
                        />
                      </svg>
                    }
                    label="Discoverable"
                    description="Anyone can find this team in the team directory"
                  />
                </div>
              </div>

              <div className="space-y-3">
                <Label>Join Policy</Label>
                <div className="space-y-2">
                  <RadioCard
                    selected={joinPolicy === "invitation"}
                    onClick={() => setJoinPolicy("invitation")}
                    disabled={saving}
                    icon={
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        className="w-5 h-5"
                      >
                        <path d="M3 4a2 2 0 00-2 2v1.161l8.441 4.221a1.25 1.25 0 001.118 0L19 7.162V6a2 2 0 00-2-2H3z" />
                        <path d="M19 8.839l-7.77 3.885a2.75 2.75 0 01-2.46 0L1 8.839V14a2 2 0 002 2h14a2 2 0 002-2V8.839z" />
                      </svg>
                    }
                    label="Invitation Only"
                    description="Users can only join through an invitation"
                  />
                  <RadioCard
                    selected={joinPolicy === "request"}
                    onClick={() => setJoinPolicy("request")}
                    disabled={saving || visibility === "private"}
                    icon={
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        className="w-5 h-5"
                      >
                        <path d="M10 8a3 3 0 100-6 3 3 0 000 6zM3.465 14.493a1.23 1.23 0 00.41 1.412A9.957 9.957 0 0010 18c2.31 0 4.438-.784 6.131-2.1.43-.333.604-.903.408-1.41a7.002 7.002 0 00-13.074.003z" />
                      </svg>
                    }
                    label="Request to Join"
                    description={
                      visibility === "private"
                        ? "Not available for private teams"
                        : "Users can request, admins approve"
                    }
                  />
                  <RadioCard
                    selected={joinPolicy === "open"}
                    onClick={() => setJoinPolicy("open")}
                    disabled={saving || visibility === "private"}
                    icon={
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        className="w-5 h-5"
                      >
                        <path d="M10 9a3 3 0 100-6 3 3 0 000 6zM6 8a2 2 0 11-4 0 2 2 0 014 0zM1.49 15.326a.78.78 0 01-.358-.442 3 3 0 014.308-3.516 6.484 6.484 0 00-1.905 3.959c-.023.222-.014.442.025.654a4.97 4.97 0 01-2.07-.655zM16.44 15.98a4.97 4.97 0 002.07-.654.78.78 0 00.357-.442 3 3 0 00-4.308-3.517 6.484 6.484 0 011.907 3.96 2.32 2.32 0 01-.026.654zM18 8a2 2 0 11-4 0 2 2 0 014 0zM5.304 16.19a.844.844 0 01-.277-.71 5 5 0 019.947 0 .843.843 0 01-.277.71A6.975 6.975 0 0110 18a6.974 6.974 0 01-4.696-1.81z" />
                      </svg>
                    }
                    label="Open"
                    description={
                      visibility === "private"
                        ? "Not available for private teams"
                        : "Anyone can join without approval"
                    }
                  />
                </div>
              </div>

              <div className="flex gap-4 pt-4">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => router.push(`/teams/${teamId}`)}
                  disabled={saving}
                >
                  Cancel
                </Button>
                <Button type="submit" disabled={saving}>
                  {saving ? "Saving..." : "Save Changes"}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        {canDelete && (
          <Card className="border-destructive/50">
            <CardHeader>
              <CardTitle className="text-destructive">Danger Zone</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground mb-4">
                Once you delete a team, there is no going back. All members will
                lose access to team content.
              </p>
              <Button
                variant="destructive"
                onClick={handleDelete}
                disabled={deleting}
              >
                {deleting ? "Deleting..." : "Delete Team"}
              </Button>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
