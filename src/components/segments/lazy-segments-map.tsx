"use client";

import dynamic from "next/dynamic";
import { Skeleton } from "@/components/ui/skeleton";
import type { Segment } from "@/lib/api";

const SegmentsMap = dynamic(
  () => import("./segments-map").then((mod) => mod.SegmentsMap),
  {
    loading: () => (
      <div className="space-y-4">
        <Skeleton className="h-[500px] w-full rounded-lg" />
        <p className="text-sm text-muted-foreground text-center">
          Loading map...
        </p>
      </div>
    ),
    ssr: false,
  }
);

interface LazySegmentsMapProps {
  segments: Segment[];
}

export function LazySegmentsMap(props: LazySegmentsMapProps) {
  return <SegmentsMap {...props} />;
}
