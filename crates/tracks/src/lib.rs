pub mod achievements_service;
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
        add_comment, all_users, create_segment, delete_activity, delete_comment,
        download_gpx_file, follow_user, get_activity, get_activity_segments, get_activity_track,
        get_comments, get_crown_leaderboard, get_distance_leaderboard, get_feed,
        get_filtered_leaderboard, get_follow_status, get_followers, get_following,
        get_kudos_givers, get_kudos_status, get_leaderboard_position, get_my_achievements,
        get_my_demographics, get_my_segment_efforts, get_nearby_segments, get_notifications,
        get_segment, get_segment_achievements, get_segment_leaderboard, get_segment_track,
        get_starred_segment_efforts, get_starred_segments, get_user_achievements,
        get_user_activities, get_user_profile, give_kudos, health_check, is_segment_starred,
        list_segments, mark_all_notifications_read, mark_notification_read, new_activity, new_user,
        remove_kudos, reprocess_segment, star_segment, unfollow_user, unstar_segment,
        update_activity, update_my_demographics,
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
        // User demographics routes
        .route(
            "/users/me/demographics",
            get(get_my_demographics).patch(update_my_demographics),
        )
        // User achievements routes
        .route("/users/me/achievements", get(get_my_achievements))
        .route("/users/{id}/achievements", get(get_user_achievements))
        // Segment routes
        .route("/segments", get(list_segments).post(create_segment))
        .route("/segments/nearby", get(get_nearby_segments))
        .route("/segments/{id}", get(get_segment))
        .route("/segments/{id}/track", get(get_segment_track))
        .route("/segments/{id}/leaderboard", get(get_segment_leaderboard))
        .route(
            "/segments/{id}/leaderboard/filtered",
            get(get_filtered_leaderboard),
        )
        .route(
            "/segments/{id}/leaderboard/position",
            get(get_leaderboard_position),
        )
        .route("/segments/{id}/achievements", get(get_segment_achievements))
        .route("/segments/{id}/my-efforts", get(get_my_segment_efforts))
        .route("/segments/{id}/reprocess", post(reprocess_segment))
        .route(
            "/segments/{id}/star",
            get(is_segment_starred)
                .post(star_segment)
                .delete(unstar_segment),
        )
        .route("/segments/starred", get(get_starred_segments))
        .route(
            "/segments/starred/efforts",
            get(get_starred_segment_efforts),
        )
        // Global leaderboards
        .route("/leaderboards/crowns", get(get_crown_leaderboard))
        .route("/leaderboards/distance", get(get_distance_leaderboard))
        // Social routes (follows)
        .route("/users/{id}/profile", get(get_user_profile))
        .route(
            "/users/{id}/follow",
            get(get_follow_status)
                .post(follow_user)
                .delete(unfollow_user),
        )
        .route("/users/{id}/followers", get(get_followers))
        .route("/users/{id}/following", get(get_following))
        // Notification routes
        .route("/notifications", get(get_notifications))
        .route("/notifications/{id}/read", post(mark_notification_read))
        .route("/notifications/read-all", post(mark_all_notifications_read))
        // Activity feed
        .route("/feed", get(get_feed))
        // Kudos routes
        .route(
            "/activities/{id}/kudos",
            get(get_kudos_status)
                .post(give_kudos)
                .delete(remove_kudos),
        )
        .route("/activities/{id}/kudos/givers", get(get_kudos_givers))
        // Comments routes
        .route(
            "/activities/{id}/comments",
            get(get_comments).post(add_comment),
        )
        .route("/comments/{id}", axum::routing::delete(delete_comment))
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
