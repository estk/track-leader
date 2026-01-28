"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { Button } from "./ui/button";
import { NotificationBell } from "./notifications/notification-bell";

export function Header() {
  const router = useRouter();
  const { user, loading, logout } = useAuth();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const handleLogout = () => {
    logout();
    router.push("/");
    setMobileMenuOpen(false);
  };

  const closeMobileMenu = () => setMobileMenuOpen(false);

  return (
    <header className="border-b" role="banner">
      <div className="container mx-auto px-4 py-4 flex items-center justify-between">
        <Link
          href="/"
          className="text-xl font-bold text-primary"
          aria-label="Track Leader - Home"
        >
          Track Leader
        </Link>

        {/* Desktop nav */}
        <nav
          className="hidden md:flex items-center gap-6"
          role="navigation"
          aria-label="Main navigation"
        >
          {user && (
            <Link href="/feed" className="text-muted-foreground hover:text-foreground">
              Feed
            </Link>
          )}
          <Link href="/activities" className="text-muted-foreground hover:text-foreground">
            Activities
          </Link>
          <Link href="/segments" className="text-muted-foreground hover:text-foreground">
            Segments
          </Link>
          <Link href="/leaderboards" className="text-muted-foreground hover:text-foreground">
            Leaderboards
          </Link>
          <div className="flex items-center gap-2 ml-4 pl-4 border-l">
            {loading ? (
              <span className="text-muted-foreground text-sm">Loading...</span>
            ) : user ? (
              <>
                <NotificationBell />
                <Link href="/profile" className="text-sm text-muted-foreground hover:text-foreground">
                  {user.name}
                </Link>
                <Button variant="ghost" size="sm" onClick={handleLogout}>
                  Sign out
                </Button>
              </>
            ) : (
              <>
                <Link href="/login" className="text-muted-foreground hover:text-foreground">
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
        </nav>

        {/* Mobile menu button */}
        <button
          className="md:hidden p-2"
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          aria-label={mobileMenuOpen ? "Close menu" : "Open menu"}
          aria-expanded={mobileMenuOpen}
          aria-controls="mobile-nav"
        >
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            {mobileMenuOpen ? (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            ) : (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6h16M4 12h16M4 18h16"
              />
            )}
          </svg>
        </button>
      </div>

      {/* Mobile nav */}
      {mobileMenuOpen && (
        <nav
          id="mobile-nav"
          className="md:hidden border-t bg-background"
          role="navigation"
          aria-label="Mobile navigation"
        >
          <div className="container mx-auto px-4 py-4 flex flex-col gap-4">
            {user && (
              <Link
                href="/feed"
                className="text-muted-foreground hover:text-foreground py-2"
                onClick={closeMobileMenu}
              >
                Feed
              </Link>
            )}
            <Link
              href="/activities"
              className="text-muted-foreground hover:text-foreground py-2"
              onClick={closeMobileMenu}
            >
              Activities
            </Link>
            <Link
              href="/segments"
              className="text-muted-foreground hover:text-foreground py-2"
              onClick={closeMobileMenu}
            >
              Segments
            </Link>
            <Link
              href="/leaderboards"
              className="text-muted-foreground hover:text-foreground py-2"
              onClick={closeMobileMenu}
            >
              Leaderboards
            </Link>
            <div className="border-t pt-4 flex flex-col gap-4">
              {loading ? (
                <span className="text-muted-foreground text-sm">Loading...</span>
              ) : user ? (
                <>
                  <Link
                    href="/notifications"
                    className="text-muted-foreground hover:text-foreground py-2"
                    onClick={closeMobileMenu}
                  >
                    Notifications
                  </Link>
                  <Link
                    href="/profile"
                    className="text-muted-foreground hover:text-foreground py-2"
                    onClick={closeMobileMenu}
                  >
                    Profile ({user.name})
                  </Link>
                  <Button variant="outline" onClick={handleLogout}>
                    Sign out
                  </Button>
                </>
              ) : (
                <>
                  <Link
                    href="/login"
                    className="text-muted-foreground hover:text-foreground py-2"
                    onClick={closeMobileMenu}
                  >
                    Sign in
                  </Link>
                  <Link
                    href="/register"
                    className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-center font-medium hover:bg-primary/90"
                    onClick={closeMobileMenu}
                  >
                    Sign up
                  </Link>
                </>
              )}
            </div>
          </div>
        </nav>
      )}
    </header>
  );
}
