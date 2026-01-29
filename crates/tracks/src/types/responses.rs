//! Response types for API endpoints.

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

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

/// Activity type resolution response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ResolveTypeResponse {
    /// "exact", "ambiguous", or "not_found"
    pub result: String,
    pub type_id: Option<Uuid>,
    pub type_ids: Option<Vec<Uuid>>,
}

/// Segment track point.
#[derive(Debug, Serialize, ToSchema)]
pub struct SegmentTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

/// Segment track data response.
#[derive(Debug, Serialize, ToSchema)]
pub struct SegmentTrackData {
    pub points: Vec<SegmentTrackPoint>,
    pub bounds: TrackBounds,
}

/// Segment preview response.
#[derive(Debug, Serialize, ToSchema)]
pub struct PreviewSegmentResponse {
    pub distance_meters: f64,
    pub elevation_gain: f64,
    pub elevation_loss: f64,
    pub average_grade: f64,
    pub max_grade: f64,
    pub climb_category: Option<i32>,
    pub validation: SegmentValidation,
}

/// Segment validation result.
#[derive(Debug, Serialize, ToSchema)]
pub struct SegmentValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Segment reprocessing result.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReprocessResult {
    pub segment_id: Uuid,
    pub activities_checked: i32,
    pub efforts_created: i32,
}

/// Segment star response.
#[derive(Debug, Serialize, ToSchema)]
pub struct StarResponse {
    pub starred: bool,
}

/// Follow status response.
#[derive(Debug, Serialize, ToSchema)]
pub struct FollowStatusResponse {
    pub is_following: bool,
}

/// Follow list response (followers or following).
#[derive(Debug, Serialize, ToSchema)]
pub struct FollowListResponse {
    pub users: Vec<crate::models::UserSummary>,
    pub total_count: i32,
}

/// Kudos action response.
#[derive(Debug, Serialize, ToSchema)]
pub struct KudosResponse {
    pub given: bool,
    pub kudos_count: i32,
}

/// Kudos status response.
#[derive(Debug, Serialize, ToSchema)]
pub struct KudosStatusResponse {
    pub has_given: bool,
}
