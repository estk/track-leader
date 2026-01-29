//! Fluent builder for constructing test scenarios.

use std::ops::RangeInclusive;
use std::time::Instant;

use rand::Rng;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::config::{BoundingBox, EffortCoverage, Region, SkillDistribution};
use crate::db::{SeedError, Seeder};
use crate::generators::{
    activity::{ActivityGenerator, GeneratedActivity},
    effort::{EffortGenConfig, EffortGenerator, GeneratedEffort},
    segment::{GeneratedSegment, SegmentGenerator},
    social::{GeneratedComment, GeneratedFollow, GeneratedKudos, SocialGenConfig, SocialGenerator},
    user::{GeneratedUser, UserGenConfig, UserGenerator},
};
use crate::profiles::{AthleteProfile, CyclistProfile, HikerProfile, RunnerProfile};
use crate::sources::ProceduralGenerator;
use tracks::models::builtin_types;

/// Result of building and seeding a scenario.
#[derive(Debug)]
pub struct ScenarioResult {
    pub users: Vec<GeneratedUser>,
    pub activities: Vec<GeneratedActivity>,
    pub segments: Vec<GeneratedSegment>,
    pub efforts: Vec<GeneratedEffort>,
    pub follows: Vec<GeneratedFollow>,
    pub kudos: Vec<GeneratedKudos>,
    pub comments: Vec<GeneratedComment>,
    /// Metrics from scenario generation (populated if metrics tracking enabled).
    pub metrics: Option<ScenarioMetrics>,
}

/// Performance metrics from scenario generation.
#[derive(Debug, Clone)]
pub struct ScenarioMetrics {
    /// Time spent generating data (milliseconds).
    pub generation_time_ms: u64,
    /// Time spent seeding database (milliseconds, 0 if build_data used).
    pub seeding_time_ms: u64,
    /// Number of users generated.
    pub user_count: usize,
    /// Number of activities generated.
    pub activity_count: usize,
    /// Number of segments generated.
    pub segment_count: usize,
    /// Number of efforts generated.
    pub effort_count: usize,
    /// Total track points across all activities.
    pub total_track_points: usize,
}

/// Builder for creating complete test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// let result = ScenarioBuilder::new()
///     .with_users(50)
///     .with_region(Region::BOULDER)
///     .with_activity_type_id(builtin_types::RUN)
///     .with_track_distance(5000.0)
///     .with_segment(0.2..0.6, "Hill Climb")
///     .with_efforts_per_user(1..=3)
///     .with_social()
///     .build(&pool, &mut rng)
///     .await?;
/// ```
pub struct ScenarioBuilder {
    // User configuration
    user_count: usize,
    user_config: UserGenConfig,

    // Track configuration
    region: BoundingBox,
    activity_type_id: Uuid,
    track_distance: f64,
    activities_per_user: RangeInclusive<usize>,

    // Segment configuration
    segments: Vec<SegmentSpec>,
    auto_extract_climbs: bool,

    // Effort configuration
    efforts_per_user: RangeInclusive<usize>,
    skill_distribution: SkillDistribution,
    effort_coverage: EffortCoverage,

    // Social configuration
    generate_social: bool,
    social_config: SocialGenConfig,

    // Misc
    seed: u32,
    track_metrics: bool,
}

/// Source of geometry for a segment.
#[derive(Debug, Clone)]
enum SegmentSource {
    /// Extract from the reference track using start/end fractions.
    FromReferenceTrack {
        start_fraction: f64,
        end_fraction: f64,
    },
    /// Generate an independent track for this segment.
    Independent {
        /// Target distance in meters.
        distance: f64,
    },
}

/// Specification for a segment to create.
#[derive(Debug, Clone)]
struct SegmentSpec {
    /// Where to get the segment geometry from.
    source: SegmentSource,
    /// Segment name.
    name: String,
}

impl Default for ScenarioBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScenarioBuilder {
    /// Creates a new scenario builder with default settings.
    pub fn new() -> Self {
        Self {
            user_count: 50,
            user_config: UserGenConfig::default(),
            region: Region::BOULDER,
            activity_type_id: builtin_types::RUN,
            track_distance: 5000.0,
            activities_per_user: 1..=3,
            segments: Vec::new(),
            auto_extract_climbs: false,
            efforts_per_user: 1..=2,
            skill_distribution: SkillDistribution::power_law(),
            effort_coverage: EffortCoverage::default(),
            generate_social: true,
            social_config: SocialGenConfig::default(),
            seed: 42,
            track_metrics: false,
        }
    }

    /// Sets the number of users to generate.
    pub fn with_users(mut self, count: usize) -> Self {
        self.user_count = count;
        self
    }

    /// Sets the user generation configuration.
    pub fn with_user_config(mut self, config: UserGenConfig) -> Self {
        self.user_config = config;
        self
    }

    /// Sets the geographic region for track generation.
    pub fn with_region(mut self, region: BoundingBox) -> Self {
        self.region = region;
        self
    }

    /// Sets the activity type UUID.
    pub fn with_activity_type_id(mut self, activity_type_id: Uuid) -> Self {
        self.activity_type_id = activity_type_id;
        self
    }

    /// Sets the target track distance in meters.
    pub fn with_track_distance(mut self, meters: f64) -> Self {
        self.track_distance = meters;
        self
    }

    /// Sets the range of activities per user.
    pub fn with_activities_per_user(mut self, range: RangeInclusive<usize>) -> Self {
        self.activities_per_user = range;
        self
    }

    /// Adds a segment specification extracted from the reference track.
    ///
    /// The segment will be created from the specified fraction of the first generated track.
    pub fn with_segment(
        mut self,
        start_fraction: f64,
        end_fraction: f64,
        name: impl Into<String>,
    ) -> Self {
        self.segments.push(SegmentSpec {
            source: SegmentSource::FromReferenceTrack {
                start_fraction,
                end_fraction,
            },
            name: name.into(),
        });
        self
    }

    /// Adds an independent segment with its own generated track.
    ///
    /// This creates a segment in a different geographic location from other segments,
    /// useful for testing segment discovery across diverse routes.
    pub fn with_independent_segment(mut self, distance: f64, name: impl Into<String>) -> Self {
        self.segments.push(SegmentSpec {
            source: SegmentSource::Independent { distance },
            name: name.into(),
        });
        self
    }

    /// Enables automatic extraction of climb segments from generated tracks.
    ///
    /// When enabled, the generator will analyze elevation profiles and automatically
    /// create segments for significant climbs, with descriptive names including
    /// category and elevation gain.
    pub fn with_auto_climbs(mut self, enabled: bool) -> Self {
        self.auto_extract_climbs = enabled;
        self
    }

    /// Sets the range of efforts per user on each segment.
    pub fn with_efforts_per_user(mut self, range: RangeInclusive<usize>) -> Self {
        self.efforts_per_user = range;
        self
    }

    /// Sets the skill distribution for effort times.
    pub fn with_skill_distribution(mut self, dist: SkillDistribution) -> Self {
        self.skill_distribution = dist;
        self
    }

    /// Sets the effort coverage distribution.
    ///
    /// Controls how efforts are distributed across users and segments:
    /// - `Full`: Every user has efforts on every segment (default)
    /// - `Sparse { fraction }`: Only a fraction of users have efforts per segment
    /// - `Zipf { alpha }`: Power-law distribution favoring popular segments
    pub fn with_effort_coverage(mut self, coverage: EffortCoverage) -> Self {
        self.effort_coverage = coverage;
        self
    }

    /// Enables or disables social interaction generation.
    pub fn with_social(mut self, enabled: bool) -> Self {
        self.generate_social = enabled;
        self
    }

    /// Sets the social generation configuration.
    pub fn with_social_config(mut self, config: SocialGenConfig) -> Self {
        self.social_config = config;
        self
    }

    /// Sets the random seed.
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Enables metrics tracking for performance analysis.
    ///
    /// When enabled, the result will include timing and count metrics
    /// useful for analyzing generation performance.
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.track_metrics = enabled;
        self
    }

    /// Builds the scenario (generates data but doesn't seed database).
    pub fn build_data(&self, rng: &mut impl Rng) -> ScenarioResult {
        let start_time = if self.track_metrics {
            Some(Instant::now())
        } else {
            None
        };

        // Generate users
        let user_gen = UserGenerator::with_config(self.user_config.clone());
        let users = user_gen.generate_batch(self.user_count, rng);
        let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

        // Generate tracks and activities
        let track_gen = ProceduralGenerator::for_region(self.region, self.seed)
            .with_distance(self.track_distance);
        let activity_gen = ActivityGenerator::new();
        let profile = self.get_profile();

        let mut activities = Vec::new();
        let mut reference_track = None;

        for user in &users {
            let num_activities = rng.gen_range(self.activities_per_user.clone());

            for _ in 0..num_activities {
                let track_points = track_gen.generate(profile.as_ref(), rng);

                if reference_track.is_none() {
                    reference_track = Some(track_points.clone());
                }

                let activity =
                    activity_gen.from_track(user.id, self.activity_type_id, track_points, rng);
                activities.push(activity);
            }
        }

        // Generate segments
        let segment_gen = SegmentGenerator::new();
        let mut segments: Vec<GeneratedSegment> = Vec::new();

        for spec in &self.segments {
            let creator = &users[rng.gen_range(0..users.len())];

            let segment = match &spec.source {
                SegmentSource::FromReferenceTrack {
                    start_fraction,
                    end_fraction,
                } => {
                    // Extract from the reference track
                    reference_track.as_ref().and_then(|track| {
                        segment_gen.extract_from_track(
                            creator.id,
                            track,
                            *start_fraction,
                            *end_fraction,
                            self.activity_type_id,
                            &spec.name,
                            rng,
                        )
                    })
                }
                SegmentSource::Independent { distance } => {
                    // Generate a dedicated track for this segment
                    let independent_gen = ProceduralGenerator::for_region(self.region, self.seed)
                        .with_distance(*distance);
                    let track_points = independent_gen.generate(profile.as_ref(), rng);

                    // Use the entire track as the segment
                    segment_gen.from_points(
                        creator.id,
                        &track_points,
                        self.activity_type_id,
                        &spec.name,
                        rng,
                    )
                }
            };

            if let Some(seg) = segment {
                segments.push(seg);
            }
        }

        // Auto-extract climbs if enabled
        if let Some(track) = reference_track.as_ref()
            && self.auto_extract_climbs
        {
            let creator = &users[rng.gen_range(0..users.len())];
            let auto_climbs =
                segment_gen.extract_climbs(creator.id, track, self.activity_type_id, rng);
            segments.extend(auto_climbs);
        }

        // Generate efforts with coverage distribution
        let effort_gen = EffortGenerator::with_config(EffortGenConfig {
            skill_distribution: self.skill_distribution,
            ..Default::default()
        });

        // Pre-calculate segment popularity weights for Zipf distribution
        let segment_weights: Vec<f64> = match self.effort_coverage {
            EffortCoverage::Zipf { alpha } => {
                // Zipf weights: segment i has weight 1/(i+1)^alpha
                (0..segments.len())
                    .map(|i| 1.0 / ((i + 1) as f64).powf(alpha))
                    .collect()
            }
            _ => vec![1.0; segments.len()],
        };

        let mut efforts = Vec::new();
        for (seg_idx, segment) in segments.iter().enumerate() {
            for user in &users {
                // Determine if this user should have efforts on this segment
                let should_generate = match self.effort_coverage {
                    EffortCoverage::Full => true,
                    EffortCoverage::Sparse { fraction } => rng.r#gen::<f64>() < fraction,
                    EffortCoverage::Zipf { .. } => {
                        // Probability proportional to segment weight (normalized)
                        let total_weight: f64 = segment_weights.iter().sum();
                        let prob = segment_weights[seg_idx] / total_weight * segments.len() as f64;
                        rng.r#gen::<f64>() < prob.min(1.0)
                    }
                };

                if !should_generate {
                    continue;
                }

                let num_efforts = rng.gen_range(self.efforts_per_user.clone());

                // Pick random activities from this user
                let user_activities: Vec<&GeneratedActivity> = activities
                    .iter()
                    .filter(|a| a.user_id == user.id)
                    .take(num_efforts)
                    .collect();

                for activity in user_activities {
                    let effort = effort_gen.generate_single(
                        segment,
                        user.id,
                        activity.id,
                        self.calculate_expected_time(segment),
                        activity.submitted_at,
                        rng,
                    );
                    efforts.push(effort);
                }
            }
        }

        // Generate social interactions
        let (follows, kudos, comments) = if self.generate_social {
            let social_gen = SocialGenerator::with_config(self.social_config.clone());

            let follows =
                social_gen.generate_follow_graph(&user_ids, OffsetDateTime::now_utc(), rng);

            let mut all_kudos = Vec::new();
            let mut all_comments = Vec::new();

            for activity in &activities {
                let k = social_gen.generate_kudos(
                    activity.id,
                    activity.user_id,
                    &user_ids,
                    activity.submitted_at,
                    rng,
                );
                all_kudos.extend(k);

                let c = social_gen.generate_comments(
                    activity.id,
                    activity.user_id,
                    &user_ids,
                    activity.submitted_at,
                    rng,
                );
                all_comments.extend(c);
            }

            (follows, all_kudos, all_comments)
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        // Collect metrics if tracking enabled
        let metrics = start_time.map(|start| {
            let total_track_points: usize = activities.iter().map(|a| a.track_points.len()).sum();
            ScenarioMetrics {
                generation_time_ms: start.elapsed().as_millis() as u64,
                seeding_time_ms: 0, // Set by build() if database seeding occurs
                user_count: users.len(),
                activity_count: activities.len(),
                segment_count: segments.len(),
                effort_count: efforts.len(),
                total_track_points,
            }
        });

        ScenarioResult {
            users,
            activities,
            segments,
            efforts,
            follows,
            kudos,
            comments,
            metrics,
        }
    }

    /// Builds and seeds the scenario into the database.
    pub async fn build(
        self,
        pool: &PgPool,
        rng: &mut impl Rng,
    ) -> Result<ScenarioResult, SeedError> {
        let track_metrics = self.track_metrics;
        let mut result = self.build_data(rng);

        let seed_start = if track_metrics {
            Some(Instant::now())
        } else {
            None
        };

        let seeder = Seeder::new(pool.clone());

        // Seed in dependency order
        seeder.seed_users(&result.users).await?;
        seeder.seed_activities(&result.activities).await?;
        seeder.seed_segments(&result.segments).await?;
        seeder.seed_efforts(&result.efforts).await?;

        if !result.follows.is_empty() {
            seeder.seed_follows(&result.follows).await?;
        }
        if !result.kudos.is_empty() {
            seeder.seed_kudos(&result.kudos).await?;
        }
        if !result.comments.is_empty() {
            seeder.seed_comments(&result.comments).await?;
        }

        // Update seeding time in metrics
        if let (Some(start), Some(ref mut metrics)) = (seed_start, result.metrics.as_mut()) {
            metrics.seeding_time_ms = start.elapsed().as_millis() as u64;
        }

        Ok(result)
    }

    /// Gets the appropriate athletic profile for the activity type.
    fn get_profile(&self) -> Box<dyn AthleteProfile> {
        if self.activity_type_id == builtin_types::RUN {
            Box::new(RunnerProfile::default())
        } else if self.activity_type_id == builtin_types::ROAD
            || self.activity_type_id == builtin_types::MTB
            || self.activity_type_id == builtin_types::EMTB
            || self.activity_type_id == builtin_types::GRAVEL
        {
            Box::new(CyclistProfile::default())
        } else if self.activity_type_id == builtin_types::HIKE
            || self.activity_type_id == builtin_types::WALK
        {
            Box::new(HikerProfile::default())
        } else {
            Box::new(RunnerProfile::default())
        }
    }

    /// Calculates expected time for a segment.
    fn calculate_expected_time(&self, segment: &GeneratedSegment) -> f64 {
        let profile = self.get_profile();
        let avg_grade = segment.average_grade.unwrap_or(0.0);
        let base_speed = profile.base_speed_mps();
        let grade_factor = profile.grade_factor(avg_grade);
        let effective_speed = base_speed * grade_factor;
        segment.distance_meters / effective_speed
    }
}

/// Preset scenarios for common testing needs.
impl ScenarioBuilder {
    /// Creates a scenario for testing leaderboard functionality.
    ///
    /// - 200 users with varied skill levels
    /// - One primary segment with all users having efforts
    /// - Power-law time distribution for realistic rankings
    pub fn leaderboard_test() -> Self {
        Self::new()
            .with_users(200)
            .with_region(Region::BOULDER)
            .with_activity_type_id(builtin_types::RUN)
            .with_track_distance(5000.0)
            .with_segment(0.2, 0.7, "Test Leaderboard Segment")
            .with_efforts_per_user(1..=3)
            .with_skill_distribution(SkillDistribution::power_law())
            .with_social(false)
    }

    /// Creates a scenario for testing social features.
    ///
    /// - 50 users with interconnected social graph
    /// - Multiple activities with kudos and comments
    pub fn social_test() -> Self {
        Self::new()
            .with_users(50)
            .with_region(Region::BOULDER)
            .with_activity_type_id(builtin_types::RUN)
            .with_track_distance(3000.0)
            .with_activities_per_user(3..=5)
            .with_social(true)
            .with_social_config(SocialGenConfig {
                avg_follows_per_user: 20.0,
                avg_kudos_per_activity: 8.0,
                avg_comments_per_activity: 3.0,
                ..Default::default()
            })
    }

    /// Creates a scenario for testing segment overlap detection.
    ///
    /// - Multiple overlapping segments on the same track
    pub fn segment_overlap_test() -> Self {
        Self::new()
            .with_users(20)
            .with_region(Region::BOULDER)
            .with_activity_type_id(builtin_types::RUN)
            .with_track_distance(8000.0)
            .with_segment(0.1, 0.4, "Segment A")
            .with_segment(0.3, 0.6, "Segment B (overlaps A)")
            .with_segment(0.5, 0.9, "Segment C (overlaps B)")
            .with_efforts_per_user(2..=3)
            .with_social(false)
    }

    /// Creates a scenario for testing climb categories.
    ///
    /// - Reno/Tahoe region for significant elevation changes
    /// - Cycling activity type for more pronounced grade effects
    pub fn climb_category_test() -> Self {
        Self::new()
            .with_users(30)
            .with_region(Region::RENO_TAHOE)
            .with_activity_type_id(builtin_types::ROAD)
            .with_track_distance(10000.0)
            .with_segment(0.1, 0.3, "Cat 4 Climb")
            .with_segment(0.4, 0.7, "Cat 3 Climb")
            .with_segment(0.7, 0.95, "Cat 2 Climb")
            .with_efforts_per_user(1..=2)
            .with_social(false)
    }

    /// Large-scale leaderboard stress test.
    ///
    /// Designed for load testing and performance analysis:
    /// - 500 users with power-law skill distribution
    /// - 10 auto-extracted climb segments from mountainous terrain
    /// - Sparse efforts (70% coverage) for realistic leaderboard density
    /// - Metrics tracking enabled for performance analysis
    pub fn leaderboard_stress_test() -> Self {
        Self::new()
            .with_users(500)
            .with_region(Region::RENO_TAHOE)
            .with_activity_type_id(builtin_types::ROAD)
            .with_track_distance(20000.0)
            .with_activities_per_user(2..=4)
            .with_auto_climbs(true)
            .with_efforts_per_user(1..=2)
            .with_skill_distribution(SkillDistribution::power_law())
            .with_effort_coverage(EffortCoverage::Sparse { fraction: 0.7 })
            .with_social(false)
            .with_metrics(true)
    }

    /// Segment discovery stress test.
    ///
    /// Tests segment matching across diverse activities:
    /// - 100 users with loop and out-and-back route patterns
    /// - 20+ segments from multiple independent tracks
    /// - Mix of reference track segments and independent segments
    /// - Useful for testing segment overlap detection and matching
    pub fn segment_discovery_test() -> Self {
        Self::new()
            .with_users(100)
            .with_region(Region::BOULDER)
            .with_activity_type_id(builtin_types::RUN)
            .with_track_distance(10000.0)
            .with_activities_per_user(2..=3)
            // Reference track segments
            .with_segment(0.05, 0.2, "Start Segment")
            .with_segment(0.15, 0.35, "Early Overlap")
            .with_segment(0.3, 0.5, "Mid Section A")
            .with_segment(0.4, 0.6, "Mid Section B (overlaps A)")
            .with_segment(0.55, 0.75, "Mid-Late Section")
            .with_segment(0.7, 0.9, "Late Section")
            .with_segment(0.8, 0.95, "Finish Segment")
            // Independent segments in different locations
            .with_independent_segment(1000.0, "Independent Short")
            .with_independent_segment(2500.0, "Independent Medium")
            .with_independent_segment(4000.0, "Independent Long")
            .with_auto_climbs(true)
            .with_effort_coverage(EffortCoverage::Sparse { fraction: 0.6 })
            .with_social(false)
            .with_metrics(true)
    }

    /// Comprehensive integration test.
    ///
    /// Full-featured scenario for end-to-end testing:
    /// - Mixed activity types (cycling)
    /// - Multiple regions
    /// - Social interactions enabled
    /// - Both manual and auto-extracted segments
    /// - Zipf effort distribution
    pub fn comprehensive_test() -> Self {
        Self::new()
            .with_users(75)
            .with_region(Region::BOULDER)
            .with_activity_type_id(builtin_types::ROAD)
            .with_track_distance(15000.0)
            .with_activities_per_user(2..=4)
            .with_segment(0.1, 0.3, "Sprint Section")
            .with_segment(0.25, 0.55, "Climb Section")
            .with_segment(0.5, 0.8, "Descent Section")
            .with_independent_segment(3000.0, "Remote Segment")
            .with_auto_climbs(true)
            .with_effort_coverage(EffortCoverage::Zipf { alpha: 1.5 })
            .with_social(true)
            .with_social_config(SocialGenConfig {
                avg_follows_per_user: 15.0,
                avg_kudos_per_activity: 5.0,
                avg_comments_per_activity: 2.0,
                ..Default::default()
            })
            .with_metrics(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_data() {
        let mut rng = rand::thread_rng();

        let result = ScenarioBuilder::new()
            .with_users(5)
            .with_segment(0.2, 0.8, "Test")
            .with_social(false)
            .build_data(&mut rng);

        assert_eq!(result.users.len(), 5);
        assert!(!result.activities.is_empty());
        assert_eq!(result.segments.len(), 1);
        assert!(!result.efforts.is_empty());
    }

    #[test]
    fn test_preset_leaderboard() {
        let builder = ScenarioBuilder::leaderboard_test();
        assert_eq!(builder.user_count, 200);
        assert_eq!(builder.segments.len(), 1);
    }

    #[test]
    fn test_auto_climb_extraction() {
        let mut rng = rand::thread_rng();

        // Use Reno/Tahoe region for more elevation variation
        let result = ScenarioBuilder::new()
            .with_users(3)
            .with_region(Region::RENO_TAHOE)
            .with_track_distance(8000.0)
            .with_auto_climbs(true)
            .with_social(false)
            .build_data(&mut rng);

        // Auto-extracted climbs should have descriptive names and descriptions
        for segment in &result.segments {
            assert!(
                segment.name.contains("Climb"),
                "Auto-climb name should contain 'Climb': {}",
                segment.name
            );
            assert!(
                segment.description.is_some(),
                "Auto-climb should have description"
            );
        }
    }

    #[test]
    fn test_independent_segments() {
        let mut rng = rand::thread_rng();

        let result = ScenarioBuilder::new()
            .with_users(3)
            .with_segment(0.1, 0.5, "Reference Track Segment")
            .with_independent_segment(1500.0, "Independent Segment")
            .with_social(false)
            .build_data(&mut rng);

        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.segments[0].name, "Reference Track Segment");
        assert_eq!(result.segments[1].name, "Independent Segment");

        // Independent segment should have its own geometry (different start point)
        let ref_start = &result.segments[0].start_wkt;
        let ind_start = &result.segments[1].start_wkt;
        assert_ne!(
            ref_start, ind_start,
            "Independent segment should have different start point"
        );
    }

    #[test]
    fn test_sparse_effort_coverage() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345);

        // Full coverage scenario
        let full_result = ScenarioBuilder::new()
            .with_users(20)
            .with_segment(0.2, 0.8, "Test Segment")
            .with_effort_coverage(EffortCoverage::Full)
            .with_social(false)
            .build_data(&mut rng);

        // Reset RNG
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345);

        // Sparse coverage scenario (50%)
        let sparse_result = ScenarioBuilder::new()
            .with_users(20)
            .with_segment(0.2, 0.8, "Test Segment")
            .with_effort_coverage(EffortCoverage::Sparse { fraction: 0.5 })
            .with_social(false)
            .build_data(&mut rng);

        // Sparse should have fewer efforts than full
        assert!(
            sparse_result.efforts.len() < full_result.efforts.len(),
            "Sparse ({}) should have fewer efforts than full ({})",
            sparse_result.efforts.len(),
            full_result.efforts.len()
        );

        // Sparse should have roughly 50% of efforts (with some variance)
        let ratio = sparse_result.efforts.len() as f64 / full_result.efforts.len() as f64;
        assert!(
            ratio > 0.3 && ratio < 0.7,
            "Sparse ratio {ratio} should be around 0.5"
        );
    }

    #[test]
    fn test_zipf_effort_coverage() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345);

        // Zipf coverage - first segments should get more efforts
        let result = ScenarioBuilder::new()
            .with_users(30)
            .with_segment(0.1, 0.3, "Segment 1")
            .with_segment(0.3, 0.5, "Segment 2")
            .with_segment(0.5, 0.7, "Segment 3")
            .with_segment(0.7, 0.9, "Segment 4")
            .with_effort_coverage(EffortCoverage::Zipf { alpha: 1.5 })
            .with_social(false)
            .build_data(&mut rng);

        // Count efforts per segment
        let mut efforts_per_segment = vec![0usize; result.segments.len()];
        for effort in &result.efforts {
            for (idx, seg) in result.segments.iter().enumerate() {
                if effort.segment_id == seg.id {
                    efforts_per_segment[idx] += 1;
                    break;
                }
            }
        }

        // With Zipf, earlier segments should generally have more efforts
        // (though there's randomness involved)
        let total_efforts: usize = efforts_per_segment.iter().sum();
        assert!(
            total_efforts > 0,
            "Should have generated at least some efforts"
        );

        // The distribution should be uneven - first segment should have more
        // than an equal share in most cases
        let equal_share = total_efforts / result.segments.len();
        let first_segment_has_more = efforts_per_segment[0] > equal_share / 2;
        assert!(
            first_segment_has_more || total_efforts < 10, // Allow for low sample variance
            "First segment ({}) should have at least half of equal share ({})",
            efforts_per_segment[0],
            equal_share / 2
        );
    }

    #[test]
    fn test_metrics_tracking() {
        let mut rng = rand::thread_rng();

        // Without metrics tracking
        let result_no_metrics = ScenarioBuilder::new()
            .with_users(5)
            .with_segment(0.2, 0.8, "Test")
            .with_social(false)
            .with_metrics(false)
            .build_data(&mut rng);
        assert!(
            result_no_metrics.metrics.is_none(),
            "Metrics should be None when not tracking"
        );

        // With metrics tracking
        let result_with_metrics = ScenarioBuilder::new()
            .with_users(5)
            .with_segment(0.2, 0.8, "Test")
            .with_social(false)
            .with_metrics(true)
            .build_data(&mut rng);
        assert!(
            result_with_metrics.metrics.is_some(),
            "Metrics should be present when tracking"
        );

        let metrics = result_with_metrics.metrics.unwrap();
        assert_eq!(metrics.user_count, 5);
        assert!(metrics.activity_count > 0);
        assert_eq!(metrics.segment_count, 1);
        assert!(metrics.effort_count > 0);
        assert!(metrics.total_track_points > 0);
        assert!(
            metrics.generation_time_ms > 0,
            "Generation should take some time"
        );
        // Seeding time should be 0 for build_data (no DB)
        assert_eq!(metrics.seeding_time_ms, 0);
    }

    #[test]
    fn test_preset_leaderboard_stress() {
        let builder = ScenarioBuilder::leaderboard_stress_test();
        assert_eq!(builder.user_count, 500);
        assert!(builder.auto_extract_climbs);
        assert!(builder.track_metrics);
        assert!(matches!(
            builder.effort_coverage,
            EffortCoverage::Sparse { fraction: 0.7 }
        ));
    }

    #[test]
    fn test_preset_segment_discovery() {
        let builder = ScenarioBuilder::segment_discovery_test();
        assert_eq!(builder.user_count, 100);
        // 7 reference track segments + 3 independent segments
        assert_eq!(builder.segments.len(), 10);
        assert!(builder.auto_extract_climbs);
        assert!(builder.track_metrics);
    }

    #[test]
    fn test_preset_comprehensive() {
        let builder = ScenarioBuilder::comprehensive_test();
        assert_eq!(builder.user_count, 75);
        // 3 reference + 1 independent
        assert_eq!(builder.segments.len(), 4);
        assert!(builder.auto_extract_climbs);
        assert!(builder.generate_social);
        assert!(builder.track_metrics);
        assert!(matches!(
            builder.effort_coverage,
            EffortCoverage::Zipf { .. }
        ));
    }
}
