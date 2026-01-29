/**
 * Client-side GPX parsing utility for activity preview.
 *
 * Parses GPX files using the native DOMParser to extract track points
 * with coordinates, elevation, and timestamps.
 */

import type { TrackPoint } from "./api";

export interface ParsedGpx {
  name: string | null;
  points: TrackPoint[];
  hasTimestamps: boolean;
}

/**
 * Parse a GPX file and extract track points.
 *
 * @param file - The GPX file to parse
 * @returns Parsed GPX data with track points
 */
export async function parseGpxFile(file: File): Promise<ParsedGpx> {
  const text = await file.text();
  return parseGpxString(text);
}

/**
 * Parse GPX XML string and extract track points.
 *
 * @param gpxString - Raw GPX XML content
 * @returns Parsed GPX data with track points
 */
export function parseGpxString(gpxString: string): ParsedGpx {
  const parser = new DOMParser();
  const doc = parser.parseFromString(gpxString, "application/xml");

  // Check for parse errors
  const parseError = doc.querySelector("parsererror");
  if (parseError) {
    throw new Error("Invalid GPX file: XML parsing failed");
  }

  // Extract activity name from metadata or first track
  const metadataName = doc.querySelector("metadata > name")?.textContent;
  const trackName = doc.querySelector("trk > name")?.textContent;
  const name = metadataName || trackName || null;

  // Extract all track points from all track segments
  const points: TrackPoint[] = [];
  let hasTimestamps = false;

  // GPX can have multiple tracks (trk) and each track can have multiple segments (trkseg)
  const trkpts = doc.querySelectorAll("trkpt");

  for (const trkpt of trkpts) {
    const lat = parseFloat(trkpt.getAttribute("lat") || "");
    const lon = parseFloat(trkpt.getAttribute("lon") || "");

    if (isNaN(lat) || isNaN(lon)) {
      continue;
    }

    // Elevation is optional
    const eleEl = trkpt.querySelector("ele");
    const ele = eleEl ? parseFloat(eleEl.textContent || "") : null;

    // Time is optional
    const timeEl = trkpt.querySelector("time");
    const time = timeEl?.textContent || null;

    if (time) {
      hasTimestamps = true;
    }

    points.push({
      lat,
      lon,
      ele: ele !== null && !isNaN(ele) ? ele : null,
      time,
    });
  }

  return {
    name,
    points,
    hasTimestamps,
  };
}

/**
 * Find the index of the track point closest to a given timestamp.
 *
 * @param points - Array of track points with timestamps
 * @param timestamp - ISO8601 timestamp to find
 * @returns Index of the closest point, or -1 if no timestamps available
 */
export function findPointIndexByTimestamp(
  points: TrackPoint[],
  timestamp: string
): number {
  const targetTime = new Date(timestamp).getTime();

  let closestIndex = -1;
  let closestDiff = Infinity;

  for (let i = 0; i < points.length; i++) {
    const point = points[i];
    if (!point.time) continue;

    const pointTime = new Date(point.time).getTime();
    const diff = Math.abs(pointTime - targetTime);

    if (diff < closestDiff) {
      closestDiff = diff;
      closestIndex = i;
    }
  }

  return closestIndex;
}

/**
 * Get the timestamp at a given point index.
 *
 * @param points - Array of track points
 * @param index - Point index
 * @returns ISO8601 timestamp or null if not available
 */
export function getTimestampAtIndex(
  points: TrackPoint[],
  index: number
): string | null {
  if (index < 0 || index >= points.length) {
    return null;
  }
  return points[index].time;
}

/**
 * Get the first and last timestamps from track points.
 *
 * @param points - Array of track points
 * @returns Object with start and end timestamps, or nulls if unavailable
 */
export function getTrackTimeRange(points: TrackPoint[]): {
  start: string | null;
  end: string | null;
} {
  let start: string | null = null;
  let end: string | null = null;

  for (const point of points) {
    if (point.time) {
      if (!start) {
        start = point.time;
      }
      end = point.time;
    }
  }

  return { start, end };
}
