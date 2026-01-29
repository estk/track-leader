//! Request body types for API endpoints.

use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Default visibility value.
pub fn default_visibility() -> String {
    "public".to_string()
}

/// Activity update request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateActivityRequest {
    pub name: Option<String>,
    pub activity_type_id: Option<Uuid>,
    pub visibility: Option<String>,
}

/// Segment creation request.
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

/// Point in a segment (for creation).
#[derive(Debug, Deserialize, ToSchema)]
pub struct SegmentPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
}

/// Segment preview request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewSegmentRequest {
    pub points: Vec<SegmentPoint>,
}

/// Comment creation request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCommentRequest {
    pub content: String,
    pub parent_id: Option<Uuid>,
}

/// Join request review request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewJoinRequestRequest {
    pub approved: bool,
}
