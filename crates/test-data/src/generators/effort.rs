//! Segment effort generation with realistic time distributions.

use rand::Rng;
use rand_distr::{Distribution, LogNormal, Normal};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::segment::GeneratedSegment;
use crate::config::SkillDistribution;
use crate::profiles::AthleteProfile;

/// Generated effort data ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedEffort {
    pub id: Uuid,
    pub segment_id: Uuid,
    pub activity_id: Uuid,
    pub user_id: Uuid,
    pub started_at: OffsetDateTime,
    pub elapsed_time_seconds: f64,
    pub moving_time_seconds: Option<f64>,
    pub average_speed_mps: Option<f64>,
    pub max_speed_mps: Option<f64>,
    pub start_fraction: f64,
    pub end_fraction: f64,
}

/// Configuration for effort generation.
#[derive(Debug, Clone)]
pub struct EffortGenConfig {
    /// Distribution of performance levels.
    pub skill_distribution: SkillDistribution,
    /// Coefficient of variation for individual effort times.
    pub time_variance: f64,
    /// Probability that moving_time differs from elapsed_time (has pauses).
    pub pause_probability: f64,
    /// Range of pause time as fraction of total time.
    pub pause_fraction_range: (f64, f64),
}

impl Default for EffortGenConfig {
    fn default() -> Self {
        Self {
            skill_distribution: SkillDistribution::power_law(),
            time_variance: 0.15,
            pause_probability: 0.1,
            pause_fraction_range: (0.02, 0.15),
        }
    }
}

/// Generates segment efforts with realistic time distributions.
pub struct EffortGenerator {
    config: EffortGenConfig,
}

impl EffortGenerator {
    /// Creates a new effort generator with default configuration.
    pub fn new() -> Self {
        Self {
            config: EffortGenConfig::default(),
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(config: EffortGenConfig) -> Self {
        Self { config }
    }

    /// Generates efforts for multiple users on a segment.
    ///
    /// Uses the skill distribution to create realistic leaderboard distributions
    /// where a few athletes are fast and most are closer to average.
    pub fn generate_for_segment<P: AthleteProfile>(
        &self,
        segment: &GeneratedSegment,
        user_ids: &[Uuid],
        activity_ids: &[Uuid],
        profile: &P,
        base_time: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> Vec<GeneratedEffort> {
        assert_eq!(user_ids.len(), activity_ids.len());

        // Calculate expected time based on profile and segment characteristics
        let expected_time = self.calculate_expected_time(segment, profile);

        user_ids
            .iter()
            .zip(activity_ids.iter())
            .enumerate()
            .map(|(i, (user_id, activity_id))| {
                self.generate_single(
                    segment,
                    *user_id,
                    *activity_id,
                    expected_time,
                    base_time + Duration::hours(i as i64),
                    rng,
                )
            })
            .collect()
    }

    /// Generates a single effort with appropriate time variance.
    pub fn generate_single(
        &self,
        segment: &GeneratedSegment,
        user_id: Uuid,
        activity_id: Uuid,
        expected_time: f64,
        started_at: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> GeneratedEffort {
        // Apply skill distribution to get this athlete's performance factor
        let skill_factor = self.sample_skill_factor(rng);

        // Apply day-to-day variance
        let variance_factor = self.sample_variance(rng);

        // Final elapsed time
        let elapsed_time = expected_time * skill_factor * variance_factor;

        // Calculate speeds
        let average_speed = segment.distance_meters / elapsed_time;

        // Maybe add pauses (moving time < elapsed time)
        let (moving_time, max_speed) = if rng.r#gen::<f64>() < self.config.pause_probability {
            let pause_fraction = rng
                .gen_range(self.config.pause_fraction_range.0..self.config.pause_fraction_range.1);
            let moving = elapsed_time * (1.0 - pause_fraction);
            let max = segment.distance_meters / moving * 1.2; // Max speed during moving portions
            (Some(moving), Some(max))
        } else {
            (Some(elapsed_time), Some(average_speed * 1.15))
        };

        GeneratedEffort {
            id: Uuid::new_v4(),
            segment_id: segment.id,
            activity_id,
            user_id,
            started_at,
            elapsed_time_seconds: elapsed_time,
            moving_time_seconds: moving_time,
            average_speed_mps: Some(average_speed),
            max_speed_mps: max_speed,
            start_fraction: 0.0, // Placeholder - set by scenario builder
            end_fraction: 1.0,   // Placeholder - set by scenario builder
        }
    }

    /// Calculates expected time for a segment based on athletic profile.
    ///
    /// Uses elevation gain and loss separately to model the asymmetry between
    /// climbing (harder/slower) and descending (easier/faster but not as much).
    /// This produces more realistic times than using average grade alone.
    fn calculate_expected_time<P: AthleteProfile>(
        &self,
        segment: &GeneratedSegment,
        profile: &P,
    ) -> f64 {
        let base_speed = profile.base_speed_mps();
        let distance = segment.distance_meters;
        let elevation_gain = segment.elevation_gain_meters.unwrap_or(0.0);
        let elevation_loss = segment.elevation_loss_meters.unwrap_or(0.0);

        // If no elevation data, use flat terrain calculation
        if elevation_gain == 0.0 && elevation_loss == 0.0 {
            return distance / base_speed;
        }

        // Calculate the horizontal distance for uphill, downhill, and flat portions.
        // We approximate by assuming elevation changes are distributed over the segment.
        let total_elevation_change = elevation_gain + elevation_loss;

        // Estimate the proportion of distance spent climbing vs descending
        // Using a simple model: steeper sections cover less horizontal distance
        let gain_fraction = if total_elevation_change > 0.0 {
            elevation_gain / total_elevation_change
        } else {
            0.5
        };
        let loss_fraction = 1.0 - gain_fraction;

        // Calculate effective grades for uphill and downhill sections
        // The grade is the elevation change divided by the horizontal distance
        let uphill_distance = distance * gain_fraction;
        let downhill_distance = distance * loss_fraction;

        // Avoid division by zero
        let uphill_grade = if uphill_distance > 1.0 {
            elevation_gain / uphill_distance
        } else {
            0.0
        };
        let downhill_grade = if downhill_distance > 1.0 {
            -(elevation_loss / downhill_distance)
        } else {
            0.0
        };

        // Calculate time for each section using profile's grade factors
        let uphill_speed = base_speed * profile.grade_factor(uphill_grade);
        let downhill_speed = base_speed * profile.grade_factor(downhill_grade);

        let uphill_time = if uphill_speed > 0.1 {
            uphill_distance / uphill_speed
        } else {
            uphill_distance / 0.5 // Minimum walking speed
        };

        let downhill_time = if downhill_speed > 0.1 {
            downhill_distance / downhill_speed
        } else {
            downhill_distance / 0.5
        };

        uphill_time + downhill_time
    }

    /// Samples a skill factor from the configured distribution.
    ///
    /// Returns a multiplier where:
    /// - < 1.0 = faster than average (elite)
    /// - 1.0 = average
    /// - > 1.0 = slower than average
    fn sample_skill_factor(&self, rng: &mut impl Rng) -> f64 {
        match self.config.skill_distribution {
            SkillDistribution::Uniform => rng.gen_range(0.7..1.5),

            SkillDistribution::Normal { mean, std_dev } => {
                let normal = Normal::new(mean, std_dev).unwrap();
                normal.sample(rng).clamp(0.5, 2.0)
            }

            SkillDistribution::PowerLaw { alpha } => {
                // Log-normal approximates power-law tail behavior
                // Parameters chosen to give mean ~1.0 with heavy right tail
                let sigma = 0.4 / alpha.sqrt();
                let mu = -0.5 * sigma * sigma; // Ensures mean = 1.0

                let log_normal = LogNormal::new(mu, sigma).unwrap();
                log_normal.sample(rng).clamp(0.5, 3.0)
            }
        }
    }

    /// Samples day-to-day variance.
    fn sample_variance(&self, rng: &mut impl Rng) -> f64 {
        let normal = Normal::new(1.0, self.config.time_variance).unwrap();
        normal.sample(rng).clamp(0.8, 1.3)
    }
}

impl Default for EffortGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::RunnerProfile;
    use tracks::models::builtin_types;

    fn make_test_segment() -> GeneratedSegment {
        GeneratedSegment {
            id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            name: "Test Climb".into(),
            description: None,
            activity_type_id: builtin_types::RUN,
            visibility: tracks::models::Visibility::Public,
            geo_wkt: "LINESTRING(0 0, 1 1)".into(),
            start_wkt: "POINT(0 0)".into(),
            end_wkt: "POINT(1 1)".into(),
            distance_meters: 1000.0,
            elevation_gain_meters: Some(50.0),
            elevation_loss_meters: Some(10.0),
            average_grade: Some(0.05),
            max_grade: Some(0.08),
            climb_category: Some(4),
        }
    }

    fn make_flat_segment() -> GeneratedSegment {
        GeneratedSegment {
            id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            name: "Flat Sprint".into(),
            description: None,
            activity_type_id: builtin_types::RUN,
            visibility: tracks::models::Visibility::Public,
            geo_wkt: "LINESTRING(0 0, 1 1)".into(),
            start_wkt: "POINT(0 0)".into(),
            end_wkt: "POINT(1 1)".into(),
            distance_meters: 1000.0,
            elevation_gain_meters: None,
            elevation_loss_meters: None,
            average_grade: None,
            max_grade: None,
            climb_category: None,
        }
    }

    fn make_descent_segment() -> GeneratedSegment {
        // Pure descent - no uphill sections
        GeneratedSegment {
            id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            name: "Downhill Run".into(),
            description: None,
            activity_type_id: builtin_types::RUN,
            visibility: tracks::models::Visibility::Public,
            geo_wkt: "LINESTRING(0 0, 1 1)".into(),
            start_wkt: "POINT(0 0)".into(),
            end_wkt: "POINT(1 1)".into(),
            distance_meters: 1000.0,
            elevation_gain_meters: Some(0.0),
            elevation_loss_meters: Some(50.0),
            average_grade: Some(-0.05),
            max_grade: Some(-0.08),
            climb_category: None,
        }
    }

    fn make_roller_segment() -> GeneratedSegment {
        // Segment with equal up and down (e.g., goes up 50m then down 50m)
        GeneratedSegment {
            id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            name: "Rolling Hills".into(),
            description: None,
            activity_type_id: builtin_types::RUN,
            visibility: tracks::models::Visibility::Public,
            geo_wkt: "LINESTRING(0 0, 1 1)".into(),
            start_wkt: "POINT(0 0)".into(),
            end_wkt: "POINT(1 1)".into(),
            distance_meters: 1000.0,
            elevation_gain_meters: Some(50.0),
            elevation_loss_meters: Some(50.0),
            average_grade: Some(0.0), // Net zero elevation change
            max_grade: Some(0.10),
            climb_category: None,
        }
    }

    #[test]
    fn test_terrain_affects_time() {
        let effort_gen = EffortGenerator::new();
        let profile = RunnerProfile::default();

        let flat = make_flat_segment();
        let climb = make_test_segment();
        let descent = make_descent_segment();
        let roller = make_roller_segment();

        let flat_time = effort_gen.calculate_expected_time(&flat, &profile);
        let climb_time = effort_gen.calculate_expected_time(&climb, &profile);
        let descent_time = effort_gen.calculate_expected_time(&descent, &profile);
        let roller_time = effort_gen.calculate_expected_time(&roller, &profile);

        // Climbing should be slowest
        assert!(
            climb_time > flat_time,
            "Climb ({climb_time:.1}s) should take longer than flat ({flat_time:.1}s)"
        );

        // Descent should be faster than flat
        assert!(
            descent_time < flat_time,
            "Descent ({descent_time:.1}s) should be faster than flat ({flat_time:.1}s)"
        );

        // Roller (up then down with net zero) should take longer than flat
        // because climbing penalty is greater than descent benefit
        assert!(
            roller_time > flat_time,
            "Roller ({roller_time:.1}s) should take longer than flat ({flat_time:.1}s) due to climb asymmetry"
        );

        // Roller should be faster than pure climb
        assert!(
            roller_time < climb_time,
            "Roller ({roller_time:.1}s) should be faster than climb ({climb_time:.1}s)"
        );
    }

    #[test]
    fn test_generate_efforts() {
        let effort_gen = EffortGenerator::new();
        let segment = make_test_segment();
        let profile = RunnerProfile::default();
        let mut rng = rand::thread_rng();

        let user_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
        let activity_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let efforts = effort_gen.generate_for_segment(
            &segment,
            &user_ids,
            &activity_ids,
            &profile,
            OffsetDateTime::now_utc(),
            &mut rng,
        );

        assert_eq!(efforts.len(), 10);

        // All efforts should have positive times
        for effort in &efforts {
            assert!(effort.elapsed_time_seconds > 0.0);
            assert!(effort.average_speed_mps.unwrap() > 0.0);
        }
    }

    #[test]
    fn test_power_law_distribution() {
        // Use fixed seed for reproducibility
        use rand::SeedableRng;
        let effort_gen = EffortGenerator::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345);

        // Sample many skill factors
        let factors: Vec<f64> = (0..1000)
            .map(|_| effort_gen.sample_skill_factor(&mut rng))
            .collect();

        // Calculate statistics
        let mean: f64 = factors.iter().sum::<f64>() / factors.len() as f64;
        let below_one = factors.iter().filter(|&&f| f < 1.0).count();

        // With log-normal approximation of power law:
        // - Mean should be around 1.0
        // - There should be a mix of above and below 1.0
        assert!(mean > 0.8 && mean < 1.4, "Mean {mean} should be near 1.0");
        assert!(
            below_one > 100,
            "Should have some fast athletes (below 1.0)"
        );
        assert!(
            below_one < 900,
            "Should have more slower athletes (above 1.0)"
        );
    }
}
