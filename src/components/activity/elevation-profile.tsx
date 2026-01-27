"use client";

import { useMemo } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
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
  const chartData = useMemo(() => {
    let cumulativeDistance = 0;
    return points
      .filter((p) => p.ele !== null)
      .map((point, i, arr) => {
        if (i > 0) {
          cumulativeDistance += calculateDistance(
            arr[i - 1].lat,
            arr[i - 1].lon,
            point.lat,
            point.lon
          );
        }
        return {
          distance: cumulativeDistance,
          elevation: point.ele!,
          index: i,
        };
      });
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
          onMouseMove={(e) => {
            const payload = e as { activePayload?: { payload: { index: number } }[] };
            if (payload.activePayload && payload.activePayload[0]) {
              onHover?.(payload.activePayload[0].payload.index);
            }
          }}
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
            content={({ payload }) => {
              if (payload && payload[0]) {
                const data = payload[0].payload;
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
