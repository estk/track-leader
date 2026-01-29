//! User management handlers.

use axum::{Extension, extract::Query, response::Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{database::Database, errors::AppError, models::User};

/// User creation query parameters.
#[derive(Deserialize, ToSchema)]
pub struct NewUserQuery {
    pub name: String,
    pub email: String,
}

/// Create a new user.
#[utoipa::path(
    get,
    path = "/users/new",
    tag = "users",
    params(
        ("name" = String, Query, description = "User's name"),
        ("email" = String, Query, description = "User's email address")
    ),
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 400, description = "Invalid input")
    )
)]
pub async fn new_user(
    Extension(db): Extension<Database>,
    Query(params): Query<NewUserQuery>,
) -> Result<Json<User>, AppError> {
    let user = User::new(params.name, params.email);
    db.new_user(&user).await?;
    Ok(Json(user))
}

/// Get all users.
#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    responses(
        (status = 200, description = "List of all users", body = Vec<User>)
    )
)]
pub async fn all_users(Extension(db): Extension<Database>) -> Result<Json<Vec<User>>, AppError> {
    let users = db.all_users().await?;
    Ok(Json(users))
}
