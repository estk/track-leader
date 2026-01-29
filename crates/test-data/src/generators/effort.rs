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
    fn calculate_expected_time<P: AthleteProfile>(
        &self,
        segment: &GeneratedSegment,
        profile: &P,
    ) -> f64 {
        let avg_grade = segment.average_grade.unwrap_or(0.0);
        let base_speed = profile.base_speed_mps();
        let grade_factor = profile.grade_factor(avg_grade);

        let effective_speed = base_speed * grade_factor;
        segment.distance_meters / effective_speed
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
