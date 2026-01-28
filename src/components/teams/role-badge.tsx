"use client";

import { TeamRole } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface RoleBadgeProps {
  role: TeamRole;
  className?: string;
}

export function getRoleBadgeClass(role: TeamRole): string {
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

export function getRoleLabel(role: TeamRole): string {
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

export function RoleBadge({ role, className }: RoleBadgeProps) {
  return (
    <Badge
      variant="outline"
      className={cn(getRoleBadgeClass(role), className)}
    >
      {getRoleLabel(role)}
    </Badge>
  );
}
