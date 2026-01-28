"use client";

import { useEffect, useState } from "react";
import { useRouter, useParams } from "next/navigation";
import {
  api,
  TeamWithMembership,
  TeamInvitation,
  TeamJoinRequest,
  TeamRole,
} from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { RoleBadge } from "@/components/teams/role-badge";

export default function TeamInvitePage() {
  const router = useRouter();
  const params = useParams();
  const teamId = params.id as string;
  const { user, loading: authLoading } = useAuth();

  const [team, setTeam] = useState<TeamWithMembership | null>(null);
  const [invitations, setInvitations] = useState<TeamInvitation[]>([]);
  const [joinRequests, setJoinRequests] = useState<TeamJoinRequest[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Invite form
  const [email, setEmail] = useState("");
  const [role, setRole] = useState<TeamRole>("member");
  const [inviting, setInviting] = useState(false);
  const [inviteError, setInviteError] = useState("");
  const [inviteSuccess, setInviteSuccess] = useState("");

  useEffect(() => {
    if (authLoading) return;
    if (!user) {
      router.push("/login");
      return;
    }

    const fetchData = async () => {
      try {
        const t = await api.getTeam(teamId);
        setTeam(t);

        if (t.user_role === "owner" || t.user_role === "admin") {
          const [invs, reqs] = await Promise.all([
            api.getTeamInvitations(teamId),
            t.join_policy === "request" ? api.getJoinRequests(teamId) : [],
          ]);
          setInvitations(invs);
          setJoinRequests(reqs);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load team");
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [teamId, user, authLoading, router]);

  if (authLoading || loading) {
    return (
      <div className="max-w-2xl mx-auto space-y-4">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-64 w-full" />
        <Skeleton className="h-64 w-full" />
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
  const isOwner = team.user_role === "owner";

  if (!canManage) {
    router.push(`/teams/${teamId}`);
    return null;
  }

  const handleInvite = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email.trim()) {
      setInviteError("Email is required");
      return;
    }

    setInviting(true);
    setInviteError("");
    setInviteSuccess("");

    try {
      const invitation = await api.inviteToTeam(teamId, email.trim(), role);
      setInvitations([invitation, ...invitations]);
      setEmail("");
      setInviteSuccess(`Invitation sent to ${email.trim()}`);
    } catch (err) {
      setInviteError(err instanceof Error ? err.message : "Failed to invite");
    } finally {
      setInviting(false);
    }
  };

  const handleRevokeInvitation = async (invitationId: string) => {
    try {
      await api.revokeInvitation(teamId, invitationId);
      setInvitations(invitations.filter((i) => i.id !== invitationId));
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to revoke");
    }
  };

  const handleReviewRequest = async (requestId: string, approved: boolean) => {
    try {
      await api.reviewJoinRequest(teamId, requestId, approved);
      setJoinRequests(joinRequests.filter((r) => r.id !== requestId));
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to review request");
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-6">Invite to {team.name}</h1>

      {error && (
        <div className="p-4 text-destructive bg-destructive/10 rounded-md mb-6">
          {error}
        </div>
      )}

      <div className="space-y-6">
        {/* Invite Form */}
        <Card>
          <CardHeader>
            <CardTitle>Send Invitation</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleInvite} className="space-y-4">
              {inviteError && (
                <div className="p-3 text-destructive bg-destructive/10 rounded-md text-sm">
                  {inviteError}
                </div>
              )}
              {inviteSuccess && (
                <div className="p-3 text-green-800 bg-green-100 dark:text-green-400 dark:bg-green-900/30 rounded-md text-sm">
                  {inviteSuccess}
                </div>
              )}

              <div className="space-y-2">
                <Label htmlFor="email">Email Address</Label>
                <Input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="user@example.com"
                  disabled={inviting}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="role">Role</Label>
                <select
                  id="role"
                  value={role}
                  onChange={(e) => setRole(e.target.value as TeamRole)}
                  disabled={inviting}
                  className="w-full px-3 py-2 border rounded-md bg-background"
                >
                  <option value="member">Member</option>
                  {isOwner && <option value="admin">Admin</option>}
                </select>
              </div>

              <Button type="submit" disabled={inviting}>
                {inviting ? "Sending..." : "Send Invitation"}
              </Button>
            </form>
          </CardContent>
        </Card>

        {/* Pending Invitations */}
        <Card>
          <CardHeader>
            <CardTitle>Pending Invitations</CardTitle>
          </CardHeader>
          <CardContent>
            {invitations.length === 0 ? (
              <p className="text-muted-foreground text-sm">
                No pending invitations
              </p>
            ) : (
              <div className="space-y-3">
                {invitations.map((invitation) => (
                  <div
                    key={invitation.id}
                    className="flex items-center justify-between p-3 bg-muted/50 rounded-lg"
                  >
                    <div>
                      <div className="font-medium">{invitation.email}</div>
                      <div className="flex items-center gap-2 text-sm text-muted-foreground">
                        <RoleBadge role={invitation.role} />
                        <span>
                          Expires{" "}
                          {new Date(invitation.expires_at).toLocaleDateString()}
                        </span>
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRevokeInvitation(invitation.id)}
                      className="text-destructive hover:text-destructive"
                    >
                      Revoke
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Join Requests (if applicable) */}
        {team.join_policy === "request" && (
          <Card>
            <CardHeader>
              <CardTitle>Join Requests</CardTitle>
            </CardHeader>
            <CardContent>
              {joinRequests.length === 0 ? (
                <p className="text-muted-foreground text-sm">
                  No pending join requests
                </p>
              ) : (
                <div className="space-y-3">
                  {joinRequests.map((request) => (
                    <div
                      key={request.id}
                      className="flex items-center justify-between p-3 bg-muted/50 rounded-lg"
                    >
                      <div>
                        <div className="font-medium">{request.user_name}</div>
                        {request.message && (
                          <p className="text-sm text-muted-foreground">
                            {request.message}
                          </p>
                        )}
                        <div className="text-xs text-muted-foreground mt-1">
                          Requested{" "}
                          {new Date(request.created_at).toLocaleDateString()}
                        </div>
                      </div>
                      <div className="flex gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleReviewRequest(request.id, false)}
                        >
                          Reject
                        </Button>
                        <Button
                          size="sm"
                          onClick={() => handleReviewRequest(request.id, true)}
                        >
                          Approve
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        )}

        <Button
          variant="outline"
          onClick={() => router.push(`/teams/${teamId}`)}
        >
          Back to Team
        </Button>
      </div>
    </div>
  );
}
