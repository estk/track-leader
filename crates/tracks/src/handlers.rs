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
    auth::AuthUser,
    database::Database,
    errors::AppError,
    models::{Activity, ActivityType, Segment, SegmentEffort, User},
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
    #[serde(default)]
    pub visibility: Option<String>,
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
        visibility: params.visibility.unwrap_or_else(|| "public".to_string()),
    };

    aq.submit(
        params.user_id,
        activity.id,
        file_type,
        file_bytes,
        activity.activity_type,
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

#[derive(Debug, Deserialize)]
pub struct UpdateActivityRequest {
    pub name: Option<String>,
    pub activity_type: Option<ActivityType>,
    pub visibility: Option<String>,
}

pub async fn update_activity(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateActivityRequest>,
) -> Result<Json<Activity>, AppError> {
    let activity = db
        .update_activity(
            id,
            req.name.as_deref(),
            req.activity_type.as_ref(),
            req.visibility.as_deref(),
        )
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

// Segment handlers

#[derive(Debug, Deserialize)]
pub struct CreateSegmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub activity_type: ActivityType,
    pub points: Vec<SegmentPoint>,
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_visibility() -> String {
    "public".to_string()
}

#[derive(Debug, Deserialize)]
pub struct SegmentPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

pub async fn create_segment(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateSegmentRequest>,
) -> Result<Json<Segment>, AppError> {
    use crate::segment_matching;

    if req.points.len() < 2 {
        return Err(AppError::InvalidInput(
            "Segment must have at least 2 points".to_string(),
        ));
    }

    // Build WKT strings for PostGIS (include elevation if available)
    let has_elevation = req.points.iter().any(|p| p.ele.is_some());
    let coords: Vec<String> = req
        .points
        .iter()
        .map(|p| {
            if has_elevation {
                format!("{} {} {}", p.lon, p.lat, p.ele.unwrap_or(0.0))
            } else {
                format!("{} {}", p.lon, p.lat)
            }
        })
        .collect();
    let geo_wkt = if has_elevation {
        format!("LINESTRING Z({})", coords.join(", "))
    } else {
        format!("LINESTRING({})", coords.join(", "))
    };

    let start = &req.points[0];
    let end = &req.points[req.points.len() - 1];
    let start_wkt = format!("POINT({} {})", start.lon, start.lat);
    let end_wkt = format!("POINT({} {})", end.lon, end.lat);

    // Calculate distance (simple haversine sum)
    let distance_meters = calculate_total_distance(&req.points);

    // Calculate elevation gain/loss
    let (elevation_gain, elevation_loss) = calculate_elevation_change(&req.points);

    let creator_id = claims.sub;

    let segment = db
        .create_segment(
            Uuid::new_v4(),
            creator_id,
            &req.name,
            req.description.as_deref(),
            &req.activity_type,
            &geo_wkt,
            &start_wkt,
            &end_wkt,
            distance_meters,
            elevation_gain,
            elevation_loss,
            &req.visibility,
        )
        .await?;

    // Automatically find and create efforts for existing activities
    let segment_id = segment.id;
    match db.find_matching_activities_for_segment(segment_id).await {
        Ok(matches) => {
            for activity_match in matches {
                // Get the activity to find its GPX file
                let activity = match db.get_activity(activity_match.activity_id).await {
                    Ok(Some(a)) => a,
                    _ => continue,
                };

                // Fetch and parse the GPX
                let file_bytes = match store.get_file(&activity.object_store_path).await {
                    Ok(bytes) => bytes,
                    Err(_) => continue,
                };

                let gpx: gpx::Gpx = match gpx::read(std::io::BufReader::new(file_bytes.as_ref())) {
                    Ok(g) => g,
                    Err(_) => continue,
                };

                // Extract timing
                let timing = match segment_matching::extract_timing_from_gpx(
                    &gpx,
                    activity_match.start_fraction,
                    activity_match.end_fraction,
                ) {
                    Some(t) => t,
                    None => continue,
                };

                // Create the effort
                if let Ok(effort) = db
                    .create_segment_effort(
                        segment_id,
                        activity_match.activity_id,
                        activity_match.user_id,
                        timing.started_at,
                        timing.elapsed_time_seconds,
                        None,
                        None,
                        None,
                    )
                    .await
                {
                    tracing::info!(
                        "Auto-created effort {} for segment {} from activity {} with time {:.1}s",
                        effort.id,
                        segment_id,
                        activity_match.activity_id,
                        timing.elapsed_time_seconds
                    );

                    // Update personal records
                    let _ = db
                        .update_personal_records(segment_id, activity_match.user_id)
                        .await;
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to find matching activities for new segment: {e}");
        }
    }

    Ok(Json(segment))
}

fn calculate_total_distance(points: &[SegmentPoint]) -> f64 {
    let mut total = 0.0;
    for i in 1..points.len() {
        total += haversine_distance(
            points[i - 1].lat,
            points[i - 1].lon,
            points[i].lat,
            points[i].lon,
        );
    }
    total
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371000.0; // Earth radius in meters
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}

fn calculate_elevation_change(points: &[SegmentPoint]) -> (Option<f64>, Option<f64>) {
    let mut gain = 0.0;
    let mut loss = 0.0;
    let mut has_elevation = false;

    for i in 1..points.len() {
        if let (Some(e1), Some(e2)) = (points[i - 1].ele, points[i].ele) {
            has_elevation = true;
            let diff = e2 - e1;
            if diff > 0.0 {
                gain += diff;
            } else {
                loss += diff.abs();
            }
        }
    }

    if has_elevation {
        (Some(gain), Some(loss))
    } else {
        (None, None)
    }
}

pub async fn get_segment(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Segment>, AppError> {
    match db.get_segment(id).await? {
        Some(segment) => Ok(Json(segment)),
        None => Err(AppError::NotFound),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSegmentsQuery {
    pub activity_type: Option<ActivityType>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn list_segments(
    Extension(db): Extension<Database>,
    Query(params): Query<ListSegmentsQuery>,
) -> Result<Json<Vec<Segment>>, AppError> {
    let segments = db
        .list_segments(params.activity_type.as_ref(), params.limit)
        .await?;
    Ok(Json(segments))
}

pub async fn get_segment_leaderboard(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SegmentEffort>>, AppError> {
    let efforts = db.get_segment_efforts(id, 100).await?;
    Ok(Json(efforts))
}

#[derive(Debug, Serialize)]
pub struct SegmentTrackData {
    pub points: Vec<SegmentTrackPoint>,
    pub bounds: TrackBounds,
}

#[derive(Debug, Serialize)]
pub struct SegmentTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

pub async fn get_segment_track(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<SegmentTrackData>, AppError> {
    let geojson = db
        .get_segment_geometry(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Parse GeoJSON to extract coordinates
    let parsed: serde_json::Value =
        serde_json::from_str(&geojson).map_err(|_| AppError::Internal)?;

    let coords = parsed["coordinates"].as_array().ok_or(AppError::Internal)?;

    let points: Vec<SegmentTrackPoint> = coords
        .iter()
        .filter_map(|c| {
            let arr = c.as_array()?;
            Some(SegmentTrackPoint {
                lon: arr.first()?.as_f64()?,
                lat: arr.get(1)?.as_f64()?,
                ele: arr.get(2).and_then(|v| v.as_f64()),
            })
        })
        .collect();

    if points.is_empty() {
        return Err(AppError::NotFound);
    }

    let min_lat = points.iter().map(|p| p.lat).fold(f64::INFINITY, f64::min);
    let max_lat = points
        .iter()
        .map(|p| p.lat)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_lon = points.iter().map(|p| p.lon).fold(f64::INFINITY, f64::min);
    let max_lon = points
        .iter()
        .map(|p| p.lon)
        .fold(f64::NEG_INFINITY, f64::max);

    Ok(Json(SegmentTrackData {
        points,
        bounds: TrackBounds {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        },
    }))
}

#[derive(Debug, Serialize)]
pub struct ReprocessResult {
    pub segment_id: Uuid,
    pub activities_checked: usize,
    pub efforts_created: usize,
}

/// Reprocess all activities to find matches for a specific segment.
/// This is useful when a new segment is created and we want to find
/// all existing activities that pass through it.
pub async fn reprocess_segment(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<ReprocessResult>, AppError> {
    use crate::segment_matching;

    // Verify segment exists
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Find all activities that match this segment
    let matches = db.find_matching_activities_for_segment(segment_id).await?;

    let activities_checked = matches.len();
    let mut efforts_created = 0;

    for activity_match in matches {
        // Check if effort already exists
        if db
            .segment_effort_exists(segment_id, activity_match.activity_id)
            .await?
        {
            continue;
        }

        // Get the activity to find its GPX file
        let activity = match db.get_activity(activity_match.activity_id).await? {
            Some(a) => a,
            None => continue,
        };

        // Fetch and parse the GPX
        let file_bytes = match store.get_file(&activity.object_store_path).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch GPX for activity {}: {e}",
                    activity_match.activity_id
                );
                continue;
            }
        };

        let gpx: gpx::Gpx = match gpx::read(std::io::BufReader::new(file_bytes.as_ref())) {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!(
                    "Failed to parse GPX for activity {}: {e}",
                    activity_match.activity_id
                );
                continue;
            }
        };

        // Extract timing
        let timing = match segment_matching::extract_timing_from_gpx(
            &gpx,
            activity_match.start_fraction,
            activity_match.end_fraction,
        ) {
            Some(t) => t,
            None => {
                tracing::warn!(
                    "Could not extract timing for activity {} on segment {}",
                    activity_match.activity_id,
                    segment_id
                );
                continue;
            }
        };

        // Create the effort
        match db
            .create_segment_effort(
                segment_id,
                activity_match.activity_id,
                activity_match.user_id,
                timing.started_at,
                timing.elapsed_time_seconds,
                None,
                None,
                None,
            )
            .await
        {
            Ok(effort) => {
                tracing::info!(
                    "Created segment effort {} for segment {} from activity {} with time {:.1}s",
                    effort.id,
                    segment_id,
                    activity_match.activity_id,
                    timing.elapsed_time_seconds
                );
                efforts_created += 1;

                // Update personal records
                if let Err(e) = db
                    .update_personal_records(segment_id, activity_match.user_id)
                    .await
                {
                    tracing::error!("Failed to update personal records: {e}");
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create segment effort for activity {}: {e}",
                    activity_match.activity_id
                );
            }
        }
    }

    Ok(Json(ReprocessResult {
        segment_id: segment.id,
        activities_checked,
        efforts_created,
    }))
}
