use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::UtcDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub filename: String,
    pub object_store_path: String,
    #[sqlx(flatten)]
    pub metrics: ActivityMetrics,
    pub submitted_at: UtcDateTime,
    pub created_at: UtcDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivityMetrics {
    pub distance: f64,
    pub ascent: f64,
    pub descent: f64,
    pub duration: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "activity_type", rename_all = "lowercase")]
pub enum ActivityType {
    Running,
    Cycling,
    Walking,
    Hiking,
    Other,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
}
