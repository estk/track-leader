import simplify from "@turf/simplify";
import { lineString } from "@turf/helpers";

interface Point {
  lat: number;
  lon: number;
  ele?: number | null;
}

/**
 * Simplify a route using Douglas-Peucker algorithm.
 * @param points - Array of points with lat, lon, and optionally ele
 * @param tolerance - Tolerance in degrees (higher = more simplification)
 *                    0.0001 (default) is about 10m at equator
 * @param highQuality - If true, uses slower but more accurate algorithm
 * @returns Simplified array of points
 */
export function simplifyRoute<T extends Point>(
  points: T[],
  tolerance: number = 0.0001,
  highQuality: boolean = false
): T[] {
  if (points.length <= 2) return points;

  // Convert to GeoJSON LineString
  const coordinates = points.map((p) => [p.lon, p.lat]);
  const line = lineString(coordinates);

  // Simplify using Douglas-Peucker
  const simplified = simplify(line, {
    tolerance,
    highQuality,
  });

  // Map back to original point structure
  const simplifiedCoords = simplified.geometry.coordinates;
  const result: T[] = [];

  for (const coord of simplifiedCoords) {
    // Find the original point closest to this coordinate
    const original = points.find(
      (p) => Math.abs(p.lon - coord[0]) < 0.000001 && Math.abs(p.lat - coord[1]) < 0.000001
    );
    if (original) {
      result.push(original);
    }
  }

  // Ensure first and last points are always included
  if (result.length > 0 && result[0] !== points[0]) {
    result.unshift(points[0]);
  }
  if (result.length > 0 && result[result.length - 1] !== points[points.length - 1]) {
    result.push(points[points.length - 1]);
  }

  return result;
}

/**
 * Determine appropriate simplification tolerance based on point count.
 * More points = more aggressive simplification.
 */
export function getAdaptiveTolerance(pointCount: number): number {
  if (pointCount < 500) return 0; // No simplification needed
  if (pointCount < 1000) return 0.00005; // ~5m
  if (pointCount < 5000) return 0.0001; // ~10m
  if (pointCount < 10000) return 0.0002; // ~20m
  return 0.0003; // ~30m for very large routes
}

/**
 * Get simplification level based on zoom.
 * At lower zoom levels, use more aggressive simplification.
 */
export function getZoomBasedTolerance(zoom: number): number {
  if (zoom >= 15) return 0; // High zoom = full detail
  if (zoom >= 13) return 0.00005;
  if (zoom >= 11) return 0.0001;
  if (zoom >= 9) return 0.0002;
  return 0.0003;
}
