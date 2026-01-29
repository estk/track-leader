//! Health check and platform statistics handlers.

use axum::{Extension, http::StatusCode, response::Json};

use crate::{database::Database, errors::AppError, models::Stats};

/// Health check endpoint.
#[utoipa::path(
    get,
    path = "/health",
    tag = "stats",
    responses(
        (status = 200, description = "Health check passed")
    )
)]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Get platform-wide statistics (active users, segments created, activities uploaded).
#[utoipa::path(
    get,
    path = "/stats",
    tag = "stats",
    responses(
        (status = 200, description = "Platform statistics", body = Stats)
    )
)]
pub async fn get_stats(Extension(db): Extension<Database>) -> Result<Json<Stats>, AppError> {
    let stats = db.get_stats().await?;
    Ok(Json(stats))
}
