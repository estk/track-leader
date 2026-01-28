"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";
import type { TrackData } from "@/lib/api";

const ActivityMap = dynamic(
  () => import("./activity-map").then((mod) => mod.ActivityMap),
  {
    loading: () => (
      <Skeleton className="w-full h-[300px] md:h-[400px] rounded-lg" />
    ),
    ssr: false,
  }
);

interface LazyActivityMapProps {
  trackData: TrackData;
  highlightIndex?: number;
  onHover?: (index: number | null) => void;
  selectionStart?: number | null;
  selectionEnd?: number | null;
}

export function LazyActivityMap(props: LazyActivityMapProps) {
  return <ActivityMap {...props} />;
}
