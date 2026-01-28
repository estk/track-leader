//! Database seeding utilities.

use sqlx::PgPool;
use thiserror::Error;
use tracing::info;

use crate::generators::{
    GeneratedActivity, GeneratedComment, GeneratedEffort, GeneratedFollow, GeneratedKudos,
    GeneratedSegment, GeneratedUser,
};
use tracks::models::{ActivityType, Gender};

/// Convert Gender enum to its database string representation.
fn gender_to_db_str(gender: &Gender) -> &'static str {
    match gender {
        Gender::Male => "male",
        Gender::Female => "female",
        Gender::Other => "other",
        Gender::PreferNotToSay => "prefer_not_to_say",
    }
}

#[derive(Debug, Error)]
pub enum SeedError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Track points required for activity")]
    NoTrackPoints,
}

/// Progress callback type for long-running operations.
#[allow(dead_code)]
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Database seeder for inserting generated test data.
pub struct Seeder {
    pool: PgPool,
    batch_size: usize,
}

impl Seeder {
    /// Creates a new seeder with the given database pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            batch_size: 50,
        }
    }

    /// Sets the batch size for bulk operations.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Seeds users into the database.
    pub async fn seed_users(&self, users: &[GeneratedUser]) -> Result<(), SeedError> {
        info!("Seeding {} users...", users.len());

        for chunk in users.chunks(self.batch_size) {
            self.insert_user_batch(chunk).await?;
        }

        info!("Seeded {} users", users.len());
        Ok(())
    }

    /// Inserts a batch of users.
    async fn insert_user_batch(&self, users: &[GeneratedUser]) -> Result<(), SeedError> {
        for user in users {
            sqlx::query(
                r#"
                INSERT INTO users (id, name, email, password_hash, auth_provider, created_at)
                VALUES ($1, $2, $3, $4, 'email', NOW())
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(user.id)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password_hash)
            .execute(&self.pool)
            .await?;

            // Update demographics if present (demographics are columns on users table)
            if user.gender.is_some() || user.birth_year.is_some() {
                sqlx::query(
                    r#"
                    UPDATE users SET
                        gender = COALESCE($2::gender, gender),
                        birth_year = COALESCE($3, birth_year),
                        weight_kg = COALESCE($4, weight_kg),
                        country = COALESCE($5, country),
                        region = COALESCE($6, region)
                    WHERE id = $1
                    "#,
                )
                .bind(user.id)
                .bind(user.gender.as_ref().map(gender_to_db_str))
                .bind(user.birth_year)
                .bind(user.weight_kg)
                .bind(&user.country)
                .bind(&user.region)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Seeds activities with their track data.
    pub async fn seed_activities(&self, activities: &[GeneratedActivity]) -> Result<(), SeedError> {
        info!("Seeding {} activities...", activities.len());

        for (i, activity) in activities.iter().enumerate() {
            self.insert_activity(activity).await?;

            if (i + 1) % self.batch_size == 0 {
                info!("  Seeded {}/{} activities", i + 1, activities.len());
            }
        }

        info!("Seeded {} activities", activities.len());
        Ok(())
    }

    /// Inserts a single activity with its track geometry.
    async fn insert_activity(&self, activity: &GeneratedActivity) -> Result<(), SeedError> {
        if activity.track_points.len() < 2 {
            return Err(SeedError::NoTrackPoints);
        }

        // Insert activity record
        let activity_type_str = match activity.activity_type {
            ActivityType::Walking => "walking",
            ActivityType::Running => "running",
            ActivityType::Hiking => "hiking",
            ActivityType::RoadCycling => "road_cycling",
            ActivityType::MountainBiking => "mountain_biking",
            ActivityType::Unknown => "unknown",
        };

        sqlx::query(
            r#"
            INSERT INTO activities (id, user_id, activity_type, name, object_store_path, submitted_at, visibility)
            VALUES ($1, $2, $3::activity_type, $4, $5, $6, $7)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(activity.id)
        .bind(activity.user_id)
        .bind(activity_type_str)
        .bind(&activity.name)
        .bind(format!("generated/{}.gpx", activity.id))
        .bind(activity.submitted_at)
        .bind(activity.visibility.as_str())
        .execute(&self.pool)
        .await?;

        // Insert track geometry
        let coords: Vec<String> = activity
            .track_points
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
        .bind(activity.user_id)
        .bind(activity.id)
        .bind(&wkt)
        .execute(&self.pool)
        .await?;

        // Insert activity scores
        sqlx::query(
            r#"
            INSERT INTO scores (user_id, activity_id, distance, duration, elevation_gain, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(activity.user_id)
        .bind(activity.id)
        .bind(activity.distance_meters)
        .bind(activity.duration_seconds)
        .bind(activity.elevation_gain_meters)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Seeds segments into the database.
    pub async fn seed_segments(&self, segments: &[GeneratedSegment]) -> Result<(), SeedError> {
        info!("Seeding {} segments...", segments.len());

        for segment in segments {
            self.insert_segment(segment).await?;
        }

        info!("Seeded {} segments", segments.len());
        Ok(())
    }

    /// Inserts a single segment.
    async fn insert_segment(&self, segment: &GeneratedSegment) -> Result<(), SeedError> {
        let activity_type_str = match segment.activity_type {
            ActivityType::Walking => "walking",
            ActivityType::Running => "running",
            ActivityType::Hiking => "hiking",
            ActivityType::RoadCycling => "road_cycling",
            ActivityType::MountainBiking => "mountain_biking",
            ActivityType::Unknown => "unknown",
        };

        sqlx::query(
            r#"
            INSERT INTO segments (
                id, creator_id, name, description, activity_type,
                geo, start_point, end_point,
                distance_meters, elevation_gain_meters, elevation_loss_meters,
                average_grade, max_grade, climb_category,
                visibility, created_at
            )
            VALUES (
                $1, $2, $3, $4, $5::activity_type,
                ST_GeogFromText($6), ST_GeogFromText($7), ST_GeogFromText($8),
                $9, $10, $11,
                $12, $13, $14,
                $15, NOW()
            )
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(segment.id)
        .bind(segment.creator_id)
        .bind(&segment.name)
        .bind(&segment.description)
        .bind(activity_type_str)
        .bind(&segment.geo_wkt)
        .bind(&segment.start_wkt)
        .bind(&segment.end_wkt)
        .bind(segment.distance_meters)
        .bind(segment.elevation_gain_meters)
        .bind(segment.elevation_loss_meters)
        .bind(segment.average_grade)
        .bind(segment.max_grade)
        .bind(segment.climb_category)
        .bind(segment.visibility.as_str())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Seeds segment efforts.
    pub async fn seed_efforts(&self, efforts: &[GeneratedEffort]) -> Result<(), SeedError> {
        info!("Seeding {} efforts...", efforts.len());

        for chunk in efforts.chunks(self.batch_size) {
            self.insert_effort_batch(chunk).await?;
        }

        info!("Seeded {} efforts", efforts.len());
        Ok(())
    }

    /// Inserts a batch of efforts.
    async fn insert_effort_batch(&self, efforts: &[GeneratedEffort]) -> Result<(), SeedError> {
        for effort in efforts {
            sqlx::query(
                r#"
                INSERT INTO segment_efforts (
                    id, segment_id, activity_id, user_id,
                    started_at, elapsed_time_seconds, moving_time_seconds,
                    average_speed_mps, max_speed_mps, is_personal_record,
                    start_fraction, end_fraction, created_at
                )
                VALUES (
                    $1, $2, $3, $4,
                    $5, $6, $7,
                    $8, $9, false,
                    $10, $11, NOW()
                )
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(effort.id)
            .bind(effort.segment_id)
            .bind(effort.activity_id)
            .bind(effort.user_id)
            .bind(effort.started_at)
            .bind(effort.elapsed_time_seconds)
            .bind(effort.moving_time_seconds)
            .bind(effort.average_speed_mps)
            .bind(effort.max_speed_mps)
            .bind(effort.start_fraction)
            .bind(effort.end_fraction)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Seeds social follows.
    pub async fn seed_follows(&self, follows: &[GeneratedFollow]) -> Result<(), SeedError> {
        info!("Seeding {} follows...", follows.len());

        for chunk in follows.chunks(self.batch_size) {
            for follow in chunk {
                sqlx::query(
                    r#"
                    INSERT INTO follows (follower_id, following_id, created_at)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (follower_id, following_id) DO NOTHING
                    "#,
                )
                .bind(follow.follower_id)
                .bind(follow.following_id)
                .bind(follow.created_at)
                .execute(&self.pool)
                .await?;
            }
        }

        info!("Seeded {} follows", follows.len());
        Ok(())
    }

    /// Seeds kudos.
    pub async fn seed_kudos(&self, kudos: &[GeneratedKudos]) -> Result<(), SeedError> {
        info!("Seeding {} kudos...", kudos.len());

        for k in kudos {
            sqlx::query(
                r#"
                INSERT INTO kudos (user_id, activity_id, created_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, activity_id) DO NOTHING
                "#,
            )
            .bind(k.user_id)
            .bind(k.activity_id)
            .bind(k.created_at)
            .execute(&self.pool)
            .await?;
        }

        info!("Seeded {} kudos", kudos.len());
        Ok(())
    }

    /// Seeds comments.
    pub async fn seed_comments(&self, comments: &[GeneratedComment]) -> Result<(), SeedError> {
        info!("Seeding {} comments...", comments.len());

        for comment in comments {
            sqlx::query(
                r#"
                INSERT INTO comments (id, user_id, activity_id, parent_id, content, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(comment.id)
            .bind(comment.user_id)
            .bind(comment.activity_id)
            .bind(comment.parent_id)
            .bind(&comment.content)
            .bind(comment.created_at)
            .execute(&self.pool)
            .await?;
        }

        info!("Seeded {} comments", comments.len());
        Ok(())
    }

    /// Clears all seeded test data.
    ///
    /// **WARNING**: This deletes all data from the tables. Use with caution.
    pub async fn clear_all(&self) -> Result<(), SeedError> {
        info!("Clearing all seeded data...");

        // Order matters due to foreign key constraints
        sqlx::query("DELETE FROM comments")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM kudos").execute(&self.pool).await?;
        sqlx::query("DELETE FROM segment_efforts")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM segments")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM tracks")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM scores")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM activities")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM follows")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM notifications")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM users").execute(&self.pool).await?;

        info!("All data cleared");
        Ok(())
    }

    /// Returns a reference to the pool for advanced usage.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
