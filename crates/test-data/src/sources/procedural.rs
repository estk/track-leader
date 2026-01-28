//! Procedural track generation.

use rand::Rng;
use rand_distr::{Distribution, Normal};
use time::{Duration, OffsetDateTime};
use tracks::models::TrackPointData;

use crate::config::BoundingBox;
use crate::profiles::{self, AthleteProfile};
use crate::terrain::ElevationGenerator;

/// Route pattern determining the shape of the generated track.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RoutePattern {
    /// Random walk with momentum - the original behavior.
    #[default]
    RandomWalk,
    /// Out-and-back: go out half the distance, then reverse and return.
    OutAndBack,
    /// Loop: curve the path back toward the starting point.
    Loop,
}

/// Configuration for procedural track generation.
#[derive(Debug, Clone)]
pub struct TrackConfig {
    /// Target distance in meters.
    pub distance_meters: f64,
    /// Starting point (lat, lon). If None, random within bounds.
    pub start_point: Option<(f64, f64)>,
    /// Geographic bounds for the track.
    pub bounds: BoundingBox,
    /// GPS position jitter standard deviation in meters.
    pub gps_jitter_m: f64,
    /// GPS elevation jitter standard deviation in meters.
    pub elevation_jitter_m: f64,
    /// Approximate distance between track points in meters.
    pub point_spacing_m: f64,
    /// Probability of inserting a pause (0.0 - 1.0).
    pub pause_probability: f64,
    /// Duration range for pauses (min, max) in seconds.
    pub pause_duration_range: (f64, f64),
}

impl Default for TrackConfig {
    fn default() -> Self {
        Self {
            distance_meters: 5000.0,
            start_point: None,
            bounds: crate::config::Region::BOULDER,
            gps_jitter_m: 3.0,
            elevation_jitter_m: 5.0,
            point_spacing_m: 10.0,
            pause_probability: 0.02,
            pause_duration_range: (30.0, 180.0),
        }
    }
}

/// Generates synthetic GPS tracks with realistic characteristics.
pub struct ProceduralGenerator {
    config: TrackConfig,
    elevation: ElevationGenerator,
    pattern: RoutePattern,
}

impl ProceduralGenerator {
    /// Creates a new procedural generator with default configuration.
    pub fn new(seed: u32) -> Self {
        Self {
            config: TrackConfig::default(),
            elevation: ElevationGenerator::boulder(seed),
            pattern: RoutePattern::default(),
        }
    }

    /// Creates a generator for a specific region.
    pub fn for_region(bounds: BoundingBox, seed: u32) -> Self {
        let elevation = if bounds.center().0 > 39.5 {
            ElevationGenerator::reno_tahoe(seed)
        } else {
            ElevationGenerator::boulder(seed)
        };

        Self {
            config: TrackConfig {
                bounds,
                ..Default::default()
            },
            elevation,
            pattern: RoutePattern::default(),
        }
    }

    /// Sets the route pattern.
    pub fn with_pattern(mut self, pattern: RoutePattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Sets the target distance.
    pub fn with_distance(mut self, meters: f64) -> Self {
        self.config.distance_meters = meters;
        self
    }

    /// Sets the starting point.
    pub fn with_start(mut self, lat: f64, lon: f64) -> Self {
        self.config.start_point = Some((lat, lon));
        self
    }

    /// Sets GPS jitter amount.
    pub fn with_gps_jitter(mut self, meters: f64) -> Self {
        self.config.gps_jitter_m = meters;
        self
    }

    /// Sets the elevation generator.
    pub fn with_elevation(mut self, elevation: ElevationGenerator) -> Self {
        self.elevation = elevation;
        self
    }

    /// Sets point spacing.
    pub fn with_point_spacing(mut self, meters: f64) -> Self {
        self.config.point_spacing_m = meters;
        self
    }

    /// Sets pause parameters.
    pub fn with_pauses(mut self, probability: f64, min_sec: f64, max_sec: f64) -> Self {
        self.config.pause_probability = probability;
        self.config.pause_duration_range = (min_sec, max_sec);
        self
    }

    /// Generates a track using the specified athletic profile.
    ///
    /// The profile determines speeds based on terrain grade.
    pub fn generate(
        &self,
        profile: &dyn AthleteProfile,
        rng: &mut impl Rng,
    ) -> Vec<TrackPointData> {
        let start = self
            .config
            .start_point
            .unwrap_or_else(|| self.config.bounds.random_point(rng));

        let path = self.generate_path(start, rng);
        self.apply_timing(path, profile, rng)
    }

    /// Generates a simple path (coordinates only, no timing).
    pub fn generate_path(&self, start: (f64, f64), rng: &mut impl Rng) -> Vec<(f64, f64)> {
        match self.pattern {
            RoutePattern::RandomWalk => self.generate_random_walk(start, rng),
            RoutePattern::OutAndBack => self.generate_out_and_back(start, rng),
            RoutePattern::Loop => self.generate_loop(start, rng),
        }
    }

    /// Generates a random walk path with momentum.
    fn generate_random_walk(&self, start: (f64, f64), rng: &mut impl Rng) -> Vec<(f64, f64)> {
        let mut path = vec![start];
        let mut current = start;
        let mut total_distance = 0.0;

        // Random walk with some momentum to create natural-looking paths
        let mut heading = rng.gen_range(0.0..std::f64::consts::TAU);

        while total_distance < self.config.distance_meters {
            // Adjust heading with some randomness
            let heading_change = rng.gen_range(-0.3..0.3);
            heading += heading_change;

            // Calculate step size (roughly config spacing, with variance)
            let step = self.config.point_spacing_m * rng.gen_range(0.8..1.2);

            // Convert step to lat/lon delta
            // Rough approximation: 1 degree lat ≈ 111km, lon varies by latitude
            let lat_delta = (step * heading.cos()) / 111_000.0;
            let lon_delta = (step * heading.sin()) / (111_000.0 * current.0.to_radians().cos());

            let next_lat = current.0 + lat_delta;
            let next_lon = current.1 + lon_delta;

            // Clamp to bounds with bounce-back
            let (next_lat, next_lon, bounced_heading) =
                self.apply_bounds(next_lat, next_lon, heading);
            heading = bounced_heading;

            current = (next_lat, next_lon);
            path.push(current);
            total_distance += step;
        }

        path
    }

    /// Generates an out-and-back path: go out half the distance, then reverse and return.
    fn generate_out_and_back(&self, start: (f64, f64), rng: &mut impl Rng) -> Vec<(f64, f64)> {
        // Generate outbound path for half the total distance
        let half_distance = self.config.distance_meters / 2.0;
        let mut outbound = vec![start];
        let mut current = start;
        let mut total_distance = 0.0;
        let mut heading = rng.gen_range(0.0..std::f64::consts::TAU);

        while total_distance < half_distance {
            let heading_change = rng.gen_range(-0.2..0.2); // Slightly less variation
            heading += heading_change;
            let step = self.config.point_spacing_m * rng.gen_range(0.8..1.2);

            let lat_delta = (step * heading.cos()) / 111_000.0;
            let lon_delta = (step * heading.sin()) / (111_000.0 * current.0.to_radians().cos());

            let next_lat = current.0 + lat_delta;
            let next_lon = current.1 + lon_delta;

            let (next_lat, next_lon, bounced_heading) =
                self.apply_bounds(next_lat, next_lon, heading);
            heading = bounced_heading;

            current = (next_lat, next_lon);
            outbound.push(current);
            total_distance += step;
        }

        // Create return path by reversing and adding slight offset to avoid exact overlap
        let mut path = outbound.clone();
        let jitter = Normal::new(0.0, 0.00001).unwrap(); // Small lat/lon offset

        for point in outbound.iter().rev().skip(1) {
            let offset_lat = point.0 + jitter.sample(rng);
            let offset_lon = point.1 + jitter.sample(rng);
            path.push((offset_lat, offset_lon));
        }

        path
    }

    /// Generates a loop path that curves back toward the starting point.
    fn generate_loop(&self, start: (f64, f64), rng: &mut impl Rng) -> Vec<(f64, f64)> {
        let mut path = vec![start];
        let mut current = start;
        let mut total_distance = 0.0;

        // Start with a random initial heading
        let initial_heading = rng.gen_range(0.0..std::f64::consts::TAU);
        let mut heading = initial_heading;

        // Target is to curve back to start over the total distance
        // Use a gradual turn rate that completes ~2π radians over the route
        let turn_rate =
            std::f64::consts::TAU / (self.config.distance_meters / self.config.point_spacing_m);

        while total_distance < self.config.distance_meters * 0.95 {
            let step = self.config.point_spacing_m * rng.gen_range(0.8..1.2);

            // Apply consistent turn with some noise
            let heading_change = turn_rate + rng.gen_range(-0.1..0.1);
            heading += heading_change;

            let lat_delta = (step * heading.cos()) / 111_000.0;
            let lon_delta = (step * heading.sin()) / (111_000.0 * current.0.to_radians().cos());

            let next_lat = current.0 + lat_delta;
            let next_lon = current.1 + lon_delta;

            let (next_lat, next_lon, bounced_heading) =
                self.apply_bounds(next_lat, next_lon, heading);
            heading = bounced_heading;

            current = (next_lat, next_lon);
            path.push(current);
            total_distance += step;
        }

        // Close the loop by adding a direct path back to start if needed
        let dist_to_start = haversine_distance(current.0, current.1, start.0, start.1);
        if dist_to_start > self.config.point_spacing_m * 2.0 {
            // Interpolate points back to start
            let steps = (dist_to_start / self.config.point_spacing_m).ceil() as usize;
            for i in 1..=steps {
                let t = i as f64 / steps as f64;
                let lat = current.0 + (start.0 - current.0) * t;
                let lon = current.1 + (start.1 - current.1) * t;
                path.push((lat, lon));
            }
        } else {
            path.push(start);
        }

        path
    }

    /// Applies bounds checking with heading reversal.
    fn apply_bounds(&self, lat: f64, lon: f64, heading: f64) -> (f64, f64, f64) {
        let b = &self.config.bounds;
        let mut new_heading = heading;

        let lat = if lat < b.min_lat {
            new_heading = std::f64::consts::PI - heading;
            b.min_lat + (b.min_lat - lat).min(0.001)
        } else if lat > b.max_lat {
            new_heading = std::f64::consts::PI - heading;
            b.max_lat - (lat - b.max_lat).min(0.001)
        } else {
            lat
        };

        let lon = if lon < b.min_lon {
            new_heading = -heading;
            b.min_lon + (b.min_lon - lon).min(0.001)
        } else if lon > b.max_lon {
            new_heading = -heading;
            b.max_lon - (lon - b.max_lon).min(0.001)
        } else {
            lon
        };

        (lat, lon, new_heading)
    }

    /// Applies timing and elevation to a path using an athletic profile.
    fn apply_timing(
        &self,
        path: Vec<(f64, f64)>,
        profile: &dyn AthleteProfile,
        rng: &mut impl Rng,
    ) -> Vec<TrackPointData> {
        if path.is_empty() {
            return Vec::new();
        }

        let jitter = Normal::new(0.0, self.config.gps_jitter_m / 111_000.0).unwrap();
        let elev_jitter = Normal::new(0.0, self.config.elevation_jitter_m).unwrap();

        let mut result = Vec::with_capacity(path.len());
        let mut timestamp = OffsetDateTime::now_utc();

        // First point
        let (lat, lon) = path[0];
        let elevation = self.elevation.elevation_at(lat, lon) + elev_jitter.sample(rng);
        result.push(TrackPointData {
            lat: lat + jitter.sample(rng),
            lon: lon + jitter.sample(rng),
            elevation: Some(elevation),
            timestamp: Some(timestamp),
        });

        for i in 1..path.len() {
            let (prev_lat, prev_lon) = path[i - 1];
            let (lat, lon) = path[i];

            // Calculate distance and grade
            let distance = haversine_distance(prev_lat, prev_lon, lat, lon);
            let prev_elev = self.elevation.elevation_at(prev_lat, prev_lon);
            let curr_elev = self.elevation.elevation_at(lat, lon);
            let grade = if distance > 0.0 {
                (curr_elev - prev_elev) / distance
            } else {
                0.0
            };

            // Calculate speed and time
            let variance = profiles::sample_variance(profile, rng);
            let speed = profiles::speed_at_grade(profile, grade, variance);
            let time_seconds = distance / speed;

            // Maybe add a pause
            let pause_seconds = if rng.r#gen::<f64>() < self.config.pause_probability {
                rng.gen_range(
                    self.config.pause_duration_range.0..self.config.pause_duration_range.1,
                )
            } else {
                0.0
            };

            timestamp += Duration::seconds_f64(time_seconds + pause_seconds);

            let elevation = curr_elev + elev_jitter.sample(rng);
            result.push(TrackPointData {
                lat: lat + jitter.sample(rng),
                lon: lon + jitter.sample(rng),
                elevation: Some(elevation),
                timestamp: Some(timestamp),
            });
        }

        result
    }
}

/// Calculates the haversine distance between two points in meters.
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
    use crate::profiles::RunnerProfile;

    #[test]
    fn test_generate_track() {
        let track_gen = ProceduralGenerator::new(42).with_distance(1000.0);
        let profile = RunnerProfile::default();
        let mut rng = rand::thread_rng();

        let track = track_gen.generate(&profile, &mut rng);

        assert!(!track.is_empty());
        assert!(track.len() > 10); // Should have many points for 1km

        // Check all points have timestamps and elevation
        for point in &track {
            assert!(point.timestamp.is_some());
            assert!(point.elevation.is_some());
        }
    }

    #[test]
    fn test_timestamps_increase() {
        let track_gen = ProceduralGenerator::new(42).with_distance(500.0);
        let profile = RunnerProfile::default();
        let mut rng = rand::thread_rng();

        let track = track_gen.generate(&profile, &mut rng);

        for window in track.windows(2) {
            let t1 = window[0].timestamp.unwrap();
            let t2 = window[1].timestamp.unwrap();
            assert!(t2 > t1, "Timestamps should increase monotonically");
        }
    }

    #[test]
    fn test_haversine() {
        // Known distance: ~111km for 1 degree of latitude
        let dist = haversine_distance(0.0, 0.0, 1.0, 0.0);
        assert!((dist - 111_000.0).abs() < 1000.0);
    }

    #[test]
    fn test_out_and_back_pattern() {
        let track_gen = ProceduralGenerator::new(42)
            .with_distance(1000.0)
            .with_pattern(RoutePattern::OutAndBack);
        let mut rng = rand::thread_rng();
        let start = (40.0, -105.3);

        let path = track_gen.generate_path(start, &mut rng);

        assert!(!path.is_empty());
        // First and last points should be close (within ~20m for small GPS variance)
        let end = path.last().unwrap();
        let dist_to_start = haversine_distance(start.0, start.1, end.0, end.1);
        assert!(
            dist_to_start < 50.0,
            "Out-and-back should return to start, got {dist_to_start}m"
        );
    }

    #[test]
    fn test_loop_pattern() {
        let track_gen = ProceduralGenerator::new(42)
            .with_distance(1000.0)
            .with_pattern(RoutePattern::Loop);
        let mut rng = rand::thread_rng();
        let start = (40.0, -105.3);

        let path = track_gen.generate_path(start, &mut rng);

        assert!(!path.is_empty());
        // First and last points should be close
        let end = path.last().unwrap();
        let dist_to_start = haversine_distance(start.0, start.1, end.0, end.1);
        assert!(
            dist_to_start < 50.0,
            "Loop should return to start, got {dist_to_start}m"
        );
    }

    #[test]
    fn test_random_walk_pattern() {
        let track_gen = ProceduralGenerator::new(42)
            .with_distance(1000.0)
            .with_pattern(RoutePattern::RandomWalk);
        let mut rng = rand::thread_rng();
        let start = (40.0, -105.3);

        let path = track_gen.generate_path(start, &mut rng);

        assert!(!path.is_empty());
        // Random walk typically doesn't return to start
        let end = path.last().unwrap();
        let dist_to_start = haversine_distance(start.0, start.1, end.0, end.1);
        // Just verify we generated something - random walk goes anywhere
        assert!(path.len() > 10);
        // Usually won't be at start (though could be by chance)
        // We don't assert dist_to_start > threshold because random walks could loop back
        let _ = dist_to_start; // Silence unused warning
    }
}
