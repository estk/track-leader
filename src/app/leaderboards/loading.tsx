import { Skeleton } from "@/components/ui/skeleton";

export default function LeaderboardsLoading() {
  return (
    <div className="space-y-6">
      <Skeleton className="h-9 w-40" />
      <div className="flex gap-2">
        <Skeleton className="h-10 w-28" />
        <Skeleton className="h-10 w-28" />
      </div>
      <div className="rounded-lg border">
        <div className="p-4 border-b">
          <Skeleton className="h-6 w-48" />
          <Skeleton className="h-4 w-72 mt-2" />
        </div>
        <div className="divide-y">
          {Array.from({ length: 10 }).map((_, i) => (
            <div key={i} className="flex items-center gap-4 p-4">
              <Skeleton className="h-8 w-8 rounded-full" />
              <Skeleton className="h-5 w-32" />
              <Skeleton className="h-5 w-12 ml-auto" />
              <Skeleton className="h-5 w-12" />
              <Skeleton className="h-5 w-12" />
              <Skeleton className="h-5 w-16" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
