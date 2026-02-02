"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { Menu } from "lucide-react";
import { useAuth } from "@/lib/auth-context";
import { useSidebar } from "@/components/navigation";
import { Button } from "./ui/button";
import { NotificationBell } from "./notifications/notification-bell";

export function Header() {
  const router = useRouter();
  const { user, loading, logout } = useAuth();
  const { toggleMobile } = useSidebar();

  const handleLogout = () => {
    logout();
    router.push("/");
  };

  return (
    <header className="border-b" role="banner">
      <div className="container mx-auto px-4 py-4 flex items-center justify-between">
        {/* Mobile: hamburger + logo */}
        <div className="flex items-center gap-3 md:hidden">
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleMobile}
            aria-label="Open menu"
            className="h-9 w-9"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <Link
            href="/"
            className="font-[family-name:var(--font-orbitron)] font-black tracking-tight"
            aria-label="TRACKS.RS - Home"
          >
            <span className="text-xl text-foreground">TRACKS</span>
            <span className="text-sm text-primary align-super font-bold">.RS</span>
          </Link>
        </div>

        {/* Desktop: spacer since logo is in sidebar */}
        <div className="hidden md:block" />

        {/* User section */}
        <div className="flex items-center gap-2">
          {loading ? (
            <span className="text-muted-foreground text-sm">Loading...</span>
          ) : user ? (
            <>
              <NotificationBell />
              <Link
                href="/profile"
                className="text-sm text-muted-foreground hover:text-foreground"
              >
                {user.name}
              </Link>
              <Button variant="ghost" size="sm" onClick={handleLogout}>
                Sign out
              </Button>
            </>
          ) : (
            <>
              <Link
                href="/login"
                className="text-muted-foreground hover:text-foreground"
              >
                Sign in
              </Link>
              <Link
                href="/register"
                className="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-sm font-medium hover:bg-primary/90"
              >
                Sign up
              </Link>
            </>
          )}
        </div>
      </div>
    </header>
  );
}
