"use client";

import { useCallback, useMemo } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";

// Filter types
export type LeaderboardScope = "all_time" | "year" | "month" | "week";
export type GenderFilter = "all" | "male" | "female";
export type AgeGroup = "all" | "18-24" | "25-34" | "35-44" | "45-54" | "55-64" | "65+";

export interface LeaderboardFilters {
  scope: LeaderboardScope;
  gender: GenderFilter;
  age_group: AgeGroup;
}

// Filter options with labels
const SCOPE_OPTIONS: { value: LeaderboardScope; label: string }[] = [
  { value: "all_time", label: "All Time" },
  { value: "year", label: "This Year" },
  { value: "month", label: "This Month" },
  { value: "week", label: "This Week" },
];

const GENDER_OPTIONS: { value: GenderFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "male", label: "Male" },
  { value: "female", label: "Female" },
];

const AGE_GROUP_OPTIONS: { value: AgeGroup; label: string }[] = [
  { value: "all", label: "All" },
  { value: "18-24", label: "18-24" },
  { value: "25-34", label: "25-34" },
  { value: "35-44", label: "35-44" },
  { value: "45-54", label: "45-54" },
  { value: "55-64", label: "55-64" },
  { value: "65+", label: "65+" },
];

const DEFAULT_FILTERS: LeaderboardFilters = {
  scope: "all_time",
  gender: "all",
  age_group: "all",
};

// Hook for managing leaderboard filters with URL persistence
export function useLeaderboardFilters() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();

  const filters: LeaderboardFilters = useMemo(() => {
    const scope = searchParams.get("scope") as LeaderboardScope | null;
    const gender = searchParams.get("gender") as GenderFilter | null;
    const ageGroup = searchParams.get("age_group") as AgeGroup | null;

    return {
      scope: scope && SCOPE_OPTIONS.some((o) => o.value === scope) ? scope : DEFAULT_FILTERS.scope,
      gender: gender && GENDER_OPTIONS.some((o) => o.value === gender) ? gender : DEFAULT_FILTERS.gender,
      age_group: ageGroup && AGE_GROUP_OPTIONS.some((o) => o.value === ageGroup) ? ageGroup : DEFAULT_FILTERS.age_group,
    };
  }, [searchParams]);

  const setFilters = useCallback(
    (newFilters: Partial<LeaderboardFilters>) => {
      const params = new URLSearchParams(searchParams.toString());
      const merged = { ...filters, ...newFilters };

      // Only set params that differ from defaults
      if (merged.scope !== DEFAULT_FILTERS.scope) {
        params.set("scope", merged.scope);
      } else {
        params.delete("scope");
      }

      if (merged.gender !== DEFAULT_FILTERS.gender) {
        params.set("gender", merged.gender);
      } else {
        params.delete("gender");
      }

      if (merged.age_group !== DEFAULT_FILTERS.age_group) {
        params.set("age_group", merged.age_group);
      } else {
        params.delete("age_group");
      }

      const queryString = params.toString();
      router.push(queryString ? `${pathname}?${queryString}` : pathname);
    },
    [filters, searchParams, router, pathname]
  );

  const resetFilters = useCallback(() => {
    router.push(pathname);
  }, [router, pathname]);

  return { filters, setFilters, resetFilters };
}

// Component props
interface LeaderboardFiltersProps {
  filters: LeaderboardFilters;
  onChange: (filters: Partial<LeaderboardFilters>) => void;
}

export function LeaderboardFiltersComponent({ filters, onChange }: LeaderboardFiltersProps) {
  return (
    <div className="flex flex-col sm:flex-row gap-4">
      {/* Time scope filter */}
      <div className="flex flex-col gap-1">
        <label htmlFor="scope-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
          Time Period
        </label>
        <select
          id="scope-filter"
          value={filters.scope}
          onChange={(e) => onChange({ scope: e.target.value as LeaderboardScope })}
          className="px-3 py-2 border rounded-md bg-background text-sm min-w-[140px]"
        >
          {SCOPE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      {/* Gender filter */}
      <div className="flex flex-col gap-1">
        <label htmlFor="gender-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
          Gender
        </label>
        <select
          id="gender-filter"
          value={filters.gender}
          onChange={(e) => onChange({ gender: e.target.value as GenderFilter })}
          className="px-3 py-2 border rounded-md bg-background text-sm min-w-[100px]"
        >
          {GENDER_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      {/* Age group filter */}
      <div className="flex flex-col gap-1">
        <label htmlFor="age-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
          Age Group
        </label>
        <select
          id="age-filter"
          value={filters.age_group}
          onChange={(e) => onChange({ age_group: e.target.value as AgeGroup })}
          className="px-3 py-2 border rounded-md bg-background text-sm min-w-[100px]"
        >
          {AGE_GROUP_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
