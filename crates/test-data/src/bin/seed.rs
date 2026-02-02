//! Default seed script - creates comprehensive test data
//!
//! Run with:
//! ```
//! cargo run -p test-data --bin seed
//! ```
//!
//! Creates test users for E2E testing:
//! - test.user1@example.com (password: "password")
//! - test.user2@example.com (password: "password")
//! - test.user3@example.com (password: "password")
//!
//! Activities are uploaded via the API to ensure DIG part extraction
//! occurs through the production code path.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::path::Path;
use test_data::api::{ApiSeeder, read_backend_url_from_dev_ports};
use test_data::builders::ScenarioBuilder;
use test_data::db::Seeder;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

/// Read the postgres port from .dev-ports file if it exists.
/// Returns None if the file doesn't exist or can't be parsed.
fn read_dev_ports_postgres_port() -> Option<u16> {
    let dev_ports_path = Path::new(".dev-ports");
    if !dev_ports_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(dev_ports_path).ok()?;
    for line in contents.lines() {
        if let Some(port_str) = line.strip_prefix("POSTGRES_PORT=") {
            return port_str.trim().parse().ok();
        }
    }
    None
}

/// Get the database URL, checking .dev-ports file for the postgres port.
fn get_database_url() -> String {
    // First check if DATABASE_URL is explicitly set
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return url;
    }

    // Try to read port from .dev-ports file
    let port = read_dev_ports_postgres_port().unwrap_or(5432);

    format!(
        "postgres://tracks_user:tracks_password@localhost:{}/tracks_db",
        port
    )
}

/// Known E2E test users with predictable credentials.
/// All users have password "password".
const E2E_TEST_USERS: &[(&str, &str, &str)] = &[
    (
        "00000000-0000-0000-0000-000000000e01",
        "test.user1@example.com",
        "Test User One",
    ),
    (
        "00000000-0000-0000-0000-000000000e02",
        "test.user2@example.com",
        "Test User Two",
    ),
    (
        "00000000-0000-0000-0000-000000000e03",
        "test.user3@example.com",
        "Test User Three",
    ),
];

/// Creates known E2E test users with predictable credentials.
async fn seed_e2e_test_users(pool: &PgPool) -> anyhow::Result<()> {
    tracing::info!("Creating E2E test users...");

    let password_hash = tracks::auth::hash_password("password")?;

    for (id_str, email, name) in E2E_TEST_USERS {
        let id = Uuid::parse_str(id_str)?;
        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password_hash, auth_provider, created_at)
            VALUES ($1, $2, $3, $4, 'email', NOW())
            ON CONFLICT (email) DO UPDATE SET
                name = EXCLUDED.name,
                password_hash = EXCLUDED.password_hash
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind(&password_hash)
        .execute(pool)
        .await?;

        tracing::info!("  Created/updated: {email}");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = get_database_url();
    tracing::info!(
        "Using database URL: {}",
        database_url.replace("tracks_password", "***")
    );

    // Check for backend URL - required for API uploads
    let backend_url = read_backend_url_from_dev_ports()
        .or_else(|| std::env::var("BACKEND_URL").ok())
        .unwrap_or_else(|| "http://localhost:3001".to_string());

    tracing::info!("Using backend URL: {}", backend_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to database");

    // Create API seeder and verify backend is reachable
    let api_seeder = ApiSeeder::new(&backend_url);
    match api_seeder.check_health().await {
        Ok(_) => tracing::info!("Backend health check passed"),
        Err(e) => {
            tracing::warn!(
                "Backend not reachable at {}: {}. Activities will be seeded directly to DB (no DIG extraction).",
                backend_url,
                e
            );
        }
    }

    // Create known E2E test users first
    seed_e2e_test_users(&pool).await?;

    let mut rng = rand::thread_rng();

    // Use dig_leaderboard_test for DIG-specific testing
    tracing::info!("Generating DIG test data...");
    let dig_result = ScenarioBuilder::dig_leaderboard_test()
        .with_seed(12345) // Reproducible data
        .with_teams(5) // Add some teams for team DIG features
        .build_data(&mut rng);

    // Also generate comprehensive test data
    tracing::info!("Generating comprehensive test data...");
    let comp_result = ScenarioBuilder::comprehensive_test()
        .with_seed(54321)
        .build_data(&mut rng);

    let seeder = Seeder::new(pool.clone());

    // Seed users from both scenarios
    tracing::info!("Seeding users...");
    seeder.seed_users(&dig_result.users).await?;
    seeder.seed_users(&comp_result.users).await?;

    // Seed teams and memberships
    tracing::info!("Seeding teams and memberships...");
    if !dig_result.teams.is_empty() {
        seeder.seed_teams(&dig_result.teams).await?;
        seeder
            .seed_team_memberships(&dig_result.team_memberships)
            .await?;
    }

    // Seed segments
    tracing::info!("Seeding segments...");
    seeder.seed_segments(&comp_result.segments).await?;

    // Upload activities via API if backend is available
    let backend_available = api_seeder.check_health().await.is_ok();

    let mut activity_id_map: HashMap<Uuid, Uuid> = HashMap::new();

    if backend_available {
        tracing::info!("Uploading activities via API (DIG extraction enabled)...");

        // Build user_id -> email mapping
        let user_emails: HashMap<Uuid, String> = dig_result
            .users
            .iter()
            .map(|u| (u.id, u.email.clone()))
            .chain(comp_result.users.iter().map(|u| (u.id, u.email.clone())))
            .collect();

        // Upload DIG activities
        let mut uploaded_dig = 0;
        for activity in &dig_result.activities {
            if let Some(email) = user_emails.get(&activity.user_id) {
                match api_seeder.login(email, "password").await {
                    Ok(token) => match api_seeder.upload_activity(&token, activity).await {
                        Ok(new_id) => {
                            activity_id_map.insert(activity.id, new_id);
                            uploaded_dig += 1;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to upload activity {}: {}", activity.name, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to login as {}: {}", email, e);
                    }
                }
            }
        }
        tracing::info!("  Uploaded {} DIG activities via API", uploaded_dig);

        // Upload comprehensive activities
        let mut uploaded_comp = 0;
        for activity in &comp_result.activities {
            if let Some(email) = user_emails.get(&activity.user_id) {
                match api_seeder.login(email, "password").await {
                    Ok(token) => match api_seeder.upload_activity(&token, activity).await {
                        Ok(new_id) => {
                            activity_id_map.insert(activity.id, new_id);
                            uploaded_comp += 1;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to upload activity {}: {}", activity.name, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to login as {}: {}", email, e);
                    }
                }
            }
        }
        tracing::info!(
            "  Uploaded {} comprehensive activities via API",
            uploaded_comp
        );
    } else {
        tracing::info!("Seeding activities directly to DB (no DIG extraction)...");
        seeder.seed_activities(&dig_result.activities).await?;
        seeder.seed_activities(&comp_result.activities).await?;

        // No ID mapping needed when seeding directly
        for activity in dig_result.activities.iter().chain(&comp_result.activities) {
            activity_id_map.insert(activity.id, activity.id);
        }
    }

    // Update effort activity IDs and seed
    tracing::info!("Seeding efforts...");
    let updated_efforts: Vec<_> = comp_result
        .efforts
        .into_iter()
        .filter_map(|mut effort| {
            if let Some(&new_id) = activity_id_map.get(&effort.activity_id) {
                effort.activity_id = new_id;
                Some(effort)
            } else {
                None
            }
        })
        .collect();
    seeder.seed_efforts(&updated_efforts).await?;

    // Seed achievements
    seeder
        .seed_achievements(&comp_result.segments, &updated_efforts, &comp_result.users)
        .await?;

    // Seed social interactions with updated activity IDs
    tracing::info!("Seeding social interactions...");
    let updated_kudos: Vec<_> = comp_result
        .kudos
        .into_iter()
        .filter_map(|mut k| {
            if let Some(&new_id) = activity_id_map.get(&k.activity_id) {
                k.activity_id = new_id;
                Some(k)
            } else {
                None
            }
        })
        .collect();
    seeder.seed_kudos(&updated_kudos).await?;

    let updated_comments: Vec<_> = comp_result
        .comments
        .into_iter()
        .filter_map(|mut c| {
            if let Some(&new_id) = activity_id_map.get(&c.activity_id) {
                c.activity_id = new_id;
                Some(c)
            } else {
                None
            }
        })
        .collect();
    seeder.seed_comments(&updated_comments).await?;

    seeder.seed_follows(&comp_result.follows).await?;

    // Seed activity-team and segment-team associations
    tracing::info!("Seeding team associations...");
    let updated_activity_teams: Vec<_> = dig_result
        .activity_teams
        .into_iter()
        .filter_map(|mut at| {
            if let Some(&new_id) = activity_id_map.get(&at.activity_id) {
                at.activity_id = new_id;
                Some(at)
            } else {
                None
            }
        })
        .collect();
    seeder.seed_activity_teams(&updated_activity_teams).await?;
    seeder.seed_segment_teams(&dig_result.segment_teams).await?;

    // Summary output
    tracing::info!("Seed completed!");
    tracing::info!("  E2E test users: {}", E2E_TEST_USERS.len());
    tracing::info!(
        "  Generated users: {} (DIG) + {} (comprehensive)",
        dig_result.users.len(),
        comp_result.users.len()
    );
    tracing::info!("  Activities: {} uploaded via API", activity_id_map.len());
    tracing::info!("  Segments: {}", comp_result.segments.len());
    tracing::info!("  Efforts: {}", updated_efforts.len());
    tracing::info!("  Follows: {}", comp_result.follows.len());
    tracing::info!("  Kudos: {}", updated_kudos.len());
    tracing::info!("  Comments: {}", updated_comments.len());
    tracing::info!("  Teams: {}", dig_result.teams.len());

    if backend_available {
        tracing::info!("");
        tracing::info!("DIG extraction was triggered via API uploads.");
        tracing::info!("Check activity_dig_parts table for extracted DIG segments.");
    }

    Ok(())
}
