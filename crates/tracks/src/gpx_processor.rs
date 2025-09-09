use bytes::Bytes;
use gpx::{read, Gpx, Waypoint};
use time::{Duration, OffsetDateTime};

use crate::{
    errors::AppError,
    models::{ActivityMetrics, ActivityType},
};

pub struct GpxProcessor;

pub struct ProcessedGpx {
    pub metrics: ActivityMetrics,
    pub activity_type: ActivityType,
}

impl GpxProcessor {
    pub fn process_gpx(content: &Bytes) -> Result<ProcessedGpx, AppError> {
        let gpx: Gpx = read(content.as_ref())
            .map_err(|e| AppError::GpxParsing(format!("Failed to parse GPX: {}", e)))?;

        let mut all_points = Vec::new();
        let mut sequence = 0;

        for track in &gpx.tracks {
            for segment in &track.segments {
                for point in &segment.points {
                    all_points.push((point.clone(), sequence));
                    sequence += 1;
                }
            }
        }

        if all_points.is_empty() {
            return Err(AppError::InvalidInput(
                "No track points found in GPX file".to_string(),
            ));
        }

        let metrics = Self::calculate_metrics(&all_points);

        Ok(ProcessedGpx {
            metrics,
            activity_type: ActivityType::Other,
        })
    }

    fn calculate_metrics(points: &[(Waypoint, i32)]) -> ActivityMetrics {
        let mut distance = 0.0;
        let mut ascent = 0.0;
        let mut descent = 0.0;
        let mut duration = Duration::ZERO;

        if points.len() < 2 {
            return ActivityMetrics {
                distance,
                ascent,
                descent,
                duration: duration.whole_seconds(),
            };
        }

        for i in 1..points.len() {
            let prev = &points[i - 1].0;
            let curr = &points[i].0;

            distance += Self::haversine_distance(
                prev.point().y(),
                prev.point().x(),
                curr.point().y(),
                curr.point().x(),
            );

            if let (Some(prev_elev), Some(curr_elev)) = (prev.elevation, curr.elevation) {
                let elev_diff = curr_elev - prev_elev;
                if elev_diff > 0.0 {
                    ascent += elev_diff;
                } else {
                    descent += elev_diff.abs();
                }
            }
        }

        if let (Some(start_time), Some(end_time)) = (
            points.first().and_then(|p| p.0.time),
            points.last().and_then(|p| p.0.time),
        ) {
            let start: OffsetDateTime = start_time.into();
            let end: OffsetDateTime = end_time.into();
            duration = end - start;
        }

        ActivityMetrics {
            distance,
            ascent,
            descent,
            duration: duration.whole_seconds(),
        }
    }

    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS: f64 = 6371000.0; // meters

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }
}
