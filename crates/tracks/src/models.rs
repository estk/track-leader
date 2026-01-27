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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub name: String,
    pub object_store_path: String,
    #[serde(with = "rfc3339")]
    pub submitted_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
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
