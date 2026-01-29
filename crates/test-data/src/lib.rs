//! Test data generation for track-leader.
//!
//! This crate provides tools for generating realistic GPS track data, users, segments,
//! efforts, and social interactions to support manual verification and integration testing.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use test_data::prelude::*;
//!
//! let scenario = ScenarioBuilder::new()
//!     .with_users(50)
//!         .skill_distribution(SkillDistribution::power_law())
//!         .done()
//!     .with_track_in_area(Region::BOULDER)
//!         .distance(5000.0)
//!         .activity_type(ActivityType::Running)
//!         .done()
//!     .with_segment_on_track(0)
//!         .fraction(0.2..0.6)
//!         .name("Hill Sprint")
//!         .done()
//!     .with_efforts_per_user(1..=3)
//!     .build(&db)
//!     .await?;
//! ```

pub mod builders;
pub mod config;
pub mod db;
pub mod generators;
pub mod profiles;
pub mod sources;
pub mod terrain;

// Re-export core types from tracks crate
pub use tracks::models::{
    builtin_types, AgeGroup, Gender, GenderFilter, LeaderboardScope, TrackPointData, Visibility,
};

pub mod prelude {
    //! Convenient re-exports for common usage.

    pub use crate::builders::{ScenarioBuilder, ScenarioMetrics, ScenarioResult};
    pub use crate::config::{BoundingBox, EffortCoverage, Region, SeedConfig, SkillDistribution};
    pub use crate::db::Seeder;
    pub use crate::generators::{
        ActivityGenerator, EffortGenerator, SegmentGenerator, SocialGenerator, UserGenerator,
    };
    pub use crate::profiles::{
        AthleteProfile, CyclistProfile, HikerProfile, RunnerProfile, sample_variance,
        speed_at_grade,
    };
    pub use crate::sources::{GpxLoader, OsmClient, ProceduralGenerator, RoutePattern};
    pub use crate::terrain::ElevationGenerator;
    pub use crate::{builtin_types, AgeGroup, Gender, TrackPointData, Visibility};
}
