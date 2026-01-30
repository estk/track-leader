//! Entity generators for test data.
//!
//! This module provides generators for creating realistic test entities:
//! - [`UserGenerator`]: Generate users with demographics
//! - [`ActivityGenerator`]: Create activities from tracks
//! - [`SegmentGenerator`]: Extract segments with climb categories
//! - [`EffortGenerator`]: Generate segment efforts with realistic time distributions
//! - [`SocialGenerator`]: Create follows, kudos, and comments
//! - [`TeamGenerator`]: Create teams with memberships and sharing

pub mod activity;
pub mod effort;
pub mod segment;
pub mod social;
pub mod team;
pub mod user;

pub use activity::{ActivityGenerator, GeneratedActivity};
pub use effort::{EffortGenerator, GeneratedEffort};
pub use segment::{GeneratedSegment, SegmentGenerator};
pub use social::{GeneratedComment, GeneratedFollow, GeneratedKudos, SocialGenerator};
pub use team::{
    GeneratedActivityTeam, GeneratedSegmentTeam, GeneratedTeam, GeneratedTeamMembership,
    TeamGenConfig, TeamGenerator, TeamJoinPolicy, TeamRole, TeamVisibility,
};
pub use user::{GeneratedUser, UserGenerator};
