use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use time::serde::rfc3339;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}
impl User {
    pub fn new(email: String, name: String) -> Self {
        let id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();
        Self {
            id,
            email,
            name,
            created_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Private,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
        }
    }
}

impl std::str::FromStr for Visibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Visibility::Public),
            "private" => Ok(Visibility::Private),
            _ => Err(format!("unknown visibility: {s}")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub name: String,
    pub object_store_path: String,
    #[serde(with = "rfc3339")]
    pub submitted_at: OffsetDateTime,
    pub visibility: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "activity_type", rename_all = "snake_case")]
pub enum ActivityType {
    Walking,
    Running,
    Hiking,
    RoadCycling,
    MountainBiking,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
}

#[derive(Debug, Clone, FromRow)]
pub struct ScoresRow {
    pub user_id: Uuid,
    pub activity_id: Uuid,
    #[sqlx(flatten)]
    pub scores: Scores,
    pub created_at: OffsetDateTime,
}
#[derive(Debug, Clone, Default, sqlx::Type)]
pub struct Scores {
    pub distance: f64,
    pub duration: f64,
    pub elevation_gain: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Segment {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub activity_type: ActivityType,
    pub distance_meters: f64,
    pub elevation_gain_meters: Option<f64>,
    pub elevation_loss_meters: Option<f64>,
    pub average_grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub climb_category: Option<i32>,
    pub visibility: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SegmentEffort {
    pub id: Uuid,
    pub segment_id: Uuid,
    pub activity_id: Uuid,
    pub user_id: Uuid,
    #[serde(with = "rfc3339")]
    pub started_at: OffsetDateTime,
    pub elapsed_time_seconds: f64,
    pub moving_time_seconds: Option<f64>,
    pub average_speed_mps: Option<f64>,
    pub max_speed_mps: Option<f64>,
    pub is_personal_record: bool,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    pub start_fraction: Option<f64>,
    pub end_fraction: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentWithStats {
    #[serde(flatten)]
    pub segment: Segment,
    pub effort_count: i64,
    pub athlete_count: i64,
    pub creator_name: String,
}

/// Segment effort with segment details, for displaying on activity detail page.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivitySegmentEffort {
    pub effort_id: Uuid,
    pub segment_id: Uuid,
    pub elapsed_time_seconds: f64,
    pub is_personal_record: bool,
    #[serde(with = "rfc3339")]
    pub started_at: OffsetDateTime,
    pub segment_name: String,
    pub segment_distance: f64,
    pub activity_type: ActivityType,
    pub rank: i64,
    pub start_fraction: Option<f64>,
    pub end_fraction: Option<f64>,
}

/// Starred segment with the user's effort stats, for the starred segments dashboard.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StarredSegmentEffort {
    // Segment basic info
    pub segment_id: Uuid,
    pub segment_name: String,
    pub activity_type: ActivityType,
    pub distance_meters: f64,
    pub elevation_gain_meters: Option<f64>,
    // User's best effort
    pub best_time_seconds: Option<f64>,
    pub best_effort_rank: Option<i64>,
    #[serde(with = "rfc3339::option")]
    pub best_effort_date: Option<OffsetDateTime>,
    // User's effort count on this segment
    pub user_effort_count: i64,
    // Segment leader time for comparison
    pub leader_time_seconds: Option<f64>,
}
