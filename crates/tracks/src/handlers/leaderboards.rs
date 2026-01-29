//! Global leaderboard handlers.

use axum::{Extension, extract::Query, response::Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    database::Database,
    errors::AppError,
    models::{CountryStats, CrownCountEntry, DistanceLeaderEntry},
};

use super::pagination::default_limit;

/// Query parameters for global leaderboards.
#[derive(Debug, Deserialize, ToSchema)]
pub struct GlobalLeaderboardQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Get global crown count leaderboard.
#[utoipa::path(
    get,
    path = "/leaderboard/crowns",
    tag = "leaderboard",
    params(
        ("limit" = i64, Query, description = "Maximum number of entries to return"),
        ("offset" = i64, Query, description = "Number of entries to skip")
    ),
    responses(
        (status = 200, description = "Crown count leaderboard", body = Vec<CrownCountEntry>)
    )
)]
pub async fn get_crown_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<CrownCountEntry>>, AppError> {
    let entries = db
        .get_crown_count_leaderboard(query.limit, query.offset)
        .await?;
    Ok(Json(entries))
}

/// Get global distance leaderboard.
#[utoipa::path(
    get,
    path = "/leaderboard/distance",
    tag = "leaderboard",
    params(
        ("limit" = i64, Query, description = "Maximum number of entries to return"),
        ("offset" = i64, Query, description = "Number of entries to skip")
    ),
    responses(
        (status = 200, description = "Distance leaderboard", body = Vec<DistanceLeaderEntry>)
    )
)]
pub async fn get_distance_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<DistanceLeaderEntry>>, AppError> {
    let entries = db
        .get_distance_leaderboard(query.limit, query.offset)
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
