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
import { TrackPoint } from "@/lib/api";

interface ElevationProfileProps {
  points: TrackPoint[];
  onHover?: (index: number | null) => void;
  selectionMode?: boolean;
  selectionStart?: number | null;
  selectionEnd?: number | null;
  onPointClick?: (index: number) => void;
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
      <ResponsiveContainer width="100%" height={200}>
        <AreaChart
          data={chartData}
          onMouseLeave={() => onHover?.(null)}
          onClick={() => {
            if (selectionMode && lastHoveredIndex.current !== null) {
              onPointClick?.(lastHoveredIndex.current);
            }
          }}
          style={{ cursor: selectionMode ? "crosshair" : "default" }}
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
            content={({ payload, active }) => {
              if (active && payload && payload[0]) {
                const data = payload[0].payload;
                const idx = data.originalIndex;
                // Schedule hover update for after render completes
                if (lastHoveredIndex.current !== idx) {
                  lastHoveredIndex.current = idx;
                  queueMicrotask(() => setCurrentHoverIndex(idx));
                }
                return (
                  <div className="bg-background border rounded-md p-2 shadow-md">
                    <p className="text-sm font-medium">
                      {data.elevation.toFixed(0)}m
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {data.distance.toFixed(2)} km
                    </p>
                    {selectionMode && (
                      <p className="text-xs text-green-600 mt-1">
                        Click to select
                      </p>
                    )}
                  </div>
                );
              }
              // Clear hover when tooltip is not active
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
          <Area
            type="monotone"
            dataKey="elevation"
            stroke="#3b82f6"
            fill="url(#elevationGradient)"
            strokeWidth={2}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
