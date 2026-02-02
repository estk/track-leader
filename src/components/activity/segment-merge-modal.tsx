"use client";

import { useMemo } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getActivityTypeName } from "@/lib/api";
import { getActivityTypeColor } from "@/components/activity/elevation-profile";

interface SegmentInfo {
  index: number;
  activityTypeId: string;
  startIndex: number;
  endIndex: number;
}

interface MergeGroup {
  startSegmentIndex: number;
  endSegmentIndex: number;
  activityTypeId: string;
  segmentCount: number;
}

interface SegmentMergeModalProps {
  segments: SegmentInfo[];
  onMerge: () => void;
  onKeepAsIs: () => void;
  onEdit: () => void;
}

function formatSegmentRange(startIndex: number, endIndex: number, totalPoints: number): string {
  const startPercent = Math.round((startIndex / totalPoints) * 100);
  const endPercent = Math.round((endIndex / totalPoints) * 100);
  return `${startPercent}% - ${endPercent}%`;
}

export function SegmentMergeModal({
  segments,
  onMerge,
  onKeepAsIs,
  onEdit,
}: SegmentMergeModalProps) {
  // Find groups of adjacent segments with the same activity type
  const mergeGroups = useMemo((): MergeGroup[] => {
    const groups: MergeGroup[] = [];
    let currentGroup: MergeGroup | null = null;

    for (let i = 0; i < segments.length; i++) {
      const segment = segments[i];

      if (currentGroup && currentGroup.activityTypeId === segment.activityTypeId) {
        // Extend the current group
        currentGroup.endSegmentIndex = i;
        currentGroup.segmentCount++;
      } else {
        // Start a new group if we have a pending group with multiple segments
        if (currentGroup && currentGroup.segmentCount > 1) {
          groups.push(currentGroup);
        }
        // Start tracking a new potential group
        currentGroup = {
          startSegmentIndex: i,
          endSegmentIndex: i,
          activityTypeId: segment.activityTypeId,
          segmentCount: 1,
        };
      }
    }

    // Don't forget the last group
    if (currentGroup && currentGroup.segmentCount > 1) {
      groups.push(currentGroup);
    }

    return groups;
  }, [segments]);

  // If no merge groups found, this modal shouldn't have been opened
  if (mergeGroups.length === 0) {
    return null;
  }

  const totalPoints = segments.length > 0
    ? Math.max(...segments.map(s => s.endIndex))
    : 0;

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
                d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25zm-1.72 6.97a.75.75 0 10-1.06 1.06L10.94 12l-1.72 1.72a.75.75 0 101.06 1.06L12 13.06l1.72 1.72a.75.75 0 101.06-1.06L13.06 12l1.72-1.72a.75.75 0 10-1.06-1.06L12 10.94l-1.72-1.72z"
                clipRule="evenodd"
              />
            </svg>
            Combine Similar Segments?
          </CardTitle>
          <CardDescription>
            We noticed some adjacent segments with the same activity type that could be combined
            into single segments.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex-1 overflow-hidden flex flex-col gap-4">
          <div className="flex-1 overflow-y-auto space-y-4 pr-1">
            {mergeGroups.map((group, groupIndex) => {
              const activityTypeName = getActivityTypeName(group.activityTypeId);
              const color = getActivityTypeColor(group.activityTypeId);

              // Get the segments in this group
              const groupSegments = segments.slice(
                group.startSegmentIndex,
                group.endSegmentIndex + 1
              );

              return (
                <div
                  key={groupIndex}
                  className="rounded-lg border-2 border-amber-500/50 bg-amber-50 dark:bg-amber-950/30 p-3"
                >
                  <div className="flex items-center gap-2 mb-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: color }}
                    />
                    <span className="font-medium text-amber-800 dark:text-amber-200">
                      {group.segmentCount} adjacent {activityTypeName} segments
                    </span>
                  </div>
                  <div className="space-y-1">
                    {groupSegments.map((segment, i) => (
                      <div
                        key={i}
                        className="text-sm text-muted-foreground flex items-center gap-2"
                      >
                        <span className="text-xs bg-muted px-1.5 py-0.5 rounded">
                          Segment {segment.index + 1}
                        </span>
                        <span>
                          {formatSegmentRange(segment.startIndex, segment.endIndex, totalPoints)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>

          <div className="text-sm text-muted-foreground">
            <p>
              <strong>Merge</strong> will combine adjacent same-type segments into single segments.
            </p>
          </div>

          <div className="flex gap-3 pt-2 border-t">
            <Button
              type="button"
              variant="ghost"
              className="flex-1"
              onClick={onEdit}
            >
              Edit Segments
            </Button>
            <Button
              type="button"
              variant="outline"
              className="flex-1"
              onClick={onKeepAsIs}
            >
              Keep As-Is
            </Button>
            <Button
              type="button"
              className="flex-1 bg-amber-600 hover:bg-amber-700"
              onClick={onMerge}
            >
              Merge
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

/**
 * Detect groups of adjacent segments that have the same activity type.
 * Returns an array of merge groups, each containing indices of segments to merge.
 */
export function detectMergeableSegments(segmentTypes: string[]): MergeGroup[] {
  const groups: MergeGroup[] = [];
  let currentGroup: MergeGroup | null = null;

  for (let i = 0; i < segmentTypes.length; i++) {
    const typeId = segmentTypes[i];

    if (currentGroup && currentGroup.activityTypeId === typeId) {
      // Extend the current group
      currentGroup.endSegmentIndex = i;
      currentGroup.segmentCount++;
    } else {
      // Save the previous group if it had multiple segments
      if (currentGroup && currentGroup.segmentCount > 1) {
        groups.push(currentGroup);
      }
      // Start a new potential group
      currentGroup = {
        startSegmentIndex: i,
        endSegmentIndex: i,
        activityTypeId: typeId,
        segmentCount: 1,
      };
    }
  }

  // Don't forget the last group
  if (currentGroup && currentGroup.segmentCount > 1) {
    groups.push(currentGroup);
  }

  return groups;
}

/**
 * Merge adjacent same-type segments by removing intermediate boundaries.
 * Returns new boundary indices and segment types arrays.
 */
export function mergeAdjacentSegments(
  boundaryIndices: number[],
  segmentTypes: string[]
): { boundaryIndices: number[]; segmentTypes: string[] } {
  // Find groups of adjacent same-type segments
  const mergeGroups = detectMergeableSegments(segmentTypes);

  if (mergeGroups.length === 0) {
    return { boundaryIndices, segmentTypes };
  }

  // Build sets of boundary indices and segment indices to remove
  const boundariesToRemove = new Set<number>();
  const segmentsToRemove = new Set<number>();

  for (const group of mergeGroups) {
    // For a group spanning segments [start, end], we need to:
    // - Remove boundaries between start and end (these are at indices start through end-1 in boundaryIndices)
    // - Keep only the first segment type, remove the rest
    for (let i = group.startSegmentIndex; i < group.endSegmentIndex; i++) {
      boundariesToRemove.add(i);
      segmentsToRemove.add(i + 1); // Remove all but the first segment in group
    }
  }

  // Filter out the boundaries and segments to remove
  const newBoundaryIndices = boundaryIndices.filter((_, i) => !boundariesToRemove.has(i));
  const newSegmentTypes = segmentTypes.filter((_, i) => !segmentsToRemove.has(i));

  return {
    boundaryIndices: newBoundaryIndices,
    segmentTypes: newSegmentTypes,
  };
}
