use sqlx::PgPool;
use std::env;
use tracks::run_server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://docker:pg@0.0.0.0".to_string());

    tracing::info!("Connecting to database at {}", database_url);

    let pool = PgPool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let object_store_path =
        env::var("OBJECT_STORE_PATH").unwrap_or_else(|_| "./uploads".to_string());

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    run_server(pool, object_store_path, port).await
}
