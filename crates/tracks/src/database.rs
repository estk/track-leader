use crate::errors::AppError;
use crate::models::{
    Achievement, AchievementHolder, AchievementType, AchievementWithSegment, Activity,
    ActivitySegmentEffort, ActivityType, CrownCountEntry, DistanceLeaderEntry, GenderFilter,
    LeaderboardEntry, LeaderboardFilters, LeaderboardScope, Notification, NotificationWithActor,
    Scores, Segment, SegmentEffort, UpdateDemographicsRequest, User, UserProfile, UserSummary,
    UserWithDemographics,
};
use crate::segment_matching::{ActivityMatch, SegmentMatch};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// A segment that is similar to a proposed new segment.
/// Used for duplicate detection when creating segments.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SimilarSegment {
    pub id: Uuid,
    pub name: String,
    pub distance_meters: f64,
}

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

    #[allow(clippy::too_many_arguments)]
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
        average_grade: Option<f64>,
        max_grade: Option<f64>,
        climb_category: Option<i32>,
        visibility: &str,
    ) -> Result<Segment, AppError> {
        let segment = sqlx::query_as(
            r#"
            INSERT INTO segments (
                id, creator_id, name, description, activity_type,
                geo, start_point, end_point,
                distance_meters, elevation_gain_meters, elevation_loss_meters,
                average_grade, max_grade, climb_category,
                visibility, created_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                ST_GeogFromText($6), ST_GeogFromText($7), ST_GeogFromText($8),
                $9, $10, $11,
                $12, $13, $14,
                $15, NOW()
            )
            RETURNING id, creator_id, name, description, activity_type,
                      distance_meters, elevation_gain_meters, elevation_loss_meters,
                      average_grade, max_grade, climb_category,
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
        .bind(average_grade)
        .bind(max_grade)
        .bind(climb_category)
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
                   average_grade, max_grade, climb_category,
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
                       average_grade, max_grade, climb_category,
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
                       average_grade, max_grade, climb_category,
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
                   average_grade, max_grade, climb_category,
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

    #[allow(clippy::too_many_arguments)]
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
        start_fraction: Option<f64>,
        end_fraction: Option<f64>,
    ) -> Result<SegmentEffort, AppError> {
        let effort = sqlx::query_as(
            r#"
            INSERT INTO segment_efforts (
                id, segment_id, activity_id, user_id,
                started_at, elapsed_time_seconds,
                moving_time_seconds, average_speed_mps, max_speed_mps,
                is_personal_record, created_at,
                start_fraction, end_fraction
            )
            VALUES (
                gen_random_uuid(), $1, $2, $3,
                $4, $5,
                $6, $7, $8,
                FALSE, NOW(),
                $9, $10
            )
            RETURNING id, segment_id, activity_id, user_id,
                      started_at, elapsed_time_seconds,
                      moving_time_seconds, average_speed_mps, max_speed_mps,
                      is_personal_record, created_at,
                      start_fraction, end_fraction
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
        .bind(start_fraction)
        .bind(end_fraction)
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
                   is_personal_record, created_at,
                   start_fraction, end_fraction
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
                   is_personal_record, created_at,
                   start_fraction, end_fraction
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

    pub async fn get_segment_geometry(&self, id: Uuid) -> Result<Option<String>, AppError> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT ST_AsGeoJSON(geo)::text
            FROM segments
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(geojson,)| geojson))
    }

    /// Get segment efforts for a specific activity, with segment details.
    pub async fn get_activity_segment_efforts(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<ActivitySegmentEffort>, AppError> {
        let efforts: Vec<ActivitySegmentEffort> = sqlx::query_as(
            r#"
            SELECT
                e.id as effort_id,
                e.segment_id,
                e.elapsed_time_seconds,
                e.is_personal_record,
                e.started_at,
                s.name as segment_name,
                s.distance_meters as segment_distance,
                s.activity_type,
                (SELECT COUNT(*) + 1 FROM segment_efforts e2
                 WHERE e2.segment_id = e.segment_id
                 AND e2.elapsed_time_seconds < e.elapsed_time_seconds) as rank,
                e.start_fraction,
                e.end_fraction
            FROM segment_efforts e
            JOIN segments s ON s.id = e.segment_id
            WHERE e.activity_id = $1
            AND s.deleted_at IS NULL
            ORDER BY e.started_at ASC
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(efforts)
    }

    // Track geometry methods

    pub async fn save_track_geometry(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
        geo_wkt: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO tracks (user_id, activity_id, geo, created_at)
            VALUES ($1, $2, ST_GeogFromText($3), NOW())
            ON CONFLICT (activity_id) DO UPDATE
            SET geo = ST_GeogFromText($3)
            "#,
        )
        .bind(user_id)
        .bind(activity_id)
        .bind(geo_wkt)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_track_geometry(&self, activity_id: Uuid) -> Result<Option<String>, AppError> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT ST_AsGeoJSON(geo)::text
            FROM tracks
            WHERE activity_id = $1
            "#,
        )
        .bind(activity_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(geojson,)| geojson))
    }

    // Segment matching methods

    /// Find segments that the activity track passes through.
    /// Uses PostGIS to check if track passes within 50m of both segment endpoints
    /// and verifies direction (start before end along the track).
    pub async fn find_matching_segments(
        &self,
        activity_id: Uuid,
        activity_type: &ActivityType,
    ) -> Result<Vec<SegmentMatch>, AppError> {
        #[derive(sqlx::FromRow)]
        struct MatchRow {
            id: Uuid,
            distance_meters: f64,
            start_pos: f64,
            end_pos: f64,
        }

        let rows: Vec<MatchRow> = sqlx::query_as(
            r#"
            SELECT s.id,
                   s.distance_meters,
                   ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry) as start_pos,
                   ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry) as end_pos
            FROM segments s
            JOIN tracks t ON t.activity_id = $1
            WHERE s.deleted_at IS NULL
              AND s.activity_type = $2
              AND ST_DWithin(t.geo, s.start_point, 50)
              AND ST_DWithin(t.geo, s.end_point, 50)
              AND ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry)
                  < ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry)
            "#,
        )
        .bind(activity_id)
        .bind(activity_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SegmentMatch {
                segment_id: r.id,
                distance_meters: r.distance_meters,
                start_fraction: r.start_pos,
                end_fraction: r.end_pos,
            })
            .collect())
    }

    /// Check if a segment effort already exists for a given segment and activity.
    pub async fn segment_effort_exists(
        &self,
        segment_id: Uuid,
        activity_id: Uuid,
    ) -> Result<bool, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM segment_efforts
            WHERE segment_id = $1 AND activity_id = $2
            LIMIT 1
            "#,
        )
        .bind(segment_id)
        .bind(activity_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    /// Increment the effort_count counter on a segment.
    pub async fn increment_segment_effort_count(&self, segment_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE segments
            SET effort_count = effort_count + 1
            WHERE id = $1
            "#,
        )
        .bind(segment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update personal records for a user on a segment.
    /// Marks the fastest effort as PR and clears PR flag from all others.
    pub async fn update_personal_records(
        &self,
        segment_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // Clear all PR flags for this user on this segment
        sqlx::query(
            r#"
            UPDATE segment_efforts
            SET is_personal_record = FALSE
            WHERE segment_id = $1 AND user_id = $2
            "#,
        )
        .bind(segment_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Set PR flag on the fastest effort
        sqlx::query(
            r#"
            UPDATE segment_efforts
            SET is_personal_record = TRUE
            WHERE id = (
                SELECT id FROM segment_efforts
                WHERE segment_id = $1 AND user_id = $2
                ORDER BY elapsed_time_seconds ASC
                LIMIT 1
            )
            "#,
        )
        .bind(segment_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all activities with their track geometry for reprocessing.
    pub async fn get_activities_with_tracks(
        &self,
        activity_type: &ActivityType,
    ) -> Result<Vec<(Uuid, Uuid)>, AppError> {
        let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT a.id, a.user_id
            FROM activities a
            JOIN tracks t ON t.activity_id = a.id
            WHERE a.deleted_at IS NULL
              AND a.activity_type = $1
            "#,
        )
        .bind(activity_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find all activities that match a specific segment.
    /// Returns activity_id, user_id, and the fractional positions along each track.
    pub async fn find_matching_activities_for_segment(
        &self,
        segment_id: Uuid,
    ) -> Result<Vec<ActivityMatch>, AppError> {
        #[derive(sqlx::FromRow)]
        struct MatchRow {
            activity_id: Uuid,
            user_id: Uuid,
            start_pos: f64,
            end_pos: f64,
        }

        let rows: Vec<MatchRow> = sqlx::query_as(
            r#"
            SELECT t.activity_id,
                   t.user_id,
                   ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry) as start_pos,
                   ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry) as end_pos
            FROM segments s
            JOIN tracks t ON ST_DWithin(t.geo, s.start_point, 50)
                         AND ST_DWithin(t.geo, s.end_point, 50)
            JOIN activities a ON a.id = t.activity_id
            WHERE s.id = $1
              AND s.deleted_at IS NULL
              AND a.deleted_at IS NULL
              AND a.activity_type = s.activity_type
              AND ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry)
                  < ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry)
            "#,
        )
        .bind(segment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ActivityMatch {
                activity_id: r.activity_id,
                user_id: r.user_id,
                start_fraction: r.start_pos,
                end_fraction: r.end_pos,
            })
            .collect())
    }

    // Segment star methods

    /// Star a segment for a user.
    pub async fn star_segment(&self, user_id: Uuid, segment_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO segment_stars (user_id, segment_id, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id, segment_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(segment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Unstar a segment for a user.
    pub async fn unstar_segment(&self, user_id: Uuid, segment_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM segment_stars
            WHERE user_id = $1 AND segment_id = $2
            "#,
        )
        .bind(user_id)
        .bind(segment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a segment is starred by a user.
    pub async fn is_segment_starred(
        &self,
        user_id: Uuid,
        segment_id: Uuid,
    ) -> Result<bool, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM segment_stars
            WHERE user_id = $1 AND segment_id = $2
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(segment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    /// Get all segments starred by a user.
    pub async fn get_user_starred_segments(&self, user_id: Uuid) -> Result<Vec<Segment>, AppError> {
        let segments: Vec<Segment> = sqlx::query_as(
            r#"
            SELECT s.id, s.creator_id, s.name, s.description, s.activity_type,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN segment_stars ss ON ss.segment_id = s.id
            WHERE ss.user_id = $1 AND s.deleted_at IS NULL
            ORDER BY ss.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(segments)
    }

    /// Get all starred segments with effort stats for a user.
    /// Returns each starred segment with the user's best effort, effort count, and leader time.
    pub async fn get_starred_segments_with_efforts(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::StarredSegmentEffort>, AppError> {
        let efforts: Vec<crate::models::StarredSegmentEffort> = sqlx::query_as(
            r#"
            SELECT
                s.id as segment_id,
                s.name as segment_name,
                s.activity_type,
                s.distance_meters,
                s.elevation_gain_meters,
                -- User's best effort (PR)
                user_best.elapsed_time_seconds as best_time_seconds,
                user_best.rank as best_effort_rank,
                user_best.started_at as best_effort_date,
                -- User's total effort count
                COALESCE(user_count.cnt, 0)::bigint as user_effort_count,
                -- Segment leader time
                leader.elapsed_time_seconds as leader_time_seconds
            FROM segments s
            JOIN segment_stars ss ON ss.segment_id = s.id AND ss.user_id = $1
            -- User's best effort (subquery to get PR with rank)
            LEFT JOIN LATERAL (
                SELECT
                    e.elapsed_time_seconds,
                    e.started_at,
                    (SELECT COUNT(*) + 1 FROM segment_efforts e2
                     WHERE e2.segment_id = s.id
                     AND e2.elapsed_time_seconds < e.elapsed_time_seconds) as rank
                FROM segment_efforts e
                WHERE e.segment_id = s.id
                  AND e.user_id = $1
                ORDER BY e.elapsed_time_seconds ASC
                LIMIT 1
            ) user_best ON true
            -- User's total effort count
            LEFT JOIN LATERAL (
                SELECT COUNT(*)::bigint as cnt
                FROM segment_efforts e
                WHERE e.segment_id = s.id AND e.user_id = $1
            ) user_count ON true
            -- Segment leader (fastest effort overall)
            LEFT JOIN LATERAL (
                SELECT e.elapsed_time_seconds
                FROM segment_efforts e
                WHERE e.segment_id = s.id
                ORDER BY e.elapsed_time_seconds ASC
                LIMIT 1
            ) leader ON true
            WHERE s.deleted_at IS NULL
            ORDER BY ss.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(efforts)
    }

    /// Find segments with similar start/end points and same activity type.
    /// Used to detect potential duplicates when creating a new segment.
    /// Returns segments where both start and end points are within 30m of the given points.
    pub async fn find_similar_segments(
        &self,
        activity_type: &ActivityType,
        start_wkt: &str,
        end_wkt: &str,
    ) -> Result<Vec<SimilarSegment>, AppError> {
        let rows: Vec<SimilarSegment> = sqlx::query_as(
            r#"
            SELECT id, name, distance_meters
            FROM segments
            WHERE activity_type = $1
              AND ST_DWithin(start_point, ST_GeogFromText($2), 30)
              AND ST_DWithin(end_point, ST_GeogFromText($3), 30)
              AND deleted_at IS NULL
            LIMIT 5
            "#,
        )
        .bind(activity_type)
        .bind(start_wkt)
        .bind(end_wkt)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find segments whose start_point is within a given radius of a point.
    pub async fn find_segments_near_point(
        &self,
        lat: f64,
        lon: f64,
        radius_meters: f64,
        limit: i64,
    ) -> Result<Vec<Segment>, AppError> {
        let point_wkt = format!("POINT({lon} {lat})");
        let segments: Vec<Segment> = sqlx::query_as(
            r#"
            SELECT id, creator_id, name, description, activity_type,
                   distance_meters, elevation_gain_meters, elevation_loss_meters,
                   average_grade, max_grade, climb_category,
                   visibility, created_at
            FROM segments
            WHERE deleted_at IS NULL
              AND visibility = 'public'
              AND ST_DWithin(
                  start_point,
                  ST_GeogFromText($1),
                  $2
              )
            ORDER BY ST_Distance(start_point, ST_GeogFromText($1))
            LIMIT $3
            "#,
        )
        .bind(&point_wkt)
        .bind(radius_meters)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(segments)
    }

    // ========================================================================
    // Enhanced Leaderboard Methods
    // ========================================================================

    /// Get filtered leaderboard entries with user info and ranking.
    /// Supports time scope, gender, and age group filtering.
    pub async fn get_filtered_leaderboard(
        &self,
        segment_id: Uuid,
        filters: &LeaderboardFilters,
    ) -> Result<(Vec<LeaderboardEntry>, i64), AppError> {
        // Build the time filter based on scope
        let time_filter = match filters.scope {
            LeaderboardScope::AllTime => None,
            LeaderboardScope::Year => Some("e.started_at >= NOW() - INTERVAL '1 year'"),
            LeaderboardScope::Month => Some("e.started_at >= NOW() - INTERVAL '1 month'"),
            LeaderboardScope::Week => Some("e.started_at >= NOW() - INTERVAL '1 week'"),
        };

        // Build the gender filter
        let gender_filter = match filters.gender {
            GenderFilter::All => None,
            GenderFilter::Male => Some("u.gender = 'male'"),
            GenderFilter::Female => Some("u.gender = 'female'"),
        };

        // Build the age filter based on current year and birth_year
        let current_year = time::OffsetDateTime::now_utc().year();
        let age_filter = filters.age_group.age_range().map(|(min_age, max_age)| {
            let max_birth_year = current_year - min_age;
            match max_age {
                Some(max) => {
                    let min_birth_year = current_year - max;
                    format!("u.birth_year BETWEEN {min_birth_year} AND {max_birth_year}")
                }
                None => format!("u.birth_year <= {max_birth_year}"),
            }
        });

        // Build WHERE clauses
        let mut where_clauses = vec!["e.segment_id = $1".to_string()];
        if let Some(tf) = time_filter {
            where_clauses.push(tf.to_string());
        }
        if let Some(gf) = gender_filter {
            where_clauses.push(gf.to_string());
        }
        if let Some(af) = age_filter {
            where_clauses.push(af);
        }
        let where_clause = where_clauses.join(" AND ");

        // Query for total count (for pagination)
        let count_query = format!(
            r#"
            SELECT COUNT(DISTINCT e.id)
            FROM segment_efforts e
            JOIN users u ON u.id = e.user_id
            WHERE {where_clause}
            "#
        );

        let total_count: (i64,) = sqlx::query_as(&count_query)
            .bind(segment_id)
            .fetch_one(&self.pool)
            .await?;

        // Main query with ranking
        let main_query = format!(
            r#"
            WITH filtered_efforts AS (
                SELECT
                    e.id as effort_id,
                    e.elapsed_time_seconds,
                    e.moving_time_seconds,
                    e.average_speed_mps,
                    e.started_at,
                    e.is_personal_record,
                    e.user_id,
                    u.name as user_name
                FROM segment_efforts e
                JOIN users u ON u.id = e.user_id
                WHERE {where_clause}
            ),
            ranked AS (
                SELECT
                    effort_id,
                    elapsed_time_seconds,
                    moving_time_seconds,
                    average_speed_mps,
                    started_at,
                    is_personal_record,
                    user_id,
                    user_name,
                    ROW_NUMBER() OVER (ORDER BY elapsed_time_seconds ASC) as rank,
                    FIRST_VALUE(elapsed_time_seconds) OVER (ORDER BY elapsed_time_seconds ASC) as leader_time
                FROM filtered_efforts
            )
            SELECT
                effort_id,
                elapsed_time_seconds,
                moving_time_seconds,
                average_speed_mps,
                started_at,
                is_personal_record,
                user_id,
                user_name,
                rank,
                CASE WHEN rank > 1 THEN elapsed_time_seconds - leader_time ELSE NULL END as gap_seconds
            FROM ranked
            ORDER BY rank
            LIMIT $2 OFFSET $3
            "#
        );

        let entries: Vec<LeaderboardEntry> = sqlx::query_as(&main_query)
            .bind(segment_id)
            .bind(filters.limit)
            .bind(filters.offset)
            .fetch_all(&self.pool)
            .await?;

        Ok((entries, total_count.0))
    }

    /// Get the user's position in the leaderboard with surrounding entries.
    pub async fn get_user_leaderboard_position(
        &self,
        segment_id: Uuid,
        user_id: Uuid,
        filters: &LeaderboardFilters,
        context_entries: i64,
    ) -> Result<
        Option<(
            LeaderboardEntry,
            Vec<LeaderboardEntry>,
            Vec<LeaderboardEntry>,
            i64,
        )>,
        AppError,
    > {
        // Build the time filter based on scope
        let time_filter = match filters.scope {
            LeaderboardScope::AllTime => None,
            LeaderboardScope::Year => Some("e.started_at >= NOW() - INTERVAL '1 year'"),
            LeaderboardScope::Month => Some("e.started_at >= NOW() - INTERVAL '1 month'"),
            LeaderboardScope::Week => Some("e.started_at >= NOW() - INTERVAL '1 week'"),
        };

        // Build the gender filter
        let gender_filter = match filters.gender {
            GenderFilter::All => None,
            GenderFilter::Male => Some("u.gender = 'male'"),
            GenderFilter::Female => Some("u.gender = 'female'"),
        };

        // Build the age filter
        let current_year = time::OffsetDateTime::now_utc().year();
        let age_filter = filters.age_group.age_range().map(|(min_age, max_age)| {
            let max_birth_year = current_year - min_age;
            match max_age {
                Some(max) => {
                    let min_birth_year = current_year - max;
                    format!("u.birth_year BETWEEN {min_birth_year} AND {max_birth_year}")
                }
                None => format!("u.birth_year <= {max_birth_year}"),
            }
        });

        // Build WHERE clauses
        let mut where_clauses = vec!["e.segment_id = $1".to_string()];
        if let Some(tf) = time_filter {
            where_clauses.push(tf.to_string());
        }
        if let Some(gf) = gender_filter {
            where_clauses.push(gf.to_string());
        }
        if let Some(af) = age_filter {
            where_clauses.push(af);
        }
        let where_clause = where_clauses.join(" AND ");

        // First get the total count
        let count_query = format!(
            r#"
            SELECT COUNT(DISTINCT e.id)
            FROM segment_efforts e
            JOIN users u ON u.id = e.user_id
            WHERE {where_clause}
            "#
        );

        let total_count: (i64,) = sqlx::query_as(&count_query)
            .bind(segment_id)
            .fetch_one(&self.pool)
            .await?;

        // Get all ranked entries including user position
        let main_query = format!(
            r#"
            WITH filtered_efforts AS (
                SELECT
                    e.id as effort_id,
                    e.elapsed_time_seconds,
                    e.moving_time_seconds,
                    e.average_speed_mps,
                    e.started_at,
                    e.is_personal_record,
                    e.user_id,
                    u.name as user_name
                FROM segment_efforts e
                JOIN users u ON u.id = e.user_id
                WHERE {where_clause}
            ),
            ranked AS (
                SELECT
                    effort_id,
                    elapsed_time_seconds,
                    moving_time_seconds,
                    average_speed_mps,
                    started_at,
                    is_personal_record,
                    user_id,
                    user_name,
                    ROW_NUMBER() OVER (ORDER BY elapsed_time_seconds ASC) as rank,
                    FIRST_VALUE(elapsed_time_seconds) OVER (ORDER BY elapsed_time_seconds ASC) as leader_time
                FROM filtered_efforts
            ),
            user_rank AS (
                SELECT rank FROM ranked WHERE user_id = $2 ORDER BY elapsed_time_seconds LIMIT 1
            )
            SELECT
                effort_id,
                elapsed_time_seconds,
                moving_time_seconds,
                average_speed_mps,
                started_at,
                is_personal_record,
                user_id,
                user_name,
                rank,
                CASE WHEN rank > 1 THEN elapsed_time_seconds - leader_time ELSE NULL END as gap_seconds
            FROM ranked
            WHERE rank BETWEEN (SELECT rank FROM user_rank) - $3 AND (SELECT rank FROM user_rank) + $3
            ORDER BY rank
            "#
        );

        let entries: Vec<LeaderboardEntry> = sqlx::query_as(&main_query)
            .bind(segment_id)
            .bind(user_id)
            .bind(context_entries)
            .fetch_all(&self.pool)
            .await?;

        // Find user's entry and split into above/below
        let user_entry_idx = entries.iter().position(|e| e.user_id == user_id);

        match user_entry_idx {
            Some(idx) => {
                let user_entry = entries[idx].clone();
                let entries_above = entries[..idx].to_vec();
                let entries_below = entries[idx + 1..].to_vec();
                Ok(Some((
                    user_entry,
                    entries_above,
                    entries_below,
                    total_count.0,
                )))
            }
            None => Ok(None),
        }
    }

    // ========================================================================
    // User Demographics Methods
    // ========================================================================

    /// Get a user with their demographics.
    pub async fn get_user_with_demographics(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserWithDemographics>, AppError> {
        let user: Option<UserWithDemographics> = sqlx::query_as(
            r#"
            SELECT id, email, name, created_at, gender, birth_year, weight_kg, country, region
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Update a user's demographics.
    pub async fn update_user_demographics(
        &self,
        user_id: Uuid,
        req: &UpdateDemographicsRequest,
    ) -> Result<UserWithDemographics, AppError> {
        let user: UserWithDemographics = sqlx::query_as(
            r#"
            UPDATE users
            SET gender = COALESCE($2, gender),
                birth_year = COALESCE($3, birth_year),
                weight_kg = COALESCE($4, weight_kg),
                country = COALESCE($5, country),
                region = COALESCE($6, region)
            WHERE id = $1
            RETURNING id, email, name, created_at, gender, birth_year, weight_kg, country, region
            "#,
        )
        .bind(user_id)
        .bind(&req.gender)
        .bind(req.birth_year)
        .bind(req.weight_kg)
        .bind(&req.country)
        .bind(&req.region)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // ========================================================================
    // Achievement Methods
    // ========================================================================

    /// Create or update an achievement (crown).
    pub async fn create_achievement(
        &self,
        user_id: Uuid,
        segment_id: Uuid,
        effort_id: Option<Uuid>,
        achievement_type: AchievementType,
        effort_count: Option<i32>,
    ) -> Result<Achievement, AppError> {
        let achievement: Achievement = sqlx::query_as(
            r#"
            INSERT INTO achievements (id, user_id, segment_id, effort_id, achievement_type, effort_count, earned_at, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING id, user_id, segment_id, effort_id, achievement_type, earned_at, lost_at, effort_count, created_at
            "#,
        )
        .bind(user_id)
        .bind(segment_id)
        .bind(effort_id)
        .bind(achievement_type)
        .bind(effort_count)
        .fetch_one(&self.pool)
        .await?;

        Ok(achievement)
    }

    /// Mark an achievement as lost (dethroned).
    pub async fn dethrone_achievement(
        &self,
        segment_id: Uuid,
        achievement_type: AchievementType,
    ) -> Result<Option<Achievement>, AppError> {
        let achievement: Option<Achievement> = sqlx::query_as(
            r#"
            UPDATE achievements
            SET lost_at = NOW()
            WHERE segment_id = $1 AND achievement_type = $2 AND lost_at IS NULL
            RETURNING id, user_id, segment_id, effort_id, achievement_type, earned_at, lost_at, effort_count, created_at
            "#,
        )
        .bind(segment_id)
        .bind(achievement_type)
        .fetch_optional(&self.pool)
        .await?;

        Ok(achievement)
    }

    /// Get the current holder of an achievement for a segment.
    pub async fn get_current_achievement_holder(
        &self,
        segment_id: Uuid,
        achievement_type: AchievementType,
    ) -> Result<Option<AchievementHolder>, AppError> {
        let holder: Option<AchievementHolder> = sqlx::query_as(
            r#"
            SELECT
                a.user_id,
                u.name as user_name,
                a.achievement_type,
                a.earned_at,
                e.elapsed_time_seconds,
                a.effort_count
            FROM achievements a
            JOIN users u ON u.id = a.user_id
            LEFT JOIN segment_efforts e ON e.id = a.effort_id
            WHERE a.segment_id = $1 AND a.achievement_type = $2 AND a.lost_at IS NULL
            "#,
        )
        .bind(segment_id)
        .bind(achievement_type)
        .fetch_optional(&self.pool)
        .await?;

        Ok(holder)
    }

    /// Get all achievements for a user.
    pub async fn get_user_achievements(
        &self,
        user_id: Uuid,
        include_lost: bool,
    ) -> Result<Vec<AchievementWithSegment>, AppError> {
        let lost_filter = if include_lost {
            ""
        } else {
            "AND a.lost_at IS NULL"
        };
        let query = format!(
            r#"
            SELECT
                a.id,
                a.user_id,
                a.segment_id,
                a.effort_id,
                a.achievement_type,
                a.earned_at,
                a.lost_at,
                a.effort_count,
                s.name as segment_name,
                s.distance_meters as segment_distance_meters,
                s.activity_type as segment_activity_type
            FROM achievements a
            JOIN segments s ON s.id = a.segment_id
            WHERE a.user_id = $1 {lost_filter}
            ORDER BY a.earned_at DESC
            "#
        );

        let achievements: Vec<AchievementWithSegment> = sqlx::query_as(&query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(achievements)
    }

    /// Get achievement counts for a user.
    pub async fn get_user_achievement_counts(
        &self,
        user_id: Uuid,
    ) -> Result<(i64, i64, i64), AppError> {
        let counts: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE achievement_type = 'kom' AND lost_at IS NULL),
                COUNT(*) FILTER (WHERE achievement_type = 'qom' AND lost_at IS NULL),
                COUNT(*) FILTER (WHERE achievement_type = 'local_legend' AND lost_at IS NULL)
            FROM achievements
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(counts)
    }

    /// Get the user's effort count on a segment in the last 90 days (for Local Legend).
    pub async fn get_user_recent_effort_count(
        &self,
        user_id: Uuid,
        segment_id: Uuid,
    ) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM segment_efforts
            WHERE user_id = $1 AND segment_id = $2
              AND started_at >= NOW() - INTERVAL '90 days'
            "#,
        )
        .bind(user_id)
        .bind(segment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Get the top effort counts on a segment in the last 90 days (for Local Legend determination).
    pub async fn get_top_recent_effort_counts(
        &self,
        segment_id: Uuid,
        limit: i64,
    ) -> Result<Vec<(Uuid, i64)>, AppError> {
        let counts: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"
            SELECT user_id, COUNT(*) as cnt
            FROM segment_efforts
            WHERE segment_id = $1 AND started_at >= NOW() - INTERVAL '90 days'
            GROUP BY user_id
            ORDER BY cnt DESC
            LIMIT $2
            "#,
        )
        .bind(segment_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(counts)
    }

    // ========================================================================
    // Global Leaderboard Methods
    // ========================================================================

    /// Get global crown count leaderboard.
    pub async fn get_crown_count_leaderboard(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CrownCountEntry>, AppError> {
        let entries: Vec<CrownCountEntry> = sqlx::query_as(
            r#"
            WITH crown_counts AS (
                SELECT
                    user_id,
                    COUNT(*) FILTER (WHERE achievement_type = 'kom') as kom_count,
                    COUNT(*) FILTER (WHERE achievement_type = 'qom') as qom_count,
                    COUNT(*) FILTER (WHERE achievement_type = 'local_legend') as local_legend_count,
                    COUNT(*) as total_crowns
                FROM achievements
                WHERE lost_at IS NULL
                GROUP BY user_id
            )
            SELECT
                cc.user_id,
                u.name as user_name,
                cc.kom_count,
                cc.qom_count,
                cc.local_legend_count,
                cc.total_crowns,
                ROW_NUMBER() OVER (ORDER BY cc.total_crowns DESC, cc.kom_count DESC) as rank
            FROM crown_counts cc
            JOIN users u ON u.id = cc.user_id
            ORDER BY rank
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get global distance leaderboard.
    pub async fn get_distance_leaderboard(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DistanceLeaderEntry>, AppError> {
        let entries: Vec<DistanceLeaderEntry> = sqlx::query_as(
            r#"
            WITH distance_totals AS (
                SELECT
                    user_id,
                    SUM(distance) as total_distance_meters,
                    COUNT(*) as activity_count
                FROM scores
                GROUP BY user_id
            )
            SELECT
                dt.user_id,
                u.name as user_name,
                dt.total_distance_meters,
                dt.activity_count,
                ROW_NUMBER() OVER (ORDER BY dt.total_distance_meters DESC) as rank
            FROM distance_totals dt
            JOIN users u ON u.id = dt.user_id
            ORDER BY rank
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    // ========================================================================
    // Social Methods (Follows, Notifications)
    // ========================================================================

    /// Follow a user.
    pub async fn follow_user(&self, follower_id: Uuid, following_id: Uuid) -> Result<(), AppError> {
        // Insert follow relationship
        sqlx::query(
            r#"
            INSERT INTO follows (follower_id, following_id, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (follower_id, following_id) DO NOTHING
            "#,
        )
        .bind(follower_id)
        .bind(following_id)
        .execute(&self.pool)
        .await?;

        // Update counts
        sqlx::query(
            r#"
            UPDATE users SET following_count = following_count + 1 WHERE id = $1
            "#,
        )
        .bind(follower_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE users SET follower_count = follower_count + 1 WHERE id = $1
            "#,
        )
        .bind(following_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Unfollow a user.
    pub async fn unfollow_user(
        &self,
        follower_id: Uuid,
        following_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM follows
            WHERE follower_id = $1 AND following_id = $2
            "#,
        )
        .bind(follower_id)
        .bind(following_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Update counts
            sqlx::query(
                r#"
                UPDATE users SET following_count = GREATEST(following_count - 1, 0) WHERE id = $1
                "#,
            )
            .bind(follower_id)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                r#"
                UPDATE users SET follower_count = GREATEST(follower_count - 1, 0) WHERE id = $1
                "#,
            )
            .bind(following_id)
            .execute(&self.pool)
            .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a user is following another user.
    pub async fn is_following(&self, follower_id: Uuid, following_id: Uuid) -> Result<bool, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM follows
            WHERE follower_id = $1 AND following_id = $2
            LIMIT 1
            "#,
        )
        .bind(follower_id)
        .bind(following_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    /// Get a user's followers.
    pub async fn get_followers(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::UserSummary>, AppError> {
        let followers: Vec<crate::models::UserSummary> = sqlx::query_as(
            r#"
            SELECT
                u.id,
                u.name,
                u.follower_count,
                u.following_count,
                f.created_at as followed_at
            FROM follows f
            JOIN users u ON u.id = f.follower_id
            WHERE f.following_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(followers)
    }

    /// Get users that a user is following.
    pub async fn get_following(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::UserSummary>, AppError> {
        let following: Vec<crate::models::UserSummary> = sqlx::query_as(
            r#"
            SELECT
                u.id,
                u.name,
                u.follower_count,
                u.following_count,
                f.created_at as followed_at
            FROM follows f
            JOIN users u ON u.id = f.following_id
            WHERE f.follower_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(following)
    }

    /// Get follower and following counts for a user.
    pub async fn get_follow_counts(&self, user_id: Uuid) -> Result<(i32, i32), AppError> {
        let counts: (i32, i32) = sqlx::query_as(
            r#"
            SELECT follower_count, following_count
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(counts)
    }

    /// Get a user profile with follow counts.
    pub async fn get_user_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<crate::models::UserProfile>, AppError> {
        let profile: Option<crate::models::UserProfile> = sqlx::query_as(
            r#"
            SELECT
                id, email, name, created_at,
                follower_count, following_count,
                gender, birth_year, weight_kg, country, region
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(profile)
    }

    /// Get a user by ID (basic info only).
    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, email, name, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    // ========================================================================
    // Activity Feed Methods
    // ========================================================================

    /// Get activity feed for a user (activities from users they follow).
    pub async fn get_activity_feed(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::FeedActivity>, AppError> {
        let activities: Vec<crate::models::FeedActivity> = sqlx::query_as(
            r#"
            SELECT
                a.id,
                a.user_id,
                a.name,
                a.activity_type,
                a.submitted_at,
                a.visibility,
                u.name as user_name,
                s.distance,
                s.duration,
                s.elevation_gain,
                COALESCE(a.kudos_count, 0) as kudos_count,
                COALESCE(a.comment_count, 0) as comment_count
            FROM activities a
            JOIN users u ON a.user_id = u.id
            LEFT JOIN scores s ON a.id = s.activity_id
            WHERE a.user_id IN (
                SELECT following_id FROM follows WHERE follower_id = $1
            )
            AND a.visibility = 'public'
            AND a.deleted_at IS NULL
            ORDER BY a.submitted_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }

    // ========================================================================
    // Notification Methods
    // ========================================================================

    /// Create a notification.
    pub async fn create_notification(
        &self,
        user_id: Uuid,
        notification_type: &str,
        actor_id: Option<Uuid>,
        target_type: Option<&str>,
        target_id: Option<Uuid>,
        message: Option<&str>,
    ) -> Result<crate::models::Notification, AppError> {
        let notification: crate::models::Notification = sqlx::query_as(
            r#"
            INSERT INTO notifications (id, user_id, notification_type, actor_id, target_type, target_id, message, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, NOW())
            RETURNING id, user_id, notification_type, actor_id, target_type, target_id, message, read_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(notification_type)
        .bind(actor_id)
        .bind(target_type)
        .bind(target_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;

        Ok(notification)
    }

    /// Get notifications for a user.
    pub async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::NotificationWithActor>, AppError> {
        let notifications: Vec<crate::models::NotificationWithActor> = sqlx::query_as(
            r#"
            SELECT
                n.id,
                n.user_id,
                n.notification_type,
                n.actor_id,
                u.name as actor_name,
                n.target_type,
                n.target_id,
                n.message,
                n.read_at,
                n.created_at
            FROM notifications n
            LEFT JOIN users u ON u.id = n.actor_id
            WHERE n.user_id = $1
            ORDER BY n.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(notifications)
    }

    /// Get unread notification count for a user.
    pub async fn get_unread_notification_count(&self, user_id: Uuid) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM notifications
            WHERE user_id = $1 AND read_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Mark a notification as read.
    pub async fn mark_notification_read(&self, notification_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET read_at = NOW()
            WHERE id = $1 AND user_id = $2 AND read_at IS NULL
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark all notifications as read for a user.
    pub async fn mark_all_notifications_read(&self, user_id: Uuid) -> Result<i64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET read_at = NOW()
            WHERE user_id = $1 AND read_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
