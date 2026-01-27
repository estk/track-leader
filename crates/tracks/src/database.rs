use crate::errors::AppError;
use crate::models::{Activity, ActivityType, Scores, Segment, SegmentEffort, User};
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
        sqlx::query(
            r#"
            INSERT INTO activities (id, user_id, activity_type, name, object_store_path,
                                    submitted_at, visibility)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(activity.id)
        .bind(activity.user_id)
        .bind(&activity.activity_type)
        .bind(&activity.name)
        .bind(&activity.object_store_path)
        .bind(activity.submitted_at)
        .bind(&activity.visibility)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_activity(&self, id: Uuid) -> Result<Option<Activity>, AppError> {
        let activity = sqlx::query_as(
            r#"
            SELECT id, user_id, activity_type, name, object_store_path, submitted_at, visibility
            FROM activities
            WHERE id = $1 AND deleted_at IS NULL
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
            SELECT id, user_id, activity_type, name, object_store_path, submitted_at, visibility
            FROM activities
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY submitted_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }

    pub async fn update_activity(
        &self,
        id: Uuid,
        name: Option<&str>,
        activity_type: Option<&crate::models::ActivityType>,
        visibility: Option<&str>,
    ) -> Result<Option<Activity>, AppError> {
        let activity = sqlx::query_as(
            r#"
            UPDATE activities
            SET name = COALESCE($2, name),
                activity_type = COALESCE($3, activity_type),
                visibility = COALESCE($4, visibility)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, user_id, activity_type, name, object_store_path, submitted_at, visibility
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(activity_type.map(|at| at as &crate::models::ActivityType))
        .bind(visibility)
        .fetch_optional(&self.pool)
        .await?;

        Ok(activity)
    }

    pub async fn delete_activity(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM activities WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn new_user(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn all_users(&self) -> Result<Vec<User>, AppError> {
        let users: Vec<User> = sqlx::query_as(
            r#"
            SELECT id, name, email, created_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn save_scores(
        &self,
        uid: Uuid,
        activity_id: Uuid,
        scores: Scores,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO scores (user_id, activity_id, distance, duration, elevation_gain, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(uid)
        .bind(activity_id)
        .bind(scores.distance)
        .bind(scores.duration)
        .bind(scores.elevation_gain)
        .bind(time::OffsetDateTime::now_utc())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Auth-related methods

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, name, email, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_user_with_password(
        &self,
        user: &User,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password_hash, auth_provider, created_at)
            VALUES ($1, $2, $3, $4, 'email', $5)
            "#,
        )
        .bind(user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(password_hash)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_with_password(
        &self,
        email: &str,
    ) -> Result<Option<(User, Option<String>)>, AppError> {
        #[derive(sqlx::FromRow)]
        struct UserWithPassword {
            id: Uuid,
            name: String,
            email: String,
            password_hash: Option<String>,
            created_at: time::OffsetDateTime,
        }

        let row: Option<UserWithPassword> = sqlx::query_as(
            r#"
            SELECT id, name, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            (
                User {
                    id: r.id,
                    name: r.name,
                    email: r.email,
                    created_at: r.created_at,
                },
                r.password_hash,
            )
        }))
    }

    // Segment methods

    pub async fn create_segment(
        &self,
        id: Uuid,
        creator_id: Uuid,
        name: &str,
        description: Option<&str>,
        activity_type: &ActivityType,
        geo_wkt: &str,
        start_wkt: &str,
        end_wkt: &str,
        distance_meters: f64,
        elevation_gain: Option<f64>,
        elevation_loss: Option<f64>,
        visibility: &str,
    ) -> Result<Segment, AppError> {
        let segment = sqlx::query_as(
            r#"
            INSERT INTO segments (
                id, creator_id, name, description, activity_type,
                geo, start_point, end_point,
                distance_meters, elevation_gain_meters, elevation_loss_meters,
                visibility, created_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                ST_GeogFromText($6), ST_GeogFromText($7), ST_GeogFromText($8),
                $9, $10, $11,
                $12, NOW()
            )
            RETURNING id, creator_id, name, description, activity_type,
                      distance_meters, elevation_gain_meters, elevation_loss_meters,
                      visibility, created_at
            "#,
        )
        .bind(id)
        .bind(creator_id)
        .bind(name)
        .bind(description)
        .bind(activity_type)
        .bind(geo_wkt)
        .bind(start_wkt)
        .bind(end_wkt)
        .bind(distance_meters)
        .bind(elevation_gain)
        .bind(elevation_loss)
        .bind(visibility)
        .fetch_one(&self.pool)
        .await?;

        Ok(segment)
    }

    pub async fn get_segment(&self, id: Uuid) -> Result<Option<Segment>, AppError> {
        let segment = sqlx::query_as(
            r#"
            SELECT id, creator_id, name, description, activity_type,
                   distance_meters, elevation_gain_meters, elevation_loss_meters,
                   visibility, created_at
            FROM segments
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(segment)
    }

    pub async fn list_segments(
        &self,
        activity_type: Option<&ActivityType>,
        limit: i64,
    ) -> Result<Vec<Segment>, AppError> {
        let segments: Vec<Segment> = if let Some(at) = activity_type {
            sqlx::query_as(
                r#"
                SELECT id, creator_id, name, description, activity_type,
                       distance_meters, elevation_gain_meters, elevation_loss_meters,
                       visibility, created_at
                FROM segments
                WHERE deleted_at IS NULL AND visibility = 'public' AND activity_type = $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(at)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, creator_id, name, description, activity_type,
                       distance_meters, elevation_gain_meters, elevation_loss_meters,
                       visibility, created_at
                FROM segments
                WHERE deleted_at IS NULL AND visibility = 'public'
                ORDER BY created_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(segments)
    }

    pub async fn get_user_segments(&self, user_id: Uuid) -> Result<Vec<Segment>, AppError> {
        let segments: Vec<Segment> = sqlx::query_as(
            r#"
            SELECT id, creator_id, name, description, activity_type,
                   distance_meters, elevation_gain_meters, elevation_loss_meters,
                   visibility, created_at
            FROM segments
            WHERE creator_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(segments)
    }

    pub async fn create_segment_effort(
        &self,
        segment_id: Uuid,
        activity_id: Uuid,
        user_id: Uuid,
        started_at: time::OffsetDateTime,
        elapsed_time_seconds: f64,
        moving_time_seconds: Option<f64>,
        average_speed_mps: Option<f64>,
        max_speed_mps: Option<f64>,
    ) -> Result<SegmentEffort, AppError> {
        let effort = sqlx::query_as(
            r#"
            INSERT INTO segment_efforts (
                id, segment_id, activity_id, user_id,
                started_at, elapsed_time_seconds,
                moving_time_seconds, average_speed_mps, max_speed_mps,
                is_personal_record, created_at
            )
            VALUES (
                gen_random_uuid(), $1, $2, $3,
                $4, $5,
                $6, $7, $8,
                FALSE, NOW()
            )
            RETURNING id, segment_id, activity_id, user_id,
                      started_at, elapsed_time_seconds,
                      moving_time_seconds, average_speed_mps, max_speed_mps,
                      is_personal_record, created_at
            "#,
        )
        .bind(segment_id)
        .bind(activity_id)
        .bind(user_id)
        .bind(started_at)
        .bind(elapsed_time_seconds)
        .bind(moving_time_seconds)
        .bind(average_speed_mps)
        .bind(max_speed_mps)
        .fetch_one(&self.pool)
        .await?;

        Ok(effort)
    }

    pub async fn get_segment_efforts(
        &self,
        segment_id: Uuid,
        limit: i64,
    ) -> Result<Vec<SegmentEffort>, AppError> {
        let efforts: Vec<SegmentEffort> = sqlx::query_as(
            r#"
            SELECT id, segment_id, activity_id, user_id,
                   started_at, elapsed_time_seconds,
                   moving_time_seconds, average_speed_mps, max_speed_mps,
                   is_personal_record, created_at
            FROM segment_efforts
            WHERE segment_id = $1
            ORDER BY elapsed_time_seconds ASC
            LIMIT $2
            "#,
        )
        .bind(segment_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(efforts)
    }

    pub async fn get_user_segment_efforts(
        &self,
        user_id: Uuid,
        segment_id: Uuid,
    ) -> Result<Vec<SegmentEffort>, AppError> {
        let efforts: Vec<SegmentEffort> = sqlx::query_as(
            r#"
            SELECT id, segment_id, activity_id, user_id,
                   started_at, elapsed_time_seconds,
                   moving_time_seconds, average_speed_mps, max_speed_mps,
                   is_personal_record, created_at
            FROM segment_efforts
            WHERE segment_id = $1 AND user_id = $2
            ORDER BY elapsed_time_seconds ASC
            "#,
        )
        .bind(segment_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(efforts)
    }
}
