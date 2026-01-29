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
    achievements_service,
    activity_queue::ActivityQueue,
    auth::{AuthUser, OptionalAuthUser},
    database::Database,
    errors::AppError,
    models::{
        AchievementType, AchievementWithSegment, Activity, ChangeMemberRoleRequest,
        CountryStats, CreateTeamRequest, CrownCountEntry, DistanceLeaderEntry, InviteToTeamRequest,
        JoinTeamRequest, LeaderboardFilters, LeaderboardFiltersResponse, LeaderboardPosition,
        LeaderboardResponse, Segment, SegmentAchievements, SegmentEffort, ShareWithTeamsRequest,
        Stats, Team, TeamInvitationWithDetails, TeamMember, TeamRole, TeamSummary,
        TeamWithMembership, UpdateDemographicsRequest, UpdateTeamRequest, User,
        UserWithDemographics,
    },
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
    pub activity_type_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub visibility: Option<String>,
    /// Comma-separated list of team IDs to share with (for teams_only visibility)
    #[serde(default)]
    pub team_ids: Option<String>,
    /// Multi-sport: JSON array of ISO-8601 timestamps marking segment boundaries
    #[serde(default)]
    pub type_boundaries: Option<Vec<time::OffsetDateTime>>,
    /// Multi-sport: JSON array of activity type UUIDs for each segment
    #[serde(default)]
    pub segment_types: Option<Vec<Uuid>>,
}

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

    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        name,
        activity_type_id,
        submitted_at: time::UtcDateTime::now().to_offset(time::UtcOffset::UTC),
        object_store_path,
        visibility: params.visibility.unwrap_or_else(|| "public".to_string()),
        type_boundaries: params.type_boundaries,
        segment_types: params.segment_types,
    };

    aq.submit(
        user_id,
        activity.id,
        file_type,
        file_bytes,
        activity.activity_type_id,
        activity.type_boundaries.clone(),
        activity.segment_types.clone(),
    )
    .map_err(AppError::Queue)?;

    db.save_activity(&activity).await?;

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

pub async fn get_activity(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Activity>, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().map_or(false, |c| c.sub == activity.user_id),
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

#[derive(Debug, Deserialize)]
pub struct UpdateActivityRequest {
    pub name: Option<String>,
    pub activity_type_id: Option<Uuid>,
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
            req.activity_type_id,
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
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().map_or(false, |c| c.sub == activity.user_id),
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
        "private" => claims.as_ref().map_or(false, |c| c.sub == activity.user_id),
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
        "private" => claims.as_ref().map_or(false, |c| c.sub == activity.user_id),
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

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// ============================================================================
// Activity Type handlers
// ============================================================================

/// List all activity types (built-in and custom).
pub async fn list_activity_types(
    Extension(db): Extension<Database>,
) -> Result<Json<Vec<crate::models::ActivityTypeRow>>, AppError> {
    let types = db.list_activity_types().await?;
    Ok(Json(types))
}

/// Get a single activity type by ID.
pub async fn get_activity_type(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::ActivityTypeRow>, AppError> {
    let activity_type = db.get_activity_type(id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(activity_type))
}

/// Create a custom activity type.
pub async fn create_activity_type(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Json(req): Json<crate::models::CreateActivityTypeRequest>,
) -> Result<Json<crate::models::ActivityTypeRow>, AppError> {
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

/// Resolve an activity type by name or alias.
#[derive(Debug, Deserialize)]
pub struct ResolveTypeQuery {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ResolveTypeResponse {
    pub result: String, // "exact", "ambiguous", "not_found"
    pub type_id: Option<Uuid>,
    pub type_ids: Option<Vec<Uuid>>,
}

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

// ============================================================================
// Segment handlers
// ============================================================================

#[derive(Debug, Deserialize)]
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
    if let Some(team_ids) = &req.team_ids {
        if !team_ids.is_empty() {
            db.share_segment_with_teams(segment.id, team_ids).await?;
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
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Segment>, AppError> {
    let segment = db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match segment.visibility.as_str() {
        "public" => true,
        "private" => claims
            .as_ref()
            .map_or(false, |c| c.sub == segment.creator_id),
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

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SegmentSortBy {
    #[default]
    CreatedAt,
    Name,
    Distance,
    ElevationGain,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Copy, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

fn default_limit() -> i64 {
    50
}

pub async fn list_segments(
    Extension(db): Extension<Database>,
    Query(params): Query<ListSegmentsQuery>,
) -> Result<Json<Vec<Segment>>, AppError> {
    let segments = db.list_segments_filtered(&params).await?;
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
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SegmentTrackData>, AppError> {
    // Verify segment exists and check access
    let segment = db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    // Check visibility-based access control
    let has_access = match segment.visibility.as_str() {
        "public" => true,
        "private" => claims
            .as_ref()
            .map_or(false, |c| c.sub == segment.creator_id),
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

#[derive(Debug, Deserialize)]
pub struct PreviewSegmentRequest {
    pub points: Vec<SegmentPoint>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct SegmentValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Calculate segment metrics from a list of points without creating the segment.
/// Useful for previewing what a segment would look like before creation.
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

// ============================================================================
// Enhanced Leaderboard Handlers
// ============================================================================

/// Get filtered leaderboard for a segment.
/// Supports time scope, gender, and age group filtering.
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

// ============================================================================
// User Demographics Handlers
// ============================================================================

/// Get the authenticated user's profile with demographics.
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
pub async fn update_my_demographics(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Json(req): Json<UpdateDemographicsRequest>,
) -> Result<Json<UserWithDemographics>, AppError> {
    let user = db.update_user_demographics(claims.sub, &req).await?;
    Ok(Json(user))
}

// ============================================================================
// Achievement Handlers
// ============================================================================

/// Get achievements for a specific user.
#[derive(Debug, Deserialize)]
pub struct GetAchievementsQuery {
    #[serde(default)]
    pub include_lost: bool,
}

pub async fn get_user_achievements(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<GetAchievementsQuery>,
) -> Result<Json<Vec<AchievementWithSegment>>, AppError> {
    let achievements = db
        .get_user_achievements(user_id, query.include_lost)
        .await?;
    Ok(Json(achievements))
}

/// Get the authenticated user's achievements.
pub async fn get_my_achievements(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Query(query): Query<GetAchievementsQuery>,
) -> Result<Json<Vec<AchievementWithSegment>>, AppError> {
    let achievements = db
        .get_user_achievements(claims.sub, query.include_lost)
        .await?;
    Ok(Json(achievements))
}

/// Get current achievement holders for a segment.
pub async fn get_segment_achievements(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<SegmentAchievements>, AppError> {
    // Verify segment exists
    db.get_segment(id).await?.ok_or(AppError::NotFound)?;

    let kom = db
        .get_current_achievement_holder(id, AchievementType::Kom)
        .await?;
    let qom = db
        .get_current_achievement_holder(id, AchievementType::Qom)
        .await?;
    let local_legend = db
        .get_current_achievement_holder(id, AchievementType::LocalLegend)
        .await?;

    Ok(Json(SegmentAchievements {
        segment_id: id,
        kom,
        qom,
        local_legend,
    }))
}

// ============================================================================
// Global Leaderboard Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GlobalLeaderboardQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Get global crown count leaderboard.
pub async fn get_crown_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<CrownCountEntry>>, AppError> {
    let entries = db
        .get_crown_count_leaderboard(query.limit, query.offset)
        .await?;
    Ok(Json(entries))
}

/// Get global distance leaderboard.
pub async fn get_distance_leaderboard(
    Extension(db): Extension<Database>,
    Query(query): Query<GlobalLeaderboardQuery>,
) -> Result<Json<Vec<DistanceLeaderEntry>>, AppError> {
    let entries = db
        .get_distance_leaderboard(query.limit, query.offset)
        .await?;
    Ok(Json(entries))
}

/// Get list of countries with user counts for the filter dropdown.
pub async fn get_countries(
    Extension(db): Extension<Database>,
) -> Result<Json<Vec<CountryStats>>, AppError> {
    let countries = db.get_countries_with_counts().await?;
    Ok(Json(countries))
}

// ============================================================================
// Social Handlers (Follows)
// ============================================================================

/// Follow a user.
pub async fn follow_user(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Can't follow yourself
    if claims.sub == user_id {
        return Err(AppError::InvalidInput("Cannot follow yourself".to_string()));
    }

    // Verify target user exists
    db.get_user(user_id).await?.ok_or(AppError::NotFound)?;

    // Check if already following
    if db.is_following(claims.sub, user_id).await? {
        return Ok(StatusCode::OK); // Idempotent
    }

    db.follow_user(claims.sub, user_id).await?;

    // Create notification for the followed user
    db.create_notification(
        user_id,
        "follow",
        Some(claims.sub),
        Some("user"),
        Some(claims.sub),
        None,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

/// Unfollow a user.
pub async fn unfollow_user(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let unfollowed = db.unfollow_user(claims.sub, user_id).await?;

    if unfollowed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

#[derive(Debug, Serialize)]
pub struct FollowStatusResponse {
    pub is_following: bool,
}

/// Check if the authenticated user is following a specific user.
pub async fn get_follow_status(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<FollowStatusResponse>, AppError> {
    let is_following = db.is_following(claims.sub, user_id).await?;
    Ok(Json(FollowStatusResponse { is_following }))
}

#[derive(Debug, Deserialize)]
pub struct FollowListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct FollowListResponse {
    pub users: Vec<crate::models::UserSummary>,
    pub total_count: i32,
}

/// Get a user's followers.
pub async fn get_followers(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<FollowListQuery>,
) -> Result<Json<FollowListResponse>, AppError> {
    // Get follow counts
    let (follower_count, _) = db.get_follow_counts(user_id).await?;

    let followers = db.get_followers(user_id, query.limit, query.offset).await?;

    Ok(Json(FollowListResponse {
        users: followers,
        total_count: follower_count,
    }))
}

/// Get users that a user is following.
pub async fn get_following(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<FollowListQuery>,
) -> Result<Json<FollowListResponse>, AppError> {
    // Get follow counts
    let (_, following_count) = db.get_follow_counts(user_id).await?;

    let following = db.get_following(user_id, query.limit, query.offset).await?;

    Ok(Json(FollowListResponse {
        users: following,
        total_count: following_count,
    }))
}

/// Get a user's profile with follow counts.
pub async fn get_user_profile(
    Extension(db): Extension<Database>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<crate::models::UserProfile>, AppError> {
    let profile = db
        .get_user_profile(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(profile))
}

// ============================================================================
// Notification Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct NotificationsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Get notifications for the authenticated user.
pub async fn get_notifications(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<crate::models::NotificationsResponse>, AppError> {
    let notifications = db
        .get_notifications(claims.sub, query.limit, query.offset)
        .await?;
    let unread_count = db.get_unread_notification_count(claims.sub).await?;
    let total_count = notifications.len() as i64;

    Ok(Json(crate::models::NotificationsResponse {
        notifications,
        unread_count,
        total_count,
    }))
}

/// Mark a notification as read.
pub async fn mark_notification_read(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let marked = db
        .mark_notification_read(notification_id, claims.sub)
        .await?;

    if marked {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

/// Mark all notifications as read.
pub async fn mark_all_notifications_read(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = db.mark_all_notifications_read(claims.sub).await?;
    Ok(Json(serde_json::json!({ "marked_count": count })))
}

// ============================================================================
// Activity Feed Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Get the activity feed for the authenticated user.
/// Returns activities from users they follow.
pub async fn get_feed(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Query(query): Query<FeedQuery>,
) -> Result<Json<Vec<crate::models::FeedActivity>>, AppError> {
    let activities = db
        .get_activity_feed(claims.sub, query.limit, query.offset)
        .await?;
    Ok(Json(activities))
}

// ============================================================================
// Kudos Handlers
// ============================================================================

#[derive(Debug, Serialize)]
pub struct KudosResponse {
    pub given: bool,
    pub kudos_count: i32,
}

/// Give kudos to an activity.
pub async fn give_kudos(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<KudosResponse>, AppError> {
    // Get activity to check it exists and get owner
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Can't give kudos to your own activity
    if activity.user_id == claims.sub {
        return Err(AppError::InvalidInput(
            "Cannot give kudos to your own activity".to_string(),
        ));
    }

    let was_new = db.give_kudos(claims.sub, activity_id).await?;

    // Create notification if this is a new kudos
    if was_new {
        db.create_notification(
            activity.user_id,
            "kudos",
            Some(claims.sub),
            Some("activity"),
            Some(activity_id),
            None,
        )
        .await?;
    }

    // Get updated count
    let _activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(KudosResponse {
        given: true,
        kudos_count: 0, // We don't have kudos_count in Activity struct yet
    }))
}

/// Remove kudos from an activity.
pub async fn remove_kudos(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    db.remove_kudos(claims.sub, activity_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct KudosStatusResponse {
    pub has_given: bool,
}

/// Check if user has given kudos to an activity.
pub async fn get_kudos_status(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<KudosStatusResponse>, AppError> {
    let has_given = db.has_given_kudos(claims.sub, activity_id).await?;
    Ok(Json(KudosStatusResponse { has_given }))
}

/// Get users who gave kudos to an activity.
pub async fn get_kudos_givers(
    Extension(db): Extension<Database>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::KudosGiver>>, AppError> {
    let givers = db.get_kudos_givers(activity_id, 100).await?;
    Ok(Json(givers))
}

// ============================================================================
// Comments Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub content: String,
    pub parent_id: Option<Uuid>,
}

/// Add a comment to an activity.
pub async fn add_comment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
    Json(req): Json<AddCommentRequest>,
) -> Result<Json<crate::models::CommentWithUser>, AppError> {
    // Verify activity exists
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let comment = db
        .add_comment(claims.sub, activity_id, &req.content, req.parent_id)
        .await?;

    // Get user name for response
    let user = db.get_user(claims.sub).await?.ok_or(AppError::NotFound)?;

    // Create notification if commenting on someone else's activity
    if activity.user_id != claims.sub {
        db.create_notification(
            activity.user_id,
            "comment",
            Some(claims.sub),
            Some("activity"),
            Some(activity_id),
            Some(&req.content),
        )
        .await?;
    }

    Ok(Json(crate::models::CommentWithUser {
        id: comment.id,
        user_id: comment.user_id,
        activity_id: comment.activity_id,
        parent_id: comment.parent_id,
        content: comment.content,
        created_at: comment.created_at,
        updated_at: comment.updated_at,
        user_name: user.name,
    }))
}

/// Get comments for an activity.
pub async fn get_comments(
    Extension(db): Extension<Database>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::CommentWithUser>>, AppError> {
    let comments = db.get_comments(activity_id).await?;
    Ok(Json(comments))
}

/// Delete a comment.
pub async fn delete_comment(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(comment_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = db.delete_comment(comment_id, claims.sub).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// Stats Handlers
// ============================================================================

/// Get platform-wide statistics (active users, segments created, activities uploaded).
pub async fn get_stats(Extension(db): Extension<Database>) -> Result<Json<Stats>, AppError> {
    let stats = db.get_stats().await?;
    Ok(Json(stats))
}

// ============================================================================
// Team Handlers
// ============================================================================

/// Create a new team.
pub async fn create_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<Team>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Team name cannot be empty".to_string(),
        ));
    }

    let team = db
        .create_team(
            &req.name,
            req.description.as_deref(),
            req.avatar_url.as_deref(),
            req.visibility,
            req.join_policy,
            claims.sub,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(team)))
}

/// Get a team by ID (with membership context for the current user).
pub async fn get_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamWithMembership>, AppError> {
    let team = db
        .get_team_with_membership(id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    // Private teams are only visible to members
    if team.team.visibility == crate::models::TeamVisibility::Private && !team.is_member {
        return Err(AppError::NotFound);
    }

    Ok(Json(team))
}

/// List teams the authenticated user is a member of.
pub async fn list_my_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<TeamWithMembership>>, AppError> {
    let teams = db.list_user_teams(claims.sub).await?;
    Ok(Json(teams))
}

#[derive(Debug, Deserialize)]
pub struct DiscoverTeamsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// List discoverable teams.
pub async fn discover_teams(
    Extension(db): Extension<Database>,
    Query(query): Query<DiscoverTeamsQuery>,
) -> Result<Json<Vec<TeamSummary>>, AppError> {
    let teams = db
        .list_discoverable_teams(query.limit, query.offset)
        .await?;
    Ok(Json(teams))
}

/// Update a team's settings.
pub async fn update_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<Json<Team>, AppError> {
    // Check membership and role
    let membership = db
        .get_team_membership(id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_modify_team() {
        return Err(AppError::Forbidden);
    }

    let team = db
        .update_team(
            id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.avatar_url.as_deref(),
            req.visibility,
            req.join_policy,
        )
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(team))
}

/// Delete a team.
pub async fn delete_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check membership and role
    let membership = db
        .get_team_membership(id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_delete_team() {
        return Err(AppError::Forbidden);
    }

    if db.delete_team(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// Team Membership Handlers
// ============================================================================

/// List members of a team.
pub async fn list_team_members(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    // Check if user is a member (only members can see member list)
    let membership = db.get_team_membership(team_id, claims.sub).await?;
    if membership.is_none() {
        // Check if team exists and is public
        let team = db.get_team(team_id).await?.ok_or(AppError::NotFound)?;
        if team.visibility == crate::models::TeamVisibility::Private {
            return Err(AppError::NotFound);
        }
    }

    let members = db.list_team_members(team_id).await?;
    Ok(Json(members))
}

/// Remove a member from a team (admin/owner) or leave team (self).
pub async fn remove_team_member(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let my_membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let is_self = claims.sub == user_id;

    // Self-removal (leaving) is always allowed for non-owners
    if is_self {
        if my_membership.role == TeamRole::Owner {
            return Err(AppError::InvalidInput(
                "Owner cannot leave the team. Transfer ownership first or delete the team."
                    .to_string(),
            ));
        }
    } else {
        // Removing someone else requires admin/owner role
        if !my_membership.role.can_manage_members() {
            return Err(AppError::Forbidden);
        }

        // Check target membership
        let target_membership = db
            .get_team_membership(team_id, user_id)
            .await?
            .ok_or(AppError::NotFound)?;

        // Can't remove someone with equal or higher role
        if target_membership.role == TeamRole::Owner {
            return Err(AppError::InvalidInput(
                "Cannot remove the team owner".to_string(),
            ));
        }
        if target_membership.role == TeamRole::Admin && my_membership.role == TeamRole::Admin {
            return Err(AppError::InvalidInput(
                "Admins cannot remove other admins".to_string(),
            ));
        }
    }

    if db.remove_team_member(team_id, user_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

/// Change a member's role.
pub async fn change_member_role(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ChangeMemberRoleRequest>,
) -> Result<Json<crate::models::TeamMembership>, AppError> {
    let my_membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !my_membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    // Only owners can change roles to/from owner
    if req.role == TeamRole::Owner || my_membership.role != TeamRole::Owner {
        if req.role == TeamRole::Owner {
            return Err(AppError::InvalidInput(
                "Use transfer ownership instead".to_string(),
            ));
        }
    }

    // Can't demote another owner
    let target_membership = db
        .get_team_membership(team_id, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_membership.role == TeamRole::Owner && claims.sub != user_id {
        return Err(AppError::InvalidInput(
            "Cannot change the owner's role".to_string(),
        ));
    }

    let membership = db
        .change_team_member_role(team_id, user_id, req.role)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(membership))
}

// ============================================================================
// Team Join Handlers
// ============================================================================

/// Request to join a team (for request-based teams) or join directly (for open teams).
pub async fn join_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<JoinTeamRequest>,
) -> Result<StatusCode, AppError> {
    let team = db.get_team(team_id).await?.ok_or(AppError::NotFound)?;

    // Check if already a member
    if db.get_team_membership(team_id, claims.sub).await?.is_some() {
        return Err(AppError::InvalidInput(
            "Already a member of this team".to_string(),
        ));
    }

    match team.join_policy {
        crate::models::TeamJoinPolicy::Open => {
            // Directly add as member
            db.add_team_member(team_id, claims.sub, TeamRole::Member, None)
                .await?;
            Ok(StatusCode::CREATED)
        }
        crate::models::TeamJoinPolicy::Request => {
            // Create a join request
            db.create_team_join_request(team_id, claims.sub, req.message.as_deref())
                .await?;
            Ok(StatusCode::ACCEPTED)
        }
        crate::models::TeamJoinPolicy::Invitation => Err(AppError::InvalidInput(
            "This team is invitation-only".to_string(),
        )),
    }
}

/// Leave a team.
pub async fn leave_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if membership.role == TeamRole::Owner {
        return Err(AppError::InvalidInput(
            "Owner cannot leave the team. Transfer ownership first or delete the team.".to_string(),
        ));
    }

    if db.remove_team_member(team_id, claims.sub).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

/// Get pending join requests for a team (admin only).
pub async fn get_join_requests(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::TeamJoinRequestWithUser>>, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    let requests = db.get_pending_join_requests(team_id).await?;
    Ok(Json(requests))
}

#[derive(Debug, Deserialize)]
pub struct ReviewJoinRequestRequest {
    pub approved: bool,
}

/// Approve or reject a join request.
pub async fn review_join_request(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, request_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ReviewJoinRequestRequest>,
) -> Result<StatusCode, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    // Get the join request
    let join_request = db
        .get_join_request(request_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if join_request.team_id != team_id {
        return Err(AppError::NotFound);
    }

    let result = db
        .review_join_request(request_id, claims.sub, req.approved)
        .await?;

    if let Some(reviewed_request) = result {
        if req.approved {
            // Add the user as a member
            db.add_team_member(
                team_id,
                reviewed_request.user_id,
                TeamRole::Member,
                Some(claims.sub),
            )
            .await?;

            // Notify the user they were accepted
            db.create_notification(
                reviewed_request.user_id,
                "team_join_approved",
                Some(claims.sub),
                Some("team"),
                Some(team_id),
                None,
            )
            .await?;
        } else {
            // Notify the user they were rejected
            db.create_notification(
                reviewed_request.user_id,
                "team_join_rejected",
                Some(claims.sub),
                Some("team"),
                Some(team_id),
                None,
            )
            .await?;
        }
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// Team Invitation Handlers
// ============================================================================

/// Create an invitation to join a team.
pub async fn invite_to_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<InviteToTeamRequest>,
) -> Result<(StatusCode, Json<crate::models::TeamInvitation>), AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    // Only owners can invite as admin
    if req.role == TeamRole::Admin && membership.role != TeamRole::Owner {
        return Err(AppError::InvalidInput(
            "Only owners can invite admins".to_string(),
        ));
    }

    // Can't invite as owner
    if req.role == TeamRole::Owner {
        return Err(AppError::InvalidInput("Cannot invite as owner".to_string()));
    }

    // Generate a secure token
    let token = Uuid::new_v4().to_string();

    // Set expiry to 7 days from now
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(7);

    let invitation = db
        .create_team_invitation(
            team_id, &req.email, claims.sub, req.role, &token, expires_at,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(invitation)))
}

/// Get pending invitations for a team (admin only).
pub async fn get_team_invitations(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::TeamInvitation>>, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    let invitations = db.get_pending_invitations(team_id).await?;
    Ok(Json(invitations))
}

/// Revoke an invitation.
pub async fn revoke_invitation(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    if db.revoke_invitation(invitation_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

/// Get invitation details by token.
pub async fn get_invitation(
    Extension(db): Extension<Database>,
    Path(token): Path<String>,
) -> Result<Json<TeamInvitationWithDetails>, AppError> {
    let invitation = db
        .get_invitation_by_token(&token)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check if expired
    if invitation.expires_at < time::OffsetDateTime::now_utc() {
        return Err(AppError::InvalidInput("Invitation has expired".to_string()));
    }

    Ok(Json(invitation))
}

/// Accept an invitation.
pub async fn accept_invitation(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(token): Path<String>,
) -> Result<StatusCode, AppError> {
    // Get invitation details
    let invitation_details = db
        .get_invitation_by_token(&token)
        .await?
        .ok_or(AppError::NotFound)?;

    // Mark invitation as accepted
    let invitation = db
        .accept_invitation(&token)
        .await?
        .ok_or(AppError::InvalidInput(
            "Invitation is invalid or has expired".to_string(),
        ))?;

    // Check if already a member
    if db
        .get_team_membership(invitation.team_id, claims.sub)
        .await?
        .is_some()
    {
        return Err(AppError::InvalidInput(
            "Already a member of this team".to_string(),
        ));
    }

    // Add as member with the invited role
    db.add_team_member(
        invitation.team_id,
        claims.sub,
        invitation.role,
        Some(invitation.invited_by),
    )
    .await?;

    // Notify the inviter
    db.create_notification(
        invitation.invited_by,
        "team_invite_accepted",
        Some(claims.sub),
        Some("team"),
        Some(invitation.team_id),
        Some(&invitation_details.team_name),
    )
    .await?;

    Ok(StatusCode::CREATED)
}

// ============================================================================
// Activity-Team Sharing Handlers
// ============================================================================

/// Get teams an activity is shared with.
pub async fn get_activity_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Vec<TeamSummary>>, AppError> {
    // Verify activity exists and user has access
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Only owner or team members can see sharing
    let is_owner = activity.user_id == claims.sub;
    let has_team_access = db
        .user_has_activity_team_access(claims.sub, activity_id)
        .await?;

    if !is_owner && !has_team_access && activity.visibility != "public" {
        return Err(AppError::NotFound);
    }

    let teams = db.get_activity_teams(activity_id).await?;
    Ok(Json(teams))
}

/// Share an activity with teams.
pub async fn share_activity_with_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
    Json(req): Json<ShareWithTeamsRequest>,
) -> Result<StatusCode, AppError> {
    // Verify activity exists and user is owner
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if activity.user_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    // Verify user is a member of all target teams
    for team_id in &req.team_ids {
        if db
            .get_team_membership(*team_id, claims.sub)
            .await?
            .is_none()
        {
            return Err(AppError::InvalidInput(format!(
                "You are not a member of team {team_id}"
            )));
        }
    }

    db.share_activity_with_teams(activity_id, &req.team_ids, claims.sub)
        .await?;

    Ok(StatusCode::OK)
}

/// Unshare an activity from a team.
pub async fn unshare_activity_from_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((activity_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Verify activity exists and user is owner
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if activity.user_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    if db.unshare_activity_from_team(activity_id, team_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// Segment-Team Sharing Handlers
// ============================================================================

/// Get teams a segment is shared with.
pub async fn get_segment_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<Vec<TeamSummary>>, AppError> {
    // Verify segment exists
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Only creator or team members can see sharing
    let is_creator = segment.creator_id == claims.sub;
    let has_team_access = db
        .user_has_segment_team_access(claims.sub, segment_id)
        .await?;

    if !is_creator && !has_team_access && segment.visibility != "public" {
        return Err(AppError::NotFound);
    }

    let teams = db.get_segment_teams(segment_id).await?;
    Ok(Json(teams))
}

/// Share a segment with teams.
pub async fn share_segment_with_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
    Json(req): Json<ShareWithTeamsRequest>,
) -> Result<StatusCode, AppError> {
    // Verify segment exists and user is creator
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if segment.creator_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    // Verify user is a member of all target teams
    for team_id in &req.team_ids {
        if db
            .get_team_membership(*team_id, claims.sub)
            .await?
            .is_none()
        {
            return Err(AppError::InvalidInput(format!(
                "You are not a member of team {team_id}"
            )));
        }
    }

    db.share_segment_with_teams(segment_id, &req.team_ids)
        .await?;

    Ok(StatusCode::OK)
}

/// Unshare a segment from a team.
pub async fn unshare_segment_from_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((segment_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Verify segment exists and user is creator
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if segment.creator_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    if db.unshare_segment_from_team(segment_id, team_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// Team Content Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TeamContentQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Get activities shared with a team.
pub async fn get_team_activities(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamContentQuery>,
) -> Result<Json<Vec<crate::models::FeedActivity>>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let activities = db
        .get_team_activities(team_id, query.limit, query.offset)
        .await?;
    Ok(Json(activities))
}

/// Get segments shared with a team.
pub async fn get_team_segments(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamContentQuery>,
) -> Result<Json<Vec<Segment>>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let segments = db
        .get_team_segments(team_id, query.limit, query.offset)
        .await?;
    Ok(Json(segments))
}
