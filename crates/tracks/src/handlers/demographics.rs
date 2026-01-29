//! User demographics handlers.

use axum::{Extension, response::Json};

use crate::{
    auth::AuthUser,
    database::Database,
    errors::AppError,
    models::{UpdateDemographicsRequest, UserWithDemographics},
};

/// Get the authenticated user's profile with demographics.
#[utoipa::path(
    get,
    path = "/users/me/demographics",
    tag = "users",
    responses(
        (status = 200, description = "User demographics", body = UserWithDemographics),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_demographics(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<UserWithDemographics>, AppError> {
    let user = db
        .get_user_with_demographics(claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

/// Update the authenticated user's demographics.
#[utoipa::path(
    patch,
    path = "/users/me/demographics",
    tag = "users",
    request_body = UpdateDemographicsRequest,
    responses(
        (status = 200, description = "Demographics updated successfully", body = UserWithDemographics),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_my_demographics(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Json(req): Json<UpdateDemographicsRequest>,
) -> Result<Json<UserWithDemographics>, AppError> {
    let user = db.update_user_demographics(claims.sub, &req).await?;
    Ok(Json(user))
}
