"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";
import type { TrackPoint } from "@/lib/api";
import type { MultiRangeSegment } from "./elevation-profile";

const ElevationProfile = dynamic(
  () => import("./elevation-profile").then((mod) => mod.ElevationProfile),
  {
    loading: () => <Skeleton className="w-full h-[200px]" />,
    ssr: false,
  }
);

interface LazyElevationProfileProps {
  points: TrackPoint[];
  onHover?: (index: number | null) => void;
  // Segment selection mode (existing)
  selectionMode?: boolean;
  selectionStart?: number | null;
  selectionEnd?: number | null;
  onPointClick?: (index: number) => void;
  // Multi-range mode (for multi-sport activities)
  multiRangeMode?: boolean;
  segments?: MultiRangeSegment[];
  onBoundaryClick?: (index: number) => void;
  selectedBoundaryIndex?: number | null;
}

export function LazyElevationProfile(props: LazyElevationProfileProps) {
  return <ElevationProfile {...props} />;
}

// Re-export types for convenience
export type { MultiRangeSegment };
