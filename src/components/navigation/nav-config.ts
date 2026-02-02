import type { LucideIcon } from "lucide-react";
import {
  Home,
  Rss,
  Calendar,
  Route,
  Trophy,
  Users,
  Compass,
  Map,
  Shovel,
  Activity,
  Settings,
  UserPlus,
} from "lucide-react";

/**
 * Navigation item that can be nested.
 * Teams are injected dynamically via the `teamPlaceholder` property.
 */
export interface NavItem {
  id: string;
  label: string;
  href?: string;
  icon?: LucideIcon;
  /** Only show when authenticated */
  authRequired?: boolean;
  /** Expandable section that can be collapsed */
  collapsible?: boolean;
  /** Child navigation items */
  children?: NavItem[];
  /** Placeholder for team items (injected by useTeamNav) */
  teamPlaceholder?: boolean;
  /** Badge count (e.g., member count) */
  badge?: number;
}

/**
 * Team navigation item structure (generated from API data)
 */
export interface TeamNavItem extends NavItem {
  teamId: string;
  role: "owner" | "admin" | "member";
  memberCount: number;
}

/**
 * Static navigation structure.
 * Teams are injected at the `teamPlaceholder` location by useTeamNav hook.
 */
export const NAV_CONFIG: NavItem[] = [
  {
    id: "home",
    label: "Home",
    href: "/",
    icon: Home,
  },
  {
    id: "explore",
    label: "Explore",
    icon: Compass,
    collapsible: true,
    children: [
      {
        id: "feed",
        label: "Feed",
        href: "/feed",
        icon: Rss,
        authRequired: true,
      },
      {
        id: "daily",
        label: "Daily Activities",
        href: "/activities/daily",
        icon: Calendar,
      },
      {
        id: "segments",
        label: "Segments",
        href: "/segments",
        icon: Route,
      },
      {
        id: "leaderboards",
        label: "Leaderboards",
        href: "/leaderboards",
        icon: Trophy,
      },
      {
        id: "dig-heatmap",
        label: "Dig Heatmap",
        href: "/dig-heatmap",
        icon: Shovel,
      },
    ],
  },
  {
    id: "my-stuff",
    label: "My Stuff",
    icon: Activity,
    collapsible: true,
    authRequired: true,
    children: [
      {
        id: "my-activities",
        label: "My Activities",
        href: "/activities",
        icon: Activity,
      },
      {
        id: "my-teams",
        label: "My Teams",
        icon: Users,
        collapsible: true,
        children: [
          // Teams are injected here by useTeamNav
          {
            id: "team-placeholder",
            label: "",
            teamPlaceholder: true,
          },
        ],
      },
    ],
  },
  {
    id: "discover-teams",
    label: "Discover Teams",
    href: "/teams?view=discover",
    icon: UserPlus,
    authRequired: true,
  },
];

/**
 * Generate navigation items for a single team
 */
export function generateTeamNavItems(
  teamId: string,
  teamName: string
): NavItem[] {
  return [
    {
      id: `team-${teamId}-daily-map`,
      label: "Daily Map",
      href: `/teams/${teamId}`,
      icon: Calendar,
    },
    {
      id: `team-${teamId}-heat-map`,
      label: "Heat Map",
      href: `/teams/${teamId}/heat-map`,
      icon: Map,
    },
    {
      id: `team-${teamId}-dig-map`,
      label: "Dig Map",
      href: `/teams/${teamId}/dig-heatmap`,
      icon: Shovel,
    },
    {
      id: `team-${teamId}-activities`,
      label: "Activities",
      href: `/teams/${teamId}/activities`,
      icon: Activity,
    },
    {
      id: `team-${teamId}-segments`,
      label: "Segments",
      href: `/teams/${teamId}/segments`,
      icon: Route,
    },
    {
      id: `team-${teamId}-members`,
      label: "Members",
      href: `/teams/${teamId}/members`,
      icon: Users,
    },
    {
      id: `team-${teamId}-settings`,
      label: "Settings",
      href: `/teams/${teamId}/settings`,
      icon: Settings,
    },
  ];
}

/**
 * Role priority for sorting teams (lower = higher priority)
 */
export const ROLE_PRIORITY: Record<string, number> = {
  owner: 0,
  admin: 1,
  member: 2,
};
