//! Example: Seed a leaderboard with 200 users competing on a segment.
//!
//! This creates realistic test data for verifying leaderboard functionality:
//! - 200 users with varied performance levels
//! - A single segment with all users having 1-3 efforts
//! - Power-law time distribution (few fast, many average)
//!
//! Run with:
//! ```
//! cargo run --example seed_leaderboard
//! ```

use sqlx::postgres::PgPoolOptions;
use test_data::builders::ScenarioBuilder;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tracks".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to database");

    // Build and seed the leaderboard scenario
    let mut rng = rand::thread_rng();

    let result = ScenarioBuilder::leaderboard_test()
        .with_seed(12345)
        .build(&pool, &mut rng)
        .await?;

    tracing::info!("Scenario seeded successfully!");
    tracing::info!("  Users: {}", result.users.len());
    tracing::info!("  Activities: {}", result.activities.len());
    tracing::info!("  Segments: {}", result.segments.len());
    tracing::info!("  Efforts: {}", result.efforts.len());

    // Print segment info
    for segment in &result.segments {
        tracing::info!(
            "  Segment '{}': {:.0}m, {:?} grade",
            segment.name,
            segment.distance_meters,
            segment.average_grade.map(|g| format!("{:.1}%", g * 100.0))
        );
    }

    // Print some leaderboard stats
    let mut effort_times: Vec<f64> = result
        .efforts
        .iter()
        .map(|e| e.elapsed_time_seconds)
        .collect();
    effort_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if !effort_times.is_empty() {
        let fastest = effort_times[0];
        let median = effort_times[effort_times.len() / 2];
        let slowest = effort_times[effort_times.len() - 1];

        tracing::info!("Effort times:");
        tracing::info!("  Fastest: {:.1}s", fastest);
        tracing::info!("  Median:  {:.1}s", median);
        tracing::info!("  Slowest: {:.1}s", slowest);
    }

    Ok(())
}
