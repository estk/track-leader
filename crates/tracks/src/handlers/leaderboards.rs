//! Global leaderboard handlers.

use axum::{Extension, extract::Query, response::Json};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    database::Database,
    errors::AppError,
    models::{
        AgeGroup, AverageSpeedLeaderEntry, CountryStats, CrownCountEntry, DigPercentageLeaderEntry,
        DigTimeLeaderEntry, DistanceLeaderEntry, GenderFilter, LeaderboardScope, WeightClass,
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
            None, // No team filter for global leaderboard
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
            None, // No team filter for global leaderboard
        )
        .await?;
    Ok(Json(entries))
}

/// Get global dig time leaderboard (total dig seconds in last 7 days).
#[utoipa::path(
    get,
    path = "/leaderboard/dig-time",
    tag = "leaderboard",
    params(GlobalLeaderboardQuery),
    responses(
        (status = 200, description = "Dig time leaderboard", body = Vec<DigTimeLeaderEntry>)
    )
)]
pub async fn get_dig_time_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<DigTimeLeaderEntry>>, AppError> {
    let entries = db
        .get_dig_time_leaderboard_filtered(
            query.limit,
            query.offset,
            query.gender,
            query.age_group,
            query.weight_class,
            query.country.as_deref(),
            None, // No team filter for global leaderboard
        )
        .await?;
    Ok(Json(entries))
}

/// Get global dig percentage leaderboard (dig_time / ride_activity_time).
#[utoipa::path(
    get,
    path = "/leaderboard/dig-percentage",
    tag = "leaderboard",
    params(GlobalLeaderboardQuery),
    responses(
        (status = 200, description = "Dig percentage leaderboard", body = Vec<DigPercentageLeaderEntry>)
    )
)]
pub async fn get_dig_percentage_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<DigPercentageLeaderEntry>>, AppError> {
    let entries = db
        .get_dig_percentage_leaderboard_filtered(
            query.limit,
            query.offset,
            query.scope,
            query.gender,
            query.age_group,
            query.weight_class,
            query.country.as_deref(),
            None, // No team filter for global leaderboard
        )
        .await?;
    Ok(Json(entries))
}

/// Get global average speed leaderboard.
#[utoipa::path(
    get,
    path = "/leaderboard/average-speed",
    tag = "leaderboard",
    params(GlobalLeaderboardQuery),
    responses(
        (status = 200, description = "Average speed leaderboard", body = Vec<AverageSpeedLeaderEntry>)
    )
)]
pub async fn get_average_speed_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<AverageSpeedLeaderEntry>>, AppError> {
    let entries = db
        .get_average_speed_leaderboard_filtered(
            query.limit,
            query.offset,
            query.scope,
            query.gender,
            query.age_group,
            query.weight_class,
            query.country.as_deref(),
            None, // No team filter for global leaderboard
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
