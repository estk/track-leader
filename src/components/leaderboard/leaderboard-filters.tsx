"use client";

import { useCallback, useMemo, useEffect, useState } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import { api, CountryStats } from "@/lib/api";

// Filter types
export type LeaderboardScope = "all_time" | "year" | "month" | "week";
export type GenderFilter = "all" | "male" | "female";
export type AgeGroup = "all" | "18-24" | "25-29" | "30-34" | "35-39" | "40-49" | "50-59" | "60+";
export type WeightClass = "all" | "featherweight" | "lightweight" | "welterweight" | "middleweight" | "cruiserweight" | "heavyweight";

export interface LeaderboardFilters {
  scope: LeaderboardScope;
  gender: GenderFilter;
  age_group: AgeGroup;
  weight_class: WeightClass;
  country: string | null;
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
  { value: "25-29", label: "25-29" },
  { value: "30-34", label: "30-34" },
  { value: "35-39", label: "35-39" },
  { value: "40-49", label: "40-49" },
  { value: "50-59", label: "50-59" },
  { value: "60+", label: "60+" },
];

const WEIGHT_CLASS_OPTIONS: { value: WeightClass; label: string }[] = [
  { value: "all", label: "All" },
  { value: "featherweight", label: "Featherweight (<55 kg)" },
  { value: "lightweight", label: "Lightweight (55-64 kg)" },
  { value: "welterweight", label: "Welterweight (65-74 kg)" },
  { value: "middleweight", label: "Middleweight (75-84 kg)" },
  { value: "cruiserweight", label: "Cruiserweight (85-94 kg)" },
  { value: "heavyweight", label: "Heavyweight (95+ kg)" },
];

const DEFAULT_FILTERS: LeaderboardFilters = {
  scope: "all_time",
  gender: "all",
  age_group: "all",
  weight_class: "all",
  country: null,
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
    const weightClass = searchParams.get("weight_class") as WeightClass | null;
    const country = searchParams.get("country");

    return {
      scope: scope && SCOPE_OPTIONS.some((o) => o.value === scope) ? scope : DEFAULT_FILTERS.scope,
      gender: gender && GENDER_OPTIONS.some((o) => o.value === gender) ? gender : DEFAULT_FILTERS.gender,
      age_group: ageGroup && AGE_GROUP_OPTIONS.some((o) => o.value === ageGroup) ? ageGroup : DEFAULT_FILTERS.age_group,
      weight_class: weightClass && WEIGHT_CLASS_OPTIONS.some((o) => o.value === weightClass) ? weightClass : DEFAULT_FILTERS.weight_class,
      country: country || DEFAULT_FILTERS.country,
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

      if (merged.weight_class !== DEFAULT_FILTERS.weight_class) {
        params.set("weight_class", merged.weight_class);
      } else {
        params.delete("weight_class");
      }

      if (merged.country !== DEFAULT_FILTERS.country && merged.country !== null) {
        params.set("country", merged.country);
      } else {
        params.delete("country");
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
  const [countries, setCountries] = useState<CountryStats[]>([]);
  const [countriesLoading, setCountriesLoading] = useState(true);

  useEffect(() => {
    api.getCountries()
      .then(setCountries)
      .catch(() => setCountries([]))
      .finally(() => setCountriesLoading(false));
  }, []);

  return (
    <div className="flex flex-col gap-4">
      {/* Primary row: Time, Gender, Age */}
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

      {/* Secondary row: Weight, Country */}
      <div className="flex flex-col sm:flex-row gap-4">
        {/* Weight class filter */}
        <div className="flex flex-col gap-1">
          <label htmlFor="weight-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
            Weight Class
          </label>
          <select
            id="weight-filter"
            value={filters.weight_class}
            onChange={(e) => onChange({ weight_class: e.target.value as WeightClass })}
            className="px-3 py-2 border rounded-md bg-background text-sm min-w-[180px]"
          >
            {WEIGHT_CLASS_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* Country filter */}
        <div className="flex flex-col gap-1">
          <label htmlFor="country-filter" className="text-xs text-muted-foreground uppercase tracking-wide">
            Country
          </label>
          <select
            id="country-filter"
            value={filters.country || ""}
            onChange={(e) => onChange({ country: e.target.value || null })}
            className="px-3 py-2 border rounded-md bg-background text-sm min-w-[200px]"
            disabled={countriesLoading}
          >
            <option value="">All Countries</option>
            {countries.map((c) => (
              <option key={c.country} value={c.country}>
                {c.country} ({c.user_count.toLocaleString()})
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
