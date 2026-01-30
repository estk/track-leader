//! Activity management handlers.

use axum::{
    Extension,
    extract::{Multipart, Path, Query},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use axum_extra::headers::{ContentType, HeaderMapExt, Mime};
use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    activity_queue::{ActivityQueue, ActivitySubmission},
    auth::{AuthUser, OptionalAuthUser},
    database::Database,
    errors::AppError,
    models::{Activity, FeedActivity},
    object_store_service::{FileType, ObjectStoreService},
};

use super::pagination::default_limit;

/// Track point in activity track data.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
    pub time: Option<String>,
}

/// Activity track data response.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrackData {
    pub points: Vec<TrackPoint>,
    pub bounds: TrackBounds,
}

/// Geographic bounds for a track.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrackBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// Activity upload query parameters.
#[derive(Deserialize, ToSchema)]
pub struct UploadQuery {
    pub activity_type_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub visibility: Option<String>,
    /// Comma-separated list of team IDs to share with (for teams_only visibility)
    #[serde(default)]
    pub team_ids: Option<String>,
    /// Multi-sport: Comma-separated ISO-8601 timestamps marking segment boundaries
    #[serde(default)]
    pub type_boundaries: Option<String>,
    /// Multi-sport: Comma-separated activity type UUIDs for each segment
    #[serde(default)]
    pub segment_types: Option<String>,
}

/// Activity update request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateActivityRequest {
    pub name: Option<String>,
    pub activity_type_id: Option<Uuid>,
    pub visibility: Option<String>,
}

/// User activities query parameters (placeholder for future pagination).
#[derive(Deserialize, ToSchema)]
pub struct UserActivitiesQuery {}

/// Query parameters for activities by date.
#[derive(Deserialize, ToSchema)]
pub struct ActivitiesByDateQuery {
    /// Date to filter activities by (YYYY-MM-DD).
    pub date: time::Date,
    /// Filter to only the authenticated user's activities.
    #[serde(default)]
    pub mine_only: Option<bool>,
    /// Maximum number of results.
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip.
    #[serde(default)]
    pub offset: i64,
}

/// Create a new activity by uploading a GPX file.
#[utoipa::path(
    post,
    path = "/activities/new",
    tag = "activities",
    params(
        ("activity_type_id" = Uuid, Query, description = "Activity type ID"),
        ("name" = String, Query, description = "Activity name"),
        ("visibility" = Option<String>, Query, description = "Visibility: public, private, or teams_only"),
        ("team_ids" = Option<String>, Query, description = "Comma-separated team IDs for teams_only visibility"),
        ("type_boundaries" = Option<String>, Query, description = "Comma-separated ISO-8601 timestamps for multi-sport segment boundaries"),
        ("segment_types" = Option<String>, Query, description = "Comma-separated activity type UUIDs for multi-sport segments")
    ),
    request_body(content_type = "multipart/form-data", description = "GPX file upload"),
    responses(
        (status = 200, description = "Activity created successfully", body = Activity),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn new_activity(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Extension(aq): Extension<ActivityQueue>,
    AuthUser(claims): AuthUser,
    Query(params): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<Activity>, AppError> {
    let user_id = claims.sub;
    let activity_id = Uuid::new_v4();
    let name = params.name;
    let activity_type_id = params.activity_type_id;

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
        .store_file(user_id, activity_id, file_type, file_bytes.clone())
        .await?;

    // Parse multi-sport parameters from comma-separated strings
    let type_boundaries: Option<Vec<time::OffsetDateTime>> =
        params.type_boundaries.as_ref().map(|s| {
            s.split(',')
                .filter_map(|ts| {
                    time::OffsetDateTime::parse(
                        ts.trim(),
                        &time::format_description::well_known::Rfc3339,
                    )
                    .ok()
                })
                .collect()
        });

    let segment_types: Option<Vec<Uuid>> = params.segment_types.as_ref().map(|s| {
        s.split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect()
    });

    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        name,
        activity_type_id,
        submitted_at: time::UtcDateTime::now().to_offset(time::UtcOffset::UTC),
        object_store_path,
        visibility: params.visibility.unwrap_or_else(|| "public".to_string()),
        type_boundaries,
        segment_types,
    };

    // Save activity to database BEFORE submitting to queue to avoid race condition.
    // The queue worker inserts scores with activity_id as a foreign key, so the
    // activity row must exist first.
    db.save_activity(&activity).await?;

    aq.submit(ActivitySubmission {
        user_id,
        activity_id: activity.id,
        file_type,
        bytes: file_bytes,
        activity_type_id: activity.activity_type_id,
        type_boundaries: activity.type_boundaries.clone(),
        segment_types: activity.segment_types.clone(),
    })
    .map_err(AppError::Queue)?;

    // Share with teams if team_ids provided
    if let Some(team_ids_str) = &params.team_ids {
        let team_ids: Vec<Uuid> = team_ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if !team_ids.is_empty() {
            db.share_activity_with_teams(activity.id, &team_ids, user_id)
                .await?;
        }
    }

    Ok(Json(activity))
}

/// Get an activity by ID.
#[utoipa::path(
    get,
    path = "/activities/{id}",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "Activity ID")
    ),
    responses(
        (status = 200, description = "Activity details", body = Activity),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn get_activity(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Activity>, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().is_some_and(|c| c.sub == activity.user_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                // Owner always has access
                if c.sub == activity.user_id {
                    true
                } else {
                    // Check team membership
                    db.user_has_activity_team_access(c.sub, id).await?
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if has_access {
        Ok(Json(activity))
    } else {
        // Return 404 to avoid leaking existence
        Err(AppError::NotFound)
    }
}

/// Update an activity.
#[utoipa::path(
    patch,
    path = "/activities/{id}",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "Activity ID")
    ),
    request_body = UpdateActivityRequest,
    responses(
        (status = 200, description = "Activity updated successfully", body = Activity),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn update_activity(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateActivityRequest>,
) -> Result<Json<Activity>, AppError> {
    let activity = db
        .update_activity(
            id,
            req.name.as_deref(),
            req.activity_type_id,
            req.visibility.as_deref(),
        )
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(activity))
}

/// Delete an activity.
#[utoipa::path(
    delete,
    path = "/activities/{id}",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "Activity ID")
    ),
    responses(
        (status = 204, description = "Activity deleted successfully"),
        (status = 404, description = "Activity not found")
    )
)]
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

/// Get activities for a user.
#[utoipa::path(
    get,
    path = "/users/{id}/activities",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "List of user's activities", body = Vec<Activity>)
    )
)]
pub async fn get_user_activities(
    Extension(db): Extension<Database>,
    Query(_params): Query<UserActivitiesQuery>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Activity>>, AppError> {
    let activities = db.get_user_activities(id).await?;
    Ok(Json(activities))
}

/// Download an activity's GPX file.
#[utoipa::path(
    get,
    path = "/activities/{id}/download",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "Activity ID")
    ),
    responses(
        (status = 200, description = "GPX file download", content_type = "application/gpx+xml"),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn download_gpx_file(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().is_some_and(|c| c.sub == activity.user_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                if c.sub == activity.user_id {
                    true
                } else {
                    db.user_has_activity_team_access(c.sub, id).await?
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if !has_access {
        return Err(AppError::NotFound);
    }

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

/// Get track data for an activity.
#[utoipa::path(
    get,
    path = "/activities/{id}/track",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "Activity ID")
    ),
    responses(
        (status = 200, description = "Track data with GPS points and bounds", body = TrackData),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn get_activity_track(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TrackData>, AppError> {
    // Verify activity exists and check access
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().is_some_and(|c| c.sub == activity.user_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                if c.sub == activity.user_id {
                    true
                } else {
                    db.user_has_activity_team_access(c.sub, id).await?
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if !has_access {
        return Err(AppError::NotFound);
    }

    // Get track points from database
    let track_points = db.get_track_points(id).await?.ok_or(AppError::NotFound)?;

    if track_points.is_empty() {
        return Err(AppError::NotFound);
    }

    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lon = f64::MAX;
    let mut max_lon = f64::MIN;

    let points: Vec<TrackPoint> = track_points
        .iter()
        .map(|pt| {
            min_lat = min_lat.min(pt.lat);
            max_lat = max_lat.max(pt.lat);
            min_lon = min_lon.min(pt.lon);
            max_lon = max_lon.max(pt.lon);

            TrackPoint {
                lat: pt.lat,
                lon: pt.lon,
                ele: pt.elevation,
                time: pt.timestamp.map(|t| {
                    t.format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default()
                }),
            }
        })
        .collect();

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

/// Get segment efforts matched in an activity.
#[utoipa::path(
    get,
    path = "/activities/{id}/segments",
    tag = "activities",
    params(
        ("id" = Uuid, Path, description = "Activity ID")
    ),
    responses(
        (status = 200, description = "Segment efforts matched in this activity", body = Vec<crate::models::ActivitySegmentEffort>),
        (status = 404, description = "Activity not found")
    )
)]
pub async fn get_activity_segments(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::ActivitySegmentEffort>>, AppError> {
    // Verify activity exists and check access
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().is_some_and(|c| c.sub == activity.user_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                if c.sub == activity.user_id {
                    true
                } else {
                    db.user_has_activity_team_access(c.sub, id).await?
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if !has_access {
        return Err(AppError::NotFound);
    }

    let efforts = db.get_activity_segment_efforts(id).await?;
    Ok(Json(efforts))
}

/// Get activities by date.
///
/// Returns activities submitted on the specified date. Visibility filtering is applied:
/// - Anonymous users only see public activities
/// - Authenticated users see public activities, their own private activities,
///   and activities shared with teams they are members of
#[utoipa::path(
    get,
    path = "/activities/by-date",
    tag = "activities",
    params(
        ("date" = String, Query, description = "Date to filter by (YYYY-MM-DD)"),
        ("mine_only" = Option<bool>, Query, description = "Filter to own activities only"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Number of results to skip")
    ),
    responses(
        (status = 200, description = "Activities for the specified date", body = Vec<FeedActivity>)
    )
)]
pub async fn get_activities_by_date(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Query(query): Query<ActivitiesByDateQuery>,
) -> Result<Json<Vec<FeedActivity>>, AppError> {
    let user_id = claims.as_ref().map(|c| c.sub);
    let mine_only = query.mine_only.unwrap_or(false);

    // If mine_only is requested but user is not authenticated, return empty
    if mine_only && user_id.is_none() {
        return Ok(Json(vec![]));
    }

    let activities = db
        .get_activities_by_date(query.date, user_id, mine_only, query.limit, query.offset)
        .await?;

    Ok(Json(activities))
}
