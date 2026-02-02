"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { api, StoppedSegment } from "@/lib/api";

interface DigTaggingModalProps {
  activityId: string;
  stoppedSegments: StoppedSegment[];
  onComplete: () => void;
  onSkip: () => void;
}

function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  if (minutes === 0) {
    return `${remainingSeconds}s`;
  }
  return `${minutes}m ${remainingSeconds}s`;
}

function formatTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function DigTaggingModal({
  activityId,
  stoppedSegments,
  onComplete,
  onSkip,
}: DigTaggingModalProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggleSelection = (id: string) => {
    setSelectedIds((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  };

  const selectAll = () => {
    setSelectedIds(new Set(stoppedSegments.map((s) => s.id)));
  };

  const selectNone = () => {
    setSelectedIds(new Set());
  };

  const handleSave = async () => {
    if (selectedIds.size === 0) {
      onSkip();
      return;
    }

    setSaving(true);
    setError(null);

    try {
      await api.createDigParts(activityId, Array.from(selectedIds));
      onComplete();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save dig tags");
    } finally {
      setSaving(false);
    }
  };

  const totalSelectedDuration = stoppedSegments
    .filter((s) => selectedIds.has(s.id))
    .reduce((sum, s) => sum + s.duration_seconds, 0);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <Card className="w-full max-w-lg mx-4 max-h-[80vh] flex flex-col">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="currentColor"
              className="w-5 h-5 text-amber-600"
            >
              <path
                fillRule="evenodd"
                d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25zM12.75 6a.75.75 0 00-1.5 0v6c0 .414.336.75.75.75h4.5a.75.75 0 000-1.5h-3.75V6z"
                clipRule="evenodd"
              />
            </svg>
            Trail Maintenance Detected
          </CardTitle>
          <CardDescription>
            We detected {stoppedSegments.length} stopped segment
            {stoppedSegments.length !== 1 ? "s" : ""} during your activity.
            Select any that represent trail maintenance (&ldquo;dig time&rdquo;).
          </CardDescription>
        </CardHeader>
        <CardContent className="flex-1 overflow-hidden flex flex-col gap-4">
          {error && (
            <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md">
              {error}
            </div>
          )}

          <div className="flex items-center justify-between text-sm">
            <div className="text-muted-foreground">
              {selectedIds.size} of {stoppedSegments.length} selected
              {selectedIds.size > 0 && (
                <span className="ml-2 font-medium text-amber-600">
                  ({formatDuration(totalSelectedDuration)} dig time)
                </span>
              )}
            </div>
            <div className="flex gap-2">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={selectAll}
              >
                Select All
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={selectNone}
              >
                Clear
              </Button>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto space-y-2 pr-1">
            {stoppedSegments.map((segment) => (
              <button
                key={segment.id}
                type="button"
                onClick={() => toggleSelection(segment.id)}
                className={`w-full text-left p-3 rounded-lg border-2 transition-colors ${
                  selectedIds.has(segment.id)
                    ? "border-amber-500 bg-amber-50 dark:bg-amber-950/30"
                    : "border-muted hover:border-muted-foreground/30"
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-5 h-5 rounded border-2 flex items-center justify-center ${
                        selectedIds.has(segment.id)
                          ? "border-amber-500 bg-amber-500 text-white"
                          : "border-muted-foreground/30"
                      }`}
                    >
                      {selectedIds.has(segment.id) && (
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          viewBox="0 0 20 20"
                          fill="currentColor"
                          className="w-3 h-3"
                        >
                          <path
                            fillRule="evenodd"
                            d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z"
                            clipRule="evenodd"
                          />
                        </svg>
                      )}
                    </div>
                    <div>
                      <div className="font-medium">
                        {formatTime(segment.start_time)} -{" "}
                        {formatTime(segment.end_time)}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        Stopped for {formatDuration(segment.duration_seconds)}
                      </div>
                    </div>
                  </div>
                </div>
              </button>
            ))}
          </div>

          <div className="flex gap-3 pt-2 border-t">
            <Button
              type="button"
              variant="outline"
              className="flex-1"
              onClick={onSkip}
              disabled={saving}
            >
              Skip
            </Button>
            <Button
              type="button"
              className="flex-1 bg-amber-600 hover:bg-amber-700"
              onClick={handleSave}
              disabled={saving}
            >
              {saving
                ? "Saving..."
                : selectedIds.size > 0
                  ? `Save ${selectedIds.size} Dig Tag${selectedIds.size !== 1 ? "s" : ""}`
                  : "Skip"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
