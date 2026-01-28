//! Example: Seed a social network with follows, kudos, and comments.
//!
//! This creates test data for verifying social features:
//! - 50 users with interconnected follow relationships
//! - Multiple activities per user
//! - Kudos and comments on activities
//!
//! Run with:
//! ```
//! cargo run --example seed_social
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

    // Build and seed the social scenario
    let mut rng = rand::thread_rng();

    let result = ScenarioBuilder::social_test()
        .with_seed(54321)
        .build(&pool, &mut rng)
        .await?;

    tracing::info!("Scenario seeded successfully!");
    tracing::info!("  Users: {}", result.users.len());
    tracing::info!("  Activities: {}", result.activities.len());
    tracing::info!("  Follows: {}", result.follows.len());
    tracing::info!("  Kudos: {}", result.kudos.len());
    tracing::info!("  Comments: {}", result.comments.len());

    // Calculate some social stats
    let avg_follows_per_user = result.follows.len() as f64 / result.users.len() as f64;
    let avg_kudos_per_activity = result.kudos.len() as f64 / result.activities.len() as f64;
    let avg_comments_per_activity = result.comments.len() as f64 / result.activities.len() as f64;

    tracing::info!("Social stats:");
    tracing::info!("  Avg follows per user: {:.1}", avg_follows_per_user);
    tracing::info!("  Avg kudos per activity: {:.1}", avg_kudos_per_activity);
    tracing::info!(
        "  Avg comments per activity: {:.1}",
        avg_comments_per_activity
    );

    // Count mutual follows
    let mut mutual_count = 0;
    for follow in &result.follows {
        if result
            .follows
            .iter()
            .any(|f| f.follower_id == follow.following_id && f.following_id == follow.follower_id)
        {
            mutual_count += 1;
        }
    }
    let mutual_rate = mutual_count as f64 / result.follows.len() as f64 * 100.0;
    tracing::info!("  Mutual follow rate: {:.1}%", mutual_rate);

    // Show some sample comments
    tracing::info!("Sample comments:");
    for comment in result.comments.iter().take(5) {
        tracing::info!("  \"{}\"", comment.content);
    }

    Ok(())
}
