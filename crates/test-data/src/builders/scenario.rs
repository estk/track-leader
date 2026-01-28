//! Fluent builder for constructing test scenarios.

use std::ops::RangeInclusive;

use rand::Rng;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::config::{BoundingBox, Region, SkillDistribution};
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
use tracks::models::ActivityType;

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
}

/// Builder for creating complete test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// let result = ScenarioBuilder::new()
///     .with_users(50)
///     .with_region(Region::BOULDER)
///     .with_activity_type(ActivityType::Running)
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
    activity_type: ActivityType,
    track_distance: f64,
    activities_per_user: RangeInclusive<usize>,

    // Segment configuration
    segments: Vec<SegmentSpec>,

    // Effort configuration
    efforts_per_user: RangeInclusive<usize>,
    skill_distribution: SkillDistribution,

    // Social configuration
    generate_social: bool,
    social_config: SocialGenConfig,

    // Misc
    seed: u32,
}

/// Specification for a segment to create.
#[derive(Debug, Clone)]
struct SegmentSpec {
    /// Start position as fraction of track (0.0 - 1.0).
    start_fraction: f64,
    /// End position as fraction of track (0.0 - 1.0).
    end_fraction: f64,
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
            activity_type: ActivityType::Running,
            track_distance: 5000.0,
            activities_per_user: 1..=3,
            segments: Vec::new(),
            efforts_per_user: 1..=2,
            skill_distribution: SkillDistribution::power_law(),
            generate_social: true,
            social_config: SocialGenConfig::default(),
            seed: 42,
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

    /// Sets the activity type.
    pub fn with_activity_type(mut self, activity_type: ActivityType) -> Self {
        self.activity_type = activity_type;
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

    /// Adds a segment specification.
    ///
    /// The segment will be created from the specified fraction of the first generated track.
    pub fn with_segment(
        mut self,
        start_fraction: f64,
        end_fraction: f64,
        name: impl Into<String>,
    ) -> Self {
        self.segments.push(SegmentSpec {
            start_fraction,
            end_fraction,
            name: name.into(),
        });
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

    /// Builds the scenario (generates data but doesn't seed database).
    pub fn build_data(&self, rng: &mut impl Rng) -> ScenarioResult {
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

                let activity = activity_gen.from_track(
                    user.id,
                    self.activity_type,
                    track_points,
                    rng,
                );
                activities.push(activity);
            }
        }

        // Generate segments from reference track
        let segment_gen = SegmentGenerator::new();
        let segments: Vec<GeneratedSegment> = if let Some(ref track) = reference_track {
            self.segments
                .iter()
                .filter_map(|spec| {
                    let creator = &users[rng.gen_range(0..users.len())];
                    segment_gen.extract_from_track(
                        creator.id,
                        track,
                        spec.start_fraction,
                        spec.end_fraction,
                        self.activity_type,
                        &spec.name,
                        rng,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };

        // Generate efforts
        let effort_gen = EffortGenerator::with_config(EffortGenConfig {
            skill_distribution: self.skill_distribution,
            ..Default::default()
        });

        let mut efforts = Vec::new();
        for segment in &segments {
            for user in &users {
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

            let follows = social_gen.generate_follow_graph(&user_ids, OffsetDateTime::now_utc(), rng);

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

        ScenarioResult {
            users,
            activities,
            segments,
            efforts,
            follows,
            kudos,
            comments,
        }
    }

    /// Builds and seeds the scenario into the database.
    pub async fn build(self, pool: &PgPool, rng: &mut impl Rng) -> Result<ScenarioResult, SeedError> {
        let result = self.build_data(rng);

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

        Ok(result)
    }

    /// Gets the appropriate athletic profile for the activity type.
    fn get_profile(&self) -> Box<dyn AthleteProfile> {
        match self.activity_type {
            ActivityType::Running => Box::new(RunnerProfile::default()),
            ActivityType::RoadCycling | ActivityType::MountainBiking => {
                Box::new(CyclistProfile::default())
            }
            ActivityType::Hiking | ActivityType::Walking => Box::new(HikerProfile::default()),
            ActivityType::Unknown => Box::new(RunnerProfile::default()),
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
            .with_activity_type(ActivityType::Running)
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
            .with_activity_type(ActivityType::Running)
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
            .with_activity_type(ActivityType::Running)
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
            .with_activity_type(ActivityType::RoadCycling)
            .with_track_distance(10000.0)
            .with_segment(0.1, 0.3, "Cat 4 Climb")
            .with_segment(0.4, 0.7, "Cat 3 Climb")
            .with_segment(0.7, 0.95, "Cat 2 Climb")
            .with_efforts_per_user(1..=2)
            .with_social(false)
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
}
