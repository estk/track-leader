use crate::errors::AppError;
use crate::models::{
    Achievement, AchievementHolder, AchievementType, AchievementWithSegment, Activity,
    ActivityAliasRow, ActivitySegmentEffort, ActivityTypeRow, ActivityWithStats, AgeGroup,
    CountryStats, CrownCountEntry, DateRangeFilter, DistanceLeaderEntry, GenderFilter,
    LeaderboardEntry, LeaderboardFilters, LeaderboardScope, ResolvedActivityType, Scores, Segment,
    SegmentEffort, Team, TeamInvitation, TeamInvitationWithDetails, TeamJoinRequest,
    TeamJoinRequestWithUser, TeamMember, TeamMembership, TeamRole, TeamSummary, TeamVisibility,
    TeamWithMembership, UpdateDemographicsRequest, User, UserWithDemographics, WeightClass,
};
use crate::query_builder::QueryBuilder;
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
            INSERT INTO activities (id, user_id, activity_type_id, name,
                                    object_store_path, started_at, submitted_at, visibility,
                                    type_boundaries, segment_types)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(activity.id)
        .bind(activity.user_id)
        .bind(activity.activity_type_id)
        .bind(&activity.name)
        .bind(&activity.object_store_path)
        .bind(activity.started_at)
        .bind(activity.submitted_at)
        .bind(&activity.visibility)
        .bind(&activity.type_boundaries)
        .bind(&activity.segment_types)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Activity Type Methods
    // ========================================================================

    /// List all activity types (built-in and custom).
    pub async fn list_activity_types(&self) -> Result<Vec<ActivityTypeRow>, AppError> {
        let types: Vec<ActivityTypeRow> = sqlx::query_as(
            r#"
            SELECT id, name, is_builtin, created_by, created_at
            FROM activity_types
            ORDER BY is_builtin DESC, name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(types)
    }

    /// Get a single activity type by ID.
    pub async fn get_activity_type(&self, id: Uuid) -> Result<Option<ActivityTypeRow>, AppError> {
        let activity_type: Option<ActivityTypeRow> = sqlx::query_as(
            r#"
            SELECT id, name, is_builtin, created_by, created_at
            FROM activity_types
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(activity_type)
    }

    /// Resolve an activity type by name or alias.
    /// Returns Exact(id) if a direct match or single alias match,
    /// Ambiguous(ids) if multiple alias matches, or NotFound if no match.
    pub async fn resolve_activity_type(
        &self,
        name_or_alias: &str,
    ) -> Result<ResolvedActivityType, AppError> {
        // First try direct name match (always exact)
        let direct_match: Option<(Uuid,)> =
            sqlx::query_as(r#"SELECT id FROM activity_types WHERE name = $1"#)
                .bind(name_or_alias)
                .fetch_optional(&self.pool)
                .await?;

        if let Some((id,)) = direct_match {
            return Ok(ResolvedActivityType::Exact(id));
        }

        // Then try aliases (may return multiple)
        let alias_matches: Vec<(Uuid,)> =
            sqlx::query_as(r#"SELECT activity_type_id FROM activity_aliases WHERE alias = $1"#)
                .bind(name_or_alias)
                .fetch_all(&self.pool)
                .await?;

        match alias_matches.len() {
            0 => Ok(ResolvedActivityType::NotFound),
            1 => Ok(ResolvedActivityType::Exact(alias_matches[0].0)),
            _ => Ok(ResolvedActivityType::Ambiguous(
                alias_matches.into_iter().map(|(id,)| id).collect(),
            )),
        }
    }

    /// Get all activity types for a given alias (for disambiguation UI).
    pub async fn get_types_for_alias(&self, alias: &str) -> Result<Vec<ActivityTypeRow>, AppError> {
        let types: Vec<ActivityTypeRow> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.is_builtin, t.created_by, t.created_at
            FROM activity_types t
            JOIN activity_aliases a ON a.activity_type_id = t.id
            WHERE a.alias = $1
            ORDER BY t.name ASC
            "#,
        )
        .bind(alias)
        .fetch_all(&self.pool)
        .await?;

        Ok(types)
    }

    /// Create a custom activity type.
    pub async fn create_activity_type(
        &self,
        name: &str,
        created_by: Uuid,
    ) -> Result<ActivityTypeRow, AppError> {
        let activity_type: ActivityTypeRow = sqlx::query_as(
            r#"
            INSERT INTO activity_types (name, is_builtin, created_by, created_at)
            VALUES ($1, false, $2, NOW())
            RETURNING id, name, is_builtin, created_by, created_at
            "#,
        )
        .bind(name)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(activity_type)
    }

    /// Create an alias for an activity type.
    pub async fn create_activity_alias(
        &self,
        alias: &str,
        activity_type_id: Uuid,
    ) -> Result<ActivityAliasRow, AppError> {
        let alias_row: ActivityAliasRow = sqlx::query_as(
            r#"
            INSERT INTO activity_aliases (alias, activity_type_id, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, alias, activity_type_id, created_at
            "#,
        )
        .bind(alias)
        .bind(activity_type_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(alias_row)
    }

    // ========================================================================
    // Activity Methods
    // ========================================================================

    pub async fn get_activity(&self, id: Uuid) -> Result<Option<Activity>, AppError> {
        let activity = sqlx::query_as(
            r#"
            SELECT id, user_id, activity_type_id, name, object_store_path,
                   started_at, submitted_at, visibility, type_boundaries, segment_types
            FROM activities
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(activity)
    }

    pub async fn get_activity_with_stats(
        &self,
        id: Uuid,
    ) -> Result<Option<ActivityWithStats>, AppError> {
        let activity = sqlx::query_as(
            r#"
            SELECT a.id, a.user_id, a.activity_type_id, a.name, a.object_store_path,
                   a.started_at, a.submitted_at, a.visibility, a.type_boundaries, a.segment_types,
                   s.distance, s.duration, s.elevation_gain
            FROM activities a
            LEFT JOIN scores s ON s.activity_id = a.id
            WHERE a.id = $1 AND a.deleted_at IS NULL
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
            SELECT id, user_id, activity_type_id, name, object_store_path,
                   started_at, submitted_at, visibility, type_boundaries, segment_types
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

    /// Get user activities with filtering, sorting, and pagination.
    pub async fn get_user_activities_filtered(
        &self,
        user_id: Uuid,
        params: &crate::handlers::UserActivitiesQuery,
    ) -> Result<Vec<Activity>, AppError> {
        use crate::models::{DateRangeFilter, VisibilityFilter};
        use crate::query_builder::QueryBuilder;

        use crate::models::ActivitySortBy;

        let mut qb = QueryBuilder::new();

        // Base condition: user_id and not deleted (use 'a.' prefix for activities table)
        qb.add_param_condition("a.user_id = ");
        qb.add_condition("a.deleted_at IS NULL");

        // Activity type filter
        if params.activity_type_id.is_some() {
            qb.add_param_condition("a.activity_type_id = ");
        }

        // Date range filter
        let date_range = params.date_range.unwrap_or_default();
        qb.add_date_range(
            date_range,
            "a.submitted_at",
            &params.start_date,
            &params.end_date,
        );

        // Visibility filter (only the owner sees their activities, so all visibilities are visible)
        if let Some(vis) = &params.visibility {
            match vis {
                VisibilityFilter::All => (),
                VisibilityFilter::Public => {
                    let _ = qb.add_condition("a.visibility = 'public'");
                }
                VisibilityFilter::Private => {
                    let _ = qb.add_condition("a.visibility = 'private'");
                }
                VisibilityFilter::TeamsOnly => {
                    let _ = qb.add_condition("a.visibility = 'teams_only'");
                }
            }
        }

        // Name search (case-insensitive)
        let search_pattern = params
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.to_lowercase()));
        if search_pattern.is_some() {
            qb.add_param_condition("LOWER(a.name) LIKE ");
        }

        // Build ORDER BY clause (join with scores for distance/duration sorting)
        let sort_by = params.sort_by.unwrap_or_default();
        let order_clause = match sort_by {
            ActivitySortBy::Recent => "a.submitted_at DESC",
            ActivitySortBy::Oldest => "a.submitted_at ASC",
            ActivitySortBy::Distance => "COALESCE(s.distance, 0) DESC",
            ActivitySortBy::Duration => "COALESCE(s.duration, 0) DESC",
        };

        // Build the final query
        let limit_idx = qb.next_param_idx();
        let offset_idx = qb.next_param_idx();
        let where_clause = qb.build_where_clause();

        // LEFT JOIN with scores to get distance/duration for sorting
        let query = format!(
            r#"
            SELECT a.id, a.user_id, a.activity_type_id, a.name, a.object_store_path,
                   a.started_at, a.submitted_at, a.visibility, a.type_boundaries, a.segment_types
            FROM activities a
            LEFT JOIN scores s ON s.activity_id = a.id
            {where_clause}
            ORDER BY {order_clause}
            LIMIT ${limit_idx} OFFSET ${offset_idx}
            "#
        );

        // Bind parameters in order
        let mut q = sqlx::query_as::<_, Activity>(&query);

        // user_id is always first
        q = q.bind(user_id);

        // Activity type filter
        if let Some(type_id) = params.activity_type_id {
            q = q.bind(type_id);
        }

        // Date range custom params
        if date_range == DateRangeFilter::Custom {
            if let Some(start) = params.start_date {
                q = q.bind(start);
            }
            if let Some(end) = params.end_date {
                q = q.bind(end);
            }
        }

        // Name search
        if let Some(ref pattern) = search_pattern {
            q = q.bind(pattern);
        }

        // Pagination
        q = q.bind(params.limit);
        q = q.bind(params.offset);

        let activities = q.fetch_all(&self.pool).await?;
        Ok(activities)
    }

    pub async fn update_activity(
        &self,
        id: Uuid,
        name: Option<&str>,
        activity_type_id: Option<Uuid>,
        visibility: Option<&str>,
    ) -> Result<Option<Activity>, AppError> {
        let activity = sqlx::query_as(
            r#"
            UPDATE activities
            SET name = COALESCE($2, name),
                activity_type_id = COALESCE($3, activity_type_id),
                visibility = COALESCE($4, visibility)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, user_id, activity_type_id, name, object_store_path,
                      started_at, submitted_at, visibility, type_boundaries, segment_types
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(activity_type_id)
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
        activity_type_id: Uuid,
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
        sqlx::query(
            r#"
            INSERT INTO segments (
                id, creator_id, name, description, activity_type_id,
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
            "#,
        )
        .bind(id)
        .bind(creator_id)
        .bind(name)
        .bind(description)
        .bind(activity_type_id)
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
        .execute(&self.pool)
        .await?;

        // Fetch the segment with the creator name via a JOIN query
        self.get_segment(id).await?.ok_or(AppError::NotFound)
    }

    pub async fn get_segment(&self, id: Uuid) -> Result<Option<Segment>, AppError> {
        let segment = sqlx::query_as(
            r#"
            SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN users u ON u.id = s.creator_id
            WHERE s.id = $1 AND s.deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(segment)
    }

    pub async fn list_segments(
        &self,
        activity_type_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<Segment>, AppError> {
        let segments: Vec<Segment> = if let Some(type_id) = activity_type_id {
            sqlx::query_as(
                r#"
                SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                       s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                       s.average_grade, s.max_grade, s.climb_category,
                       s.visibility, s.created_at
                FROM segments s
                JOIN users u ON u.id = s.creator_id
                WHERE s.deleted_at IS NULL AND s.visibility = 'public' AND s.activity_type_id = $1
                ORDER BY s.created_at DESC
                LIMIT $2
                "#,
            )
            .bind(type_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                       s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                       s.average_grade, s.max_grade, s.climb_category,
                       s.visibility, s.created_at
                FROM segments s
                JOIN users u ON u.id = s.creator_id
                WHERE s.deleted_at IS NULL AND s.visibility = 'public'
                ORDER BY s.created_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(segments)
    }

    /// List segments with filtering and sorting support.
    /// Supports: name search, activity type filter, distance range, climb category, sorting.
    pub async fn list_segments_filtered(
        &self,
        params: &crate::handlers::ListSegmentsQuery,
    ) -> Result<Vec<Segment>, AppError> {
        use crate::handlers::{SegmentSortBy, SortOrder};

        // Build WHERE clause dynamically
        let mut conditions: Vec<String> = vec![
            "s.deleted_at IS NULL".into(),
            "s.visibility = 'public'".into(),
        ];
        let mut param_idx = 1;

        // Activity type filter
        if params.activity_type_id.is_some() {
            conditions.push(format!("s.activity_type_id = ${param_idx}"));
            param_idx += 1;
        }

        // Name search (case-insensitive)
        let search_pattern = params
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.to_lowercase()));
        if search_pattern.is_some() {
            conditions.push(format!("LOWER(s.name) LIKE ${param_idx}"));
            param_idx += 1;
        }

        // Min distance filter
        if params.min_distance_meters.is_some() {
            conditions.push(format!("s.distance_meters >= ${param_idx}"));
            param_idx += 1;
        }

        // Max distance filter
        if params.max_distance_meters.is_some() {
            conditions.push(format!("s.distance_meters <= ${param_idx}"));
            param_idx += 1;
        }

        // Climb category filter
        if let Some(ref cat) = params.climb_category {
            if cat.is_flat() {
                conditions.push("s.climb_category IS NULL".into());
            } else {
                conditions.push(format!("s.climb_category = ${param_idx}"));
                param_idx += 1;
            }
        }

        // Build ORDER BY clause
        let order_col = match params.sort_by {
            SegmentSortBy::CreatedAt => "s.created_at",
            SegmentSortBy::Name => "s.name",
            SegmentSortBy::Distance => "s.distance_meters",
            SegmentSortBy::ElevationGain => "COALESCE(s.elevation_gain_meters, 0)",
        };
        let order_dir = match params.sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        // Build the final query
        let limit_param_idx = param_idx;
        let where_clause = conditions.join(" AND ");
        let query = format!(
            r#"
            SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN users u ON u.id = s.creator_id
            WHERE {where_clause}
            ORDER BY {order_col} {order_dir}
            LIMIT ${limit_param_idx}
            "#
        );

        // Build the query with bindings
        let mut q = sqlx::query_as::<_, Segment>(&query);

        // Bind parameters in the same order we added conditions
        if let Some(type_id) = params.activity_type_id {
            q = q.bind(type_id);
        }
        if let Some(ref pattern) = search_pattern {
            q = q.bind(pattern);
        }
        if let Some(min) = params.min_distance_meters {
            q = q.bind(min);
        }
        if let Some(max) = params.max_distance_meters {
            q = q.bind(max);
        }
        if let Some(ref cat) = params.climb_category
            && !cat.is_flat()
        {
            q = q.bind(cat.to_db_value());
        }
        q = q.bind(params.limit);

        let segments = q.fetch_all(&self.pool).await?;
        Ok(segments)
    }

    pub async fn get_user_segments(&self, user_id: Uuid) -> Result<Vec<Segment>, AppError> {
        let segments: Vec<Segment> = sqlx::query_as(
            r#"
            SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN users u ON u.id = s.creator_id
            WHERE s.creator_id = $1 AND s.deleted_at IS NULL
            ORDER BY s.created_at DESC
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
            WITH inserted AS (
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
            )
            SELECT i.id, i.segment_id, i.activity_id, i.user_id,
                   u.name AS user_name,
                   i.started_at, i.elapsed_time_seconds,
                   i.moving_time_seconds, i.average_speed_mps, i.max_speed_mps,
                   i.is_personal_record, i.created_at,
                   i.start_fraction, i.end_fraction
            FROM inserted i
            JOIN users u ON u.id = i.user_id
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
            SELECT se.id, se.segment_id, se.activity_id, se.user_id,
                   u.name AS user_name,
                   se.started_at, se.elapsed_time_seconds,
                   se.moving_time_seconds, se.average_speed_mps, se.max_speed_mps,
                   se.is_personal_record, se.created_at,
                   se.start_fraction, se.end_fraction
            FROM segment_efforts se
            JOIN users u ON u.id = se.user_id
            WHERE se.segment_id = $1
            ORDER BY se.elapsed_time_seconds ASC
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
            SELECT se.id, se.segment_id, se.activity_id, se.user_id,
                   u.name AS user_name,
                   se.started_at, se.elapsed_time_seconds,
                   se.moving_time_seconds, se.average_speed_mps, se.max_speed_mps,
                   se.is_personal_record, se.created_at,
                   se.start_fraction, se.end_fraction
            FROM segment_efforts se
            JOIN users u ON u.id = se.user_id
            WHERE se.segment_id = $1 AND se.user_id = $2
            ORDER BY se.elapsed_time_seconds ASC
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
                s.activity_type_id,
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

    /// Save track geometry with elevation and timestamps as a LineStringZM.
    /// Points are stored as (X=lon, Y=lat, Z=elevation, M=unix_epoch).
    /// Missing elevation defaults to 0, missing timestamp defaults to 0.
    pub async fn save_track_geometry_with_data(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
        points: &[crate::models::TrackPointData],
    ) -> Result<(), AppError> {
        if points.len() < 2 {
            return Err(AppError::InvalidInput(
                "Track must have at least 2 points".to_string(),
            ));
        }

        // Build LineStringZM WKT: LINESTRING ZM(lon lat ele time, ...)
        let coords: Vec<String> = points
            .iter()
            .map(|p| {
                let epoch = p
                    .timestamp
                    .map(|t| t.unix_timestamp() as f64)
                    .unwrap_or(0.0);
                let ele = p.elevation.unwrap_or(0.0);
                format!("{} {} {} {}", p.lon, p.lat, ele, epoch)
            })
            .collect();
        let wkt = format!("LINESTRING ZM({})", coords.join(", "));

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
        .bind(&wkt)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Save track geometry from a simple WKT string (for backwards compatibility).
    /// Used when creating segments from existing activities without full data.
    pub async fn save_track_geometry(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
        geo_wkt: &str,
    ) -> Result<(), AppError> {
        // For simple 2D WKT, we need to force it to 4D before storing
        sqlx::query(
            r#"
            INSERT INTO tracks (user_id, activity_id, geo, created_at)
            VALUES ($1, $2, ST_Force4D(ST_GeogFromText($3)::geometry)::geography, NOW())
            ON CONFLICT (activity_id) DO UPDATE
            SET geo = ST_Force4D(ST_GeogFromText($3)::geometry)::geography
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

    /// Get track points with all 4 dimensions (lat, lon, elevation, timestamp).
    /// Uses ST_DumpPoints to extract individual points from the LineStringZM.
    pub async fn get_track_points(
        &self,
        activity_id: Uuid,
    ) -> Result<Option<Vec<crate::models::TrackPointData>>, AppError> {
        #[derive(sqlx::FromRow)]
        struct PointRow {
            lon: f64,
            lat: f64,
            elevation: f64,
            epoch: f64,
        }

        let rows: Vec<PointRow> = sqlx::query_as(
            r#"
            SELECT
                ST_X(geom) as lon,
                ST_Y(geom) as lat,
                ST_Z(geom) as elevation,
                ST_M(geom) as epoch
            FROM tracks t,
            LATERAL ST_DumpPoints(t.geo::geometry) AS dp(path, geom)
            WHERE t.activity_id = $1
            ORDER BY dp.path[1]
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let points: Vec<crate::models::TrackPointData> = rows
            .into_iter()
            .map(|r| crate::models::TrackPointData {
                lat: r.lat,
                lon: r.lon,
                elevation: if r.elevation != 0.0 {
                    Some(r.elevation)
                } else {
                    None
                },
                timestamp: if r.epoch != 0.0 {
                    time::OffsetDateTime::from_unix_timestamp(r.epoch as i64).ok()
                } else {
                    None
                },
            })
            .collect();

        Ok(Some(points))
    }

    // Segment matching methods

    /// Find segments that the activity track passes through.
    /// Uses PostGIS to check if track passes within 50m of both segment endpoints
    /// and verifies direction (start before end along the track).
    ///
    /// For single-sport activities: filters by activity_type_id directly.
    /// For multi-sport activities: finds all geometric matches, then filters by
    /// the activity type at each segment's position on the track.
    pub async fn find_matching_segments(
        &self,
        activity_id: Uuid,
        activity_type_id: Uuid,
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
              AND s.activity_type_id = $2
              AND ST_DWithin(t.geo, s.start_point, 50)
              AND ST_DWithin(t.geo, s.end_point, 50)
              AND ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry)
                  < ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry)
            "#,
        )
        .bind(activity_id)
        .bind(activity_type_id)
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

    /// Find segments that the activity track passes through for multi-sport activities.
    /// This version finds all geometric matches first, then the caller filters by
    /// the activity type at each segment's position on the track.
    pub async fn find_matching_segments_any_type(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<(SegmentMatch, Uuid)>, AppError> {
        #[derive(sqlx::FromRow)]
        struct MatchRow {
            id: Uuid,
            activity_type_id: Uuid,
            distance_meters: f64,
            start_pos: f64,
            end_pos: f64,
        }

        let rows: Vec<MatchRow> = sqlx::query_as(
            r#"
            SELECT s.id,
                   s.activity_type_id,
                   s.distance_meters,
                   ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry) as start_pos,
                   ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry) as end_pos
            FROM segments s
            JOIN tracks t ON t.activity_id = $1
            WHERE s.deleted_at IS NULL
              AND ST_DWithin(t.geo, s.start_point, 50)
              AND ST_DWithin(t.geo, s.end_point, 50)
              AND ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry)
                  < ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry)
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    SegmentMatch {
                        segment_id: r.id,
                        distance_meters: r.distance_meters,
                        start_fraction: r.start_pos,
                        end_fraction: r.end_pos,
                    },
                    r.activity_type_id,
                )
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
        activity_type_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid)>, AppError> {
        let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT a.id, a.user_id
            FROM activities a
            JOIN tracks t ON t.activity_id = a.id
            WHERE a.deleted_at IS NULL
              AND a.activity_type_id = $1
            "#,
        )
        .bind(activity_type_id)
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
              AND a.activity_type_id = s.activity_type_id
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
            SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN segment_stars ss ON ss.segment_id = s.id
            JOIN users u ON u.id = s.creator_id
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
                s.activity_type_id,
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
        activity_type_id: Uuid,
        start_wkt: &str,
        end_wkt: &str,
    ) -> Result<Vec<SimilarSegment>, AppError> {
        let rows: Vec<SimilarSegment> = sqlx::query_as(
            r#"
            SELECT id, name, distance_meters
            FROM segments
            WHERE activity_type_id = $1
              AND ST_DWithin(start_point, ST_GeogFromText($2), 30)
              AND ST_DWithin(end_point, ST_GeogFromText($3), 30)
              AND deleted_at IS NULL
            LIMIT 5
            "#,
        )
        .bind(activity_type_id)
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
            SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN users u ON u.id = s.creator_id
            WHERE s.deleted_at IS NULL
              AND s.visibility = 'public'
              AND ST_DWithin(
                  s.start_point,
                  ST_GeogFromText($1),
                  $2
              )
            ORDER BY ST_Distance(s.start_point, ST_GeogFromText($1))
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

        // Build the weight class filter
        let weight_filter =
            filters
                .weight_class
                .weight_range()
                .map(|(min_kg, max_kg)| match (min_kg, max_kg) {
                    (None, Some(max)) => format!("u.weight_kg < {max}"),
                    (Some(min), Some(max)) => {
                        format!("u.weight_kg >= {min} AND u.weight_kg < {max}")
                    }
                    (Some(min), None) => format!("u.weight_kg >= {min}"),
                    (None, None) => unreachable!(),
                });

        // Build the country filter
        let country_filter = filters
            .country
            .as_ref()
            .map(|c| format!("u.country = '{}'", c.replace('\'', "''")));

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
        if let Some(wf) = weight_filter {
            where_clauses.push(wf);
        }
        if let Some(cf) = country_filter {
            where_clauses.push(cf);
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

        // Build the weight class filter
        let weight_filter =
            filters
                .weight_class
                .weight_range()
                .map(|(min_kg, max_kg)| match (min_kg, max_kg) {
                    (None, Some(max)) => format!("u.weight_kg < {max}"),
                    (Some(min), Some(max)) => {
                        format!("u.weight_kg >= {min} AND u.weight_kg < {max}")
                    }
                    (Some(min), None) => format!("u.weight_kg >= {min}"),
                    (None, None) => unreachable!(),
                });

        // Build the country filter
        let country_filter = filters
            .country
            .as_ref()
            .map(|c| format!("u.country = '{}'", c.replace('\'', "''")));

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
        if let Some(wf) = weight_filter {
            where_clauses.push(wf);
        }
        if let Some(cf) = country_filter {
            where_clauses.push(cf);
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
        .bind(req.gender)
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
                s.activity_type_id as segment_activity_type_id
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
    pub async fn get_user_achievement_counts(&self, user_id: Uuid) -> Result<(i64, i64), AppError> {
        let counts: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE achievement_type = 'kom' AND lost_at IS NULL),
                COUNT(*) FILTER (WHERE achievement_type = 'qom' AND lost_at IS NULL)
            FROM achievements
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
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

    /// Get filtered crown count leaderboard with demographic and time scope filtering.
    ///
    /// Filters:
    /// - scope: Time period for counting crowns (based on segment_efforts.started_at)
    /// - gender: Filter users by gender
    /// - age_group: Filter users by age (calculated from birth_year)
    /// - weight_class: Filter users by weight
    /// - country: Filter users by country
    /// - activity_type_id: Filter crowns by segment activity type
    #[allow(clippy::too_many_arguments)]
    pub async fn get_crown_leaderboard_filtered(
        &self,
        limit: i64,
        offset: i64,
        scope: LeaderboardScope,
        gender: GenderFilter,
        age_group: AgeGroup,
        weight_class: WeightClass,
        country: Option<&str>,
        activity_type_id: Option<Uuid>,
        team_id: Option<Uuid>,
    ) -> Result<Vec<CrownCountEntry>, AppError> {
        // Start at index 3 since $1 and $2 are used for LIMIT and OFFSET
        let mut qb = QueryBuilder::with_start_index(3);

        // Base condition: only active crowns
        qb.add_condition("a.lost_at IS NULL");

        // Time scope filter on the effort that earned the crown
        match scope {
            LeaderboardScope::AllTime => {}
            LeaderboardScope::Year => {
                qb.add_condition("se.started_at >= NOW() - INTERVAL '1 year'");
            }
            LeaderboardScope::Month => {
                qb.add_condition("se.started_at >= NOW() - INTERVAL '1 month'");
            }
            LeaderboardScope::Week => {
                qb.add_condition("se.started_at >= NOW() - INTERVAL '7 days'");
            }
        }

        // Gender filter
        match gender {
            GenderFilter::All => {}
            GenderFilter::Male => {
                qb.add_condition("u.gender = 'male'");
            }
            GenderFilter::Female => {
                qb.add_condition("u.gender = 'female'");
            }
        }

        // Age group filter (calculate age from birth_year)
        if let Some((min_age, max_age)) = age_group.age_range() {
            let current_year = time::OffsetDateTime::now_utc().year();
            // birth_year = current_year - age, so for min_age we want birth_year <= current_year - min_age
            let max_birth_year = current_year - min_age;
            qb.add_condition(format!("u.birth_year <= {max_birth_year}"));
            if let Some(max) = max_age {
                let min_birth_year = current_year - max;
                qb.add_condition(format!("u.birth_year >= {min_birth_year}"));
            }
        }

        // Weight class filter
        if let Some((min_kg, max_kg)) = weight_class.weight_range() {
            if let Some(min) = min_kg {
                qb.add_condition(format!("u.weight_kg >= {min}"));
            }
            if let Some(max) = max_kg {
                qb.add_condition(format!("u.weight_kg <= {max}"));
            }
        }

        // Country filter
        if country.is_some() {
            qb.add_param_condition("u.country = ");
        }

        // Activity type filter (filter by segment's activity type)
        if activity_type_id.is_some() {
            qb.add_param_condition("s.activity_type_id = ");
        }

        // Team filter
        if team_id.is_some() {
            qb.add_param_condition("tm.team_id = ");
        }

        let where_clause = qb.build_where_clause();

        // Join with team_memberships if team filter is applied
        let team_join = if team_id.is_some() {
            "JOIN team_memberships tm ON tm.user_id = a.user_id"
        } else {
            ""
        };

        let query = format!(
            r#"
            WITH crown_counts AS (
                SELECT
                    a.user_id,
                    COUNT(*) FILTER (WHERE a.achievement_type = 'kom') as kom_count,
                    COUNT(*) FILTER (WHERE a.achievement_type = 'qom') as qom_count,
                    COUNT(*) as total_crowns
                FROM achievements a
                JOIN users u ON u.id = a.user_id
                JOIN segments s ON s.id = a.segment_id
                LEFT JOIN segment_efforts se ON se.id = a.effort_id
                {team_join}
                {where_clause}
                GROUP BY a.user_id
            )
            SELECT
                cc.user_id,
                u.name as user_name,
                cc.kom_count,
                cc.qom_count,
                cc.total_crowns,
                ROW_NUMBER() OVER (ORDER BY cc.total_crowns DESC, cc.kom_count DESC) as rank
            FROM crown_counts cc
            JOIN users u ON u.id = cc.user_id
            ORDER BY rank
            LIMIT $1 OFFSET $2
            "#
        );

        let mut query_builder = sqlx::query_as::<_, CrownCountEntry>(&query)
            .bind(limit)
            .bind(offset);

        // Bind optional parameters in the order they were added
        if let Some(c) = country {
            query_builder = query_builder.bind(c);
        }
        if let Some(type_id) = activity_type_id {
            query_builder = query_builder.bind(type_id);
        }
        if let Some(tid) = team_id {
            query_builder = query_builder.bind(tid);
        }

        let entries = query_builder.fetch_all(&self.pool).await?;
        Ok(entries)
    }

    /// Get filtered distance leaderboard with demographic and time scope filtering.
    ///
    /// Filters:
    /// - scope: Time period for summing distance (based on scores.created_at via activity submitted_at)
    /// - gender: Filter users by gender
    /// - age_group: Filter users by age (calculated from birth_year)
    /// - weight_class: Filter users by weight
    /// - country: Filter users by country
    /// - team_id: Filter to team members only
    #[allow(clippy::too_many_arguments)]
    pub async fn get_distance_leaderboard_filtered(
        &self,
        limit: i64,
        offset: i64,
        scope: LeaderboardScope,
        gender: GenderFilter,
        age_group: AgeGroup,
        weight_class: WeightClass,
        country: Option<&str>,
        team_id: Option<Uuid>,
    ) -> Result<Vec<DistanceLeaderEntry>, AppError> {
        // Start at index 3 since $1 and $2 are used for LIMIT and OFFSET
        let mut qb = QueryBuilder::with_start_index(3);

        // Time scope filter on scores.created_at
        match scope {
            LeaderboardScope::AllTime => {}
            LeaderboardScope::Year => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '1 year'");
            }
            LeaderboardScope::Month => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '1 month'");
            }
            LeaderboardScope::Week => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '7 days'");
            }
        }

        // Gender filter
        match gender {
            GenderFilter::All => {}
            GenderFilter::Male => {
                qb.add_condition("u.gender = 'male'");
            }
            GenderFilter::Female => {
                qb.add_condition("u.gender = 'female'");
            }
        }

        // Age group filter (calculate age from birth_year)
        if let Some((min_age, max_age)) = age_group.age_range() {
            let current_year = time::OffsetDateTime::now_utc().year();
            let max_birth_year = current_year - min_age;
            qb.add_condition(format!("u.birth_year <= {max_birth_year}"));
            if let Some(max) = max_age {
                let min_birth_year = current_year - max;
                qb.add_condition(format!("u.birth_year >= {min_birth_year}"));
            }
        }

        // Weight class filter
        if let Some((min_kg, max_kg)) = weight_class.weight_range() {
            if let Some(min) = min_kg {
                qb.add_condition(format!("u.weight_kg >= {min}"));
            }
            if let Some(max) = max_kg {
                qb.add_condition(format!("u.weight_kg <= {max}"));
            }
        }

        // Country filter
        if country.is_some() {
            qb.add_param_condition("u.country = ");
        }

        // Team filter
        if team_id.is_some() {
            qb.add_param_condition("tm.team_id = ");
        }

        let where_clause = if qb.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", qb.build_where())
        };

        // Join with team_memberships if team filter is applied
        let team_join = if team_id.is_some() {
            "JOIN team_memberships tm ON tm.user_id = sc.user_id"
        } else {
            ""
        };

        let query = format!(
            r#"
            WITH distance_totals AS (
                SELECT
                    sc.user_id,
                    SUM(sc.distance) as total_distance_meters,
                    COUNT(*) as activity_count
                FROM scores sc
                JOIN users u ON u.id = sc.user_id
                {team_join}
                {where_clause}
                GROUP BY sc.user_id
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
            "#
        );

        let mut query_builder = sqlx::query_as::<_, DistanceLeaderEntry>(&query)
            .bind(limit)
            .bind(offset);

        // Bind optional parameters in the order they were added
        if let Some(c) = country {
            query_builder = query_builder.bind(c);
        }
        if let Some(tid) = team_id {
            query_builder = query_builder.bind(tid);
        }

        let entries = query_builder.fetch_all(&self.pool).await?;
        Ok(entries)
    }

    /// Get filtered dig time leaderboard (total dig seconds in last 7 days).
    ///
    /// Filters:
    /// - gender: Filter users by gender
    /// - age_group: Filter users by age (calculated from birth_year)
    /// - weight_class: Filter users by weight
    /// - country: Filter users by country
    /// - team_id: Filter to team members only
    #[allow(clippy::too_many_arguments)]
    pub async fn get_dig_time_leaderboard_filtered(
        &self,
        limit: i64,
        offset: i64,
        gender: GenderFilter,
        age_group: AgeGroup,
        weight_class: WeightClass,
        country: Option<&str>,
        team_id: Option<Uuid>,
    ) -> Result<Vec<crate::models::DigTimeLeaderEntry>, AppError> {
        // Start at index 3 since $1 and $2 are used for LIMIT and OFFSET
        let mut qb = QueryBuilder::with_start_index(3);

        // Dig time is always weekly (last 7 days)
        qb.add_condition("ads.created_at >= NOW() - INTERVAL '7 days'");

        // Gender filter
        match gender {
            GenderFilter::All => {}
            GenderFilter::Male => {
                qb.add_condition("u.gender = 'male'");
            }
            GenderFilter::Female => {
                qb.add_condition("u.gender = 'female'");
            }
        }

        // Age group filter (calculate age from birth_year)
        if let Some((min_age, max_age)) = age_group.age_range() {
            let current_year = time::OffsetDateTime::now_utc().year();
            let max_birth_year = current_year - min_age;
            qb.add_condition(format!("u.birth_year <= {max_birth_year}"));
            if let Some(max) = max_age {
                let min_birth_year = current_year - max;
                qb.add_condition(format!("u.birth_year >= {min_birth_year}"));
            }
        }

        // Weight class filter
        if let Some((min_kg, max_kg)) = weight_class.weight_range() {
            if let Some(min) = min_kg {
                qb.add_condition(format!("u.weight_kg >= {min}"));
            }
            if let Some(max) = max_kg {
                qb.add_condition(format!("u.weight_kg <= {max}"));
            }
        }

        // Country filter
        if country.is_some() {
            qb.add_param_condition("u.country = ");
        }

        // Team filter
        if team_id.is_some() {
            qb.add_param_condition("tm.team_id = ");
        }

        let where_clause = if qb.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", qb.build_where())
        };

        // Join with team_memberships if team filter is applied
        let team_join = if team_id.is_some() {
            "JOIN team_memberships tm ON tm.user_id = a.user_id"
        } else {
            ""
        };

        let query = format!(
            r#"
            WITH dig_totals AS (
                SELECT
                    a.user_id,
                    SUM(ads.duration_seconds) as total_dig_time_seconds,
                    COUNT(*) as dig_part_count
                FROM activity_dig_parts ads
                JOIN activities a ON a.id = ads.activity_id
                JOIN users u ON u.id = a.user_id
                {team_join}
                {where_clause}
                GROUP BY a.user_id
            )
            SELECT
                dt.user_id,
                u.name as user_name,
                dt.total_dig_time_seconds,
                dt.dig_part_count,
                ROW_NUMBER() OVER (ORDER BY dt.total_dig_time_seconds DESC) as rank
            FROM dig_totals dt
            JOIN users u ON u.id = dt.user_id
            ORDER BY rank
            LIMIT $1 OFFSET $2
            "#
        );

        let mut query_builder = sqlx::query_as::<_, crate::models::DigTimeLeaderEntry>(&query)
            .bind(limit)
            .bind(offset);

        // Bind optional parameters in the order they were added
        if let Some(c) = country {
            query_builder = query_builder.bind(c);
        }
        if let Some(tid) = team_id {
            query_builder = query_builder.bind(tid);
        }

        let entries = query_builder.fetch_all(&self.pool).await?;
        Ok(entries)
    }

    /// Get filtered dig percentage leaderboard (dig_time / ride_activity_time).
    /// Only includes MTB, eMTB, Road, and Gravel activities.
    ///
    /// Filters:
    /// - scope: Time period for calculating percentages
    /// - gender: Filter users by gender
    /// - age_group: Filter users by age (calculated from birth_year)
    /// - weight_class: Filter users by weight
    /// - country: Filter users by country
    /// - team_id: Filter to team members only
    #[allow(clippy::too_many_arguments)]
    pub async fn get_dig_percentage_leaderboard_filtered(
        &self,
        limit: i64,
        offset: i64,
        scope: LeaderboardScope,
        gender: GenderFilter,
        age_group: AgeGroup,
        weight_class: WeightClass,
        country: Option<&str>,
        team_id: Option<Uuid>,
    ) -> Result<Vec<crate::models::DigPercentageLeaderEntry>, AppError> {
        use crate::models::builtin_types;

        // Start at index 3 since $1 and $2 are used for LIMIT and OFFSET
        let mut qb = QueryBuilder::with_start_index(3);

        // Time scope filter
        match scope {
            LeaderboardScope::AllTime => {}
            LeaderboardScope::Year => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '1 year'");
            }
            LeaderboardScope::Month => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '1 month'");
            }
            LeaderboardScope::Week => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '7 days'");
            }
        }

        // Only include ride activity types
        let ride_type_ids = format!(
            "a.activity_type_id IN ('{}', '{}', '{}', '{}')",
            builtin_types::MTB,
            builtin_types::EMTB,
            builtin_types::ROAD,
            builtin_types::GRAVEL
        );
        qb.add_condition(&ride_type_ids);

        // Gender filter
        match gender {
            GenderFilter::All => {}
            GenderFilter::Male => {
                qb.add_condition("u.gender = 'male'");
            }
            GenderFilter::Female => {
                qb.add_condition("u.gender = 'female'");
            }
        }

        // Age group filter
        if let Some((min_age, max_age)) = age_group.age_range() {
            let current_year = time::OffsetDateTime::now_utc().year();
            let max_birth_year = current_year - min_age;
            qb.add_condition(format!("u.birth_year <= {max_birth_year}"));
            if let Some(max) = max_age {
                let min_birth_year = current_year - max;
                qb.add_condition(format!("u.birth_year >= {min_birth_year}"));
            }
        }

        // Weight class filter
        if let Some((min_kg, max_kg)) = weight_class.weight_range() {
            if let Some(min) = min_kg {
                qb.add_condition(format!("u.weight_kg >= {min}"));
            }
            if let Some(max) = max_kg {
                qb.add_condition(format!("u.weight_kg <= {max}"));
            }
        }

        // Country filter
        if country.is_some() {
            qb.add_param_condition("u.country = ");
        }

        // Team filter
        if team_id.is_some() {
            qb.add_param_condition("tm.team_id = ");
        }

        let where_clause = if qb.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", qb.build_where())
        };

        // Join with team_memberships if team filter is applied
        let team_join = if team_id.is_some() {
            "JOIN team_memberships tm ON tm.user_id = a.user_id"
        } else {
            ""
        };

        let query = format!(
            r#"
            WITH user_totals AS (
                SELECT
                    a.user_id,
                    COALESCE(SUM(ads.duration_seconds), 0) as total_dig_time_seconds,
                    COALESCE(SUM(sc.duration), 0) as total_activity_duration_seconds
                FROM activities a
                JOIN scores sc ON sc.activity_id = a.id
                JOIN users u ON u.id = a.user_id
                LEFT JOIN activity_dig_parts ads ON ads.activity_id = a.id
                {team_join}
                {where_clause}
                GROUP BY a.user_id
                HAVING SUM(sc.duration) > 0 AND COALESCE(SUM(ads.duration_seconds), 0) > 0
            )
            SELECT
                ut.user_id,
                u.name as user_name,
                CASE
                    WHEN ut.total_activity_duration_seconds > 0
                    THEN (ut.total_dig_time_seconds / ut.total_activity_duration_seconds) * 100.0
                    ELSE 0.0
                END as dig_percentage,
                ut.total_dig_time_seconds,
                ut.total_activity_duration_seconds,
                ROW_NUMBER() OVER (ORDER BY
                    CASE
                        WHEN ut.total_activity_duration_seconds > 0
                        THEN (ut.total_dig_time_seconds / ut.total_activity_duration_seconds)
                        ELSE 0.0
                    END DESC
                ) as rank
            FROM user_totals ut
            JOIN users u ON u.id = ut.user_id
            ORDER BY rank
            LIMIT $1 OFFSET $2
            "#
        );

        let mut query_builder =
            sqlx::query_as::<_, crate::models::DigPercentageLeaderEntry>(&query)
                .bind(limit)
                .bind(offset);

        // Bind optional parameters in the order they were added
        if let Some(c) = country {
            query_builder = query_builder.bind(c);
        }
        if let Some(tid) = team_id {
            query_builder = query_builder.bind(tid);
        }

        let entries = query_builder.fetch_all(&self.pool).await?;
        Ok(entries)
    }

    /// Get filtered average speed leaderboard (mean average_speed_mps across ride activities).
    ///
    /// Filters:
    /// - scope: Time period for calculating averages
    /// - gender: Filter users by gender
    /// - age_group: Filter users by age (calculated from birth_year)
    /// - weight_class: Filter users by weight
    /// - country: Filter users by country
    /// - team_id: Filter to team members only
    #[allow(clippy::too_many_arguments)]
    pub async fn get_average_speed_leaderboard_filtered(
        &self,
        limit: i64,
        offset: i64,
        scope: LeaderboardScope,
        gender: GenderFilter,
        age_group: AgeGroup,
        weight_class: WeightClass,
        country: Option<&str>,
        team_id: Option<Uuid>,
    ) -> Result<Vec<crate::models::AverageSpeedLeaderEntry>, AppError> {
        // Start at index 3 since $1 and $2 are used for LIMIT and OFFSET
        let mut qb = QueryBuilder::with_start_index(3);

        // Time scope filter on scores.created_at
        match scope {
            LeaderboardScope::AllTime => {}
            LeaderboardScope::Year => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '1 year'");
            }
            LeaderboardScope::Month => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '1 month'");
            }
            LeaderboardScope::Week => {
                qb.add_condition("sc.created_at >= NOW() - INTERVAL '7 days'");
            }
        }

        // Only include activities with valid duration
        qb.add_condition("sc.duration > 0");

        // Gender filter
        match gender {
            GenderFilter::All => {}
            GenderFilter::Male => {
                qb.add_condition("u.gender = 'male'");
            }
            GenderFilter::Female => {
                qb.add_condition("u.gender = 'female'");
            }
        }

        // Age group filter
        if let Some((min_age, max_age)) = age_group.age_range() {
            let current_year = time::OffsetDateTime::now_utc().year();
            let max_birth_year = current_year - min_age;
            qb.add_condition(format!("u.birth_year <= {max_birth_year}"));
            if let Some(max) = max_age {
                let min_birth_year = current_year - max;
                qb.add_condition(format!("u.birth_year >= {min_birth_year}"));
            }
        }

        // Weight class filter
        if let Some((min_kg, max_kg)) = weight_class.weight_range() {
            if let Some(min) = min_kg {
                qb.add_condition(format!("u.weight_kg >= {min}"));
            }
            if let Some(max) = max_kg {
                qb.add_condition(format!("u.weight_kg <= {max}"));
            }
        }

        // Country filter
        if country.is_some() {
            qb.add_param_condition("u.country = ");
        }

        // Team filter
        if team_id.is_some() {
            qb.add_param_condition("tm.team_id = ");
        }

        let where_clause = if qb.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", qb.build_where())
        };

        // Join with team_memberships if team filter is applied
        let team_join = if team_id.is_some() {
            "JOIN team_memberships tm ON tm.user_id = sc.user_id"
        } else {
            ""
        };

        // Calculate average speed as total_distance / total_duration (weighted average)
        let query = format!(
            r#"
            WITH speed_totals AS (
                SELECT
                    sc.user_id,
                    SUM(sc.distance) / NULLIF(SUM(sc.duration), 0) as average_speed_mps,
                    COUNT(*) as activity_count
                FROM scores sc
                JOIN users u ON u.id = sc.user_id
                {team_join}
                {where_clause}
                GROUP BY sc.user_id
            )
            SELECT
                st.user_id,
                u.name as user_name,
                st.average_speed_mps,
                st.activity_count,
                ROW_NUMBER() OVER (ORDER BY st.average_speed_mps DESC) as rank
            FROM speed_totals st
            JOIN users u ON u.id = st.user_id
            WHERE st.average_speed_mps IS NOT NULL
            ORDER BY rank
            LIMIT $1 OFFSET $2
            "#
        );

        let mut query_builder = sqlx::query_as::<_, crate::models::AverageSpeedLeaderEntry>(&query)
            .bind(limit)
            .bind(offset);

        // Bind optional parameters in the order they were added
        if let Some(c) = country {
            query_builder = query_builder.bind(c);
        }
        if let Some(tid) = team_id {
            query_builder = query_builder.bind(tid);
        }

        let entries = query_builder.fetch_all(&self.pool).await?;
        Ok(entries)
    }

    /// Get list of countries with user counts for filter dropdown.
    pub async fn get_countries_with_counts(&self) -> Result<Vec<CountryStats>, AppError> {
        let countries: Vec<CountryStats> = sqlx::query_as(
            r#"
            SELECT
                country,
                COUNT(*) as user_count
            FROM users
            WHERE country IS NOT NULL AND country != ''
            GROUP BY country
            ORDER BY user_count DESC, country ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(countries)
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
    pub async fn is_following(
        &self,
        follower_id: Uuid,
        following_id: Uuid,
    ) -> Result<bool, AppError> {
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
                a.activity_type_id,
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

    /// Get activity feed for a user with filtering support.
    /// Filters activities from users they follow by activity type and date range.
    pub async fn get_activity_feed_filtered(
        &self,
        user_id: Uuid,
        activity_type_id: Option<Uuid>,
        date_range: DateRangeFilter,
        start_date: Option<time::Date>,
        end_date: Option<time::Date>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::FeedActivity>, AppError> {
        // Build dynamic WHERE conditions
        let mut qb = QueryBuilder::new();

        // Always filter to followed users (uses $1)
        // Note: We construct the full subquery condition here to avoid the closing paren
        // being treated as a separate condition joined by AND
        let subquery_idx = qb.next_param_idx();
        qb.add_condition(format!(
            "a.user_id IN (SELECT following_id FROM follows WHERE follower_id = ${subquery_idx})"
        ));

        // Always require public visibility and not deleted
        qb.add_condition("a.visibility = 'public'");
        qb.add_condition("a.deleted_at IS NULL");

        // Optional activity type filter
        qb.add_optional(&activity_type_id, |idx| {
            format!("a.activity_type_id = ${idx}")
        });

        // Date range filter
        qb.add_date_range(date_range, "a.submitted_at", &start_date, &end_date);

        let where_clause = qb.build_where_clause();
        let limit_idx = qb.next_param_idx();
        let offset_idx = qb.next_param_idx();

        let query = format!(
            r#"
            SELECT
                a.id,
                a.user_id,
                a.name,
                a.activity_type_id,
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
            {where_clause}
            ORDER BY a.submitted_at DESC
            LIMIT ${limit_idx} OFFSET ${offset_idx}
            "#
        );

        // Build and execute query with dynamic bindings
        let mut sqlx_query = sqlx::query_as::<_, crate::models::FeedActivity>(&query);

        // Bind in order: user_id is always first
        sqlx_query = sqlx_query.bind(user_id);

        // Bind optional activity_type_id
        if let Some(type_id) = activity_type_id {
            sqlx_query = sqlx_query.bind(type_id);
        }

        // Bind custom date range params if applicable
        if date_range == DateRangeFilter::Custom {
            if let Some(start) = start_date {
                sqlx_query = sqlx_query.bind(start);
            }
            if let Some(end) = end_date {
                sqlx_query = sqlx_query.bind(end);
            }
        }

        // Bind pagination
        sqlx_query = sqlx_query.bind(limit);
        sqlx_query = sqlx_query.bind(offset);

        let activities = sqlx_query.fetch_all(&self.pool).await?;

        Ok(activities)
    }

    /// Get activities for a specific date with visibility filtering.
    ///
    /// - If `user_id` is None (anonymous), only public activities are returned.
    /// - If `user_id` is Some and `mine_only` is true, only that user's activities are returned.
    /// - If `user_id` is Some and `mine_only` is false, returns public activities,
    ///   the user's own private activities, and activities shared with teams the user is a member of.
    pub async fn get_activities_by_date(
        &self,
        date: time::Date,
        user_id: Option<Uuid>,
        mine_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::FeedActivity>, AppError> {
        let activities: Vec<crate::models::FeedActivity> = match (user_id, mine_only) {
            // mine_only requires authentication and filters to user's activities only
            (Some(uid), true) => {
                sqlx::query_as(
                    r#"
                    SELECT
                        a.id,
                        a.user_id,
                        a.name,
                        a.activity_type_id,
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
                    WHERE DATE(COALESCE(a.started_at, a.submitted_at)) = $1
                    AND a.user_id = $2
                    AND a.deleted_at IS NULL
                    ORDER BY COALESCE(a.started_at, a.submitted_at) DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(date)
                .bind(uid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            // Authenticated user sees public + own private + team-shared
            (Some(uid), false) => {
                sqlx::query_as(
                    r#"
                    SELECT
                        a.id,
                        a.user_id,
                        a.name,
                        a.activity_type_id,
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
                    WHERE DATE(COALESCE(a.started_at, a.submitted_at)) = $1
                    AND a.deleted_at IS NULL
                    AND (
                        a.visibility = 'public'
                        OR a.user_id = $2
                        OR (a.visibility = 'teams_only' AND EXISTS (
                            SELECT 1 FROM activity_teams at
                            JOIN team_memberships tm ON tm.team_id = at.team_id
                            WHERE at.activity_id = a.id AND tm.user_id = $2
                        ))
                    )
                    ORDER BY COALESCE(a.started_at, a.submitted_at) DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(date)
                .bind(uid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            // Anonymous user sees only public activities
            (None, _) => {
                sqlx::query_as(
                    r#"
                    SELECT
                        a.id,
                        a.user_id,
                        a.name,
                        a.activity_type_id,
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
                    WHERE DATE(COALESCE(a.started_at, a.submitted_at)) = $1
                    AND a.visibility = 'public'
                    AND a.deleted_at IS NULL
                    ORDER BY COALESCE(a.started_at, a.submitted_at) DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(date)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(activities)
    }

    // ========================================================================
    // Kudos Methods
    // ========================================================================

    /// Give kudos to an activity.
    pub async fn give_kudos(&self, user_id: Uuid, activity_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO kudos (user_id, activity_id, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id, activity_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(activity_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Update kudos count
            sqlx::query(r#"UPDATE activities SET kudos_count = kudos_count + 1 WHERE id = $1"#)
                .bind(activity_id)
                .execute(&self.pool)
                .await?;
            Ok(true)
        } else {
            Ok(false) // Already gave kudos
        }
    }

    /// Remove kudos from an activity.
    pub async fn remove_kudos(&self, user_id: Uuid, activity_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(r#"DELETE FROM kudos WHERE user_id = $1 AND activity_id = $2"#)
            .bind(user_id)
            .bind(activity_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() > 0 {
            // Update kudos count
            sqlx::query(
                r#"UPDATE activities SET kudos_count = GREATEST(kudos_count - 1, 0) WHERE id = $1"#,
            )
            .bind(activity_id)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if user gave kudos to an activity.
    pub async fn has_given_kudos(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
    ) -> Result<bool, AppError> {
        let row: Option<(i32,)> =
            sqlx::query_as(r#"SELECT 1 FROM kudos WHERE user_id = $1 AND activity_id = $2"#)
                .bind(user_id)
                .bind(activity_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.is_some())
    }

    /// Get users who gave kudos to an activity.
    pub async fn get_kudos_givers(
        &self,
        activity_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::models::KudosGiver>, AppError> {
        let givers: Vec<crate::models::KudosGiver> = sqlx::query_as(
            r#"
            SELECT u.id as user_id, u.name as user_name, k.created_at
            FROM kudos k
            JOIN users u ON u.id = k.user_id
            WHERE k.activity_id = $1
            ORDER BY k.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(activity_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(givers)
    }

    // ========================================================================
    // Comments Methods
    // ========================================================================

    /// Add a comment to an activity.
    pub async fn add_comment(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
        content: &str,
        parent_id: Option<Uuid>,
    ) -> Result<crate::models::Comment, AppError> {
        let comment: crate::models::Comment = sqlx::query_as(
            r#"
            INSERT INTO comments (id, user_id, activity_id, parent_id, content, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
            RETURNING id, user_id, activity_id, parent_id, content, created_at, updated_at, deleted_at
            "#,
        )
        .bind(user_id)
        .bind(activity_id)
        .bind(parent_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        // Update comment count
        sqlx::query(r#"UPDATE activities SET comment_count = comment_count + 1 WHERE id = $1"#)
            .bind(activity_id)
            .execute(&self.pool)
            .await?;

        Ok(comment)
    }

    /// Get comments for an activity.
    pub async fn get_comments(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<crate::models::CommentWithUser>, AppError> {
        let comments: Vec<crate::models::CommentWithUser> = sqlx::query_as(
            r#"
            SELECT
                c.id,
                c.user_id,
                c.activity_id,
                c.parent_id,
                c.content,
                c.created_at,
                c.updated_at,
                u.name as user_name
            FROM comments c
            JOIN users u ON u.id = c.user_id
            WHERE c.activity_id = $1 AND c.deleted_at IS NULL
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    /// Delete a comment (soft delete).
    pub async fn delete_comment(&self, comment_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        // Get the activity_id before deleting
        let activity_id: Option<(Uuid,)> = sqlx::query_as(
            r#"SELECT activity_id FROM comments WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"#,
        )
        .bind(comment_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((activity_id,)) = activity_id {
            sqlx::query(r#"UPDATE comments SET deleted_at = NOW() WHERE id = $1 AND user_id = $2"#)
                .bind(comment_id)
                .bind(user_id)
                .execute(&self.pool)
                .await?;

            // Update comment count
            sqlx::query(
                r#"UPDATE activities SET comment_count = GREATEST(comment_count - 1, 0) WHERE id = $1"#,
            )
            .bind(activity_id)
            .execute(&self.pool)
            .await?;

            Ok(true)
        } else {
            Ok(false)
        }
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
    pub async fn mark_notification_read(
        &self,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
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

    // ========================================================================
    // Stats Methods
    // ========================================================================

    /// Get platform-wide statistics.
    pub async fn get_stats(&self) -> Result<crate::models::Stats, AppError> {
        let active_users: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id)
            FROM users
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let segments_created: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM segments
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let activities_uploaded: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM activities
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(crate::models::Stats {
            active_users: active_users.0,
            segments_created: segments_created.0,
            activities_uploaded: activities_uploaded.0,
        })
    }

    // ========================================================================
    // Team Methods
    // ========================================================================

    /// Create a new team.
    pub async fn create_team(
        &self,
        name: &str,
        description: Option<&str>,
        avatar_url: Option<&str>,
        visibility: TeamVisibility,
        join_policy: crate::models::TeamJoinPolicy,
        owner_id: Uuid,
    ) -> Result<Team, AppError> {
        let team: Team = sqlx::query_as(
            r#"
            INSERT INTO teams (name, description, avatar_url, visibility, join_policy, owner_id, member_count, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, 1, NOW())
            RETURNING id, name, description, avatar_url, visibility, join_policy, owner_id,
                      member_count, activity_count, segment_count, featured_leaderboard,
                      created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(avatar_url)
        .bind(visibility)
        .bind(join_policy)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        // Add owner as a member with 'owner' role
        sqlx::query(
            r#"
            INSERT INTO team_memberships (team_id, user_id, role, joined_at)
            VALUES ($1, $2, 'owner', NOW())
            "#,
        )
        .bind(team.id)
        .bind(owner_id)
        .execute(&self.pool)
        .await?;

        Ok(team)
    }

    /// Get a team by ID.
    pub async fn get_team(&self, id: Uuid) -> Result<Option<Team>, AppError> {
        let team: Option<Team> = sqlx::query_as(
            r#"
            SELECT id, name, description, avatar_url, visibility, join_policy, owner_id,
                   member_count, activity_count, segment_count, featured_leaderboard,
                   created_at, updated_at
            FROM teams
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    /// Get a team with membership context for a user.
    pub async fn get_team_with_membership(
        &self,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<TeamWithMembership>, AppError> {
        #[derive(sqlx::FromRow)]
        struct TeamRow {
            id: Uuid,
            name: String,
            description: Option<String>,
            avatar_url: Option<String>,
            visibility: TeamVisibility,
            join_policy: crate::models::TeamJoinPolicy,
            owner_id: Uuid,
            member_count: i32,
            activity_count: i32,
            segment_count: i32,
            featured_leaderboard: Option<crate::models::LeaderboardType>,
            created_at: time::OffsetDateTime,
            updated_at: Option<time::OffsetDateTime>,
            owner_name: String,
            user_role: Option<TeamRole>,
        }

        let row: Option<TeamRow> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.description, t.avatar_url, t.visibility, t.join_policy, t.owner_id,
                   t.member_count, t.activity_count, t.segment_count, t.featured_leaderboard,
                   t.created_at, t.updated_at,
                   u.name as owner_name,
                   tm.role as user_role
            FROM teams t
            JOIN users u ON u.id = t.owner_id
            LEFT JOIN team_memberships tm ON tm.team_id = t.id AND tm.user_id = $2
            WHERE t.id = $1 AND t.deleted_at IS NULL
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TeamWithMembership {
            team: Team {
                id: r.id,
                name: r.name,
                description: r.description,
                avatar_url: r.avatar_url,
                visibility: r.visibility,
                join_policy: r.join_policy,
                owner_id: r.owner_id,
                member_count: r.member_count,
                activity_count: r.activity_count,
                segment_count: r.segment_count,
                featured_leaderboard: r.featured_leaderboard,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            user_role: r.user_role,
            is_member: r.user_role.is_some(),
            owner_name: r.owner_name,
        }))
    }

    /// Update a team.
    pub async fn update_team(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        avatar_url: Option<&str>,
        visibility: Option<TeamVisibility>,
        join_policy: Option<crate::models::TeamJoinPolicy>,
        featured_leaderboard: Option<crate::models::LeaderboardType>,
    ) -> Result<Option<Team>, AppError> {
        let team: Option<Team> = sqlx::query_as(
            r#"
            UPDATE teams
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                avatar_url = COALESCE($4, avatar_url),
                visibility = COALESCE($5, visibility),
                join_policy = COALESCE($6, join_policy),
                featured_leaderboard = COALESCE($7, featured_leaderboard),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, name, description, avatar_url, visibility, join_policy, owner_id,
                      member_count, activity_count, segment_count, featured_leaderboard,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(avatar_url)
        .bind(visibility)
        .bind(join_policy)
        .bind(featured_leaderboard)
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    /// Delete a team (soft delete).
    pub async fn delete_team(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE teams SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List teams for a user (teams they are a member of).
    pub async fn list_user_teams(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<TeamWithMembership>, AppError> {
        #[derive(sqlx::FromRow)]
        struct TeamRow {
            id: Uuid,
            name: String,
            description: Option<String>,
            avatar_url: Option<String>,
            visibility: TeamVisibility,
            join_policy: crate::models::TeamJoinPolicy,
            owner_id: Uuid,
            member_count: i32,
            activity_count: i32,
            segment_count: i32,
            featured_leaderboard: Option<crate::models::LeaderboardType>,
            created_at: time::OffsetDateTime,
            updated_at: Option<time::OffsetDateTime>,
            owner_name: String,
            user_role: TeamRole,
        }

        let rows: Vec<TeamRow> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.description, t.avatar_url, t.visibility, t.join_policy, t.owner_id,
                   t.member_count, t.activity_count, t.segment_count, t.featured_leaderboard,
                   t.created_at, t.updated_at,
                   u.name as owner_name,
                   tm.role as user_role
            FROM teams t
            JOIN team_memberships tm ON tm.team_id = t.id
            JOIN users u ON u.id = t.owner_id
            WHERE tm.user_id = $1 AND t.deleted_at IS NULL
            ORDER BY tm.joined_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| TeamWithMembership {
                team: Team {
                    id: r.id,
                    name: r.name,
                    description: r.description,
                    avatar_url: r.avatar_url,
                    visibility: r.visibility,
                    join_policy: r.join_policy,
                    owner_id: r.owner_id,
                    member_count: r.member_count,
                    activity_count: r.activity_count,
                    segment_count: r.segment_count,
                    featured_leaderboard: r.featured_leaderboard,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                },
                user_role: Some(r.user_role),
                is_member: true,
                owner_name: r.owner_name,
            })
            .collect())
    }

    /// List discoverable teams (for browsing).
    pub async fn list_discoverable_teams(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TeamSummary>, AppError> {
        let teams: Vec<TeamSummary> = sqlx::query_as(
            r#"
            SELECT id, name, description, avatar_url, member_count, activity_count, segment_count
            FROM teams
            WHERE visibility = 'public' AND deleted_at IS NULL
            ORDER BY member_count DESC, created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    // ========================================================================
    // Team Membership Methods
    // ========================================================================

    /// Get a user's membership in a team.
    pub async fn get_team_membership(
        &self,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<TeamMembership>, AppError> {
        let membership: Option<TeamMembership> = sqlx::query_as(
            r#"
            SELECT team_id, user_id, role, invited_by, joined_at
            FROM team_memberships
            WHERE team_id = $1 AND user_id = $2
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(membership)
    }

    /// Add a member to a team.
    pub async fn add_team_member(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        role: TeamRole,
        invited_by: Option<Uuid>,
    ) -> Result<TeamMembership, AppError> {
        let membership: TeamMembership = sqlx::query_as(
            r#"
            INSERT INTO team_memberships (team_id, user_id, role, invited_by, joined_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING team_id, user_id, role, invited_by, joined_at
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(role)
        .bind(invited_by)
        .fetch_one(&self.pool)
        .await?;

        // Update member count
        sqlx::query(r#"UPDATE teams SET member_count = member_count + 1 WHERE id = $1"#)
            .bind(team_id)
            .execute(&self.pool)
            .await?;

        Ok(membership)
    }

    /// Remove a member from a team.
    pub async fn remove_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM team_memberships WHERE team_id = $1 AND user_id = $2
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Update member count
            sqlx::query(
                r#"UPDATE teams SET member_count = GREATEST(member_count - 1, 0) WHERE id = $1"#,
            )
            .bind(team_id)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Change a member's role in a team.
    pub async fn change_team_member_role(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        new_role: TeamRole,
    ) -> Result<Option<TeamMembership>, AppError> {
        let membership: Option<TeamMembership> = sqlx::query_as(
            r#"
            UPDATE team_memberships
            SET role = $3
            WHERE team_id = $1 AND user_id = $2
            RETURNING team_id, user_id, role, invited_by, joined_at
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(new_role)
        .fetch_optional(&self.pool)
        .await?;

        Ok(membership)
    }

    /// List members of a team.
    pub async fn list_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>, AppError> {
        let members: Vec<TeamMember> = sqlx::query_as(
            r#"
            SELECT
                tm.user_id,
                u.name as user_name,
                tm.role,
                tm.joined_at,
                tm.invited_by,
                ib.name as invited_by_name
            FROM team_memberships tm
            JOIN users u ON u.id = tm.user_id
            LEFT JOIN users ib ON ib.id = tm.invited_by
            WHERE tm.team_id = $1
            ORDER BY
                CASE tm.role
                    WHEN 'owner' THEN 1
                    WHEN 'admin' THEN 2
                    ELSE 3
                END,
                tm.joined_at ASC
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    // ========================================================================
    // Team Join Request Methods
    // ========================================================================

    /// Create a join request for a team.
    pub async fn create_team_join_request(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        message: Option<&str>,
    ) -> Result<TeamJoinRequest, AppError> {
        let request: TeamJoinRequest = sqlx::query_as(
            r#"
            INSERT INTO team_join_requests (team_id, user_id, message, status, created_at)
            VALUES ($1, $2, $3, 'pending', NOW())
            ON CONFLICT (team_id, user_id) DO UPDATE SET
                message = COALESCE($3, team_join_requests.message),
                status = 'pending',
                reviewed_by = NULL,
                reviewed_at = NULL,
                created_at = NOW()
            RETURNING id, team_id, user_id, message, status, reviewed_by, reviewed_at, created_at
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    /// Get pending join requests for a team.
    pub async fn get_pending_join_requests(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<TeamJoinRequestWithUser>, AppError> {
        let requests: Vec<TeamJoinRequestWithUser> = sqlx::query_as(
            r#"
            SELECT
                jr.id, jr.team_id, jr.user_id, u.name as user_name,
                jr.message, jr.status, jr.created_at
            FROM team_join_requests jr
            JOIN users u ON u.id = jr.user_id
            WHERE jr.team_id = $1 AND jr.status = 'pending'
            ORDER BY jr.created_at ASC
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(requests)
    }

    /// Approve or reject a join request.
    pub async fn review_join_request(
        &self,
        request_id: Uuid,
        reviewer_id: Uuid,
        approved: bool,
    ) -> Result<Option<TeamJoinRequest>, AppError> {
        let status = if approved { "approved" } else { "rejected" };
        let request: Option<TeamJoinRequest> = sqlx::query_as(
            r#"
            UPDATE team_join_requests
            SET status = $2, reviewed_by = $3, reviewed_at = NOW()
            WHERE id = $1 AND status = 'pending'
            RETURNING id, team_id, user_id, message, status, reviewed_by, reviewed_at, created_at
            "#,
        )
        .bind(request_id)
        .bind(status)
        .bind(reviewer_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(request)
    }

    /// Get a join request by ID.
    pub async fn get_join_request(&self, id: Uuid) -> Result<Option<TeamJoinRequest>, AppError> {
        let request: Option<TeamJoinRequest> = sqlx::query_as(
            r#"
            SELECT id, team_id, user_id, message, status, reviewed_by, reviewed_at, created_at
            FROM team_join_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(request)
    }

    /// Check if a user has a pending join request for a team.
    pub async fn has_pending_join_request(
        &self,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM team_join_requests
            WHERE team_id = $1 AND user_id = $2 AND status = 'pending'
            LIMIT 1
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    // ========================================================================
    // Team Invitation Methods
    // ========================================================================

    /// Create an invitation to join a team.
    pub async fn create_team_invitation(
        &self,
        team_id: Uuid,
        email: &str,
        invited_by: Uuid,
        role: TeamRole,
        token: &str,
        expires_at: time::OffsetDateTime,
    ) -> Result<TeamInvitation, AppError> {
        let invitation: TeamInvitation = sqlx::query_as(
            r#"
            INSERT INTO team_invitations (team_id, email, invited_by, role, token, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (team_id, email) DO UPDATE SET
                invited_by = $3,
                role = $4,
                token = $5,
                expires_at = $6,
                accepted_at = NULL,
                created_at = NOW()
            RETURNING id, team_id, email, invited_by, role, token, expires_at, accepted_at, created_at
            "#,
        )
        .bind(team_id)
        .bind(email)
        .bind(invited_by)
        .bind(role)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(invitation)
    }

    /// Get a team invitation by token.
    pub async fn get_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<Option<TeamInvitationWithDetails>, AppError> {
        let invitation: Option<TeamInvitationWithDetails> = sqlx::query_as(
            r#"
            SELECT
                ti.id, ti.team_id, t.name as team_name, ti.email,
                ti.invited_by, u.name as invited_by_name, ti.role,
                ti.expires_at, ti.created_at
            FROM team_invitations ti
            JOIN teams t ON t.id = ti.team_id
            JOIN users u ON u.id = ti.invited_by
            WHERE ti.token = $1 AND ti.accepted_at IS NULL AND t.deleted_at IS NULL
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(invitation)
    }

    /// Mark an invitation as accepted.
    pub async fn accept_invitation(&self, token: &str) -> Result<Option<TeamInvitation>, AppError> {
        let invitation: Option<TeamInvitation> = sqlx::query_as(
            r#"
            UPDATE team_invitations
            SET accepted_at = NOW()
            WHERE token = $1 AND accepted_at IS NULL AND expires_at > NOW()
            RETURNING id, team_id, email, invited_by, role, token, expires_at, accepted_at, created_at
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(invitation)
    }

    /// Get pending invitations for a team.
    pub async fn get_pending_invitations(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<TeamInvitation>, AppError> {
        let invitations: Vec<TeamInvitation> = sqlx::query_as(
            r#"
            SELECT id, team_id, email, invited_by, role, token, expires_at, accepted_at, created_at
            FROM team_invitations
            WHERE team_id = $1 AND accepted_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(invitations)
    }

    /// Revoke (delete) an invitation.
    pub async fn revoke_invitation(&self, invitation_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM team_invitations WHERE id = $1 AND accepted_at IS NULL
            "#,
        )
        .bind(invitation_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Activity-Team Sharing Methods
    // ========================================================================

    /// Share an activity with teams.
    pub async fn share_activity_with_teams(
        &self,
        activity_id: Uuid,
        team_ids: &[Uuid],
        shared_by: Uuid,
    ) -> Result<(), AppError> {
        for team_id in team_ids {
            sqlx::query(
                r#"
                INSERT INTO activity_teams (activity_id, team_id, shared_at, shared_by)
                VALUES ($1, $2, NOW(), $3)
                ON CONFLICT (activity_id, team_id) DO NOTHING
                "#,
            )
            .bind(activity_id)
            .bind(team_id)
            .bind(shared_by)
            .execute(&self.pool)
            .await?;

            // Update team activity count
            sqlx::query(r#"UPDATE teams SET activity_count = activity_count + 1 WHERE id = $1"#)
                .bind(team_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Get teams an activity is shared with.
    pub async fn get_activity_teams(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<TeamSummary>, AppError> {
        let teams: Vec<TeamSummary> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.description, t.avatar_url, t.member_count, t.activity_count, t.segment_count
            FROM teams t
            JOIN activity_teams at ON at.team_id = t.id
            WHERE at.activity_id = $1 AND t.deleted_at IS NULL
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    /// Unshare an activity from a team.
    pub async fn unshare_activity_from_team(
        &self,
        activity_id: Uuid,
        team_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM activity_teams WHERE activity_id = $1 AND team_id = $2
            "#,
        )
        .bind(activity_id)
        .bind(team_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Update team activity count
            sqlx::query(
                r#"UPDATE teams SET activity_count = GREATEST(activity_count - 1, 0) WHERE id = $1"#,
            )
            .bind(team_id)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if user has access to an activity through team membership.
    pub async fn user_has_activity_team_access(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
    ) -> Result<bool, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM activity_teams at
            JOIN team_memberships tm ON tm.team_id = at.team_id
            WHERE at.activity_id = $1 AND tm.user_id = $2
            LIMIT 1
            "#,
        )
        .bind(activity_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    // ========================================================================
    // Segment-Team Sharing Methods
    // ========================================================================

    /// Share a segment with teams.
    pub async fn share_segment_with_teams(
        &self,
        segment_id: Uuid,
        team_ids: &[Uuid],
    ) -> Result<(), AppError> {
        for team_id in team_ids {
            sqlx::query(
                r#"
                INSERT INTO segment_teams (segment_id, team_id, shared_at)
                VALUES ($1, $2, NOW())
                ON CONFLICT (segment_id, team_id) DO NOTHING
                "#,
            )
            .bind(segment_id)
            .bind(team_id)
            .execute(&self.pool)
            .await?;

            // Update team segment count
            sqlx::query(r#"UPDATE teams SET segment_count = segment_count + 1 WHERE id = $1"#)
                .bind(team_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Get teams a segment is shared with.
    pub async fn get_segment_teams(&self, segment_id: Uuid) -> Result<Vec<TeamSummary>, AppError> {
        let teams: Vec<TeamSummary> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.description, t.avatar_url, t.member_count, t.activity_count, t.segment_count
            FROM teams t
            JOIN segment_teams st ON st.team_id = t.id
            WHERE st.segment_id = $1 AND t.deleted_at IS NULL
            "#,
        )
        .bind(segment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    /// Unshare a segment from a team.
    pub async fn unshare_segment_from_team(
        &self,
        segment_id: Uuid,
        team_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM segment_teams WHERE segment_id = $1 AND team_id = $2
            "#,
        )
        .bind(segment_id)
        .bind(team_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Update team segment count
            sqlx::query(
                r#"UPDATE teams SET segment_count = GREATEST(segment_count - 1, 0) WHERE id = $1"#,
            )
            .bind(team_id)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if user has access to a segment through team membership.
    pub async fn user_has_segment_team_access(
        &self,
        user_id: Uuid,
        segment_id: Uuid,
    ) -> Result<bool, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM segment_teams st
            JOIN team_memberships tm ON tm.team_id = st.team_id
            WHERE st.segment_id = $1 AND tm.user_id = $2
            LIMIT 1
            "#,
        )
        .bind(segment_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    // ========================================================================
    // Team Activity/Segment List Methods
    // ========================================================================

    /// Get activities shared with a team.
    pub async fn get_team_activities(
        &self,
        team_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::FeedActivity>, AppError> {
        let activities: Vec<crate::models::FeedActivity> = sqlx::query_as(
            r#"
            SELECT
                a.id,
                a.user_id,
                a.name,
                a.activity_type_id,
                a.submitted_at,
                a.visibility,
                u.name as user_name,
                s.distance,
                s.duration,
                s.elevation_gain,
                COALESCE(a.kudos_count, 0) as kudos_count,
                COALESCE(a.comment_count, 0) as comment_count
            FROM activities a
            JOIN activity_teams at ON at.activity_id = a.id
            JOIN users u ON a.user_id = u.id
            LEFT JOIN scores s ON a.id = s.activity_id
            WHERE at.team_id = $1 AND a.deleted_at IS NULL
            ORDER BY at.shared_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(team_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }

    /// Get segments shared with a team.
    pub async fn get_team_segments(
        &self,
        team_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Segment>, AppError> {
        let segments: Vec<Segment> = sqlx::query_as(
            r#"
            SELECT s.id, s.creator_id, u.name as creator_name, s.name, s.description, s.activity_type_id,
                   s.distance_meters, s.elevation_gain_meters, s.elevation_loss_meters,
                   s.average_grade, s.max_grade, s.climb_category,
                   s.visibility, s.created_at
            FROM segments s
            JOIN segment_teams st ON st.segment_id = s.id
            JOIN users u ON u.id = s.creator_id
            WHERE st.team_id = $1 AND s.deleted_at IS NULL
            ORDER BY st.shared_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(team_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(segments)
    }

    /// Get activities shared with a team for a specific date.
    pub async fn get_team_activities_by_date(
        &self,
        team_id: Uuid,
        date: time::Date,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::FeedActivity>, AppError> {
        let activities: Vec<crate::models::FeedActivity> = sqlx::query_as(
            r#"
            SELECT
                a.id,
                a.user_id,
                a.name,
                a.activity_type_id,
                a.submitted_at,
                a.visibility,
                u.name as user_name,
                s.distance,
                s.duration,
                s.elevation_gain,
                COALESCE(a.kudos_count, 0) as kudos_count,
                COALESCE(a.comment_count, 0) as comment_count
            FROM activities a
            JOIN activity_teams at ON at.activity_id = a.id
            JOIN users u ON a.user_id = u.id
            LEFT JOIN scores s ON a.id = s.activity_id
            WHERE at.team_id = $1
            AND DATE(COALESCE(a.started_at, a.submitted_at)) = $2
            AND a.deleted_at IS NULL
            ORDER BY COALESCE(a.started_at, a.submitted_at) DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(team_id)
        .bind(date)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }

    /// Get team names for an activity (for teams_only visibility display).
    pub async fn get_activity_team_names(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let names: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT t.name
            FROM teams t
            JOIN activity_teams at ON at.team_id = t.id
            WHERE at.activity_id = $1
            ORDER BY t.name
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(names.into_iter().map(|(name,)| name).collect())
    }

    /// Get team names for multiple activities in one query (for efficiency).
    pub async fn get_activities_team_names(
        &self,
        activity_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<String>>, AppError> {
        if activity_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows: Vec<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT at.activity_id, t.name
            FROM teams t
            JOIN activity_teams at ON at.team_id = t.id
            WHERE at.activity_id = ANY($1)
            ORDER BY t.name
            "#,
        )
        .bind(activity_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut result: std::collections::HashMap<Uuid, Vec<String>> =
            std::collections::HashMap::new();
        for (activity_id, team_name) in rows {
            result.entry(activity_id).or_default().push(team_name);
        }

        Ok(result)
    }

    // ========================================================================
    // Stopped Segment / Dig Tagging Methods
    // ========================================================================

    /// Save detected stopped segments for an activity.
    pub async fn save_stopped_segments(
        &self,
        activity_id: Uuid,
        segments: &[crate::activity_queue::DetectedStoppedSegment],
    ) -> Result<(), AppError> {
        for segment in segments {
            sqlx::query(
                r#"
                INSERT INTO activity_stopped_segments
                    (activity_id, start_time, end_time, duration_seconds)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(activity_id)
            .bind(segment.start_time)
            .bind(segment.end_time)
            .bind(segment.duration_seconds)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get stopped segments for an activity.
    pub async fn get_stopped_segments(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<crate::models::StoppedSegment>, AppError> {
        let segments: Vec<crate::models::StoppedSegment> = sqlx::query_as(
            r#"
            SELECT id, activity_id, start_time, end_time, duration_seconds, created_at
            FROM activity_stopped_segments
            WHERE activity_id = $1
            ORDER BY start_time
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(segments)
    }

    /// Create dig segments from stopped segment IDs.
    pub async fn create_dig_parts(
        &self,
        activity_id: Uuid,
        stopped_segment_ids: &[Uuid],
    ) -> Result<Vec<crate::models::DigPart>, AppError> {
        // First verify all stopped segments belong to this activity
        let stopped_segments: Vec<crate::models::StoppedSegment> = sqlx::query_as(
            r#"
            SELECT id, activity_id, start_time, end_time, duration_seconds, created_at
            FROM activity_stopped_segments
            WHERE id = ANY($1) AND activity_id = $2
            "#,
        )
        .bind(stopped_segment_ids)
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        if stopped_segments.len() != stopped_segment_ids.len() {
            return Err(AppError::InvalidInput(
                "Some stopped segment IDs are invalid or don't belong to this activity".to_string(),
            ));
        }

        // Create dig segments from the stopped segments
        let mut dig_parts = Vec::new();
        for stopped in &stopped_segments {
            let dig: crate::models::DigPart = sqlx::query_as(
                r#"
                INSERT INTO activity_dig_parts
                    (activity_id, start_time, end_time, duration_seconds)
                VALUES ($1, $2, $3, $4)
                RETURNING id, activity_id, start_time, end_time, duration_seconds, created_at
                "#,
            )
            .bind(activity_id)
            .bind(stopped.start_time)
            .bind(stopped.end_time)
            .bind(stopped.duration_seconds)
            .fetch_one(&self.pool)
            .await?;

            dig_parts.push(dig);
        }

        Ok(dig_parts)
    }

    /// Save dig segments extracted from multi-sport activities.
    /// Used when processing activities with DIG activity type segments.
    pub async fn save_dig_parts_batch(
        &self,
        activity_id: Uuid,
        segments: &[crate::activity_queue::DetectedStoppedSegment],
    ) -> Result<(), AppError> {
        for segment in segments {
            sqlx::query(
                r#"
                INSERT INTO activity_dig_parts
                    (activity_id, start_time, end_time, duration_seconds)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(activity_id)
            .bind(segment.start_time)
            .bind(segment.end_time)
            .bind(segment.duration_seconds)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Get dig segments for an activity.
    pub async fn get_dig_parts(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<crate::models::DigPart>, AppError> {
        let segments: Vec<crate::models::DigPart> = sqlx::query_as(
            r#"
            SELECT id, activity_id, start_time, end_time, duration_seconds, created_at
            FROM activity_dig_parts
            WHERE activity_id = $1
            ORDER BY start_time
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(segments)
    }

    /// Get dig time summary for an activity.
    pub async fn get_dig_time_summary(
        &self,
        activity_id: Uuid,
    ) -> Result<crate::models::DigTimeSummary, AppError> {
        let row: Option<(f64, i64, Option<f64>)> = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(ads.duration_seconds), 0.0) as total_dig_time,
                COUNT(ads.id) as dig_count,
                s.duration as activity_duration
            FROM activities a
            LEFT JOIN activity_dig_parts ads ON ads.activity_id = a.id
            LEFT JOIN scores s ON s.activity_id = a.id
            WHERE a.id = $1
            GROUP BY a.id, s.duration
            "#,
        )
        .bind(activity_id)
        .fetch_optional(&self.pool)
        .await?;

        let (total_dig_time_seconds, dig_part_count, activity_duration_seconds) =
            row.unwrap_or((0.0, 0, None));

        Ok(crate::models::DigTimeSummary {
            activity_id,
            total_dig_time_seconds,
            dig_part_count,
            activity_duration_seconds,
        })
    }

    /// Delete a dig segment.
    pub async fn delete_dig_part(
        &self,
        activity_id: Uuid,
        dig_part_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM activity_dig_parts
            WHERE id = $1 AND activity_id = $2
            "#,
        )
        .bind(dig_part_id)
        .bind(activity_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Insert a dig part if it doesn't already exist (based on activity_id and start_time).
    /// Returns true if a new row was inserted, false if it already existed.
    pub async fn insert_dig_part_if_not_exists(
        &self,
        activity_id: Uuid,
        start_time: time::OffsetDateTime,
        end_time: time::OffsetDateTime,
        duration_seconds: f64,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO activity_dig_parts (activity_id, start_time, end_time, duration_seconds)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(activity_id)
        .bind(start_time)
        .bind(end_time)
        .bind(duration_seconds)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get dig heatmap data showing geographic locations where trail maintenance occurred.
    /// Extracts track points that fall within dig segment time ranges, aggregates by
    /// rounded coordinates (4 decimal places ≈ 10m grid), and returns summary statistics.
    pub async fn get_dig_heatmap_data(
        &self,
        team_id: Option<Uuid>,
        since: Option<time::OffsetDateTime>,
        limit: i64,
    ) -> Result<crate::models::DigHeatmapResponse, AppError> {
        // Build the query dynamically based on filters
        let team_join = if team_id.is_some() {
            "JOIN activity_teams tas ON tas.activity_id = a.id"
        } else {
            ""
        };

        let team_where = if team_id.is_some() {
            "AND tas.team_id = $2"
        } else {
            ""
        };

        let since_where = if since.is_some() {
            if team_id.is_some() {
                "AND ads.start_time >= $3"
            } else {
                "AND ads.start_time >= $2"
            }
        } else {
            ""
        };

        let query = format!(
            r#"
            WITH dig_points AS (
                SELECT
                    ROUND(ST_X(dp.geom)::numeric, 4) as lon,
                    ROUND(ST_Y(dp.geom)::numeric, 4) as lat,
                    ads.duration_seconds
                FROM activity_dig_parts ads
                JOIN activities a ON a.id = ads.activity_id
                JOIN tracks t ON t.activity_id = a.id
                {team_join}
                CROSS JOIN LATERAL ST_DumpPoints(t.geo::geometry) AS dp(path, geom)
                WHERE a.deleted_at IS NULL
                AND ST_M(dp.geom) BETWEEN EXTRACT(EPOCH FROM ads.start_time) AND EXTRACT(EPOCH FROM ads.end_time)
                {team_where}
                {since_where}
            ),
            aggregated AS (
                SELECT
                    lon::float8 as lon,
                    lat::float8 as lat,
                    SUM(duration_seconds) as total_duration_seconds,
                    COUNT(*) as frequency
                FROM dig_points
                GROUP BY lon, lat
                ORDER BY total_duration_seconds DESC
                LIMIT $1
            )
            SELECT lon, lat, total_duration_seconds, frequency
            FROM aggregated
            "#
        );

        // Bind parameters based on which filters are present
        // $1 = limit, $2 = team_id OR since (if no team), $3 = since (if team present)
        let points: Vec<crate::models::DigHeatmapPoint> = match (team_id, since) {
            (Some(tid), Some(s)) => {
                sqlx::query_as(&query)
                    .bind(limit)
                    .bind(tid)
                    .bind(s)
                    .fetch_all(&self.pool)
                    .await?
            }
            (Some(tid), None) => {
                sqlx::query_as(&query)
                    .bind(limit)
                    .bind(tid)
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, Some(s)) => {
                sqlx::query_as(&query)
                    .bind(limit)
                    .bind(s)
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, None) => {
                sqlx::query_as(&query)
                    .bind(limit)
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        // Calculate bounds and totals
        let bounds = if points.is_empty() {
            None
        } else {
            let min_lat = points.iter().map(|p| p.lat).fold(f64::INFINITY, f64::min);
            let max_lat = points
                .iter()
                .map(|p| p.lat)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_lon = points.iter().map(|p| p.lon).fold(f64::INFINITY, f64::min);
            let max_lon = points
                .iter()
                .map(|p| p.lon)
                .fold(f64::NEG_INFINITY, f64::max);

            Some(crate::models::DigHeatmapBounds {
                min_lat,
                max_lat,
                min_lon,
                max_lon,
            })
        };

        let total_dig_time_seconds: f64 = points.iter().map(|p| p.total_duration_seconds).sum();
        let total_dig_count: i64 = points.iter().map(|p| p.frequency).sum();

        Ok(crate::models::DigHeatmapResponse {
            points,
            bounds,
            total_dig_time_seconds,
            total_dig_count,
        })
    }

    // ========================================================================
    // Sensor Data Methods
    // ========================================================================

    /// Save sensor data for an activity.
    /// Arrays should be parallel to track points.
    pub async fn save_sensor_data(
        &self,
        activity_id: Uuid,
        sensor_data: &crate::file_parsers::SensorData,
    ) -> Result<(), AppError> {
        // Only save if there's actual data
        if !sensor_data.has_any_data() {
            return Ok(());
        }

        sqlx::query(
            r#"
            INSERT INTO activity_sensor_data (activity_id, heart_rates, cadences, powers, temperatures)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (activity_id) DO UPDATE
            SET heart_rates = EXCLUDED.heart_rates,
                cadences = EXCLUDED.cadences,
                powers = EXCLUDED.powers,
                temperatures = EXCLUDED.temperatures
            "#,
        )
        .bind(activity_id)
        .bind(&sensor_data.heart_rates)
        .bind(&sensor_data.cadences)
        .bind(&sensor_data.powers)
        .bind(&sensor_data.temperatures)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get sensor data for an activity, including calculated distances from track geometry.
    pub async fn get_sensor_data(
        &self,
        activity_id: Uuid,
    ) -> Result<Option<crate::models::ActivitySensorDataResponse>, AppError> {
        // First get the sensor data arrays
        let sensor_row: Option<(
            Option<Vec<Option<i32>>>,
            Option<Vec<Option<i32>>>,
            Option<Vec<Option<i32>>>,
            Option<Vec<Option<f64>>>,
        )> = sqlx::query_as(
            r#"
            SELECT heart_rates, cadences, powers, temperatures
            FROM activity_sensor_data
            WHERE activity_id = $1
            "#,
        )
        .bind(activity_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((heart_rates, cadences, powers, temperatures)) = sensor_row else {
            return Ok(None);
        };

        // Calculate distances from track geometry
        // We extract each point and calculate cumulative distance
        let distances: Vec<f64> = sqlx::query_scalar(
            r#"
            WITH points AS (
                SELECT
                    (ST_DumpPoints(geo::geometry)).geom AS pt,
                    (ST_DumpPoints(geo::geometry)).path[1] AS idx
                FROM tracks
                WHERE activity_id = $1
            ),
            with_prev AS (
                SELECT
                    idx,
                    pt,
                    LAG(pt) OVER (ORDER BY idx) AS prev_pt
                FROM points
            )
            SELECT
                COALESCE(
                    SUM(ST_Distance(pt::geography, prev_pt::geography)) OVER (ORDER BY idx),
                    0
                )::float8 AS cum_distance
            FROM with_prev
            ORDER BY idx
            "#,
        )
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await?;

        // Check what data we have
        let has_heart_rate = heart_rates
            .as_ref()
            .map(|v| v.iter().any(|x| x.is_some()))
            .unwrap_or(false);
        let has_cadence = cadences
            .as_ref()
            .map(|v| v.iter().any(|x| x.is_some()))
            .unwrap_or(false);
        let has_power = powers
            .as_ref()
            .map(|v| v.iter().any(|x| x.is_some()))
            .unwrap_or(false);
        let has_temperature = temperatures
            .as_ref()
            .map(|v| v.iter().any(|x| x.is_some()))
            .unwrap_or(false);

        Ok(Some(crate::models::ActivitySensorDataResponse {
            activity_id,
            has_heart_rate,
            has_cadence,
            has_power,
            has_temperature,
            distances,
            heart_rates: if has_heart_rate { heart_rates } else { None },
            cadences: if has_cadence { cadences } else { None },
            powers: if has_power { powers } else { None },
            temperatures: if has_temperature { temperatures } else { None },
        }))
    }

    // ========================================================================
    // Recovery Methods
    // ========================================================================

    /// Find activities that have been uploaded but not fully processed.
    /// These are activities with an object_store_path but no corresponding track geometry.
    pub async fn find_orphaned_activities(
        &self,
    ) -> Result<Vec<crate::models::OrphanedActivity>, AppError> {
        let rows = sqlx::query_as::<_, crate::models::OrphanedActivity>(
            r#"
            SELECT a.id, a.user_id, a.activity_type_id, a.object_store_path,
                   a.type_boundaries, a.segment_types
            FROM activities a
            LEFT JOIN tracks t ON a.id = t.activity_id
            WHERE t.id IS NULL
              AND a.deleted_at IS NULL
            ORDER BY a.submitted_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
