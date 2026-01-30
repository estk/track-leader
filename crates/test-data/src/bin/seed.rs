//! CLI tool for seeding test data.

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
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://tracks_user:tracks_password@localhost:5432/tracks_db".to_string()
    });

    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected! Seeding test data...");

    // Build and seed both scenarios
    let mut rng = rand::thread_rng();

    // Seed leaderboard scenario (200 users, segments, efforts)
    tracing::info!("Creating leaderboard test scenario...");
    let leaderboard_result = ScenarioBuilder::leaderboard_test()
        .with_seed(12345)
        .build(&pool, &mut rng)
        .await?;

    tracing::info!(
        "Leaderboard scenario: {} users, {} activities, {} segments, {} efforts",
        leaderboard_result.users.len(),
        leaderboard_result.activities.len(),
        leaderboard_result.segments.len(),
        leaderboard_result.efforts.len()
    );

    // Seed social scenario (50 more users with follows/kudos/comments)
    tracing::info!("Creating social test scenario...");
    let social_result = ScenarioBuilder::social_test()
        .with_seed(54321)
        .build(&pool, &mut rng)
        .await?;

    tracing::info!(
        "Social scenario: {} users, {} activities, {} follows, {} kudos, {} comments",
        social_result.users.len(),
        social_result.activities.len(),
        social_result.follows.len(),
        social_result.kudos.len(),
        social_result.comments.len()
    );

    // Seed team scenario (100 more users with teams and sharing)
    tracing::info!("Creating team test scenario...");
    let team_result = ScenarioBuilder::team_test()
        .with_seed(67890)
        .build(&pool, &mut rng)
        .await?;

    tracing::info!(
        "Team scenario: {} users, {} teams, {} memberships, {} activity-team shares, {} segment-team shares",
        team_result.users.len(),
        team_result.teams.len(),
        team_result.team_memberships.len(),
        team_result.activity_teams.len(),
        team_result.segment_teams.len()
    );

    tracing::info!("Done! Test data seeded successfully.");

    Ok(())
}
