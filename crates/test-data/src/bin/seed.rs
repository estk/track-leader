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

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use test_data::builders::ScenarioBuilder;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

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

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://tracks_user:tracks_password@localhost:5432/tracks_db".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to database");

    // Create known E2E test users first
    seed_e2e_test_users(&pool).await?;

    let mut rng = rand::thread_rng();

    let result = ScenarioBuilder::comprehensive_test()
        .with_seed(12345) // Reproducible data
        .build(&pool, &mut rng)
        .await?;

    // Summary output
    tracing::info!("Seed completed!");
    tracing::info!("  E2E test users: {}", E2E_TEST_USERS.len());
    tracing::info!("  Generated users: {}", result.users.len());
    tracing::info!("  Activities: {}", result.activities.len());
    tracing::info!("  Segments: {}", result.segments.len());
    tracing::info!("  Efforts: {}", result.efforts.len());
    tracing::info!("  Follows: {}", result.follows.len());
    tracing::info!("  Kudos: {}", result.kudos.len());
    tracing::info!("  Comments: {}", result.comments.len());

    Ok(())
}
