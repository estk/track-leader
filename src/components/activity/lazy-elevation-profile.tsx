"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";
import type { TrackPoint } from "@/lib/api";

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
  selectionMode?: boolean;
  selectionStart?: number | null;
  selectionEnd?: number | null;
  onPointClick?: (index: number) => void;
}

export function LazyElevationProfile(props: LazyElevationProfileProps) {
  return <ElevationProfile {...props} />;
}
