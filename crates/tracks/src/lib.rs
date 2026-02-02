pub mod achievements_service;
pub mod activity_queue;
pub mod auth;
pub mod database;
pub mod errors;
pub mod file_parsers;
pub mod handlers;
pub mod models;
pub mod object_store_service;
pub mod query_builder;
pub mod request_id;
pub mod scoring;
pub mod segment_matching;
pub mod types;

use axum::{
    Extension, Router,
    http::{HeaderValue, Method, header},
    middleware,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

use crate::request_id::request_id_middleware;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    activity_queue::ActivityQueue,
    auth::{login, me, register},
    database::Database,
    handlers::{
        accept_invitation, add_comment, all_users, change_member_role, create_activity_type,
        create_dig_parts, create_segment, create_team, delete_activity, delete_comment,
        delete_dig_part, delete_team, discover_teams, download_gpx_file, follow_user,
        get_activities_by_date, get_activity, get_activity_segments, get_activity_sensor_data,
        get_activity_teams, get_activity_track, get_activity_type, get_average_speed_leaderboard,
        get_comments, get_countries, get_crown_leaderboard, get_dig_percentage_leaderboard,
        get_dig_parts, get_dig_time, get_dig_time_leaderboard, get_distance_leaderboard,
        get_feed, get_filtered_leaderboard, get_follow_status, get_followers, get_following,
        get_global_dig_heatmap, get_invitation, get_join_requests, get_kudos_givers,
        get_kudos_status, get_leaderboard_position, get_my_achievements, get_my_demographics,
        get_my_segment_efforts, get_nearby_segments, get_notifications, get_segment,
        get_segment_achievements, get_segment_leaderboard, get_segment_teams, get_segment_track,
        get_starred_segment_efforts, get_starred_segments, get_stats, get_stopped_segments,
        get_team, get_team_activities, get_team_activities_by_date, get_team_dig_heatmap,
        get_team_invitations, get_team_leaderboard, get_team_segments, get_user_achievements,
        get_user_activities, get_user_profile, give_kudos, health_check, invite_to_team,
        is_segment_starred, join_team, leave_team, list_activity_types, list_my_teams,
        list_segments, list_team_members, mark_all_notifications_read, mark_notification_read,
        new_activity, new_user, preview_activity, preview_segment, remove_kudos,
        remove_team_member, reprocess_dig_parts, reprocess_segment, resolve_activity_type,
        review_join_request, revoke_invitation, share_activity_with_teams,
        share_segment_with_teams, star_segment, unfollow_user, unshare_activity_from_team,
        unshare_segment_from_team, unstar_segment, update_activity, update_my_demographics,
        update_team,
    },
    object_store_service::ObjectStoreService,
};
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Track Leader API",
        description = "API for Track Leader - Activity tracking and segment leaderboards",
        version = "1.0.0",
        license(name = "MIT"),
    ),
    servers(
        (url = "http://localhost:8000", description = "Local development server"),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "activities", description = "Activity management endpoints"),
        (name = "segments", description = "Segment management endpoints"),
        (name = "leaderboards", description = "Leaderboard endpoints"),
        (name = "social", description = "Social features (follows, kudos, comments)"),
        (name = "notifications", description = "Notification endpoints"),
        (name = "teams", description = "Team management endpoints"),
        (name = "stats", description = "Platform statistics"),
    ),
    paths(
        // Auth
        auth::register,
        auth::login,
        auth::me,
        // Users
        handlers::new_user,
        handlers::all_users,
        // Activities
        handlers::new_activity,
        handlers::preview_activity,
        handlers::get_activity,
        handlers::update_activity,
        handlers::delete_activity,
        handlers::get_user_activities,
        handlers::download_gpx_file,
        handlers::get_activity_track,
        handlers::get_activity_segments,
        handlers::get_activities_by_date,
        handlers::get_stopped_segments,
        handlers::get_dig_parts,
        handlers::create_dig_parts,
        handlers::get_dig_time,
        handlers::delete_dig_part,
        handlers::reprocess_dig_parts,
        handlers::get_activity_sensor_data,
        // Activity types
        handlers::health_check,
        handlers::list_activity_types,
        handlers::get_activity_type,
        handlers::create_activity_type,
        handlers::resolve_activity_type,
        // Segments
        handlers::create_segment,
        handlers::get_segment,
        handlers::list_segments,
        handlers::get_segment_leaderboard,
        handlers::get_my_segment_efforts,
        handlers::get_segment_track,
        handlers::preview_segment,
        handlers::reprocess_segment,
        handlers::star_segment,
        handlers::unstar_segment,
        handlers::is_segment_starred,
        handlers::get_starred_segments,
        handlers::get_starred_segment_efforts,
        handlers::get_nearby_segments,
        handlers::get_filtered_leaderboard,
        handlers::get_leaderboard_position,
        // Demographics
        handlers::get_my_demographics,
        handlers::update_my_demographics,
        // Achievements
        handlers::get_user_achievements,
        handlers::get_my_achievements,
        handlers::get_segment_achievements,
        // Global leaderboards
        handlers::get_crown_leaderboard,
        handlers::get_distance_leaderboard,
        handlers::get_dig_time_leaderboard,
        handlers::get_dig_percentage_leaderboard,
        handlers::get_average_speed_leaderboard,
        handlers::get_countries,
        // Social
        handlers::follow_user,
        handlers::unfollow_user,
        handlers::get_follow_status,
        handlers::get_followers,
        handlers::get_following,
        handlers::get_user_profile,
        // Notifications
        handlers::get_notifications,
        handlers::mark_notification_read,
        handlers::mark_all_notifications_read,
        // Feed
        handlers::get_feed,
        // Kudos
        handlers::give_kudos,
        handlers::remove_kudos,
        handlers::get_kudos_status,
        handlers::get_kudos_givers,
        // Comments
        handlers::add_comment,
        handlers::get_comments,
        handlers::delete_comment,
        // Stats
        handlers::get_stats,
        // Teams
        handlers::create_team,
        handlers::get_team,
        handlers::list_my_teams,
        handlers::discover_teams,
        handlers::update_team,
        handlers::delete_team,
        handlers::list_team_members,
        handlers::remove_team_member,
        handlers::change_member_role,
        handlers::join_team,
        handlers::leave_team,
        handlers::get_join_requests,
        handlers::review_join_request,
        handlers::invite_to_team,
        handlers::get_team_invitations,
        handlers::revoke_invitation,
        handlers::get_invitation,
        handlers::accept_invitation,
        // Team sharing
        handlers::get_activity_teams,
        handlers::share_activity_with_teams,
        handlers::unshare_activity_from_team,
        handlers::get_segment_teams,
        handlers::share_segment_with_teams,
        handlers::unshare_segment_from_team,
        handlers::get_team_activities,
        handlers::get_team_activities_by_date,
        handlers::get_team_segments,
        handlers::get_team_leaderboard,
        // Dig heatmap
        handlers::get_team_dig_heatmap,
        handlers::get_global_dig_heatmap,
    ),
    components(
        schemas(
            // Auth types
            auth::RegisterRequest,
            auth::LoginRequest,
            auth::AuthResponse,
            auth::UserResponse,
            // Core models
            models::User,
            models::Activity,
            models::Segment,
            models::SegmentEffort,
            models::ActivityTypeRow,
            models::CreateActivityTypeRequest,
            // Visibility and enums
            models::Visibility,
            models::Gender,
            models::TeamRole,
            models::TeamVisibility,
            models::TeamJoinPolicy,
            // Leaderboard types
            models::LeaderboardScope,
            models::AgeGroup,
            models::GenderFilter,
            models::WeightClass,
            models::LeaderboardFilters,
            // Activity filter types
            models::DateRangeFilter,
            models::VisibilityFilter,
            models::ActivitySortBy,
            models::SortOrder,
            models::LeaderboardEntry,
            models::LeaderboardResponse,
            models::LeaderboardFiltersResponse,
            models::LeaderboardPosition,
            models::CountryStats,
            // Achievement types
            models::AchievementType,
            models::Achievement,
            models::AchievementWithSegment,
            models::AchievementHolder,
            models::SegmentAchievements,
            // User types
            models::UserWithDemographics,
            models::UpdateDemographicsRequest,
            models::UserProfile,
            models::UserSummary,
            // Global leaderboards
            models::CrownCountEntry,
            models::DistanceLeaderEntry,
            models::LeaderboardType,
            models::DigTimeLeaderEntry,
            models::DigPercentageLeaderEntry,
            models::AverageSpeedLeaderEntry,
            // Social types
            models::Follow,
            models::NotificationType,
            models::Notification,
            models::NotificationWithActor,
            models::NotificationsResponse,
            // Feed types
            models::FeedActivity,
            models::FeedActivityWithTeams,
            // Stopped/Dig segment types
            models::StoppedSegment,
            models::DigPart,
            models::CreateDigPartsRequest,
            models::DigTimeSummary,
            handlers::ReprocessDigPartsResult,
            // Sensor data types
            models::ActivitySensorDataResponse,
            // Kudos/Comments
            models::KudosGiver,
            models::Comment,
            models::CommentWithUser,
            // Stats
            models::Stats,
            // Segment types
            models::SegmentWithStats,
            models::ActivitySegmentEffort,
            models::StarredSegmentEffort,
            // Team types
            models::Team,
            models::TeamWithMembership,
            models::TeamMembership,
            models::TeamMember,
            models::TeamJoinRequest,
            models::TeamJoinRequestWithUser,
            models::TeamInvitation,
            models::TeamInvitationWithDetails,
            models::CreateTeamRequest,
            models::UpdateTeamRequest,
            models::InviteToTeamRequest,
            models::ChangeMemberRoleRequest,
            models::JoinTeamRequest,
            models::ShareWithTeamsRequest,
            models::TeamSummary,
            // Dig heatmap types
            models::DigHeatmapPoint,
            models::DigHeatmapBounds,
            models::DigHeatmapResponse,
            // Handler request/response types
            handlers::TrackPoint,
            handlers::TrackData,
            handlers::TrackBounds,
            handlers::PreviewTrackPoint,
            handlers::PreviewSportSegment,
            handlers::PreviewActivityResponse,
            handlers::UploadQuery,
            handlers::UpdateActivityRequest,
            handlers::UserActivitiesQuery,
            handlers::ActivitiesByDateQuery,
            handlers::ResolveTypeQuery,
            handlers::ResolveTypeResponse,
            handlers::CreateSegmentRequest,
            handlers::SegmentPoint,
            handlers::SegmentSortBy,
            handlers::SortOrder,
            handlers::ClimbCategoryFilter,
            handlers::ListSegmentsQuery,
            handlers::SegmentTrackData,
            handlers::SegmentTrackPoint,
            handlers::PreviewSegmentRequest,
            handlers::PreviewSegmentResponse,
            handlers::SegmentValidation,
            handlers::ReprocessResult,
            handlers::StarResponse,
            handlers::NearbySegmentsQuery,
            handlers::GetAchievementsQuery,
            handlers::GlobalLeaderboardQuery,
            handlers::FollowStatusResponse,
            handlers::FollowListQuery,
            handlers::FollowListResponse,
            handlers::NotificationsQuery,
            handlers::FeedQuery,
            handlers::KudosResponse,
            handlers::KudosStatusResponse,
            handlers::AddCommentRequest,
            handlers::DiscoverTeamsQuery,
            handlers::ReviewJoinRequestRequest,
            handlers::TeamContentQuery,
            handlers::TeamActivitiesByDateQuery,
            handlers::TeamLeaderboardQuery,
            handlers::TeamLeaderboardResponse,
            handlers::DigHeatmapQuery,
        )
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

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
        .route("/stats", get(get_stats))
        // Auth routes
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        // User routes
        .route("/users/new", get(new_user))
        .route("/users", get(all_users))
        // Activity routes
        .route("/activities/new", post(new_activity))
        .route("/activities/preview", post(preview_activity))
        .route("/activities/by-date", get(get_activities_by_date))
        .route(
            "/activities/{id}",
            get(get_activity)
                .patch(update_activity)
                .delete(delete_activity),
        )
        .route("/activities/{id}/track", get(get_activity_track))
        .route("/activities/{id}/segments", get(get_activity_segments))
        .route("/activities/{id}/download", get(download_gpx_file))
        .route(
            "/activities/{id}/stopped-segments",
            get(get_stopped_segments),
        )
        .route(
            "/activities/{id}/dig-parts",
            get(get_dig_parts).post(create_dig_parts),
        )
        .route("/activities/{id}/dig-time", get(get_dig_time))
        .route(
            "/activities/{id}/reprocess-dig-parts",
            post(reprocess_dig_parts),
        )
        .route(
            "/activities/{activity_id}/dig-parts/{segment_id}",
            axum::routing::delete(delete_dig_part),
        )
        .route(
            "/activities/{id}/sensor-data",
            get(get_activity_sensor_data),
        )
        .route("/users/{id}/activities", get(get_user_activities))
        // User demographics routes
        .route(
            "/users/me/demographics",
            get(get_my_demographics).patch(update_my_demographics),
        )
        // User achievements routes
        .route("/users/me/achievements", get(get_my_achievements))
        .route("/users/{id}/achievements", get(get_user_achievements))
        // Activity type routes
        .route(
            "/activity-types",
            get(list_activity_types).post(create_activity_type),
        )
        .route("/activity-types/resolve", get(resolve_activity_type))
        .route("/activity-types/{id}", get(get_activity_type))
        // Segment routes
        .route("/segments", get(list_segments).post(create_segment))
        .route("/segments/preview", post(preview_segment))
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
        .route("/leaderboards/dig-time", get(get_dig_time_leaderboard))
        .route(
            "/leaderboards/dig-percentage",
            get(get_dig_percentage_leaderboard),
        )
        .route(
            "/leaderboards/average-speed",
            get(get_average_speed_leaderboard),
        )
        .route("/leaderboards/countries", get(get_countries))
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
            get(get_kudos_status).post(give_kudos).delete(remove_kudos),
        )
        .route("/activities/{id}/kudos/givers", get(get_kudos_givers))
        // Comments routes
        .route(
            "/activities/{id}/comments",
            get(get_comments).post(add_comment),
        )
        .route("/comments/{id}", axum::routing::delete(delete_comment))
        // Team routes
        .route("/teams", get(list_my_teams).post(create_team))
        .route("/teams/discover", get(discover_teams))
        .route(
            "/teams/{id}",
            get(get_team).patch(update_team).delete(delete_team),
        )
        .route("/teams/{id}/members", get(list_team_members))
        .route(
            "/teams/{id}/members/{user_id}",
            axum::routing::delete(remove_team_member),
        )
        .route(
            "/teams/{id}/members/{user_id}/role",
            axum::routing::patch(change_member_role),
        )
        .route("/teams/{id}/join", post(join_team))
        .route("/teams/{id}/leave", post(leave_team))
        .route("/teams/{id}/join-requests", get(get_join_requests))
        .route(
            "/teams/{id}/join-requests/{request_id}",
            post(review_join_request),
        )
        .route(
            "/teams/{id}/invitations",
            get(get_team_invitations).post(invite_to_team),
        )
        .route(
            "/teams/{id}/invitations/{invitation_id}",
            axum::routing::delete(revoke_invitation),
        )
        .route("/teams/{id}/activities", get(get_team_activities))
        .route(
            "/teams/{id}/activities/daily",
            get(get_team_activities_by_date),
        )
        .route("/teams/{id}/segments", get(get_team_segments))
        .route(
            "/teams/{id}/leaderboard/{leaderboard_type}",
            get(get_team_leaderboard),
        )
        .route("/teams/{id}/dig-heatmap", get(get_team_dig_heatmap))
        // Global dig heatmap (public route)
        .route("/dig-heatmap", get(get_global_dig_heatmap))
        // Invitation acceptance (public route, token-based)
        .route("/invitations/{token}", get(get_invitation))
        .route("/invitations/{token}/accept", post(accept_invitation))
        // Activity-team sharing
        .route(
            "/activities/{id}/teams",
            get(get_activity_teams).post(share_activity_with_teams),
        )
        .route(
            "/activities/{id}/teams/{team_id}",
            axum::routing::delete(unshare_activity_from_team),
        )
        // Segment-team sharing
        .route(
            "/segments/{id}/teams",
            get(get_segment_teams).post(share_segment_with_teams),
        )
        .route(
            "/segments/{id}/teams/{team_id}",
            axum::routing::delete(unshare_segment_from_team),
        )
        // OpenAPI / Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(Extension(db))
        .layer(Extension(store))
        .layer(Extension(aq))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(request_id_middleware))
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
}

pub async fn run_server(pool: PgPool, object_store_path: String, port: u16) -> anyhow::Result<()> {
    // Create core components
    let db = Database::new(pool.clone());
    let aq = ActivityQueue::new(db.clone());
    let store = ObjectStoreService::new_local(object_store_path.clone());

    // Recover orphaned activities (uploaded but not processed due to restart)
    match db.find_orphaned_activities().await {
        Ok(orphaned) if !orphaned.is_empty() => {
            tracing::info!(
                count = orphaned.len(),
                "Found orphaned activities, reprocessing"
            );
            for activity in orphaned {
                if let Err(e) = aq.reprocess_orphaned(activity, &store).await {
                    tracing::error!("Failed to reprocess orphaned activity: {e}");
                }
            }
        }
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Failed to check for orphaned activities: {e}");
        }
    }

    let app = create_router(pool, object_store_path);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("Server running on http://0.0.0.0:{}", port);
    println!(
        "Swagger UI available at http://0.0.0.0:{}/swagger-ui/",
        port
    );

    axum::serve(listener, app).await?;

    Ok(())
}
