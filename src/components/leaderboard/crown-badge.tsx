import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Crown, Star, CheckCircle, Medal } from "lucide-react";
import { cn } from "@/lib/utils";

// Size variants shared across badge components
const sizeConfig = {
  sm: { icon: 12, text: "text-xs", padding: "px-1.5 py-0.5", gap: "gap-0.5" },
  md: { icon: 14, text: "text-sm", padding: "px-2 py-0.5", gap: "gap-1" },
  lg: { icon: 18, text: "text-base", padding: "px-2.5 py-1", gap: "gap-1.5" },
};

// CrownBadge Component
type CrownType = "kom" | "qom" | "local_legend" | "course_record";

const crownConfig: Record<
  CrownType,
  { label: string; tooltip: string; colors: string }
> = {
  kom: {
    label: "KOM",
    tooltip: "King of the Mountain",
    colors: "bg-amber-100 text-amber-800 border-amber-300",
  },
  qom: {
    label: "QOM",
    tooltip: "Queen of the Mountain",
    colors: "bg-amber-100 text-amber-800 border-amber-300",
  },
  local_legend: {
    label: "Local Legend",
    tooltip: "Local Legend - Most efforts on this segment",
    colors: "bg-purple-100 text-purple-800 border-purple-300",
  },
  course_record: {
    label: "CR",
    tooltip: "Course Record",
    colors: "bg-emerald-100 text-emerald-800 border-emerald-300",
  },
};

export interface CrownBadgeProps {
  type: CrownType;
  size?: "sm" | "md" | "lg";
  showLabel?: boolean;
  className?: string;
}

export function CrownBadge({
  type,
  size = "md",
  showLabel = true,
  className,
}: CrownBadgeProps) {
  const config = crownConfig[type];
  const sizeValues = sizeConfig[size];

  const Icon =
    type === "local_legend"
      ? Star
      : type === "course_record"
        ? CheckCircle
        : Crown;

  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border font-semibold",
        sizeValues.padding,
        sizeValues.gap,
        sizeValues.text,
        config.colors,
        className
      )}
      title={config.tooltip}
    >
      <Icon size={sizeValues.icon} className="flex-shrink-0" />
      {showLabel && <span>{config.label}</span>}
    </span>
  );
}

// PRBadge Component
export interface PRBadgeProps {
  size?: "sm" | "md" | "lg";
  className?: string;
}

export function PRBadge({ size = "md", className }: PRBadgeProps) {
  const sizeValues = sizeConfig[size];

  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border font-semibold",
        "bg-green-100 text-green-800 border-green-300",
        sizeValues.padding,
        sizeValues.text,
        className
      )}
      title="Personal Record"
    >
      PR
    </span>
  );
}

// RankBadge Component
export interface RankBadgeProps {
  rank: number;
  size?: "sm" | "md" | "lg";
  className?: string;
}

const rankMedals: Record<number, { emoji: string; colors: string }> = {
  1: { emoji: "🥇", colors: "bg-amber-50 text-amber-900 border-amber-200" },
  2: { emoji: "🥈", colors: "bg-slate-50 text-slate-700 border-slate-200" },
  3: { emoji: "🥉", colors: "bg-orange-50 text-orange-800 border-orange-200" },
};

export function RankBadge({ rank, size = "md", className }: RankBadgeProps) {
  const sizeValues = sizeConfig[size];
  const medal = rankMedals[rank];

  if (medal) {
    return (
      <span
        className={cn(
          "inline-flex items-center justify-center rounded-full border font-semibold",
          sizeValues.padding,
          sizeValues.text,
          medal.colors,
          className
        )}
        title={`${rank}${rank === 1 ? "st" : rank === 2 ? "nd" : "rd"} place`}
      >
        {medal.emoji}
      </span>
    );
  }

  return (
    <span
      className={cn(
        "inline-flex items-center justify-center rounded-full border font-semibold",
        "bg-gray-100 text-gray-700 border-gray-300",
        sizeValues.padding,
        sizeValues.text,
        className
      )}
      title={`${rank}th place`}
    >
      {rank}
    </span>
  );
}
