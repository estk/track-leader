"use client";

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import type { NavItem } from "./nav-config";
import { useAuth } from "@/lib/auth-context";

interface NavTreeProps {
  items: NavItem[];
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
  collapsed?: boolean;
}

/**
 * Renders the navigation tree with collapsible sections.
 */
export function NavTree({
  items,
  expandedIds,
  onToggle,
  collapsed = false,
}: NavTreeProps) {
  const { user } = useAuth();
  const pathname = usePathname();

  // Filter items based on auth state
  const visibleItems = items.filter((item) => {
    if (item.authRequired && !user) return false;
    return true;
  });

  return (
    <nav className="space-y-1" role="navigation" aria-label="Main navigation">
      {visibleItems.map((item) => (
        <NavTreeItem
          key={item.id}
          item={item}
          expandedIds={expandedIds}
          onToggle={onToggle}
          pathname={pathname}
          collapsed={collapsed}
          depth={0}
        />
      ))}
    </nav>
  );
}

interface NavTreeItemProps {
  item: NavItem;
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
  pathname: string;
  collapsed: boolean;
  depth: number;
}

function NavTreeItem({
  item,
  expandedIds,
  onToggle,
  pathname,
  collapsed,
  depth,
}: NavTreeItemProps) {
  const { user } = useAuth();

  // Skip placeholder items or auth-required items when not authenticated
  if (item.teamPlaceholder) return null;
  if (item.authRequired && !user) return null;

  const isExpanded = expandedIds.has(item.id);
  // Filter out placeholder items when checking for children
  const visibleChildren = item.children?.filter(
    (child) => !child.teamPlaceholder && (!child.authRequired || user)
  );
  const hasChildren = visibleChildren && visibleChildren.length > 0;
  const isActive = item.href ? pathname === item.href || pathname.startsWith(item.href + "/") : false;
  const Icon = item.icon;

  // Check if any child is active (for highlighting parent)
  const hasActiveChild = hasChildren && item.children!.some((child) =>
    child.href ? pathname === child.href || pathname.startsWith(child.href + "/") : false
  );

  const handleClick = () => {
    if (item.collapsible && hasChildren) {
      onToggle(item.id);
    }
  };

  // Padding based on depth
  const paddingLeft = collapsed ? "pl-3" : `pl-${3 + depth * 4}`;

  // Common classes for nav items
  const itemClasses = cn(
    "group flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
    "hover:bg-accent hover:text-accent-foreground",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
    isActive && "bg-accent text-accent-foreground",
    hasActiveChild && !isActive && "text-foreground",
    !isActive && !hasActiveChild && "text-muted-foreground"
  );

  // When sidebar is collapsed, only show icon
  if (collapsed) {
    if (!Icon) return null;

    const content = (
      <div
        className={cn(
          "flex items-center justify-center rounded-md p-2 transition-colors",
          "hover:bg-accent hover:text-accent-foreground",
          isActive && "bg-accent text-accent-foreground"
        )}
        title={item.label}
      >
        <Icon className="h-5 w-5" />
      </div>
    );

    if (item.href) {
      return (
        <Link href={item.href} className="block">
          {content}
        </Link>
      );
    }

    return content;
  }

  // Full sidebar mode
  // Render as collapsible if marked collapsible OR has visible children
  if (item.collapsible) {
    return (
      <div>
        <button
          onClick={handleClick}
          className={cn(itemClasses, "w-full justify-between")}
          style={{ paddingLeft: `${0.75 + depth * 1}rem` }}
          aria-expanded={isExpanded}
        >
          <span className="flex items-center gap-3">
            {Icon && <Icon className="h-4 w-4 shrink-0" />}
            <span className="truncate">{item.label}</span>
          </span>
          <span className="flex items-center gap-2">
            {item.badge !== undefined && (
              <span className="rounded-full bg-muted px-2 py-0.5 text-xs">
                {item.badge}
              </span>
            )}
            <ChevronRight
              className={cn(
                "h-4 w-4 shrink-0 transition-transform",
                isExpanded && "rotate-90"
              )}
            />
          </span>
        </button>
        {isExpanded && visibleChildren && visibleChildren.length > 0 && (
          <div className="mt-1">
            {visibleChildren.map((child) => (
              <NavTreeItem
                key={child.id}
                item={child}
                expandedIds={expandedIds}
                onToggle={onToggle}
                pathname={pathname}
                collapsed={collapsed}
                depth={depth + 1}
              />
            ))}
          </div>
        )}
      </div>
    );
  }

  // Leaf node (link)
  if (item.href) {
    return (
      <Link
        href={item.href}
        className={itemClasses}
        style={{ paddingLeft: `${0.75 + depth * 1}rem` }}
        aria-current={isActive ? "page" : undefined}
      >
        {Icon && <Icon className="h-4 w-4 shrink-0" />}
        <span className="truncate">{item.label}</span>
        {item.badge !== undefined && (
          <span className="ml-auto rounded-full bg-muted px-2 py-0.5 text-xs">
            {item.badge}
          </span>
        )}
      </Link>
    );
  }

  // Non-link item (section header)
  return (
    <div
      className={cn(itemClasses, "cursor-default")}
      style={{ paddingLeft: `${0.75 + depth * 1}rem` }}
    >
      {Icon && <Icon className="h-4 w-4 shrink-0" />}
      <span className="truncate">{item.label}</span>
    </div>
  );
}
