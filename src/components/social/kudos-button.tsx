"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";

interface KudosButtonProps {
  activityId: string;
  initialHasGiven: boolean;
  initialCount: number;
  disabled?: boolean;
  onKudosChange?: (hasGiven: boolean, count: number) => void;
}

export function KudosButton({
  activityId,
  initialHasGiven,
  initialCount,
  disabled = false,
  onKudosChange,
}: KudosButtonProps) {
  const [hasGiven, setHasGiven] = useState(initialHasGiven);
  const [count, setCount] = useState(initialCount);
  const [loading, setLoading] = useState(false);

  const handleToggle = async () => {
    if (disabled) return;

    setLoading(true);
    try {
      if (hasGiven) {
        await api.removeKudos(activityId);
        setHasGiven(false);
        setCount((c) => Math.max(0, c - 1));
        onKudosChange?.(false, count - 1);
      } else {
        await api.giveKudos(activityId);
        setHasGiven(true);
        setCount((c) => c + 1);
        onKudosChange?.(true, count + 1);
      }
    } catch {
      // Revert on error
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={handleToggle}
      disabled={loading || disabled}
      className={cn(
        "gap-1",
        hasGiven && "text-orange-500 hover:text-orange-600"
      )}
    >
      <span className={cn("text-lg", hasGiven && "animate-pulse")}>
        {hasGiven ? "👏" : "👋"}
      </span>
      <span>{count}</span>
    </Button>
  );
}
