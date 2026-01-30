"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { SensorData } from "@/lib/api";

interface SensorGraphsProps {
  sensorData: SensorData;
  onHover?: (distanceKm: number | null) => void;
}

// Sensor type colors
const SENSOR_COLORS = {
  heartRate: "#ef4444", // red
  cadence: "#3b82f6", // blue
  power: "#eab308", // yellow/gold
  temperature: "#22c55e", // green
};

interface ChartDataPoint {
  distanceKm: number;
  heartRate?: number | null;
  cadence?: number | null;
  power?: number | null;
  temperature?: number | null;
}

export function SensorGraphs({ sensorData, onHover }: SensorGraphsProps) {
  const lastHoveredDistance = useRef<number | null>(null);
  const [currentHoverDistance, setCurrentHoverDistance] = useState<number | null>(null);

  // Use effect to propagate hover state changes to parent
  useEffect(() => {
    onHover?.(currentHoverDistance);
  }, [currentHoverDistance, onHover]);

  // Transform data into chart format
  const chartData = useMemo(() => {
    const data: ChartDataPoint[] = [];

    for (let i = 0; i < sensorData.distances.length; i++) {
      const point: ChartDataPoint = {
        distanceKm: sensorData.distances[i] / 1000, // Convert meters to km
      };

      if (sensorData.heart_rates) {
        point.heartRate = sensorData.heart_rates[i];
      }
      if (sensorData.cadences) {
        point.cadence = sensorData.cadences[i];
      }
      if (sensorData.powers) {
        point.power = sensorData.powers[i];
      }
      if (sensorData.temperatures) {
        point.temperature = sensorData.temperatures[i];
      }

      data.push(point);
    }

    return data;
  }, [sensorData]);

  // Calculate stats for each metric
  const stats = useMemo(() => {
    const result: {
      heartRate?: { avg: number; max: number };
      cadence?: { avg: number; max: number };
      power?: { avg: number; max: number };
      temperature?: { avg: number; max: number; min: number };
    } = {};

    if (sensorData.has_heart_rate && sensorData.heart_rates) {
      const values = sensorData.heart_rates.filter((v): v is number => v !== null);
      if (values.length > 0) {
        result.heartRate = {
          avg: Math.round(values.reduce((a, b) => a + b, 0) / values.length),
          max: Math.max(...values),
        };
      }
    }

    if (sensorData.has_cadence && sensorData.cadences) {
      const values = sensorData.cadences.filter((v): v is number => v !== null);
      if (values.length > 0) {
        result.cadence = {
          avg: Math.round(values.reduce((a, b) => a + b, 0) / values.length),
          max: Math.max(...values),
        };
      }
    }

    if (sensorData.has_power && sensorData.powers) {
      const values = sensorData.powers.filter((v): v is number => v !== null);
      if (values.length > 0) {
        result.power = {
          avg: Math.round(values.reduce((a, b) => a + b, 0) / values.length),
          max: Math.max(...values),
        };
      }
    }

    if (sensorData.has_temperature && sensorData.temperatures) {
      const values = sensorData.temperatures.filter((v): v is number => v !== null);
      if (values.length > 0) {
        result.temperature = {
          avg: Math.round(values.reduce((a, b) => a + b, 0) / values.length * 10) / 10,
          max: Math.max(...values),
          min: Math.min(...values),
        };
      }
    }

    return result;
  }, [sensorData]);

  // Determine which metrics to show
  const hasAnyData =
    sensorData.has_heart_rate ||
    sensorData.has_cadence ||
    sensorData.has_power ||
    sensorData.has_temperature;

  if (!hasAnyData || chartData.length === 0) {
    return (
      <div className="h-[200px] flex items-center justify-center text-muted-foreground">
        No sensor data available
      </div>
    );
  }

  // Calculate domain for Y axes
  const hrDomain = stats.heartRate
    ? [Math.max(0, stats.heartRate.avg - 50), stats.heartRate.max + 10]
    : [0, 200];

  const cadenceDomain = stats.cadence
    ? [0, stats.cadence.max + 20]
    : [0, 120];

  const powerDomain = stats.power
    ? [0, stats.power.max + 50]
    : [0, 400];

  // Helper to handle hover state from tooltip
  const handleTooltipActive = (distanceKm: number) => {
    if (lastHoveredDistance.current !== distanceKm) {
      lastHoveredDistance.current = distanceKm;
      queueMicrotask(() => setCurrentHoverDistance(distanceKm));
    }
  };

  const handleTooltipInactive = () => {
    if (lastHoveredDistance.current !== null) {
      lastHoveredDistance.current = null;
      queueMicrotask(() => setCurrentHoverDistance(null));
    }
  };

  return (
    <div className="space-y-4">
      {/* Stats summary */}
      <div className="flex flex-wrap gap-4 text-sm">
        {stats.heartRate && (
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full" style={{ backgroundColor: SENSOR_COLORS.heartRate }} />
            <span className="text-muted-foreground">
              HR: <span className="font-medium text-foreground">{stats.heartRate.avg} avg</span> / {stats.heartRate.max} max bpm
            </span>
          </div>
        )}
        {stats.cadence && (
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full" style={{ backgroundColor: SENSOR_COLORS.cadence }} />
            <span className="text-muted-foreground">
              Cadence: <span className="font-medium text-foreground">{stats.cadence.avg} avg</span> / {stats.cadence.max} max rpm
            </span>
          </div>
        )}
        {stats.power && (
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full" style={{ backgroundColor: SENSOR_COLORS.power }} />
            <span className="text-muted-foreground">
              Power: <span className="font-medium text-foreground">{stats.power.avg} avg</span> / {stats.power.max} max W
            </span>
          </div>
        )}
        {stats.temperature && (
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full" style={{ backgroundColor: SENSOR_COLORS.temperature }} />
            <span className="text-muted-foreground">
              Temp: {stats.temperature.min}&deg;C - {stats.temperature.max}&deg;C
            </span>
          </div>
        )}
      </div>

      {/* Heart Rate Graph */}
      {sensorData.has_heart_rate && (
        <div className="space-y-1">
          <h4 className="text-sm font-medium">Heart Rate</h4>
          <ResponsiveContainer width="100%" height={150}>
            <LineChart data={chartData}>
              <XAxis
                dataKey="distanceKm"
                tickFormatter={(val) => `${val.toFixed(1)}`}
                stroke="#888888"
                fontSize={10}
              />
              <YAxis
                domain={hrDomain}
                tickFormatter={(val) => `${val}`}
                stroke="#888888"
                fontSize={10}
                width={40}
              />
              <Tooltip
                content={({ payload, active }) => {
                  if (active && payload && payload[0]) {
                    const data = payload[0].payload;
                    handleTooltipActive(data.distanceKm);
                    return (
                      <div className="bg-background border rounded-md p-2 shadow-md">
                        <p className="text-sm font-medium" style={{ color: SENSOR_COLORS.heartRate }}>
                          {data.heartRate ?? "-"} bpm
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {data.distanceKm.toFixed(2)} km
                        </p>
                      </div>
                    );
                  }
                  handleTooltipInactive();
                  return null;
                }}
              />
              <Line
                type="monotone"
                dataKey="heartRate"
                stroke={SENSOR_COLORS.heartRate}
                strokeWidth={1.5}
                dot={false}
                connectNulls
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Cadence Graph */}
      {sensorData.has_cadence && (
        <div className="space-y-1">
          <h4 className="text-sm font-medium">Cadence</h4>
          <ResponsiveContainer width="100%" height={150}>
            <LineChart data={chartData}>
              <XAxis
                dataKey="distanceKm"
                tickFormatter={(val) => `${val.toFixed(1)}`}
                stroke="#888888"
                fontSize={10}
              />
              <YAxis
                domain={cadenceDomain}
                tickFormatter={(val) => `${val}`}
                stroke="#888888"
                fontSize={10}
                width={40}
              />
              <Tooltip
                content={({ payload, active }) => {
                  if (active && payload && payload[0]) {
                    const data = payload[0].payload;
                    handleTooltipActive(data.distanceKm);
                    return (
                      <div className="bg-background border rounded-md p-2 shadow-md">
                        <p className="text-sm font-medium" style={{ color: SENSOR_COLORS.cadence }}>
                          {data.cadence ?? "-"} rpm
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {data.distanceKm.toFixed(2)} km
                        </p>
                      </div>
                    );
                  }
                  handleTooltipInactive();
                  return null;
                }}
              />
              <Line
                type="monotone"
                dataKey="cadence"
                stroke={SENSOR_COLORS.cadence}
                strokeWidth={1.5}
                dot={false}
                connectNulls
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Power Graph */}
      {sensorData.has_power && (
        <div className="space-y-1">
          <h4 className="text-sm font-medium">Power</h4>
          <ResponsiveContainer width="100%" height={150}>
            <LineChart data={chartData}>
              <XAxis
                dataKey="distanceKm"
                tickFormatter={(val) => `${val.toFixed(1)}`}
                stroke="#888888"
                fontSize={10}
              />
              <YAxis
                domain={powerDomain}
                tickFormatter={(val) => `${val}`}
                stroke="#888888"
                fontSize={10}
                width={40}
              />
              <Tooltip
                content={({ payload, active }) => {
                  if (active && payload && payload[0]) {
                    const data = payload[0].payload;
                    handleTooltipActive(data.distanceKm);
                    return (
                      <div className="bg-background border rounded-md p-2 shadow-md">
                        <p className="text-sm font-medium" style={{ color: SENSOR_COLORS.power }}>
                          {data.power ?? "-"} W
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {data.distanceKm.toFixed(2)} km
                        </p>
                      </div>
                    );
                  }
                  handleTooltipInactive();
                  return null;
                }}
              />
              <Line
                type="monotone"
                dataKey="power"
                stroke={SENSOR_COLORS.power}
                strokeWidth={1.5}
                dot={false}
                connectNulls
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Temperature Graph */}
      {sensorData.has_temperature && (
        <div className="space-y-1">
          <h4 className="text-sm font-medium">Temperature</h4>
          <ResponsiveContainer width="100%" height={150}>
            <LineChart data={chartData}>
              <XAxis
                dataKey="distanceKm"
                tickFormatter={(val) => `${val.toFixed(1)}`}
                stroke="#888888"
                fontSize={10}
              />
              <YAxis
                tickFormatter={(val) => `${val}`}
                stroke="#888888"
                fontSize={10}
                width={40}
              />
              <Tooltip
                content={({ payload, active }) => {
                  if (active && payload && payload[0]) {
                    const data = payload[0].payload;
                    handleTooltipActive(data.distanceKm);
                    return (
                      <div className="bg-background border rounded-md p-2 shadow-md">
                        <p className="text-sm font-medium" style={{ color: SENSOR_COLORS.temperature }}>
                          {data.temperature?.toFixed(1) ?? "-"}&deg;C
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {data.distanceKm.toFixed(2)} km
                        </p>
                      </div>
                    );
                  }
                  handleTooltipInactive();
                  return null;
                }}
              />
              <Line
                type="monotone"
                dataKey="temperature"
                stroke={SENSOR_COLORS.temperature}
                strokeWidth={1.5}
                dot={false}
                connectNulls
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}
