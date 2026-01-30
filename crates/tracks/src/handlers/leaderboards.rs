//! Global leaderboard handlers.

use axum::{Extension, extract::Query, response::Json};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    database::Database,
    errors::AppError,
    models::{
        AgeGroup, CountryStats, CrownCountEntry, DistanceLeaderEntry, GenderFilter,
        LeaderboardScope, WeightClass,
    },
};

use super::pagination::default_limit;

/// Query parameters for global leaderboards.
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GlobalLeaderboardQuery {
    /// Maximum number of entries to return
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of entries to skip
    #[serde(default)]
    pub offset: i64,
    /// Time scope for filtering (default: all_time)
    #[serde(default)]
    pub scope: LeaderboardScope,
    /// Gender filter
    #[serde(default)]
    pub gender: GenderFilter,
    /// Age group filter
    #[serde(default)]
    pub age_group: AgeGroup,
    /// Weight class filter
    #[serde(default)]
    pub weight_class: WeightClass,
    /// Country filter (ISO country code)
    pub country: Option<String>,
    /// Filter crowns by activity type (only for crown leaderboard)
    pub activity_type_id: Option<Uuid>,
}

/// Get global crown count leaderboard.
#[utoipa::path(
    get,
    path = "/leaderboard/crowns",
    tag = "leaderboard",
    params(GlobalLeaderboardQuery),
    responses(
        (status = 200, description = "Crown count leaderboard", body = Vec<CrownCountEntry>)
    )
)]
pub async fn get_crown_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<CrownCountEntry>>, AppError> {
    let entries = db
        .get_crown_leaderboard_filtered(
            query.limit,
            query.offset,
            query.scope,
            query.gender,
            query.age_group,
            query.weight_class,
            query.country.as_deref(),
            query.activity_type_id,
        )
        .await?;
    Ok(Json(entries))
}

/// Get global distance leaderboard.
#[utoipa::path(
    get,
    path = "/leaderboard/distance",
    tag = "leaderboard",
    params(GlobalLeaderboardQuery),
    responses(
        (status = 200, description = "Distance leaderboard", body = Vec<DistanceLeaderEntry>)
    )
)]
pub async fn get_distance_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<DistanceLeaderEntry>>, AppError> {
    let entries = db
        .get_distance_leaderboard_filtered(
            query.limit,
            query.offset,
            query.scope,
            query.gender,
            query.age_group,
            query.weight_class,
            query.country.as_deref(),
        )
        .await?;
    Ok(Json(entries))
}

/// Get list of countries with user counts for the filter dropdown.
#[utoipa::path(
    get,
    path = "/countries",
    tag = "reference",
    responses(
        (status = 200, description = "List of countries with user counts", body = Vec<CountryStats>)
    )
)]
pub async fn get_countries(
    Extension(db): Extension<Database>,
) -> Result<Json<Vec<CountryStats>>, AppError> {
    let countries = db.get_countries_with_counts().await?;
    Ok(Json(countries))
}
