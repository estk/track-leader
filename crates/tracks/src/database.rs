use crate::errors::AppError;
use crate::models::{Activity, Scores, TrackScoringMetricTags, User, UserPreferences};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_activity(&self, activity: &Activity) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO activities (id, user_id, activity_type, name, object_store_path,
                                    submitted_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            activity.id,
            activity.user_id,
            activity.activity_type as _,
            activity.name,
            activity.object_store_path,
            activity.submitted_at.into(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_activity(&self, id: Uuid) -> Result<Option<Activity>, AppError> {
        let activity = sqlx::query_as(
            r#"
            SELECT id, user_id, activity_type , filename, object_store_path,
                   distance, ascent, descent, duration,
                   submitted_at, created_at
            FROM activities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(activity)
    }

    pub async fn get_user_activities(&self, user_id: Uuid) -> Result<Vec<Activity>, AppError> {
        let activities: Vec<Activity> = sqlx::query_as(
            r#"
            SELECT id, user_id, activity_type, filename, object_store_path,
                   distance, ascent, descent, duration,
                   submitted_at, created_at
            FROM activities
            WHERE user_id = $1
            ORDER BY submitted_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }

    pub async fn new_user(&self, user: &User) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, name, email, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            user.id,
            user.name,
            user.email,
            user.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_scoring_metrics(
        &self,
        uid: Uuid,
    ) -> Result<TrackScoringMetricTags, AppError> {
        let prefs: UserPreferences = sqlx::query_as(
            r#"
            SELECT user_id, scoring_metric_tags
            FROM user_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(uid)
        .fetch_one(&self.pool)
        .await?;

        Ok(prefs.scoring_metric_tags)
    }

    pub async fn save_scores(
        &self,
        uid: Uuid,
        activity_id: Uuid,
        scores: Scores,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO scores (user_id, activity_id, distance, duration, elevation_gain, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            uid,
            activity_id,
            scores.distance,
            scores.duration,
            scores.elevation_gain,
            time::OffsetDateTime::now_utc()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
