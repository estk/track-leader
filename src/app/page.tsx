import { Features } from "@/components/marketing/features";
import { FAQ } from "@/components/marketing/faq";

interface Stats {
  active_users: number;
  segments_created: number;
  activities_uploaded: number;
}

async function getStats(): Promise<Stats | null> {
  try {
    const apiBase = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';
    const response = await fetch(`${apiBase}/stats`, {
      cache: 'no-store',
    });
    if (!response.ok) {
      return null;
    }
    return response.json();
  } catch {
    return null;
  }
}

export default async function Home() {
  const stats = await getStats();

  return (
    <div className="space-y-12">
      {/* Hero Section */}
      <section className="text-center py-12">
        <h1 className="text-4xl font-bold tracking-tight sm:text-6xl">
          Open Leaderboards for
          <span className="text-primary"> Every Trail</span>
        </h1>
        <p className="mt-6 text-lg text-muted-foreground max-w-2xl mx-auto">
          Create segments, compete on your terms, and join a community of trail
          enthusiasts. Track Leader puts you in control of how you compete.
        </p>
        <div className="mt-10 flex items-center justify-center gap-4">
          <a
            href="/register"
            className="rounded-md bg-primary px-6 py-3 text-sm font-semibold text-primary-foreground shadow-sm hover:bg-primary/90"
          >
            Get Started
          </a>
          <a
            href="/segments"
            className="rounded-md border border-input px-6 py-3 text-sm font-semibold shadow-sm hover:bg-accent hover:text-accent-foreground"
          >
            Explore Segments
          </a>
        </div>
      </section>

      {/* Features Grid */}
      <Features />

      {/* Stats Section */}
      <section className="rounded-lg border bg-card p-8">
        <div className="grid gap-8 md:grid-cols-3 text-center">
          <div>
            <div className="text-4xl font-bold text-primary">
              {stats?.active_users ?? '-'}
            </div>
            <div className="text-sm text-muted-foreground">Active Users</div>
          </div>
          <div>
            <div className="text-4xl font-bold text-primary">
              {stats?.segments_created ?? '-'}
            </div>
            <div className="text-sm text-muted-foreground">Segments Created</div>
          </div>
          <div>
            <div className="text-4xl font-bold text-primary">
              {stats?.activities_uploaded ?? '-'}
            </div>
            <div className="text-sm text-muted-foreground">Activities Uploaded</div>
          </div>
        </div>
      </section>

      {/* FAQ Section */}
      <FAQ />
    </div>
  );
}
