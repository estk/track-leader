"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";

const TeamHeatmap = dynamic(
  () => import("./team-heatmap").then((mod) => mod.TeamHeatmap),
  {
    loading: () => (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <Skeleton className="h-7 w-40" />
          <Skeleton className="h-10 w-[150px]" />
        </div>
        <Skeleton className="h-[400px] w-full rounded-lg" />
        <p className="text-sm text-muted-foreground text-center">
          Loading map...
        </p>
      </div>
    ),
    ssr: false,
  }
);

interface LazyTeamHeatmapProps {
  teamId: string;
}

export function LazyTeamHeatmap(props: LazyTeamHeatmapProps) {
  return <TeamHeatmap {...props} />;
}
