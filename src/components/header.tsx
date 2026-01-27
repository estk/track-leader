"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth-context";
import { Button } from "./ui/button";

export function Header() {
  const router = useRouter();
  const { user, loading, logout } = useAuth();

  const handleLogout = () => {
    logout();
    router.push("/");
  };

  return (
    <header className="border-b">
      <div className="container mx-auto px-4 py-4 flex items-center justify-between">
        <Link href="/" className="text-xl font-bold text-primary">
          Track Leader
        </Link>
        <nav className="flex items-center gap-6">
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
      </div>
    </header>
  );
}
