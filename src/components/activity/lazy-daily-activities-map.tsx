"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";
import type { FeedActivity } from "@/lib/api";

const DailyActivitiesMap = dynamic(
  () => import("./daily-activities-map").then((mod) => mod.DailyActivitiesMap),
  {
    loading: () => (
      <div className="space-y-4">
        <Skeleton className="h-[750px] w-full rounded-lg" />
        <p className="text-sm text-muted-foreground text-center">
          Loading map...
        </p>
      </div>
    ),
    ssr: false,
  }
);

interface LazyDailyActivitiesMapProps {
  activities: FeedActivity[];
}

export function LazyDailyActivitiesMap(props: LazyDailyActivitiesMapProps) {
  return <DailyActivitiesMap {...props} />;
}
