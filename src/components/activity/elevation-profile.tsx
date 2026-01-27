"use client";

import { useMemo, useRef } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { TrackPoint } from "@/lib/api";

interface ElevationProfileProps {
  points: TrackPoint[];
  onHover?: (index: number | null) => void;
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

export function ElevationProfile({ points, onHover }: ElevationProfileProps) {
  const lastHoveredIndex = useRef<number | null>(null);

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
        >
          <defs>
            <linearGradient id="elevationGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8} />
              <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.1} />
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
                // Only call onHover when index changes to avoid render loops
                if (lastHoveredIndex.current !== idx) {
                  lastHoveredIndex.current = idx;
                  setTimeout(() => onHover?.(idx), 0);
                }
                return (
                  <div className="bg-background border rounded-md p-2 shadow-md">
                    <p className="text-sm font-medium">
                      {data.elevation.toFixed(0)}m
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {data.distance.toFixed(2)} km
                    </p>
                  </div>
                );
              }
              // Clear hover when tooltip is not active
              if (!active && lastHoveredIndex.current !== null) {
                lastHoveredIndex.current = null;
                setTimeout(() => onHover?.(null), 0);
              }
              return null;
            }}
          />
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
