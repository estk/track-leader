"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LazyDigHeatmap } from "@/components/maps/lazy-dig-heatmap";
import { Shovel } from "lucide-react";

export default function GlobalDigHeatmapPage() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-xl flex items-center gap-2">
            <Shovel className="h-6 w-6" />
            Global Dig Heat Map
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            See where trail digging and maintenance work has occurred across all activities
          </p>
        </CardHeader>
        <CardContent>
          <LazyDigHeatmap />
        </CardContent>
      </Card>
    </div>
  );
}
