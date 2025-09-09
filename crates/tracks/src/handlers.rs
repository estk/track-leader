use axum::{
    extract::{Multipart, Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    Extension,
};
use bytes::Bytes;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::Database,
    errors::AppError,
    gpx_processor::GpxProcessor,
    models::{Activity, ActivityType},
    object_store_service::ObjectStoreService,
};

#[derive(Deserialize)]
pub struct UploadQuery {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
}

pub async fn upload_gpx(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Query(params): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<Activity>, AppError> {
    let mut filename = String::new();
    let mut file_bytes = Bytes::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::InvalidInput("Failed to process multipart data".to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            filename = field.file_name().unwrap_or("unknown.gpx").to_string();
            file_bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::InvalidInput("Failed to read file data".to_string()))?;
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::InvalidInput("No file provided".to_string()));
    }

    let activity_id = Uuid::new_v4();

    // Process GPX for metrics calculation
    let processed_gpx = GpxProcessor::process_gpx(&file_bytes)?;

    // Store the file in object store
    let object_store_path = store
        .store_file(file_bytes, params.user_id, &filename, activity_id)
        .await?;

    let activity = Activity {
        id: Uuid::new_v4(),
        user_id: params.user_id,
        metrics: processed_gpx.metrics,
        filename,
        activity_type: processed_gpx.activity_type,
        submitted_at: time::UtcDateTime::now(),
        created_at: processed_gpx.created_at,

        object_store_path,
    };

    db.save_activity(&activity).await?;
    Ok(Json(activity))
}

pub async fn get_activity(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Activity>, AppError> {
    match db.get_activity(id).await? {
        Some(activity) => Ok(Json(activity)),
        None => Err(AppError::NotFound),
    }
}

#[derive(Deserialize)]
pub struct UserActivitiesQuery {
    pub user_id: Uuid,
}

pub async fn get_user_activities(
    Extension(db): Extension<Database>,
    Query(params): Query<UserActivitiesQuery>,
) -> Result<Json<Vec<Activity>>, AppError> {
    let activities = db.get_user_activities(params.user_id).await?;
    Ok(Json(activities))
}

pub async fn download_gpx_file(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    let file_bytes = store.get_file(&activity.object_store_path).await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/gpx+xml".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", activity.filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, file_bytes).into_response())
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
