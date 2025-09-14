use enumflags2::{bitflags, BitFlags};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub name: String,
    pub object_store_path: String,
    pub submitted_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "activity_type", rename_all = "lowercase")]
pub enum ActivityType {
    Running,
    Cycling,
    MountainBiking,
    Walking,
    Hiking,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
}

#[bitflags]
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[repr(u8)]
pub enum TrackScoringMetricTag {
    Distance,
    Duration,
    ElevationGain,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub scoring_metric_tags: BitFlags<TrackScoringMetricTag>,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct Scores {
    pub distance: f64,
    pub duration: f64,
    pub elevation_gain: f64,
}
