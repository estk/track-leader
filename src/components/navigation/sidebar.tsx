"use client";

import * as React from "react";
import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { PanelLeftClose, PanelLeft } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { NavTree } from "./nav-tree";
import { NAV_CONFIG, NavItem } from "./nav-config";
import { useTeamNav, injectTeamsIntoNav } from "@/hooks/use-team-nav";
import { useSidebar } from "./sidebar-context";

const STORAGE_KEY = "tracks-nav-expanded";
const SIDEBAR_COLLAPSED_KEY = "tracks-sidebar-collapsed";

interface SidebarProps {
  className?: string;
}

/**
 * Main sidebar component with responsive behavior.
 * - Desktop: Fixed sidebar that can be collapsed
 * - Mobile: Slide-out drawer (Sheet)
 */
export function Sidebar({ className }: SidebarProps) {
  const pathname = usePathname();
  const { isMobileOpen, setMobileOpen } = useSidebar();
  const { teamNavItems, loading } = useTeamNav();
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set(["explore", "my-stuff"]));
  const [isCollapsed, setIsCollapsed] = useState(false);

  // Load persisted state on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        setExpandedIds(new Set(JSON.parse(stored)));
      }
      const collapsedStored = localStorage.getItem(SIDEBAR_COLLAPSED_KEY);
      if (collapsedStored) {
        setIsCollapsed(JSON.parse(collapsedStored));
      }
    } catch {
      // Ignore localStorage errors
    }
  }, []);

  // Persist expanded state
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([...expandedIds]));
    } catch {
      // Ignore localStorage errors
    }
  }, [expandedIds]);

  // Persist collapsed state
  useEffect(() => {
    try {
      localStorage.setItem(SIDEBAR_COLLAPSED_KEY, JSON.stringify(isCollapsed));
    } catch {
      // Ignore localStorage errors
    }
  }, [isCollapsed]);

  // Close mobile menu on navigation
  useEffect(() => {
    setMobileOpen(false);
  }, [pathname, setMobileOpen]);

  const handleToggle = useCallback((id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  // Inject teams into nav config
  const navItems: NavItem[] = React.useMemo(() => {
    return injectTeamsIntoNav(NAV_CONFIG, teamNavItems);
  }, [teamNavItems]);

  const sidebarContent = (
    <div className="flex h-full flex-col">
      {/* Navigation tree */}
      <div className="flex-1 overflow-y-auto py-4">
        {loading ? (
          <div className="px-3 py-2 text-sm text-muted-foreground">
            Loading...
          </div>
        ) : (
          <NavTree
            items={navItems}
            expandedIds={expandedIds}
            onToggle={handleToggle}
            collapsed={isCollapsed}
          />
        )}
      </div>
    </div>
  );

  return (
    <>
      {/* Desktop sidebar */}
      <aside
        className={cn(
          "hidden md:flex flex-col border-r bg-background transition-all duration-300",
          isCollapsed ? "w-16" : "w-64",
          className
        )}
      >
        {/* Sidebar header with collapse toggle */}
        <div className="flex h-14 items-center justify-between border-b px-3">
          {!isCollapsed && (
            <Link
              href="/"
              className="font-[family-name:var(--font-orbitron)] font-black tracking-tight"
            >
              <span className="text-lg text-foreground">TRACKS</span>
              <span className="text-xs text-primary align-super font-bold">.RS</span>
            </Link>
          )}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsCollapsed(!isCollapsed)}
            className={cn("h-8 w-8", isCollapsed && "mx-auto")}
            aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            {isCollapsed ? (
              <PanelLeft className="h-4 w-4" />
            ) : (
              <PanelLeftClose className="h-4 w-4" />
            )}
          </Button>
        </div>
        {sidebarContent}
      </aside>

      {/* Mobile sheet/drawer */}
      <Sheet open={isMobileOpen} onOpenChange={setMobileOpen}>
        <SheetContent side="left" className="w-72 p-0">
          <SheetHeader className="border-b px-4 py-3">
            <SheetTitle className="text-left">
              <Link
                href="/"
                className="font-[family-name:var(--font-orbitron)] font-black tracking-tight"
                onClick={() => setMobileOpen(false)}
              >
                <span className="text-lg text-foreground">TRACKS</span>
                <span className="text-xs text-primary align-super font-bold">.RS</span>
              </Link>
            </SheetTitle>
          </SheetHeader>
          <div className="py-2">
            <NavTree
              items={navItems}
              expandedIds={expandedIds}
              onToggle={handleToggle}
              collapsed={false}
            />
          </div>
        </SheetContent>
      </Sheet>
    </>
  );
}
