"use client";

import { useEffect, useState } from "react";
import { api, TeamWithMembership } from "@/lib/api";
import { useAuth } from "@/lib/auth-context";
import {
  NavItem,
  TeamNavItem,
  generateTeamNavItems,
  ROLE_PRIORITY,
} from "@/components/navigation/nav-config";
import { Users } from "lucide-react";

interface UseTeamNavResult {
  teamNavItems: TeamNavItem[];
  loading: boolean;
  error: Error | null;
}

/**
 * Hook that fetches user's teams and builds navigation items.
 * Teams are sorted by role (owner first, then admin, then member).
 */
export function useTeamNav(): UseTeamNavResult {
  const { user } = useAuth();
  const [teamNavItems, setTeamNavItems] = useState<TeamNavItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!user) {
      setTeamNavItems([]);
      return;
    }

    let cancelled = false;

    async function fetchTeams() {
      setLoading(true);
      setError(null);

      try {
        const teams = await api.listMyTeams();

        if (cancelled) return;

        // Sort by role priority (owner first, then admin, then member)
        const sortedTeams = [...teams].sort((a, b) => {
          const priorityA = ROLE_PRIORITY[a.user_role || "member"] ?? 99;
          const priorityB = ROLE_PRIORITY[b.user_role || "member"] ?? 99;
          if (priorityA !== priorityB) return priorityA - priorityB;
          // Secondary sort by name
          return a.name.localeCompare(b.name);
        });

        // Convert to nav items
        const navItems: TeamNavItem[] = sortedTeams.map((team) => ({
          id: `team-${team.id}`,
          label: team.name,
          icon: Users,
          teamId: team.id,
          role: team.user_role || "member",
          memberCount: team.member_count,
          badge: team.member_count,
          collapsible: true,
          children: generateTeamNavItems(team.id, team.name),
        }));

        setTeamNavItems(navItems);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err : new Error("Failed to fetch teams"));
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    fetchTeams();

    return () => {
      cancelled = true;
    };
  }, [user]);

  return { teamNavItems, loading, error };
}

/**
 * Inject team nav items into the static nav config.
 * Replaces the teamPlaceholder with actual team items.
 */
export function injectTeamsIntoNav(
  navConfig: NavItem[],
  teamNavItems: TeamNavItem[]
): NavItem[] {
  return navConfig.map((item) => {
    if (item.children) {
      const newChildren: NavItem[] = [];
      for (const child of item.children) {
        if (child.teamPlaceholder) {
          // Replace placeholder with actual team items
          newChildren.push(...teamNavItems);
        } else if (child.children) {
          // Recurse into nested children
          newChildren.push({
            ...child,
            children: injectTeamsIntoNav([child], teamNavItems)[0].children,
          });
        } else {
          newChildren.push(child);
        }
      }
      return { ...item, children: newChildren };
    }
    return item;
  });
}
