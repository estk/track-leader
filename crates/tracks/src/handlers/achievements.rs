//! Achievement handlers.

use axum::{
    Extension,
    extract::{Path, Query},
    response::Json,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    database::Database,
    errors::AppError,
    models::{AchievementType, AchievementWithSegment, SegmentAchievements},
};

/// Query parameters for achievements endpoints.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetAchievementsQuery {
    #[serde(default)]
    pub include_lost: bool,
}

/// Get achievements for a specific user.
#[utoipa::path(
    get,
    path = "/users/{id}/achievements",
    tag = "achievements",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        GetAchievementsQuery
    ),
    responses(
        (status = 200, description = "User achievements", body = Vec<AchievementWithSegment>),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user_achievements(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<GetAchievementsQuery>,
) -> Result<Json<Vec<AchievementWithSegment>>, AppError> {
    let achievements = db
        .get_user_achievements(user_id, query.include_lost)
        .await?;
    Ok(Json(achievements))
}

/// Get the authenticated user's achievements.
#[utoipa::path(
    get,
    path = "/me/achievements",
    tag = "achievements",
    params(GetAchievementsQuery),
    responses(
        (status = 200, description = "User achievements", body = Vec<AchievementWithSegment>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_achievements(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Query(query): Query<GetAchievementsQuery>,
) -> Result<Json<Vec<AchievementWithSegment>>, AppError> {
    let achievements = db
        .get_user_achievements(claims.sub, query.include_lost)
        .await?;
    Ok(Json(achievements))
}

/// Get current achievement holders for a segment.
#[utoipa::path(
    get,
    path = "/segments/{id}/achievements",
    tag = "achievements",
    params(("id" = Uuid, Path, description = "Segment ID")),
    responses(
        (status = 200, description = "Segment achievement holders", body = SegmentAchievements),
        (status = 404, description = "Segment not found")
    )
)]
pub async fn get_segment_achievements(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<SegmentAchievements>, AppError> {
    // Verify segment exists
    db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    let kom = db
        .get_current_achievement_holder(id, AchievementType::Kom)
        .await?;
    let qom = db
        .get_current_achievement_holder(id, AchievementType::Qom)
        .await?;
    let local_legend = db
        .get_current_achievement_holder(id, AchievementType::LocalLegend)
        .await?;

    Ok(Json(SegmentAchievements {
        segment_id: id,
        kom,
        qom,
        local_legend,
    }))
}
