"use client";

import { useMemo } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceDot,
} from "recharts";
import type { SegmentEffort } from "@/lib/api";

interface ChartDataPoint {
  date: number;
  time: number;
  isPR: boolean;
  formattedDate: string;
}

interface PRHistoryChartProps {
  efforts: SegmentEffort[];
}

export function PRHistoryChart({ efforts }: PRHistoryChartProps) {
  const chartData: ChartDataPoint[] = useMemo(() => {
    return efforts
      .slice()
      .sort((a, b) => new Date(a.started_at).getTime() - new Date(b.started_at).getTime())
      .map((effort) => ({
        date: new Date(effort.started_at).getTime(),
        time: effort.elapsed_time_seconds,
        isPR: effort.is_personal_record,
        formattedDate: new Date(effort.started_at).toLocaleDateString(),
      }));
  }, [efforts]);

  const prPoints = chartData.filter((d) => d.isPR);

  const formatTooltipTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <div className="h-64">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
          <XAxis
            dataKey="date"
            type="number"
            domain={["dataMin", "dataMax"]}
            tickFormatter={(value) => new Date(value).toLocaleDateString(undefined, { month: "short", day: "numeric" })}
            className="text-xs fill-muted-foreground"
          />
          <YAxis
            dataKey="time"
            tickFormatter={formatTooltipTime}
            className="text-xs fill-muted-foreground"
            width={50}
          />
          <Tooltip
            labelFormatter={(value) => new Date(value as number).toLocaleDateString()}
            formatter={(value) => [formatTooltipTime(value as number), "Time"]}
            contentStyle={{
              backgroundColor: "hsl(var(--card))",
              border: "1px solid hsl(var(--border))",
              borderRadius: "var(--radius)",
            }}
          />
          <Line
            type="monotone"
            dataKey="time"
            stroke="hsl(var(--primary))"
            strokeWidth={2}
            dot={{ r: 4, fill: "hsl(var(--primary))" }}
            activeDot={{ r: 6 }}
          />
          {prPoints.map((point) => (
            <ReferenceDot
              key={point.date}
              x={point.date}
              y={point.time}
              r={8}
              fill="hsl(var(--chart-1))"
              stroke="hsl(var(--background))"
              strokeWidth={2}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
      <div className="flex justify-center gap-6 mt-2 text-sm text-muted-foreground">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-primary" />
          <span>Effort</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full" style={{ backgroundColor: "hsl(var(--chart-1))" }} />
          <span>Personal Record</span>
        </div>
      </div>
    </div>
  );
}
