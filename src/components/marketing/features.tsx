import {
  Mountain,
  Trophy,
  Users,
  Upload,
  Filter,
  Award,
  MapPin,
  BarChart3,
} from "lucide-react";

const features = [
  {
    icon: Mountain,
    title: "Create Segments",
    description:
      "Define your own segments from any activity. Select start and end points on the elevation profile to create competition-worthy courses.",
  },
  {
    icon: Trophy,
    title: "Compete Openly",
    description:
      "Transparent leaderboards show everyone's times. No hidden algorithms - just pure performance comparisons.",
  },
  {
    icon: Filter,
    title: "Demographic Filters",
    description:
      "Compare yourself against similar athletes. Filter by age group, gender, or see the overall rankings.",
  },
  {
    icon: Award,
    title: "Earn Crowns",
    description:
      "Claim KOMs and QOMs. Track your crown collection across all segments.",
  },
  {
    icon: Users,
    title: "Community Driven",
    description:
      "Segments created and verified by the community. Follow other athletes and see their activities in your feed.",
  },
  {
    icon: Upload,
    title: "GPX Upload",
    description:
      "Upload GPX files from any device or app. Automatic segment matching finds all your efforts instantly.",
  },
  {
    icon: MapPin,
    title: "Nearby Segments",
    description:
      "Discover segments near you with location-based search. Find trails you might have missed.",
  },
  {
    icon: BarChart3,
    title: "Track Progress",
    description:
      "See your PR history and improvement over time. Detailed stats help you train smarter.",
  },
];

export function Features() {
  return (
    <section className="py-16" aria-labelledby="features-heading">
      <div className="text-center mb-12">
        <h2 id="features-heading" className="text-3xl font-bold tracking-tight">
          Everything You Need to Compete
        </h2>
        <p className="mt-4 text-lg text-muted-foreground max-w-2xl mx-auto">
          Track Leader gives you the tools to create, compete, and connect with
          trail athletes worldwide.
        </p>
      </div>
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        {features.map((feature) => (
          <div
            key={feature.title}
            className="rounded-lg border bg-card p-6 hover:shadow-md transition-shadow"
          >
            <div className="text-primary mb-4">
              <feature.icon className="h-8 w-8" aria-hidden="true" />
            </div>
            <h3 className="font-semibold mb-2">{feature.title}</h3>
            <p className="text-sm text-muted-foreground">{feature.description}</p>
          </div>
        ))}
      </div>
    </section>
  );
}
