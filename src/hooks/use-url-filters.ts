"use client";

import { useRouter, useSearchParams, usePathname } from "next/navigation";
import { useCallback, useMemo } from "react";

export type FilterValues = { [key: string]: string | undefined };

export function useUrlFilters<T extends FilterValues>(
  defaultValues: T
): [T, (updates: Partial<T>) => void, () => void] {
  const router = useRouter();
  const searchParams = useSearchParams();
  const pathname = usePathname();

  const filters = useMemo(() => {
    const result = { ...defaultValues } as T;
    for (const key of Object.keys(defaultValues)) {
      const value = searchParams.get(key);
      if (value !== null) {
        (result as FilterValues)[key] = value;
      }
    }
    return result;
  }, [searchParams, defaultValues]);

  const setFilters = useCallback(
    (updates: Partial<T>) => {
      const params = new URLSearchParams(searchParams.toString());
      for (const [key, value] of Object.entries(updates)) {
        if (value === undefined || value === "" || value === defaultValues[key]) {
          params.delete(key);
        } else {
          params.set(key, value);
        }
      }
      const queryString = params.toString();
      router.push(queryString ? `${pathname}?${queryString}` : pathname, { scroll: false });
    },
    [router, searchParams, pathname, defaultValues]
  );

  const resetFilters = useCallback(() => {
    router.push(pathname, { scroll: false });
  }, [router, pathname]);

  return [filters, setFilters, resetFilters];
}
