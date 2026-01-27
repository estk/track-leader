pub mod activity_queue;
pub mod auth;
pub mod database;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod object_store_service;
pub mod scoring;
pub mod segment_matching;

use axum::{
    Extension, Router,
    http::Method,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    activity_queue::ActivityQueue,
    auth::{login, me, register},
    database::Database,
    handlers::{
        all_users, create_segment, delete_activity, download_gpx_file, get_activity,
        get_activity_segments, get_activity_track, get_segment, get_segment_leaderboard,
        get_segment_track, get_user_activities, health_check, list_segments, new_activity,
        new_user, reprocess_segment, update_activity,
    },
    object_store_service::ObjectStoreService,
};

pub fn create_router(pool: PgPool, object_store_path: String) -> Router {
    let db = Database::new(pool);
    let aq = ActivityQueue::new(db.clone());
    let store = ObjectStoreService::new_local(object_store_path);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health_check))
        // Auth routes
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        // User routes
        .route("/users/new", get(new_user))
        .route("/users", get(all_users))
        // Activity routes
        .route("/activities/new", post(new_activity))
        .route(
            "/activities/{id}",
            get(get_activity)
                .patch(update_activity)
                .delete(delete_activity),
        )
        .route("/activities/{id}/track", get(get_activity_track))
        .route("/activities/{id}/segments", get(get_activity_segments))
        .route("/activities/{id}/download", get(download_gpx_file))
        .route("/users/{id}/activities", get(get_user_activities))
        // Segment routes
        .route("/segments", get(list_segments).post(create_segment))
        .route("/segments/{id}", get(get_segment))
        .route("/segments/{id}/track", get(get_segment_track))
        .route("/segments/{id}/leaderboard", get(get_segment_leaderboard))
        .route("/segments/{id}/reprocess", post(reprocess_segment))
        .layer(Extension(db))
        .layer(Extension(store))
        .layer(Extension(aq))
        .layer(cors)
}

pub async fn run_server(pool: PgPool, object_store_path: String, port: u16) -> anyhow::Result<()> {
    let app = create_router(pool, object_store_path);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("Server running on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
