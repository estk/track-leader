use axum::{
    Extension,
    extract::{Multipart, Path, Query},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use axum_extra::headers::{ContentType, HeaderMapExt, Mime};
use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    activity_queue::ActivityQueue,
    database::Database,
    errors::AppError,
    models::{Activity, ActivityType, User},
    object_store_service::{FileType, ObjectStoreService},
};

#[derive(Debug, Serialize)]
pub struct TrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
    pub time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackData {
    pub points: Vec<TrackPoint>,
    pub bounds: TrackBounds,
}

#[derive(Debug, Serialize)]
pub struct TrackBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

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

pub async fn all_users(Extension(db): Extension<Database>) -> Result<Json<Vec<User>>, AppError> {
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

    aq.submit(params.user_id, activity.id, file_type, file_bytes)
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

#[derive(Debug, Deserialize)]
pub struct UpdateActivityRequest {
    pub name: Option<String>,
    pub activity_type: Option<ActivityType>,
}

pub async fn update_activity(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateActivityRequest>,
) -> Result<Json<Activity>, AppError> {
    let activity = db
        .update_activity(id, req.name.as_deref(), req.activity_type.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(activity))
}

pub async fn delete_activity(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if db.delete_activity(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

#[derive(Deserialize)]
pub struct UserActivitiesQuery {}

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

pub async fn get_activity_track(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Path(id): Path<Uuid>,
) -> Result<Json<TrackData>, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;
    let file_bytes = store.get_file(&activity.object_store_path).await?;

    let gpx: gpx::Gpx = gpx::read(std::io::BufReader::new(file_bytes.as_ref()))
        .map_err(|e| AppError::InvalidInput(format!("Failed to parse GPX: {e}")))?;

    let mut points = Vec::new();
    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lon = f64::MAX;
    let mut max_lon = f64::MIN;

    for track in &gpx.tracks {
        for segment in &track.segments {
            for pt in &segment.points {
                let lat = pt.point().y();
                let lon = pt.point().x();
                let ele = pt.elevation;
                let time = pt.time.as_ref().map(|t| t.format().unwrap_or_default());

                min_lat = min_lat.min(lat);
                max_lat = max_lat.max(lat);
                min_lon = min_lon.min(lon);
                max_lon = max_lon.max(lon);

                points.push(TrackPoint {
                    lat,
                    lon,
                    ele,
                    time,
                });
            }
        }
    }

    Ok(Json(TrackData {
        points,
        bounds: TrackBounds {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        },
    }))
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
