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

pub async fn get_activity_segments(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::ActivitySegmentEffort>>, AppError> {
    // Verify activity exists
    db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    let efforts = db.get_activity_segment_efforts(id).await?;
    Ok(Json(efforts))
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// Segment handlers

#[derive(Debug, Deserialize)]
pub struct CreateSegmentRequest {
    pub name: String,
    pub description: Option<String>,
    /// Optional if source_activity_id is provided (inherits from the activity).
    /// Required if source_activity_id is not provided.
    pub activity_type: Option<ActivityType>,
    pub points: Vec<SegmentPoint>,
    #[serde(default = "default_visibility")]
    pub visibility: String,
    /// Optional: the activity this segment was created from.
    /// If provided, the segment inherits its activity_type and guarantees that activity gets the first effort.
    pub source_activity_id: Option<Uuid>,
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

    // Validation: minimum point count
    const MIN_POINTS: usize = 10;
    if req.points.len() < MIN_POINTS {
        return Err(AppError::InvalidInput(format!(
            "Segment must have at least {MIN_POINTS} points (got {})",
            req.points.len()
        )));
    }

    // Calculate distance early for validation
    let distance_meters = calculate_total_distance(&req.points);

    // Validation: minimum length (100m)
    const MIN_LENGTH_METERS: f64 = 100.0;
    if distance_meters < MIN_LENGTH_METERS {
        return Err(AppError::InvalidInput(format!(
            "Segment must be at least {MIN_LENGTH_METERS}m long (got {:.0}m)",
            distance_meters
        )));
    }

    // Validation: maximum length (50km)
    const MAX_LENGTH_METERS: f64 = 50_000.0;
    if distance_meters > MAX_LENGTH_METERS {
        return Err(AppError::InvalidInput(format!(
            "Segment must be at most {}km long (got {:.1}km)",
            MAX_LENGTH_METERS / 1000.0,
            distance_meters / 1000.0
        )));
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

    // Calculate elevation gain/loss
    let (elevation_gain, elevation_loss) = calculate_elevation_change(&req.points);

    // Calculate grade metrics
    let (average_grade, max_grade) = calculate_grades(&req.points);

    // Calculate climb category
    let climb_category = calculate_climb_category(elevation_gain, distance_meters, average_grade);

    let creator_id = claims.sub;

    // Resolve activity_type: inherit from source activity if provided, otherwise require in request
    // Also store the source activity for later use (to save track geometry if needed)
    let (activity_type, source_activity) = if let Some(source_id) = req.source_activity_id {
        let activity = db.get_activity(source_id).await?.ok_or_else(|| {
            AppError::InvalidInput(format!("Source activity {source_id} not found"))
        })?;
        let activity_type = activity.activity_type.clone();
        (activity_type, Some(activity))
    } else {
        let activity_type = req.activity_type.clone().ok_or_else(|| {
            AppError::InvalidInput(
                "activity_type is required when source_activity_id is not provided".to_string(),
            )
        })?;
        (activity_type, None)
    };

    // Check for duplicate segments (same activity type, similar start/end points)
    let similar_segments = db
        .find_similar_segments(&activity_type, &start_wkt, &end_wkt)
        .await?;
    if !similar_segments.is_empty() {
        return Err(AppError::SimilarSegmentsExist(similar_segments));
    }

    let segment = db
        .create_segment(
            Uuid::new_v4(),
            creator_id,
            &req.name,
            req.description.as_deref(),
            &activity_type,
            &geo_wkt,
            &start_wkt,
            &end_wkt,
            distance_meters,
            elevation_gain,
            elevation_loss,
            average_grade,
            max_grade,
            climb_category,
            &req.visibility,
        )
        .await?;

    // If source_activity provided, ensure its track is in the database
    // (it might not be if the activity was uploaded before track storage was implemented)
    if let Some(activity) = &source_activity {
        let source_id = activity.id;
        // Check if track already exists
        if db
            .get_track_geometry(source_id)
            .await
            .ok()
            .flatten()
            .is_none()
        {
            // Track not in database, try to save it
            if let Ok(file_bytes) = store.get_file(&activity.object_store_path).await {
                if let Ok(gpx) = gpx::read(std::io::BufReader::new(file_bytes.as_ref())) {
                    if let Some(wkt) = build_track_wkt(&gpx) {
                        if let Err(e) = db
                            .save_track_geometry(activity.user_id, source_id, &wkt)
                            .await
                        {
                            tracing::warn!(
                                "Failed to save track geometry for source activity {source_id}: {e}"
                            );
                        } else {
                            tracing::info!("Saved track geometry for source activity {source_id}");
                        }
                    }
                }
            }
        }
    }

    // Automatically find and create efforts for existing activities
    let segment_id = segment.id;
    match db.find_matching_activities_for_segment(segment_id).await {
        Ok(matches) => {
            tracing::info!(
                "Found {} matching activities for segment {}",
                matches.len(),
                segment_id
            );
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

                // Calculate average speed: distance / time
                let average_speed_mps = if timing.elapsed_time_seconds > 0.0 {
                    Some(distance_meters / timing.elapsed_time_seconds)
                } else {
                    None
                };

                // Create the effort
                if let Ok(effort) = db
                    .create_segment_effort(
                        segment_id,
                        activity_match.activity_id,
                        activity_match.user_id,
                        timing.started_at,
                        timing.elapsed_time_seconds,
                        Some(timing.moving_time_seconds),
                        average_speed_mps,
                        None, // max_speed_mps
                        Some(activity_match.start_fraction),
                        Some(activity_match.end_fraction),
                    )
                    .await
                {
                    tracing::info!(
                        "Auto-created effort {} for segment {} from activity {} with time {:.1}s (moving: {:.1}s)",
                        effort.id,
                        segment_id,
                        activity_match.activity_id,
                        timing.elapsed_time_seconds,
                        timing.moving_time_seconds
                    );

                    // Update effort count
                    let _ = db.increment_segment_effort_count(segment_id).await;

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

/// Build a WKT LINESTRING from all track points
fn build_track_wkt(gpx: &gpx::Gpx) -> Option<String> {
    let mut coords: Vec<String> = Vec::new();

    for track in &gpx.tracks {
        for seg in &track.segments {
            for pt in &seg.points {
                let lon = pt.point().x();
                let lat = pt.point().y();
                coords.push(format!("{lon} {lat}"));
            }
        }
    }

    if coords.len() < 2 {
        return None;
    }

    Some(format!("LINESTRING({})", coords.join(", ")))
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

/// Calculate average and maximum grade (slope) for a segment.
/// Returns (average_grade, max_grade) as percentages (e.g., 5.0 = 5% grade).
fn calculate_grades(points: &[SegmentPoint]) -> (Option<f64>, Option<f64>) {
    let mut max_grade: f64 = 0.0;
    let mut total_horizontal = 0.0;
    let mut total_vertical = 0.0;
    let mut has_data = false;

    for i in 1..points.len() {
        if let (Some(e1), Some(e2)) = (points[i - 1].ele, points[i].ele) {
            let horizontal = haversine_distance(
                points[i - 1].lat,
                points[i - 1].lon,
                points[i].lat,
                points[i].lon,
            );

            if horizontal > 1.0 {
                // Avoid division by very small numbers
                has_data = true;
                let vertical = e2 - e1;
                let grade = (vertical / horizontal) * 100.0;

                total_horizontal += horizontal;
                total_vertical += vertical;

                // Track max grade (absolute value for steepest section)
                if grade.abs() > max_grade.abs() {
                    max_grade = grade;
                }
            }
        }
    }

    if has_data && total_horizontal > 0.0 {
        let avg_grade = (total_vertical / total_horizontal) * 100.0;
        (Some(avg_grade), Some(max_grade))
    } else {
        (None, None)
    }
}

/// Calculate climb category based on elevation gain and distance.
/// Categories: 4 (easiest), 3, 2, 1, 0 (HC/hardest), None (not a climb)
/// Uses a points system: points = elevation_gain * (distance_km * grade_factor)
fn calculate_climb_category(
    elevation_gain: Option<f64>,
    distance_meters: f64,
    average_grade: Option<f64>,
) -> Option<i32> {
    let gain = elevation_gain?;
    let grade = average_grade?;

    // Only categorize actual climbs (positive elevation gain and grade)
    if gain < 20.0 || grade < 1.0 {
        return None;
    }

    let distance_km = distance_meters / 1000.0;

    // Grade factor increases difficulty for steeper climbs
    let grade_factor = if grade < 4.0 {
        1.0
    } else if grade < 6.0 {
        1.5
    } else if grade < 8.0 {
        2.0
    } else if grade < 10.0 {
        2.5
    } else {
        3.0
    };

    let points = gain * distance_km * grade_factor / 100.0;

    // Map points to category
    if points >= 320.0 {
        Some(0) // HC (Hors Catégorie)
    } else if points >= 160.0 {
        Some(1) // Cat 1
    } else if points >= 80.0 {
        Some(2) // Cat 2
    } else if points >= 40.0 {
        Some(3) // Cat 3
    } else if points >= 20.0 {
        Some(4) // Cat 4
    } else {
        None // Not categorized
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

pub async fn get_my_segment_efforts(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SegmentEffort>>, AppError> {
    let efforts = db.get_user_segment_efforts(claims.sub, id).await?;
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

        // Calculate average speed: distance / time
        let average_speed_mps = if timing.elapsed_time_seconds > 0.0 {
            Some(segment.distance_meters / timing.elapsed_time_seconds)
        } else {
            None
        };

        // Create the effort
        match db
            .create_segment_effort(
                segment_id,
                activity_match.activity_id,
                activity_match.user_id,
                timing.started_at,
                timing.elapsed_time_seconds,
                Some(timing.moving_time_seconds),
                average_speed_mps,
                None, // max_speed_mps
                Some(activity_match.start_fraction),
                Some(activity_match.end_fraction),
            )
            .await
        {
            Ok(effort) => {
                tracing::info!(
                    "Created segment effort {} for segment {} from activity {} with time {:.1}s (moving: {:.1}s)",
                    effort.id,
                    segment_id,
                    activity_match.activity_id,
                    timing.elapsed_time_seconds,
                    timing.moving_time_seconds
                );
                efforts_created += 1;

                // Update effort count
                if let Err(e) = db.increment_segment_effort_count(segment_id).await {
                    tracing::error!("Failed to increment effort count: {e}");
                }

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

// Segment star handlers

#[derive(Debug, Serialize)]
pub struct StarResponse {
    pub starred: bool,
}

/// Star a segment for the authenticated user.
pub async fn star_segment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<StarResponse>, AppError> {
    // Verify segment exists
    db.get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    db.star_segment(claims.sub, segment_id).await?;
    Ok(Json(StarResponse { starred: true }))
}

/// Unstar a segment for the authenticated user.
pub async fn unstar_segment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<StarResponse>, AppError> {
    db.unstar_segment(claims.sub, segment_id).await?;
    Ok(Json(StarResponse { starred: false }))
}

/// Check if a segment is starred by the authenticated user.
pub async fn is_segment_starred(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<StarResponse>, AppError> {
    let starred = db.is_segment_starred(claims.sub, segment_id).await?;
    Ok(Json(StarResponse { starred }))
}

/// Get all segments starred by the authenticated user.
pub async fn get_starred_segments(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<Segment>>, AppError> {
    let segments = db.get_user_starred_segments(claims.sub).await?;
    Ok(Json(segments))
}

/// Get all starred segments with effort stats for the authenticated user.
/// Returns each starred segment with the user's best effort, effort count, and leader time.
pub async fn get_starred_segment_efforts(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<crate::models::StarredSegmentEffort>>, AppError> {
    let efforts = db.get_starred_segments_with_efforts(claims.sub).await?;
    Ok(Json(efforts))
}

#[derive(Debug, Deserialize)]
pub struct NearbySegmentsQuery {
    lat: f64,
    lon: f64,
    radius_meters: Option<f64>,
    limit: Option<i64>,
}

pub async fn get_nearby_segments(
    Extension(db): Extension<Database>,
    Query(query): Query<NearbySegmentsQuery>,
) -> Result<Json<Vec<Segment>>, AppError> {
    let radius = query.radius_meters.unwrap_or(5000.0);
    let limit = query.limit.unwrap_or(20);
    let segments = db
        .find_segments_near_point(query.lat, query.lon, radius, limit)
        .await?;
    Ok(Json(segments))
}
