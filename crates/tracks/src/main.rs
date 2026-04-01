use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracks::object_store_service::ObjectStoreService;
use tracks::run_server;

fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://docker:pg@0.0.0.0".to_string());

    tracing::info!("Connecting to database at {database_url}");

    // Configure connection pool for production workloads
    let pool = PgPoolOptions::new()
        .max_connections(
            env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        )
        .min_connections(
            env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
        )
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let store = match env::var("OBJECT_STORE_BACKEND").as_deref() {
        Ok("s3") => {
            let bucket =
                env::var("AWS_S3_BUCKET").expect("AWS_S3_BUCKET is required when using S3");
            let region = env::var("AWS_REGION").expect("AWS_REGION is required when using S3");
            let prefix = env::var("AWS_S3_PREFIX").ok();
            tracing::info!(%bucket, %region, prefix = prefix.as_deref().unwrap_or(""), "Using S3 object store");
            ObjectStoreService::new_s3(&bucket, &region, prefix.as_deref())
        }
        _ => {
            let path = env::var("OBJECT_STORE_PATH").unwrap_or_else(|_| "./uploads".to_string());
            tracing::info!(%path, "Using local object store");
            ObjectStoreService::new_local(path)
        }
    };

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    run_server(pool, store, port).await
}
