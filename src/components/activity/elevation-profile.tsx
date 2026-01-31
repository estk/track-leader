"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  ReferenceArea,
  ReferenceLine,
} from "recharts";
import { TrackPoint, ACTIVITY_TYPE_IDS, getActivityTypeName } from "@/lib/api";

// Colors for activity type segments (distinct, accessible colors)
const ACTIVITY_TYPE_COLORS: Record<string, string> = {
  [ACTIVITY_TYPE_IDS.WALK]: "#22c55e",    // green
  [ACTIVITY_TYPE_IDS.RUN]: "#ef4444",     // red
  [ACTIVITY_TYPE_IDS.HIKE]: "#84cc16",    // lime
  [ACTIVITY_TYPE_IDS.ROAD]: "#3b82f6",    // blue
  [ACTIVITY_TYPE_IDS.MTB]: "#f97316",     // orange
  [ACTIVITY_TYPE_IDS.EMTB]: "#eab308",    // yellow
  [ACTIVITY_TYPE_IDS.GRAVEL]: "#a855f7",  // purple
  [ACTIVITY_TYPE_IDS.UNKNOWN]: "#6b7280", // gray
  [ACTIVITY_TYPE_IDS.DIG]: "#78716c",     // stone/brown for trail work
};

// Default color for unknown activity types
const DEFAULT_SEGMENT_COLOR = "#6b7280";

export function getActivityTypeColor(typeId: string): string {
  return ACTIVITY_TYPE_COLORS[typeId] || DEFAULT_SEGMENT_COLOR;
}

// Multi-range mode segment definition
export interface MultiRangeSegment {
  startIndex: number;      // Index into points array (first boundary = 0)
  endIndex: number;        // Index into points array (last boundary = points.length - 1)
  activityTypeId: string;  // Activity type UUID
}

interface ElevationProfileProps {
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
  // Index of boundary being dragged/selected (for visual feedback)
  selectedBoundaryIndex?: number | null;
}

function calculateDistance(
  lat1: number,
  lon1: number,
  lat2: number,
  lon2: number
): number {
  const R = 6371; // Earth's radius in km
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLon = ((lon2 - lon1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLon / 2) *
      Math.sin(dLon / 2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c;
}

export function ElevationProfile({
  points,
  onHover,
  selectionMode,
  selectionStart,
  selectionEnd,
  onPointClick,
  multiRangeMode,
  segments,
  onBoundaryClick,
  selectedBoundaryIndex,
}: ElevationProfileProps) {
  const lastHoveredIndex = useRef<number | null>(null);
  const [currentHoverIndex, setCurrentHoverIndex] = useState<number | null>(null);

  // Use effect to propagate hover state changes to parent
  useEffect(() => {
    onHover?.(currentHoverIndex);
  }, [currentHoverIndex, onHover]);

  const chartData = useMemo(() => {
    let cumulativeDistance = 0;
    const data: { distance: number; elevation: number; originalIndex: number }[] = [];

    for (let i = 0; i < points.length; i++) {
      const point = points[i];
      if (point.ele === null) continue;

      if (data.length > 0) {
        const prevPoint = points[data[data.length - 1].originalIndex];
        cumulativeDistance += calculateDistance(
          prevPoint.lat,
          prevPoint.lon,
          point.lat,
          point.lon
        );
      }

      data.push({
        distance: cumulativeDistance,
        elevation: point.ele,
        originalIndex: i,
      });
    }

    return data;
  }, [points]);

  const { minEle, maxEle, totalDistance, totalGain } = useMemo(() => {
    const elevations = chartData.map((d) => d.elevation);
    const min = Math.min(...elevations);
    const max = Math.max(...elevations);
    const total = chartData[chartData.length - 1]?.distance || 0;

    let gain = 0;
    for (let i = 1; i < chartData.length; i++) {
      const diff = chartData[i].elevation - chartData[i - 1].elevation;
      if (diff > 0) gain += diff;
    }

    return {
      minEle: min,
      maxEle: max,
      totalDistance: total,
      totalGain: gain,
    };
  }, [chartData]);

  // Find the distances for selection markers
  const selectionDistances = useMemo(() => {
    if (selectionStart === null && selectionEnd === null) return null;

    const startData = chartData.find((d) => d.originalIndex === selectionStart);
    const endData = chartData.find((d) => d.originalIndex === selectionEnd);

    return {
      start: startData?.distance ?? null,
      end: endData?.distance ?? null,
    };
  }, [chartData, selectionStart, selectionEnd]);

  // Convert segments (point indices) to distances for rendering
  const segmentDistances = useMemo(() => {
    if (!multiRangeMode || !segments || segments.length === 0) return null;

    const result = segments.map((segment, idx) => {
      // Find the chart data points for start and end indices
      const startData = chartData.find((d) => d.originalIndex >= segment.startIndex);
      const endData = [...chartData].reverse().find((d) => d.originalIndex <= segment.endIndex);

      const startDistance = startData?.distance ?? 0;
      const endDistance = endData?.distance ?? (chartData[chartData.length - 1]?.distance ?? 0);


      return {
        startDistance,
        endDistance,
        activityTypeId: segment.activityTypeId,
        color: getActivityTypeColor(segment.activityTypeId),
      };
    });

    return result;
  }, [chartData, multiRangeMode, segments]);

  // Extract boundary indices from segments (for rendering boundary lines)
  const boundaryIndices = useMemo(() => {
    if (!multiRangeMode || !segments || segments.length === 0) return [];

    // Collect all unique boundary indices (excluding start=0 and end=last)
    const indices = new Set<number>();
    for (const segment of segments) {
      // Add interior boundaries only (not the very first or very last)
      if (segment.startIndex > 0) {
        indices.add(segment.startIndex);
      }
      if (segment.endIndex < points.length - 1) {
        indices.add(segment.endIndex);
      }
    }
    return Array.from(indices).sort((a, b) => a - b);
  }, [multiRangeMode, segments, points.length]);

  // Convert boundary indices to distances
  const boundaryDistances = useMemo(() => {
    return boundaryIndices.map((idx) => {
      const data = chartData.find((d) => d.originalIndex >= idx);
      return {
        index: idx,
        distance: data?.distance ?? 0,
      };
    });
  }, [boundaryIndices, chartData]);

  // Get unique activity types for legend
  const legendItems = useMemo(() => {
    if (!multiRangeMode || !segments || segments.length === 0) return [];

    const seen = new Set<string>();
    const items: { typeId: string; color: string; name: string }[] = [];

    for (const seg of segments) {
      if (!seen.has(seg.activityTypeId)) {
        seen.add(seg.activityTypeId);
        items.push({
          typeId: seg.activityTypeId,
          color: getActivityTypeColor(seg.activityTypeId),
          name: getActivityTypeName(seg.activityTypeId),
        });
      }
    }

    return items;
  }, [multiRangeMode, segments]);

  if (chartData.length === 0) {
    return (
      <div className="h-[200px] flex items-center justify-center text-muted-foreground">
        No elevation data available
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2 md:gap-4 text-xs md:text-sm text-muted-foreground">
        <span>Distance: {totalDistance.toFixed(2)} km</span>
        <span>Gain: {totalGain.toFixed(0)} m</span>
        <span>
          Range: {minEle.toFixed(0)}m - {maxEle.toFixed(0)}m
        </span>
      </div>
      {/* Multi-range legend */}
      {legendItems.length > 0 && (
        <div className="flex flex-wrap gap-3 text-xs">
          {legendItems.map((item) => (
            <div key={item.typeId} className="flex items-center gap-1.5">
              <div
                className="w-3 h-3 rounded-sm"
                style={{ backgroundColor: item.color, opacity: 0.4 }}
              />
              <span className="text-muted-foreground">{item.name}</span>
            </div>
          ))}
        </div>
      )}
      <div className="[&_*]:outline-none [&_*]:focus:outline-none">
      <ResponsiveContainer width="100%" height={200}>
        <AreaChart
          data={chartData}
          onMouseLeave={() => {
            lastHoveredIndex.current = null;
            setCurrentHoverIndex(null);
            onHover?.(null);
          }}
          onClick={() => {
            if (selectionMode && lastHoveredIndex.current !== null) {
              onPointClick?.(lastHoveredIndex.current);
            }
            if (multiRangeMode && lastHoveredIndex.current !== null) {
              onBoundaryClick?.(lastHoveredIndex.current);
            }
          }}
          style={{ cursor: selectionMode || multiRangeMode ? "crosshair" : "default", outline: "none" }}
        >
          <defs>
            <linearGradient id="elevationGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8} />
              <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.1} />
            </linearGradient>
            <linearGradient id="selectionGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#22c55e" stopOpacity={0.6} />
              <stop offset="95%" stopColor="#22c55e" stopOpacity={0.2} />
            </linearGradient>
          </defs>
          <XAxis
            dataKey="distance"
            type="number"
            domain={[0, totalDistance]}
            tickFormatter={(val) => `${val.toFixed(1)} km`}
            stroke="#888888"
            fontSize={12}
          />
          <YAxis
            domain={[minEle - 50, maxEle + 50]}
            tickFormatter={(val) => `${val}m`}
            stroke="#888888"
            fontSize={12}
          />
          <Tooltip
            position={{ y: 0 }}
            offset={20}
            content={({ payload, active }) => {
              if (active && payload && payload[0]) {
                const data = payload[0].payload;
                const idx = data.originalIndex;
                if (lastHoveredIndex.current !== idx) {
                  lastHoveredIndex.current = idx;
                  queueMicrotask(() => setCurrentHoverIndex(idx));
                }
                return (
                  <div className="bg-background/90 border rounded-md px-2 py-1 shadow-sm text-xs">
                    <span className="font-medium">{data.elevation.toFixed(0)}m</span>
                    <span className="text-muted-foreground ml-2">{data.distance.toFixed(2)} km</span>
                  </div>
                );
              }
              if (!active && lastHoveredIndex.current !== null) {
                lastHoveredIndex.current = null;
                queueMicrotask(() => setCurrentHoverIndex(null));
              }
              return null;
            }}
          />
          {/* Selection range highlight */}
          {selectionDistances &&
            selectionDistances.start !== null &&
            selectionDistances.end !== null && (
              <ReferenceArea
                x1={Math.min(selectionDistances.start, selectionDistances.end)}
                x2={Math.max(selectionDistances.start, selectionDistances.end)}
                fill="#22c55e"
                fillOpacity={0.3}
                stroke="#22c55e"
                strokeOpacity={0.8}
              />
            )}
          {/* Start marker */}
          {selectionDistances && selectionDistances.start !== null && (
            <ReferenceLine
              x={selectionDistances.start}
              stroke="#22c55e"
              strokeWidth={2}
              label={{ value: "Start", position: "top", fill: "#22c55e", fontSize: 12 }}
            />
          )}
          {/* End marker */}
          {selectionDistances && selectionDistances.end !== null && (
            <ReferenceLine
              x={selectionDistances.end}
              stroke="#ef4444"
              strokeWidth={2}
              label={{ value: "End", position: "top", fill: "#ef4444", fontSize: 12 }}
            />
          )}
          {/* Main elevation area - rendered first so segments appear on top */}
          <Area
            type="monotone"
            dataKey="elevation"
            stroke="#3b82f6"
            fill="url(#elevationGradient)"
            strokeWidth={2}
          />
          {/* Multi-range segment backgrounds - subtle tint over the elevation area */}
          {segmentDistances?.map((seg, idx) => (
            <ReferenceArea
              key={`segment-${idx}`}
              x1={seg.startDistance}
              x2={seg.endDistance}
              fill={seg.color}
              fillOpacity={0.15}
              stroke="none"
            />
          ))}
          {/* Multi-range boundary lines */}
          {boundaryDistances.map((boundary, idx) => {
            const isSelected = selectedBoundaryIndex !== null &&
              boundaryIndices.indexOf(boundary.index) === selectedBoundaryIndex;
            return (
              <ReferenceLine
                key={`boundary-${idx}`}
                x={boundary.distance}
                stroke={isSelected ? "#3b82f6" : "#1f2937"}
                strokeWidth={isSelected ? 3 : 2}
                strokeDasharray={isSelected ? "none" : "5 3"}
              />
            );
          })}
        </AreaChart>
      </ResponsiveContainer>
      </div>
      {/* Instructions for interactive modes */}
      {multiRangeMode && (
        <p className="text-xs text-muted-foreground">
          Click on the chart to add or remove segment boundaries
        </p>
      )}
      {selectionMode && (
        <p className="text-xs text-muted-foreground">
          Click to select start and end points
        </p>
      )}
    </div>
  );
}
