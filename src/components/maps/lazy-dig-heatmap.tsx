"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";

const DigHeatmap = dynamic(
  () => import("./dig-heatmap").then((mod) => mod.DigHeatmap),
  {
    loading: () => (
      <div className="space-y-4">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <Skeleton className="h-7 w-48" />
          <div className="flex items-center gap-2">
            <Skeleton className="h-9 w-[180px]" />
            <Skeleton className="h-10 w-[150px]" />
          </div>
        </div>
        <Skeleton className="h-[750px] w-full rounded-lg" />
        <p className="text-sm text-muted-foreground text-center">
          Loading map...
        </p>
      </div>
    ),
    ssr: false,
  }
);

interface LazyDigHeatmapProps {
  teamId?: string;
}

export function LazyDigHeatmap(props: LazyDigHeatmapProps) {
  return <DigHeatmap {...props} />;
}
