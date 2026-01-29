//! Query parameter types for API endpoints.

use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Default pagination limit.
pub fn default_limit() -> i64 {
    50
}

/// User creation query parameters.
#[derive(Deserialize, ToSchema)]
pub struct NewUserQuery {
    pub name: String,
    pub email: String,
}

/// Activity upload query parameters.
#[derive(Deserialize, ToSchema)]
pub struct UploadQuery {
    pub activity_type_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub visibility: Option<String>,
    /// Comma-separated list of team IDs to share with (for teams_only visibility)
    #[serde(default)]
    pub team_ids: Option<String>,
    /// Multi-sport: JSON array of ISO-8601 timestamps marking segment boundaries
    #[serde(default)]
    pub type_boundaries: Option<Vec<time::OffsetDateTime>>,
    /// Multi-sport: JSON array of activity type UUIDs for each segment
    #[serde(default)]
    pub segment_types: Option<Vec<Uuid>>,
}

/// User activities query parameters (placeholder for future pagination).
#[derive(Deserialize, ToSchema)]
pub struct UserActivitiesQuery {}

/// Activity type resolution query.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResolveTypeQuery {
    pub name: String,
}

/// Segment listing query parameters.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ListSegmentsQuery {
    /// Filter by activity type
    pub activity_type_id: Option<Uuid>,
    /// Filter by creator
    pub creator_id: Option<Uuid>,
    /// Search by name (partial match)
    pub search: Option<String>,
    /// Sort field
    #[serde(default)]
    pub sort_by: SegmentSortBy,
    /// Sort direction
    #[serde(default)]
    pub sort_order: SortOrder,
    /// Filter by climb category (0-5, where 0 = no climb, 5 = HC)
    pub climb_category: Option<ClimbCategoryFilter>,
    /// Minimum distance in meters
    pub min_distance: Option<f64>,
    /// Maximum distance in meters
    pub max_distance: Option<f64>,
    /// Minimum elevation gain in meters
    pub min_elevation_gain: Option<f64>,
    /// Maximum elevation gain in meters
    pub max_elevation_gain: Option<f64>,
    /// Pagination limit
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Pagination offset
    #[serde(default)]
    pub offset: i64,
}

/// Segment sort field options.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SegmentSortBy {
    #[default]
    CreatedAt,
    Name,
    Distance,
    ElevationGain,
    EffortCount,
    AverageGrade,
}

/// Sort direction.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Climb category filter options.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClimbCategoryFilter {
    /// No significant climb (category 0)
    None,
    /// Category 4 climb
    Cat4,
    /// Category 3 climb
    Cat3,
    /// Category 2 climb
    Cat2,
    /// Category 1 climb
    Cat1,
    /// Hors Catégorie (beyond categorization)
    Hc,
}

impl ClimbCategoryFilter {
    pub fn to_db_value(&self) -> Option<i32> {
        match self {
            ClimbCategoryFilter::None => Some(0),
            ClimbCategoryFilter::Cat4 => Some(1),
            ClimbCategoryFilter::Cat3 => Some(2),
            ClimbCategoryFilter::Cat2 => Some(3),
            ClimbCategoryFilter::Cat1 => Some(4),
            ClimbCategoryFilter::Hc => Some(5),
        }
    }
}

/// Nearby segments query parameters.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct NearbySegmentsQuery {
    pub lat: f64,
    pub lon: f64,
    pub radius_meters: Option<f64>,
    pub limit: Option<i64>,
}

/// Achievement query parameters.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetAchievementsQuery {
    /// Filter by achievement type
    pub achievement_type: Option<crate::models::AchievementType>,
}

/// Global leaderboard query parameters.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GlobalLeaderboardQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Follow list query parameters.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FollowListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Notifications query parameters.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NotificationsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Activity feed query parameters.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FeedQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Team discovery query parameters.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct DiscoverTeamsQuery {
    /// Search by name (partial match)
    pub search: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Team content (activities/segments) query parameters.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TeamContentQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
