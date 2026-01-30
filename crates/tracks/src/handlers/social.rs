//! Social feature handlers: follows, notifications, feed, kudos, and comments.

use axum::{
    Extension,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{auth::AuthUser, database::Database, errors::AppError, models::DateRangeFilter};

use super::pagination::default_limit;

// ============================================================================
// Follow Types
// ============================================================================

/// Response for follow status check.
#[derive(Debug, Serialize, ToSchema)]
pub struct FollowStatusResponse {
    pub is_following: bool,
}

/// Query parameters for follow list endpoints.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FollowListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Response for follower/following lists.
#[derive(Debug, Serialize, ToSchema)]
pub struct FollowListResponse {
    pub users: Vec<crate::models::UserSummary>,
    pub total_count: i32,
}

// ============================================================================
// Follow Handlers
// ============================================================================

/// Follow a user.
#[utoipa::path(
    post,
    path = "/users/{id}/follow",
    tag = "social",
    params(("id" = Uuid, Path, description = "User ID to follow")),
    responses(
        (status = 201, description = "Successfully followed user"),
        (status = 200, description = "Already following (idempotent)"),
        (status = 400, description = "Cannot follow yourself"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn follow_user(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Can't follow yourself
    if claims.sub == user_id {
        return Err(AppError::InvalidInput("Cannot follow yourself".to_string()));
    }

    // Verify target user exists
    db.get_user(user_id).await?.ok_or(AppError::NotFound)?;

    // Check if already following
    if db.is_following(claims.sub, user_id).await? {
        return Ok(StatusCode::OK); // Idempotent
    }

    db.follow_user(claims.sub, user_id).await?;

    // Create notification for the followed user
    db.create_notification(
        user_id,
        "follow",
        Some(claims.sub),
        Some("user"),
        Some(claims.sub),
        None,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

/// Unfollow a user.
#[utoipa::path(
    delete,
    path = "/users/{id}/follow",
    tag = "social",
    params(("id" = Uuid, Path, description = "User ID to unfollow")),
    responses(
        (status = 204, description = "Successfully unfollowed user"),
        (status = 404, description = "Was not following this user"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn unfollow_user(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let unfollowed = db.unfollow_user(claims.sub, user_id).await?;

    if unfollowed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

/// Check if the authenticated user is following a specific user.
#[utoipa::path(
    get,
    path = "/users/{id}/follow/status",
    tag = "social",
    params(("id" = Uuid, Path, description = "User ID to check follow status for")),
    responses(
        (status = 200, description = "Follow status retrieved", body = FollowStatusResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_follow_status(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<FollowStatusResponse>, AppError> {
    let is_following = db.is_following(claims.sub, user_id).await?;
    Ok(Json(FollowStatusResponse { is_following }))
}

/// Get a user's followers.
#[utoipa::path(
    get,
    path = "/users/{id}/followers",
    tag = "social",
    params(
        ("id" = Uuid, Path, description = "User ID to get followers for"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "Followers retrieved", body = FollowListResponse)
    )
)]
pub async fn get_followers(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<FollowListQuery>,
) -> Result<Json<FollowListResponse>, AppError> {
    // Get follow counts
    let (follower_count, _) = db.get_follow_counts(user_id).await?;

    let followers = db.get_followers(user_id, query.limit, query.offset).await?;

    Ok(Json(FollowListResponse {
        users: followers,
        total_count: follower_count,
    }))
}

/// Get users that a user is following.
#[utoipa::path(
    get,
    path = "/users/{id}/following",
    tag = "social",
    params(
        ("id" = Uuid, Path, description = "User ID to get following list for"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "Following list retrieved", body = FollowListResponse)
    )
)]
pub async fn get_following(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<FollowListQuery>,
) -> Result<Json<FollowListResponse>, AppError> {
    // Get follow counts
    let (_, following_count) = db.get_follow_counts(user_id).await?;

    let following = db.get_following(user_id, query.limit, query.offset).await?;

    Ok(Json(FollowListResponse {
        users: following,
        total_count: following_count,
    }))
}

/// Get a user's profile with follow counts.
#[utoipa::path(
    get,
    path = "/users/{id}/profile",
    tag = "social",
    params(("id" = Uuid, Path, description = "User ID to get profile for")),
    responses(
        (status = 200, description = "User profile retrieved", body = crate::models::UserProfile),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user_profile(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<crate::models::UserProfile>, AppError> {
    let profile = db
        .get_user_profile(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(profile))
}

// ============================================================================
// Notification Types
// ============================================================================

/// Query parameters for notifications.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NotificationsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

// ============================================================================
// Notification Handlers
// ============================================================================

/// Get notifications for the authenticated user.
#[utoipa::path(
    get,
    path = "/notifications",
    tag = "notifications",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of notifications to return"),
        ("offset" = Option<i64>, Query, description = "Number of notifications to skip")
    ),
    responses(
        (status = 200, description = "List of notifications", body = crate::models::NotificationsResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_notifications(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<crate::models::NotificationsResponse>, AppError> {
    let notifications = db
        .get_notifications(claims.sub, query.limit, query.offset)
        .await?;
    let unread_count = db.get_unread_notification_count(claims.sub).await?;
    let total_count = notifications.len() as i64;

    Ok(Json(crate::models::NotificationsResponse {
        notifications,
        unread_count,
        total_count,
    }))
}

/// Mark a notification as read.
#[utoipa::path(
    put,
    path = "/notifications/{id}/read",
    tag = "notifications",
    params(
        ("id" = Uuid, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found")
    )
)]
pub async fn mark_notification_read(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let marked = db
        .mark_notification_read(notification_id, claims.sub)
        .await?;

    if marked {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

/// Mark all notifications as read.
#[utoipa::path(
    put,
    path = "/notifications/read-all",
    tag = "notifications",
    responses(
        (status = 200, description = "All notifications marked as read", body = Value),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn mark_all_notifications_read(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = db.mark_all_notifications_read(claims.sub).await?;
    Ok(Json(serde_json::json!({ "marked_count": count })))
}

// ============================================================================
// Feed Types
// ============================================================================

/// Query parameters for activity feed with filtering support.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct FeedQuery {
    /// Maximum number of feed items to return
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of feed items to skip
    #[serde(default)]
    pub offset: i64,
    /// Filter by activity type
    pub activity_type_id: Option<Uuid>,
    /// Preset date range filter (defaults to All)
    #[serde(default)]
    pub date_range: DateRangeFilter,
    /// Start date for custom date range (requires date_range=custom)
    pub start_date: Option<time::Date>,
    /// End date for custom date range (requires date_range=custom)
    pub end_date: Option<time::Date>,
}

// ============================================================================
// Feed Handlers
// ============================================================================

/// Get the activity feed for the authenticated user.
/// Returns activities from users they follow, with optional filtering.
#[utoipa::path(
    get,
    path = "/feed",
    tag = "feed",
    params(FeedQuery),
    responses(
        (status = 200, description = "Activity feed", body = Vec<crate::models::FeedActivity>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_feed(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Query(query): Query<FeedQuery>,
) -> Result<Json<Vec<crate::models::FeedActivity>>, AppError> {
    let activities = db
        .get_activity_feed_filtered(
            claims.sub,
            query.activity_type_id,
            query.date_range,
            query.start_date,
            query.end_date,
            query.limit,
            query.offset,
        )
        .await?;
    Ok(Json(activities))
}

// ============================================================================
// Kudos Types
// ============================================================================

/// Response for kudos operations.
#[derive(Debug, Serialize, ToSchema)]
pub struct KudosResponse {
    pub given: bool,
    pub kudos_count: i32,
}

/// Response for kudos status check.
#[derive(Debug, Serialize, ToSchema)]
pub struct KudosStatusResponse {
    pub has_given: bool,
}

// ============================================================================
// Kudos Handlers
// ============================================================================

/// Give kudos to an activity.
#[utoipa::path(
    post,
    path = "/activities/{id}/kudos",
    tag = "kudos",
    params(("id" = Uuid, Path, description = "Activity ID")),
    responses(
        (status = 200, description = "Kudos given successfully", body = KudosResponse),
        (status = 400, description = "Cannot give kudos to own activity"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn give_kudos(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<KudosResponse>, AppError> {
    // Get activity to check it exists and get owner
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Can't give kudos to your own activity
    if activity.user_id == claims.sub {
        return Err(AppError::InvalidInput(
            "Cannot give kudos to your own activity".to_string(),
        ));
    }

    let was_new = db.give_kudos(claims.sub, activity_id).await?;

    // Create notification if this is a new kudos
    if was_new {
        db.create_notification(
            activity.user_id,
            "kudos",
            Some(claims.sub),
            Some("activity"),
            Some(activity_id),
            None,
        )
        .await?;
    }

    // Get updated count
    let _activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(KudosResponse {
        given: true,
        kudos_count: 0, // We don't have kudos_count in Activity struct yet
    }))
}

/// Remove kudos from an activity.
#[utoipa::path(
    delete,
    path = "/activities/{id}/kudos",
    tag = "kudos",
    params(("id" = Uuid, Path, description = "Activity ID")),
    responses(
        (status = 204, description = "Kudos removed successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn remove_kudos(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    db.remove_kudos(claims.sub, activity_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Check if user has given kudos to an activity.
#[utoipa::path(
    get,
    path = "/activities/{id}/kudos/status",
    tag = "kudos",
    params(("id" = Uuid, Path, description = "Activity ID")),
    responses(
        (status = 200, description = "Kudos status retrieved", body = KudosStatusResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_kudos_status(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<KudosStatusResponse>, AppError> {
    let has_given = db.has_given_kudos(claims.sub, activity_id).await?;
    Ok(Json(KudosStatusResponse { has_given }))
}

/// Get users who gave kudos to an activity.
#[utoipa::path(
    get,
    path = "/activities/{id}/kudos",
    tag = "kudos",
    params(("id" = Uuid, Path, description = "Activity ID")),
    responses(
        (status = 200, description = "List of users who gave kudos", body = Vec<crate::models::KudosGiver>)
    )
)]
pub async fn get_kudos_givers(
    Extension(db): Extension<Database>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::KudosGiver>>, AppError> {
    let givers = db.get_kudos_givers(activity_id, 100).await?;
    Ok(Json(givers))
}

// ============================================================================
// Comment Types
// ============================================================================

/// Request to add a comment.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCommentRequest {
    pub content: String,
    pub parent_id: Option<Uuid>,
}

// ============================================================================
// Comment Handlers
// ============================================================================

/// Add a comment to an activity.
#[utoipa::path(
    post,
    path = "/activities/{id}/comments",
    tag = "comments",
    params(("id" = Uuid, Path, description = "Activity ID")),
    request_body = AddCommentRequest,
    responses(
        (status = 200, description = "Comment added successfully", body = crate::models::CommentWithUser),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn add_comment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
    Json(req): Json<AddCommentRequest>,
) -> Result<Json<crate::models::CommentWithUser>, AppError> {
    // Verify activity exists
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let comment = db
        .add_comment(claims.sub, activity_id, &req.content, req.parent_id)
        .await?;

    // Get user name for response
    let user = db.get_user(claims.sub).await?.ok_or(AppError::NotFound)?;

    // Create notification if commenting on someone else's activity
    if activity.user_id != claims.sub {
        db.create_notification(
            activity.user_id,
            "comment",
            Some(claims.sub),
            Some("activity"),
            Some(activity_id),
            Some(&req.content),
        )
        .await?;
    }

    Ok(Json(crate::models::CommentWithUser {
        id: comment.id,
        user_id: comment.user_id,
        activity_id: comment.activity_id,
        parent_id: comment.parent_id,
        content: comment.content,
        created_at: comment.created_at,
        updated_at: comment.updated_at,
        user_name: user.name,
    }))
}

/// Get comments for an activity.
#[utoipa::path(
    get,
    path = "/activities/{id}/comments",
    tag = "comments",
    params(("id" = Uuid, Path, description = "Activity ID")),
    responses(
        (status = 200, description = "List of comments for the activity", body = Vec<crate::models::CommentWithUser>)
    )
)]
pub async fn get_comments(
    Extension(db): Extension<Database>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::CommentWithUser>>, AppError> {
    let comments = db.get_comments(activity_id).await?;
    Ok(Json(comments))
}

/// Delete a comment.
#[utoipa::path(
    delete,
    path = "/comments/{id}",
    tag = "comments",
    params(("id" = Uuid, Path, description = "Comment ID")),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Comment not found or not owned by user")
    )
)]
pub async fn delete_comment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(comment_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = db.delete_comment(comment_id, claims.sub).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
