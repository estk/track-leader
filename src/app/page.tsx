import { Mountain, Trophy, Users, Upload } from "lucide-react";

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
      <section className="grid gap-8 md:grid-cols-2 lg:grid-cols-4">
        <FeatureCard
          icon={<Mountain className="h-8 w-8" />}
          title="Create Segments"
          description="Define your own segments from any activity. Your trail, your rules."
        />
        <FeatureCard
          icon={<Trophy className="h-8 w-8" />}
          title="Compete Openly"
          description="Transparent leaderboards with demographic filters. Find your competition."
        />
        <FeatureCard
          icon={<Users className="h-8 w-8" />}
          title="Community Driven"
          description="Segments created and verified by the community. Quality through collaboration."
        />
        <FeatureCard
          icon={<Upload className="h-8 w-8" />}
          title="Upload & Track"
          description="Upload GPX files and automatically match to segments. See your progress."
        />
      </section>

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
    </div>
  );
}

function FeatureCard({
  icon,
  title,
  description,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="rounded-lg border bg-card p-6">
      <div className="text-primary mb-4">{icon}</div>
      <h3 className="font-semibold mb-2">{title}</h3>
      <p className="text-sm text-muted-foreground">{description}</p>
    </div>
  );
}
