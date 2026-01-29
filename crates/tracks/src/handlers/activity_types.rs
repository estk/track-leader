//! Activity type management handlers.

use axum::{
    Extension,
    extract::{Path, Query},
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{auth::AuthUser, database::Database, errors::AppError, models::ActivityTypeRow};

/// List all activity types (built-in and custom).
#[utoipa::path(
    get,
    path = "/activity-types",
    tag = "activity-types",
    responses(
        (status = 200, description = "List of all activity types", body = Vec<ActivityTypeRow>)
    )
)]
pub async fn list_activity_types(
    Extension(db): Extension<Database>,
) -> Result<Json<Vec<ActivityTypeRow>>, AppError> {
    let types = db.list_activity_types().await?;
    Ok(Json(types))
}

/// Get a single activity type by ID.
#[utoipa::path(
    get,
    path = "/activity-types/{id}",
    tag = "activity-types",
    params(
        ("id" = Uuid, Path, description = "Activity type ID")
    ),
    responses(
        (status = 200, description = "Activity type details", body = ActivityTypeRow),
        (status = 404, description = "Activity type not found")
    )
)]
pub async fn get_activity_type(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<ActivityTypeRow>, AppError> {
    let activity_type = db.get_activity_type(id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(activity_type))
}

/// Create a custom activity type.
#[utoipa::path(
    post,
    path = "/activity-types",
    tag = "activity-types",
    request_body = crate::models::CreateActivityTypeRequest,
    responses(
        (status = 200, description = "Activity type created successfully", body = ActivityTypeRow),
        (status = 400, description = "Invalid input")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_activity_type(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Json(req): Json<crate::models::CreateActivityTypeRequest>,
) -> Result<Json<ActivityTypeRow>, AppError> {
    // Validate name: must be non-empty, alphanumeric with underscores
    let name = req.name.trim().to_lowercase();
    if name.is_empty() {
        return Err(AppError::InvalidInput("Name cannot be empty".to_string()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AppError::InvalidInput(
            "Name must be alphanumeric with underscores only".to_string(),
        ));
    }

    let activity_type = db.create_activity_type(&name, claims.sub).await?;
    Ok(Json(activity_type))
}

/// Activity type resolution query.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResolveTypeQuery {
    pub name: String,
}

/// Activity type resolution response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ResolveTypeResponse {
    /// "exact", "ambiguous", or "not_found"
    pub result: String,
    pub type_id: Option<Uuid>,
    pub type_ids: Option<Vec<Uuid>>,
}

/// Resolve an activity type by name or alias.
#[utoipa::path(
    get,
    path = "/activity-types/resolve",
    tag = "activity-types",
    params(
        ("name" = String, Query, description = "Activity type name or alias to resolve")
    ),
    responses(
        (status = 200, description = "Resolution result", body = ResolveTypeResponse)
    )
)]
pub async fn resolve_activity_type(
    Extension(db): Extension<Database>,
    Query(query): Query<ResolveTypeQuery>,
) -> Result<Json<ResolveTypeResponse>, AppError> {
    let resolved = db.resolve_activity_type(&query.name).await?;

    let response = match resolved {
        crate::models::ResolvedActivityType::Exact(id) => ResolveTypeResponse {
            result: "exact".to_string(),
            type_id: Some(id),
            type_ids: None,
        },
        crate::models::ResolvedActivityType::Ambiguous(ids) => ResolveTypeResponse {
            result: "ambiguous".to_string(),
            type_id: None,
            type_ids: Some(ids),
        },
        crate::models::ResolvedActivityType::NotFound => ResolveTypeResponse {
            result: "not_found".to_string(),
            type_id: None,
            type_ids: None,
        },
    };

    Ok(Json(response))
}
