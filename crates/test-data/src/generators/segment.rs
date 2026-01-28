//! Segment extraction and generation.

use rand::Rng;
use uuid::Uuid;

use tracks::models::{ActivityType, TrackPointData, Visibility};

/// Generated segment data ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedSegment {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub activity_type: ActivityType,
    pub visibility: Visibility,
    /// Segment geometry as WKT LineString.
    pub geo_wkt: String,
    /// Start point as WKT Point.
    pub start_wkt: String,
    /// End point as WKT Point.
    pub end_wkt: String,
    /// Distance in meters.
    pub distance_meters: f64,
    /// Total elevation gain in meters.
    pub elevation_gain_meters: Option<f64>,
    /// Total elevation loss in meters.
    pub elevation_loss_meters: Option<f64>,
    /// Average grade as a fraction (0.05 = 5%).
    pub average_grade: Option<f64>,
    /// Maximum grade as a fraction.
    pub max_grade: Option<f64>,
    /// Climb category (0=HC, 1-4=categories, None=uncategorized).
    pub climb_category: Option<i32>,
}

/// Configuration for segment extraction.
#[derive(Debug, Clone)]
pub struct SegmentExtractConfig {
    /// Minimum segment length in meters.
    pub min_length_m: f64,
    /// Maximum segment length in meters.
    pub max_length_m: f64,
    /// Minimum elevation gain to be considered a climb (meters).
    pub min_climb_gain_m: f64,
}

impl Default for SegmentExtractConfig {
    fn default() -> Self {
        Self {
            min_length_m: 200.0,
            max_length_m: 5000.0,
            min_climb_gain_m: 10.0,
        }
    }
}

/// Generates segments from track data.
pub struct SegmentGenerator {
    config: SegmentExtractConfig,
}

impl SegmentGenerator {
    /// Creates a new segment generator with default configuration.
    pub fn new() -> Self {
        Self {
            config: SegmentExtractConfig::default(),
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(config: SegmentExtractConfig) -> Self {
        Self { config }
    }

    /// Extracts a segment from a portion of a track.
    ///
    /// # Arguments
    /// * `creator_id` - User who creates the segment
    /// * `points` - Full track points
    /// * `start_fraction` - Start position as fraction of track (0.0 - 1.0)
    /// * `end_fraction` - End position as fraction of track (0.0 - 1.0)
    /// * `activity_type` - Activity type for the segment
    /// * `name` - Segment name
    pub fn extract_from_track(
        &self,
        creator_id: Uuid,
        points: &[TrackPointData],
        start_fraction: f64,
        end_fraction: f64,
        activity_type: ActivityType,
        name: impl Into<String>,
        rng: &mut impl Rng,
    ) -> Option<GeneratedSegment> {
        if points.len() < 2 || start_fraction >= end_fraction {
            return None;
        }

        let start_idx = (start_fraction * points.len() as f64) as usize;
        let end_idx = ((end_fraction * points.len() as f64) as usize).min(points.len());

        if end_idx <= start_idx + 1 {
            return None;
        }

        let segment_points = &points[start_idx..end_idx];
        self.from_points(creator_id, segment_points, activity_type, name, rng)
    }

    /// Creates a segment from a set of points.
    pub fn from_points(
        &self,
        creator_id: Uuid,
        points: &[TrackPointData],
        activity_type: ActivityType,
        name: impl Into<String>,
        rng: &mut impl Rng,
    ) -> Option<GeneratedSegment> {
        if points.len() < 2 {
            return None;
        }

        let (distance, gain, loss, avg_grade, max_grade) = self.calculate_stats(points);

        if distance < self.config.min_length_m || distance > self.config.max_length_m {
            return None;
        }

        let climb_category = self.calculate_climb_category(gain, distance, avg_grade);

        let geo_wkt = self.points_to_linestring_wkt(points);
        let start_wkt = format!("POINT({} {})", points[0].lon, points[0].lat);
        let end_wkt = format!(
            "POINT({} {})",
            points[points.len() - 1].lon,
            points[points.len() - 1].lat
        );

        let visibility = if rng.r#gen::<f64>() < 0.95 {
            Visibility::Public
        } else {
            Visibility::Private
        };

        Some(GeneratedSegment {
            id: Uuid::new_v4(),
            creator_id,
            name: name.into(),
            description: None,
            activity_type,
            visibility,
            geo_wkt,
            start_wkt,
            end_wkt,
            distance_meters: distance,
            elevation_gain_meters: if gain > 0.0 { Some(gain) } else { None },
            elevation_loss_meters: if loss > 0.0 { Some(loss) } else { None },
            average_grade: if avg_grade.abs() > 0.001 { Some(avg_grade) } else { None },
            max_grade: if max_grade.abs() > 0.001 { Some(max_grade) } else { None },
            climb_category,
        })
    }

    /// Automatically finds and extracts climb segments from a track.
    ///
    /// Identifies uphill sections that meet the minimum gain threshold.
    pub fn extract_climbs(
        &self,
        creator_id: Uuid,
        points: &[TrackPointData],
        activity_type: ActivityType,
        rng: &mut impl Rng,
    ) -> Vec<GeneratedSegment> {
        if points.len() < 3 {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut climb_start: Option<usize> = None;
        let mut current_gain = 0.0;
        let mut current_loss = 0.0;

        for i in 1..points.len() {
            let prev_elev = points[i - 1].elevation.unwrap_or(0.0);
            let curr_elev = points[i].elevation.unwrap_or(0.0);
            let delta = curr_elev - prev_elev;

            if delta > 0.0 {
                // Going uphill
                if climb_start.is_none() {
                    climb_start = Some(i - 1);
                    current_gain = 0.0;
                    current_loss = 0.0;
                }
                current_gain += delta;
            } else if delta < 0.0 {
                current_loss += -delta;

                // If we lose significant elevation, end the climb
                if current_loss > self.config.min_climb_gain_m / 2.0 {
                    if let Some(start) = climb_start {
                        if current_gain >= self.config.min_climb_gain_m {
                            let climb_points = &points[start..i];
                            let name = format!("Climb {}", segments.len() + 1);
                            if let Some(seg) = self.from_points(
                                creator_id,
                                climb_points,
                                activity_type,
                                name,
                                rng,
                            ) {
                                segments.push(seg);
                            }
                        }
                    }
                    climb_start = None;
                    current_gain = 0.0;
                    current_loss = 0.0;
                }
            }
        }

        // Handle climb at end of track
        if let Some(start) = climb_start {
            if current_gain >= self.config.min_climb_gain_m {
                let climb_points = &points[start..];
                let name = format!("Climb {}", segments.len() + 1);
                if let Some(seg) = self.from_points(creator_id, climb_points, activity_type, name, rng) {
                    segments.push(seg);
                }
            }
        }

        segments
    }

    /// Calculates segment statistics from points.
    fn calculate_stats(&self, points: &[TrackPointData]) -> (f64, f64, f64, f64, f64) {
        if points.len() < 2 {
            return (0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let mut total_distance = 0.0;
        let mut total_gain = 0.0;
        let mut total_loss = 0.0;
        let mut max_grade = 0.0_f64;
        let mut grades = Vec::new();

        for window in points.windows(2) {
            let p1 = &window[0];
            let p2 = &window[1];

            let segment_dist = haversine_distance(p1.lat, p1.lon, p2.lat, p2.lon);
            total_distance += segment_dist;

            if let (Some(e1), Some(e2)) = (p1.elevation, p2.elevation) {
                let delta = e2 - e1;
                if delta > 0.0 {
                    total_gain += delta;
                } else {
                    total_loss += -delta;
                }

                if segment_dist > 1.0 {
                    let grade = delta / segment_dist;
                    grades.push(grade);
                    if grade.abs() > max_grade.abs() {
                        max_grade = grade;
                    }
                }
            }
        }

        let avg_grade = if total_distance > 0.0 {
            let start_elev = points.first().and_then(|p| p.elevation).unwrap_or(0.0);
            let end_elev = points.last().and_then(|p| p.elevation).unwrap_or(0.0);
            (end_elev - start_elev) / total_distance
        } else {
            0.0
        };

        (total_distance, total_gain, total_loss, avg_grade, max_grade)
    }

    /// Calculates climb category based on the formula from migration 006.
    ///
    /// Points = elevation_gain_meters * (distance_meters / 1000) * grade_factor
    /// where grade_factor increases with steepness.
    fn calculate_climb_category(
        &self,
        elevation_gain: f64,
        distance: f64,
        avg_grade: f64,
    ) -> Option<i32> {
        if elevation_gain < self.config.min_climb_gain_m {
            return None;
        }

        // Grade factor: steeper = harder
        let grade_factor = 1.0 + (avg_grade.abs() * 10.0);
        let points = elevation_gain * (distance / 1000.0) * grade_factor;

        // Category thresholds from migration 006
        match points {
            p if p >= 320.0 => Some(0), // HC
            p if p >= 160.0 => Some(1), // Cat 1
            p if p >= 80.0 => Some(2),  // Cat 2
            p if p >= 40.0 => Some(3),  // Cat 3
            p if p >= 20.0 => Some(4),  // Cat 4
            _ => None,
        }
    }

    /// Converts points to WKT LineString format.
    fn points_to_linestring_wkt(&self, points: &[TrackPointData]) -> String {
        let coords: Vec<String> = points
            .iter()
            .map(|p| format!("{} {}", p.lon, p.lat))
            .collect();
        format!("LINESTRING({})", coords.join(", "))
    }
}

impl Default for SegmentGenerator {
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

    fn make_climb_track() -> Vec<TrackPointData> {
        // 500m distance with 50m elevation gain
        (0..50)
            .map(|i| TrackPointData {
                lat: 40.0 + (i as f64 * 0.0001),
                lon: -105.3,
                elevation: Some(1650.0 + i as f64),
                timestamp: None,
            })
            .collect()
    }

    #[test]
    fn test_extract_from_track() {
        let segment_gen = SegmentGenerator::new();
        let mut rng = rand::thread_rng();
        let points = make_climb_track();

        let segment = segment_gen.extract_from_track(
            Uuid::new_v4(),
            &points,
            0.1,
            0.9,
            ActivityType::Running,
            "Test Segment",
            &mut rng,
        );

        assert!(segment.is_some());
        let seg = segment.unwrap();
        assert_eq!(seg.name, "Test Segment");
        assert!(seg.distance_meters > 0.0);
    }

    #[test]
    fn test_climb_category() {
        let segment_gen = SegmentGenerator::new();

        // Small climb: Cat 4
        let cat = segment_gen.calculate_climb_category(30.0, 500.0, 0.06);
        assert_eq!(cat, Some(4));

        // No category for flat
        let cat = segment_gen.calculate_climb_category(5.0, 1000.0, 0.005);
        assert_eq!(cat, None);
    }
}
