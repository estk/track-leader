//! Segment matching utilities for finding and timing segment efforts from GPX tracks.

use time::OffsetDateTime;
use uuid::Uuid;

/// A segment that an activity track passes through.
#[derive(Debug, Clone)]
pub struct SegmentMatch {
    pub segment_id: Uuid,
    pub distance_meters: f64,
    /// Fractional position (0-1) of segment start along the track
    pub start_fraction: f64,
    /// Fractional position (0-1) of segment end along the track
    pub end_fraction: f64,
}

/// An activity whose track matches a segment.
#[derive(Debug, Clone)]
pub struct ActivityMatch {
    pub activity_id: Uuid,
    pub user_id: Uuid,
    /// Fractional position (0-1) of segment start along the track
    pub start_fraction: f64,
    /// Fractional position (0-1) of segment end along the track
    pub end_fraction: f64,
}

/// Timing information extracted from GPX for a segment effort.
#[derive(Debug, Clone)]
pub struct SegmentTiming {
    pub started_at: OffsetDateTime,
    pub elapsed_time_seconds: f64,
}

/// Extract timing from GPX track for a segment match.
///
/// Uses the fractional positions (0-1) from PostGIS ST_LineLocatePoint to find
/// the corresponding timestamps in the GPX track points.
pub fn extract_timing_from_gpx(
    gpx: &gpx::Gpx,
    start_fraction: f64,
    end_fraction: f64,
) -> Option<SegmentTiming> {
    // Collect all points with timestamps
    let mut points_with_time: Vec<(f64, gpx::Time)> = Vec::new();
    let mut cumulative_distance = 0.0;

    for track in &gpx.tracks {
        for seg in &track.segments {
            let mut prev_point: Option<geo::geometry::Point<f64>> = None;

            for pt in &seg.points {
                if let Some(prev) = prev_point {
                    let distance =
                        haversine_distance(prev.y(), prev.x(), pt.point().y(), pt.point().x());
                    cumulative_distance += distance;
                }

                if let Some(time) = pt.time {
                    points_with_time.push((cumulative_distance, time));
                }

                prev_point = Some(pt.point());
            }
        }
    }

    if points_with_time.is_empty() || cumulative_distance == 0.0 {
        return None;
    }

    // Normalize distances to fractions (0-1)
    let total_distance = cumulative_distance;
    for (dist, _) in &mut points_with_time {
        *dist /= total_distance;
    }

    // Find timestamps at start and end fractions by interpolation
    let start_time = interpolate_time(&points_with_time, start_fraction)?;
    let end_time = interpolate_time(&points_with_time, end_fraction)?;

    let start_dt: OffsetDateTime = start_time.into();
    let end_dt: OffsetDateTime = end_time.into();

    let elapsed = (end_dt - start_dt).as_seconds_f64();

    if elapsed <= 0.0 {
        return None;
    }

    Some(SegmentTiming {
        started_at: start_dt,
        elapsed_time_seconds: elapsed,
    })
}

/// Interpolate time at a given fractional position along the track.
fn interpolate_time(points: &[(f64, gpx::Time)], fraction: f64) -> Option<gpx::Time> {
    if points.is_empty() {
        return None;
    }

    // Find the two points that bracket the fraction
    let mut lower_idx = 0;
    for (i, (dist, _)) in points.iter().enumerate() {
        if *dist <= fraction {
            lower_idx = i;
        } else {
            break;
        }
    }

    // If at or past the last point, return last time
    if lower_idx == points.len() - 1 {
        return Some(points[lower_idx].1);
    }

    let (dist1, time1) = points[lower_idx];
    let (dist2, time2) = points[lower_idx + 1];

    // Linear interpolation between the two points
    if (dist2 - dist1).abs() < f64::EPSILON {
        return Some(time1);
    }

    let t = (fraction - dist1) / (dist2 - dist1);

    let time1_dt: OffsetDateTime = time1.into();
    let time2_dt: OffsetDateTime = time2.into();

    let duration = time2_dt - time1_dt;
    let interpolated = time1_dt + duration * t;

    // Convert back to gpx::Time
    Some(gpx::Time::from(interpolated))
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371000.0; // Earth radius in meters
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}
