import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export interface ClimbCategoryInfo {
  label: string;
  tooltip: string;
}

export function getClimbCategoryInfo(category: number | null): ClimbCategoryInfo | null {
  if (category === null) return null;
  switch (category) {
    case 0:
      return {
        label: "HC",
        tooltip: "Hors Categorie: The most difficult climbs, typically 800m+ elevation gain. Beyond normal categorization.",
      };
    case 1:
      return {
        label: "Cat 1",
        tooltip: "Category 1: Very difficult climbs, typically 640-800m gain over 10+ km at 7-9% gradient.",
      };
    case 2:
      return {
        label: "Cat 2",
        tooltip: "Category 2: Difficult climbs, typically 320-640m gain over 5-10 km at 6-9% gradient.",
      };
    case 3:
      return {
        label: "Cat 3",
        tooltip: "Category 3: Moderate climbs, typically 160-320m gain over 4-5 km at 6-8% gradient.",
      };
    case 4:
      return {
        label: "Cat 4",
        tooltip: "Category 4: Easy climbs, typically 80-160m gain over 1-3 km at 3-6% gradient.",
      };
    default:
      return null;
  }
}

export function formatClimbCategory(category: number | null): string | null {
  const info = getClimbCategoryInfo(category);
  return info?.label ?? null;
}

export function formatDistanceToNow(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);
  const diffWeeks = Math.floor(diffDays / 7);
  const diffMonths = Math.floor(diffDays / 30);

  if (diffSeconds < 60) {
    return "just now";
  } else if (diffMinutes < 60) {
    return `${diffMinutes}m ago`;
  } else if (diffHours < 24) {
    return `${diffHours}h ago`;
  } else if (diffDays < 7) {
    return `${diffDays}d ago`;
  } else if (diffWeeks < 4) {
    return `${diffWeeks}w ago`;
  } else if (diffMonths < 12) {
    return `${diffMonths}mo ago`;
  } else {
    return date.toLocaleDateString();
  }
}
