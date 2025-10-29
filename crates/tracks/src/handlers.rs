use axum::{
    extract::{Multipart, Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    Extension,
};
use axum_extra::{
    headers::{ContentType, HeaderMapExt, Mime},
};
use bytes::BytesMut;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    activity_queue::ActivityQueue,
    database::Database,
    errors::AppError,
    models::{Activity, ActivityType, User},
    object_store_service::{FileType, ObjectStoreService},
};

#[derive(Deserialize)]
pub struct NewUserQuery {
    pub name: String,
    pub email: String,
}

pub async fn new_user(
    Extension(db): Extension<Database>,
    Query(params): Query<NewUserQuery>,
) -> Result<Json<User>, AppError> {
    let user = User::new(params.name, params.email);
    db.new_user(&user).await?;
    Ok(Json(user))
}

pub async fn all_users(
    Extension(db): Extension<Database>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = db.all_users().await?;
    Ok(Json(users))
}

#[derive(Deserialize)]
pub struct UploadQuery {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub name: String,
}

pub async fn new_activity(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Extension(aq): Extension<ActivityQueue>,
    Query(params): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<Activity>, AppError> {
    let activity_id = Uuid::new_v4();
    let name = params.name;
    let activity_type = params.activity_type;


    let (mime_hdr, file_bytes) =
        {
            let mut file_bytes = BytesMut::new();
            let mut mime_hdr = None;

            while let Some(field) = multipart.next_field().await.map_err(|_| {
                AppError::InvalidInput("Failed to process multipart data".to_string())
            })? {
                if field.name() == Some("file") {
                    mime_hdr = field.headers().typed_get::<ContentType>();
                    let chunk = field.bytes().await.map_err(|_| {
                        AppError::InvalidInput("Failed to read file data".to_string())
                    })?;
                    file_bytes.extend(chunk);
                } else {
                    tracing::warn!("Unexpected field: {:?}", field.name());
                }
            }

            if file_bytes.is_empty() {
                return Err(AppError::InvalidInput("No file provided".to_string()));
            }
            (mime_hdr, file_bytes.freeze())
        };

    let file_type = mime_hdr.map_or(FileType::Other, |ct| {
        let mime = Mime::from(ct);
        FileType::from(mime)
    });

    // Store the file in object store
    let object_store_path = store
        .store_file(params.user_id, activity_id, file_type, file_bytes.clone())
        .await?;

    let activity = Activity {
        id: Uuid::new_v4(),
        user_id: params.user_id,
        name,
        activity_type,
        submitted_at: time::UtcDateTime::now().to_offset(time::UtcOffset::UTC),

        object_store_path,
    };

    aq.submit(
        params.user_id,
        activity.id,
        file_type,
        file_bytes,
    )
    .map_err(AppError::Queue)?;

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
}

pub async fn get_user_activities(
    Extension(db): Extension<Database>,
    Query(_params): Query<UserActivitiesQuery>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Activity>>, AppError> {
    let activities = db.get_user_activities(id).await?;
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
        format!("attachment; filename=\"{}\"", activity.name)
            .parse()
            .unwrap(),
    );

    Ok((headers, file_bytes).into_response())
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
