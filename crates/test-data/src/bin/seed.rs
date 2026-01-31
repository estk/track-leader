//! Default seed script - creates comprehensive test data
//!
//! Run with:
//! ```
//! cargo run -p test-data --bin seed
//! ```

use sqlx::postgres::PgPoolOptions;
use test_data::builders::ScenarioBuilder;
use tracing_subscriber::EnvFilter;

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

    let mut rng = rand::thread_rng();

    let result = ScenarioBuilder::comprehensive_test()
        .with_seed(12345) // Reproducible data
        .build(&pool, &mut rng)
        .await?;

    // Summary output
    tracing::info!("Seed completed!");
    tracing::info!("  Users: {}", result.users.len());
    tracing::info!("  Activities: {}", result.activities.len());
    tracing::info!("  Segments: {}", result.segments.len());
    tracing::info!("  Efforts: {}", result.efforts.len());
    tracing::info!("  Follows: {}", result.follows.len());
    tracing::info!("  Kudos: {}", result.kudos.len());
    tracing::info!("  Comments: {}", result.comments.len());

    Ok(())
}
