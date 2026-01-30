//! Segment management handlers.

use axum::{
    Extension,
    extract::{Path, Query},
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    achievements_service,
    auth::{AuthUser, OptionalAuthUser},
    database::Database,
    errors::AppError,
    models::{
        LeaderboardFilters, LeaderboardFiltersResponse, LeaderboardPosition, LeaderboardResponse,
        Segment, SegmentEffort, StarredSegmentEffort,
    },
    object_store_service::ObjectStoreService,
    segment_matching,
};

use super::activities::TrackBounds;
use super::pagination::default_limit;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSegmentRequest {
    pub name: String,
    pub description: Option<String>,
    /// Optional if source_activity_id is provided (inherits from the activity).
    /// Required if source_activity_id is not provided.
    pub activity_type_id: Option<Uuid>,
    pub points: Vec<SegmentPoint>,
    #[serde(default = "default_visibility")]
    pub visibility: String,
    /// Optional: the activity this segment was created from.
    /// If provided, the segment inherits its activity_type_id and guarantees that activity gets the first effort.
    pub source_activity_id: Option<Uuid>,
    /// Team IDs to share the segment with (for teams_only visibility)
    #[serde(default)]
    pub team_ids: Option<Vec<Uuid>>,
}

fn default_visibility() -> String {
    "public".to_string()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SegmentPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

#[utoipa::path(
    post,
    path = "/segments",
    tag = "segments",
    request_body = CreateSegmentRequest,
    responses(
        (status = 200, description = "Segment created successfully", body = Segment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_segment(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateSegmentRequest>,
) -> Result<Json<Segment>, AppError> {
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

    // Resolve activity_type_id: inherit from source activity if provided, otherwise require in request
    // Also store the source activity for later use (to save track geometry if needed)
    let (activity_type_id, source_activity) = if let Some(source_id) = req.source_activity_id {
        let activity = db.get_activity(source_id).await?.ok_or_else(|| {
            AppError::InvalidInput(format!("Source activity {source_id} not found"))
        })?;
        (activity.activity_type_id, Some(activity))
    } else {
        let activity_type_id = req.activity_type_id.ok_or_else(|| {
            AppError::InvalidInput(
                "activity_type_id is required when source_activity_id is not provided".to_string(),
            )
        })?;
        (activity_type_id, None)
    };

    // Check for duplicate segments (same activity type, similar start/end points)
    let similar_segments = db
        .find_similar_segments(activity_type_id, &start_wkt, &end_wkt)
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
            activity_type_id,
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
            if let Ok(file_bytes) = store.get_file(&activity.object_store_path).await
                && let Ok(gpx) = gpx::read(std::io::BufReader::new(file_bytes.as_ref()))
                && let Some(wkt) = build_track_wkt(&gpx)
            {
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

                    // Process achievements (KOM/QOM and Local Legend)
                    if let Err(e) = achievements_service::process_achievements(
                        &db,
                        segment_id,
                        activity_match.user_id,
                        effort.id,
                        timing.elapsed_time_seconds,
                    )
                    .await
                    {
                        tracing::error!("Failed to process achievements: {e}");
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to find matching activities for new segment: {e}");
        }
    }

    // Share with teams if team_ids provided
    if let Some(team_ids) = req.team_ids.as_ref().filter(|ids| !ids.is_empty()) {
        db.share_segment_with_teams(segment.id, team_ids).await?;
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
///
/// Uses the standard Strava formula: points = distance_meters * gradient_percent
/// This is mathematically equivalent to: elevation_gain_meters * 100
///
/// Categories (from easiest to hardest):
/// - Cat 4: 8,000+ points (roughly 80m+ gain at any gradient meeting minimum)
/// - Cat 3: 16,000+ points (roughly 160m+ gain)
/// - Cat 2: 32,000+ points (roughly 320m+ gain)
/// - Cat 1: 64,000+ points (roughly 640m+ gain)
/// - HC: 80,000+ points (roughly 800m+ gain, "beyond categorization")
///
/// Minimum requirements: 3% gradient and 8,000+ points score.
fn calculate_climb_category(
    elevation_gain: Option<f64>,
    distance_meters: f64,
    average_grade: Option<f64>,
) -> Option<i32> {
    // Require both elevation gain and grade data to exist
    let _ = elevation_gain?;
    let grade = average_grade?;

    // Minimum 3% gradient required for categorization
    if grade < 3.0 {
        return None;
    }

    // Standard Strava formula: distance (meters) * gradient (percentage)
    // Mathematically equivalent to elevation_gain * 100
    let points = distance_meters * grade;

    // Minimum score required for any category
    if points < 8000.0 {
        return None;
    }

    // Map points to category using standard thresholds
    if points >= 80000.0 {
        Some(0) // HC (Hors Catégorie) - "beyond categorization"
    } else if points >= 64000.0 {
        Some(1) // Cat 1 - long, demanding climbs (10+ km at 7-9%)
    } else if points >= 32000.0 {
        Some(2) // Cat 2 - significant climbs (5-10 km at 6-9%)
    } else if points >= 16000.0 {
        Some(3) // Cat 3 - moderate climbs (4-5 km at 6-8%)
    } else {
        Some(4) // Cat 4 - short climbs (1-3 km at 3-6%)
    }
}

#[utoipa::path(
    get,
    path = "/segments/{id}",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Segment details", body = Segment),
        (status = 404, description = "Segment not found")
    )
)]
pub async fn get_segment(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Segment>, AppError> {
    let segment = db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match segment.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().is_some_and(|c| c.sub == segment.creator_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                // Creator always has access
                if c.sub == segment.creator_id {
                    true
                } else {
                    // Check team membership
                    db.user_has_segment_team_access(c.sub, id).await?
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if has_access {
        Ok(Json(segment))
    } else {
        // Return 404 to avoid leaking existence
        Err(AppError::NotFound)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SegmentSortBy {
    #[default]
    CreatedAt,
    Name,
    Distance,
    ElevationGain,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClimbCategoryFilter {
    Hc,
    Cat1,
    Cat2,
    Cat3,
    Cat4,
    Flat,
}

impl ClimbCategoryFilter {
    /// Returns the database value for this filter.
    /// `None` means flat/uncategorized, `Some(n)` means category n (0=HC, 1-4=Cat 1-4).
    pub fn to_db_value(self) -> Option<i32> {
        match self {
            ClimbCategoryFilter::Hc => Some(0),
            ClimbCategoryFilter::Cat1 => Some(1),
            ClimbCategoryFilter::Cat2 => Some(2),
            ClimbCategoryFilter::Cat3 => Some(3),
            ClimbCategoryFilter::Cat4 => Some(4),
            ClimbCategoryFilter::Flat => None,
        }
    }

    /// Whether this filter matches flat/uncategorized segments.
    pub fn is_flat(self) -> bool {
        matches!(self, ClimbCategoryFilter::Flat)
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListSegmentsQuery {
    pub activity_type_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Case-insensitive name search
    pub search: Option<String>,
    /// Sort field
    #[serde(default)]
    pub sort_by: SegmentSortBy,
    /// Sort order
    #[serde(default)]
    pub sort_order: SortOrder,
    /// Minimum distance in meters
    pub min_distance_meters: Option<f64>,
    /// Maximum distance in meters
    pub max_distance_meters: Option<f64>,
    /// Filter by climb category
    pub climb_category: Option<ClimbCategoryFilter>,
}

#[utoipa::path(
    get,
    path = "/segments",
    tag = "segments",
    params(ListSegmentsQuery),
    responses(
        (status = 200, description = "List of segments", body = Vec<Segment>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_segments(
    Extension(db): Extension<Database>,
    Query(params): Query<ListSegmentsQuery>,
) -> Result<Json<Vec<Segment>>, AppError> {
    let segments = db.list_segments_filtered(&params).await?;
    Ok(Json(segments))
}

#[utoipa::path(
    get,
    path = "/segments/{id}/leaderboard",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Segment leaderboard", body = Vec<SegmentEffort>),
        (status = 404, description = "Segment not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_segment_leaderboard(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SegmentEffort>>, AppError> {
    let efforts = db.get_segment_efforts(id, 100).await?;
    Ok(Json(efforts))
}

#[utoipa::path(
    get,
    path = "/segments/{id}/efforts/me",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "User's efforts on segment", body = Vec<SegmentEffort>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Segment not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_segment_efforts(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SegmentEffort>>, AppError> {
    let efforts = db.get_user_segment_efforts(claims.sub, id).await?;
    Ok(Json(efforts))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SegmentTrackData {
    pub points: Vec<SegmentTrackPoint>,
    pub bounds: TrackBounds,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SegmentTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

#[utoipa::path(
    get,
    path = "/segments/{id}/track",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Segment track data with points and bounds", body = SegmentTrackData),
        (status = 401, description = "Unauthorized - private segment requires authentication"),
        (status = 404, description = "Segment not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_segment_track(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SegmentTrackData>, AppError> {
    // Verify segment exists and check access
    let segment = db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match segment.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().is_some_and(|c| c.sub == segment.creator_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                if c.sub == segment.creator_id {
                    true
                } else {
                    db.user_has_segment_team_access(c.sub, id).await?
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

// -- Segment Preview Endpoint --

#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewSegmentRequest {
    pub points: Vec<SegmentPoint>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreviewSegmentResponse {
    pub distance_meters: f64,
    pub elevation_gain_meters: Option<f64>,
    pub elevation_loss_meters: Option<f64>,
    pub average_grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub climb_category: Option<i32>,
    pub point_count: usize,
    pub validation: SegmentValidation,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SegmentValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Calculate segment metrics from a list of points without creating the segment.
/// Useful for previewing what a segment would look like before creation.
#[utoipa::path(
    post,
    path = "/segments/preview",
    tag = "segments",
    request_body = PreviewSegmentRequest,
    responses(
        (status = 200, description = "Preview of segment metrics and validation", body = PreviewSegmentResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn preview_segment(
    Json(req): Json<PreviewSegmentRequest>,
) -> Result<Json<PreviewSegmentResponse>, AppError> {
    let mut errors = Vec::new();

    // Validation checks (same as create_segment but we collect all errors)
    const MIN_POINTS: usize = 10;
    if req.points.len() < MIN_POINTS {
        errors.push(format!(
            "Segment must have at least {MIN_POINTS} points (got {})",
            req.points.len()
        ));
    }

    // Calculate distance
    let distance_meters = calculate_total_distance(&req.points);

    // Validation: minimum length (100m)
    const MIN_LENGTH_METERS: f64 = 100.0;
    if distance_meters < MIN_LENGTH_METERS {
        errors.push(format!(
            "Segment must be at least {MIN_LENGTH_METERS}m long (got {:.0}m)",
            distance_meters
        ));
    }

    // Validation: maximum length (50km)
    const MAX_LENGTH_METERS: f64 = 50_000.0;
    if distance_meters > MAX_LENGTH_METERS {
        errors.push(format!(
            "Segment must be at most {}km long (got {:.1}km)",
            MAX_LENGTH_METERS / 1000.0,
            distance_meters / 1000.0
        ));
    }

    // Calculate elevation metrics
    let (elevation_gain, elevation_loss) = calculate_elevation_change(&req.points);

    // Calculate grades
    let (average_grade, max_grade) = calculate_grades(&req.points);

    // Calculate climb category
    let climb_category = calculate_climb_category(elevation_gain, distance_meters, average_grade);

    Ok(Json(PreviewSegmentResponse {
        distance_meters,
        elevation_gain_meters: elevation_gain,
        elevation_loss_meters: elevation_loss,
        average_grade,
        max_grade,
        climb_category,
        point_count: req.points.len(),
        validation: SegmentValidation {
            is_valid: errors.is_empty(),
            errors,
        },
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReprocessResult {
    pub segment_id: Uuid,
    pub activities_checked: usize,
    pub efforts_created: usize,
}

/// Reprocess all activities to find matches for a specific segment.
/// This is useful when a new segment is created and we want to find
/// all existing activities that pass through it.
#[utoipa::path(
    post,
    path = "/segments/{id}/reprocess",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Reprocessing results", body = ReprocessResult),
        (status = 404, description = "Segment not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn reprocess_segment(
    Extension(db): Extension<Database>,
    Extension(store): Extension<ObjectStoreService>,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<ReprocessResult>, AppError> {
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

                // Process achievements (KOM/QOM and Local Legend)
                if let Err(e) = achievements_service::process_achievements(
                    &db,
                    segment_id,
                    activity_match.user_id,
                    effort.id,
                    timing.elapsed_time_seconds,
                )
                .await
                {
                    tracing::error!("Failed to process achievements: {e}");
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

#[derive(Debug, Serialize, ToSchema)]
pub struct StarResponse {
    pub starred: bool,
}

/// Star a segment for the authenticated user.
#[utoipa::path(
    post,
    path = "/segments/{id}/star",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Segment starred successfully", body = StarResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Segment not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
#[utoipa::path(
    delete,
    path = "/segments/{id}/star",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Segment unstarred successfully", body = StarResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unstar_segment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<StarResponse>, AppError> {
    db.unstar_segment(claims.sub, segment_id).await?;
    Ok(Json(StarResponse { starred: false }))
}

/// Check if a segment is starred by the authenticated user.
#[utoipa::path(
    get,
    path = "/segments/{id}/starred",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID")
    ),
    responses(
        (status = 200, description = "Starred status", body = StarResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn is_segment_starred(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<StarResponse>, AppError> {
    let starred = db.is_segment_starred(claims.sub, segment_id).await?;
    Ok(Json(StarResponse { starred }))
}

/// Get all segments starred by the authenticated user.
#[utoipa::path(
    get,
    path = "/segments/starred",
    tag = "segments",
    responses(
        (status = 200, description = "List of starred segments", body = Vec<Segment>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_starred_segments(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<Segment>>, AppError> {
    let segments = db.get_user_starred_segments(claims.sub).await?;
    Ok(Json(segments))
}

/// Get all starred segments with effort stats for the authenticated user.
/// Returns each starred segment with the user's best effort, effort count, and leader time.
#[utoipa::path(
    get,
    path = "/segments/starred/efforts",
    tag = "segments",
    responses(
        (status = 200, description = "List of starred segments with effort stats", body = Vec<StarredSegmentEffort>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_starred_segment_efforts(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<crate::models::StarredSegmentEffort>>, AppError> {
    let efforts = db.get_starred_segments_with_efforts(claims.sub).await?;
    Ok(Json(efforts))
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct NearbySegmentsQuery {
    lat: f64,
    lon: f64,
    radius_meters: Option<f64>,
    limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/segments/nearby",
    tag = "segments",
    params(NearbySegmentsQuery),
    responses(
        (status = 200, description = "List of nearby segments", body = Vec<Segment>),
        (status = 500, description = "Internal server error")
    )
)]
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

// ============================================================================
// Filtered Leaderboard Handlers
// ============================================================================

/// Get filtered leaderboard for a segment.
/// Supports time scope, gender, and age group filtering.
#[utoipa::path(
    get,
    path = "/segments/{id}/leaderboard/filtered",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID"),
        LeaderboardFilters
    ),
    responses(
        (status = 200, description = "Filtered leaderboard", body = LeaderboardResponse),
        (status = 404, description = "Segment not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_filtered_leaderboard(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
    Query(filters): Query<LeaderboardFilters>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    // Verify segment exists
    db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    let (entries, total_count) = db.get_filtered_leaderboard(id, &filters).await?;

    Ok(Json(LeaderboardResponse {
        entries,
        total_count,
        filters: LeaderboardFiltersResponse {
            scope: filters.scope,
            gender: filters.gender,
            age_group: filters.age_group,
            weight_class: filters.weight_class,
            country: filters.country.clone(),
            limit: filters.limit,
            offset: filters.offset,
        },
    }))
}

/// Get the authenticated user's position in a segment leaderboard.
#[utoipa::path(
    get,
    path = "/segments/{id}/leaderboard/position",
    tag = "segments",
    params(
        ("id" = Uuid, Path, description = "Segment ID"),
        LeaderboardFilters
    ),
    responses(
        (status = 200, description = "User's leaderboard position with surrounding entries", body = LeaderboardPosition),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Segment not found or user has no efforts"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_leaderboard_position(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Query(filters): Query<LeaderboardFilters>,
) -> Result<Json<LeaderboardPosition>, AppError> {
    // Verify segment exists
    db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    // Get user's position with 3 entries above and below
    let result = db
        .get_user_leaderboard_position(id, claims.sub, &filters, 3)
        .await?;

    match result {
        Some((user_entry, entries_above, entries_below, total_count)) => {
            Ok(Json(LeaderboardPosition {
                user_rank: user_entry.rank,
                user_entry,
                entries_above,
                entries_below,
                total_count,
            }))
        }
        None => Err(AppError::NotFound),
    }
}
