pub mod database;
pub mod errors;
pub mod gpx_processor;
pub mod handlers;
pub mod models;
pub mod object_store_service;

use axum::{
    http::Method,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    database::Database,
    handlers::{download_gpx_file, get_activity, get_user_activities, health_check, upload_gpx},
    object_store_service::ObjectStoreService,
};

pub fn create_router(pool: PgPool, object_store_path: String) -> Router {
    let db = Database::new(pool);
    let store = ObjectStoreService::new_local(object_store_path);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/activities/upload", post(upload_gpx))
        .route("/activities/:id", get(get_activity))
        .route("/activities/:id/download", get(download_gpx_file))
        .route("/activities", get(get_user_activities))
        .layer(Extension(db))
        .layer(Extension(store))
        .layer(cors)
}

pub async fn run_server(pool: PgPool, object_store_path: String, port: u16) -> anyhow::Result<()> {
    let app = create_router(pool, object_store_path);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("Server running on http://0.0.0.0:{}", port);

    axum::Server::from_tcp(listener.into_std()?)
        .unwrap()
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
