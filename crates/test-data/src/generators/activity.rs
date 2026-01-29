//! Activity generation from tracks.

use rand::Rng;
use time::OffsetDateTime;
use uuid::Uuid;

use tracks::models::{builtin_types, TrackPointData, Visibility};

/// Generated activity data ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedActivity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub activity_type_id: Uuid,
    pub visibility: Visibility,
    pub submitted_at: OffsetDateTime,
    pub track_points: Vec<TrackPointData>,
    /// Calculated from track: total distance in meters.
    pub distance_meters: f64,
    /// Calculated from track: total duration in seconds.
    pub duration_seconds: f64,
    /// Calculated from track: moving time in seconds (excluding pauses).
    pub moving_time_seconds: f64,
    /// Calculated from track: total elevation gain in meters.
    pub elevation_gain_meters: f64,
}

/// Configuration for activity name generation.
#[derive(Debug, Clone)]
pub struct ActivityNameConfig {
    /// Prefix options for different activity types.
    pub running_prefixes: Vec<String>,
    pub cycling_prefixes: Vec<String>,
    pub hiking_prefixes: Vec<String>,
    /// Time-of-day adjectives.
    pub time_adjectives: Vec<String>,
    /// Location suffixes.
    pub location_suffixes: Vec<String>,
}

impl Default for ActivityNameConfig {
    fn default() -> Self {
        Self {
            running_prefixes: vec![
                "Morning Run".into(),
                "Evening Run".into(),
                "Trail Run".into(),
                "Easy Run".into(),
                "Tempo Run".into(),
                "Long Run".into(),
            ],
            cycling_prefixes: vec![
                "Morning Ride".into(),
                "Evening Ride".into(),
                "Mountain Bike".into(),
                "Road Ride".into(),
                "Gravel Ride".into(),
            ],
            hiking_prefixes: vec![
                "Morning Hike".into(),
                "Afternoon Hike".into(),
                "Summit Attempt".into(),
                "Trail Hike".into(),
                "Nature Walk".into(),
            ],
            time_adjectives: vec![
                "Sunrise".into(),
                "Sunset".into(),
                "Midday".into(),
                "After Work".into(),
                "Weekend".into(),
            ],
            location_suffixes: vec![
                "on the Flatirons".into(),
                "at the Mesa".into(),
                "by the Lake".into(),
                "in the Mountains".into(),
                "through the Park".into(),
            ],
        }
    }
}

/// Generates activities from track data.
pub struct ActivityGenerator {
    name_config: ActivityNameConfig,
}

impl ActivityGenerator {
    /// Creates a new activity generator with default naming.
    pub fn new() -> Self {
        Self {
            name_config: ActivityNameConfig::default(),
        }
    }

    /// Creates an activity from track points.
    pub fn from_track(
        &self,
        user_id: Uuid,
        activity_type_id: Uuid,
        track_points: Vec<TrackPointData>,
        rng: &mut impl Rng,
    ) -> GeneratedActivity {
        let id = Uuid::new_v4();
        let name = self.generate_name(activity_type_id, rng);

        // Calculate stats from track
        let (distance_meters, elevation_gain_meters) =
            self.calculate_distance_and_gain(&track_points);
        let (duration_seconds, moving_time_seconds) = self.calculate_times(&track_points);

        let submitted_at = track_points
            .first()
            .and_then(|p| p.timestamp)
            .unwrap_or_else(OffsetDateTime::now_utc);

        let visibility = if rng.r#gen::<f64>() < 0.9 {
            Visibility::Public
        } else {
            Visibility::Private
        };

        GeneratedActivity {
            id,
            user_id,
            name,
            activity_type_id,
            visibility,
            submitted_at,
            track_points,
            distance_meters,
            duration_seconds,
            moving_time_seconds,
            elevation_gain_meters,
        }
    }

    /// Generates an appropriate name for an activity.
    fn generate_name(&self, activity_type_id: Uuid, rng: &mut impl Rng) -> String {
        let prefixes = if activity_type_id == builtin_types::RUN {
            &self.name_config.running_prefixes
        } else if activity_type_id == builtin_types::ROAD
            || activity_type_id == builtin_types::MTB
            || activity_type_id == builtin_types::EMTB
            || activity_type_id == builtin_types::GRAVEL
        {
            &self.name_config.cycling_prefixes
        } else if activity_type_id == builtin_types::HIKE
            || activity_type_id == builtin_types::WALK
        {
            &self.name_config.hiking_prefixes
        } else {
            &self.name_config.running_prefixes
        };

        let prefix = &prefixes[rng.gen_range(0..prefixes.len())];

        // Sometimes add a location suffix
        if rng.r#gen::<f64>() < 0.3 {
            let suffix = &self.name_config.location_suffixes
                [rng.gen_range(0..self.name_config.location_suffixes.len())];
            format!("{prefix} {suffix}")
        } else {
            prefix.clone()
        }
    }

    /// Calculates total distance and elevation gain from track points.
    fn calculate_distance_and_gain(&self, points: &[TrackPointData]) -> (f64, f64) {
        if points.len() < 2 {
            return (0.0, 0.0);
        }

        let mut total_distance = 0.0;
        let mut total_gain = 0.0;

        for window in points.windows(2) {
            let p1 = &window[0];
            let p2 = &window[1];

            // Haversine distance
            total_distance += haversine_distance(p1.lat, p1.lon, p2.lat, p2.lon);

            // Elevation gain (only count uphill)
            if let (Some(e1), Some(e2)) = (p1.elevation, p2.elevation)
                && e2 > e1
            {
                total_gain += e2 - e1;
            }
        }

        (total_distance, total_gain)
    }

    /// Calculates total duration and moving time from track points.
    ///
    /// Moving time excludes periods where speed is below a threshold (pauses).
    fn calculate_times(&self, points: &[TrackPointData]) -> (f64, f64) {
        if points.len() < 2 {
            return (0.0, 0.0);
        }

        let first_ts = points.first().and_then(|p| p.timestamp);
        let last_ts = points.last().and_then(|p| p.timestamp);

        let total_duration = match (first_ts, last_ts) {
            (Some(t1), Some(t2)) => (t2 - t1).as_seconds_f64(),
            _ => 0.0,
        };

        // Calculate moving time by summing intervals where speed > threshold
        let pause_threshold_mps = 0.5; // Below this speed is considered stopped
        let mut moving_time = 0.0;

        for window in points.windows(2) {
            let p1 = &window[0];
            let p2 = &window[1];

            if let (Some(t1), Some(t2)) = (p1.timestamp, p2.timestamp) {
                let interval = (t2 - t1).as_seconds_f64();
                let distance = haversine_distance(p1.lat, p1.lon, p2.lat, p2.lon);

                if interval > 0.0 {
                    let speed = distance / interval;
                    if speed > pause_threshold_mps {
                        moving_time += interval;
                    }
                }
            }
        }

        (total_duration, moving_time)
    }
}

impl Default for ActivityGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Haversine distance calculation.
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_M * c
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[test]
    fn test_activity_from_track() {
        let activity_gen = ActivityGenerator::new();
        let mut rng = rand::thread_rng();

        let now = OffsetDateTime::now_utc();
        let points = vec![
            TrackPointData {
                lat: 40.0,
                lon: -105.3,
                elevation: Some(1650.0),
                timestamp: Some(now),
            },
            TrackPointData {
                lat: 40.001,
                lon: -105.299,
                elevation: Some(1660.0),
                timestamp: Some(now + Duration::seconds(60)),
            },
            TrackPointData {
                lat: 40.002,
                lon: -105.298,
                elevation: Some(1670.0),
                timestamp: Some(now + Duration::seconds(120)),
            },
        ];

        let activity =
            activity_gen.from_track(Uuid::new_v4(), builtin_types::RUN, points, &mut rng);

        assert!(!activity.name.is_empty());
        assert!(activity.distance_meters > 0.0);
        assert!(activity.duration_seconds > 0.0);
        assert!(activity.elevation_gain_meters > 0.0);
    }
}
