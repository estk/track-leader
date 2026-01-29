//! Visibility access control helpers.
//!
//! This module provides common patterns for checking access to
//! activities and segments based on visibility settings.

use uuid::Uuid;

use crate::{auth::Claims, database::Database, errors::AppError};

/// Resource type for visibility checks.
pub enum ResourceType {
    Activity { owner_id: Uuid },
    Segment { creator_id: Uuid },
}

/// Check if a user has access to a resource based on visibility settings.
///
/// Returns `true` if access is granted, `false` otherwise.
pub async fn check_visibility_access(
    db: &Database,
    claims: Option<&Claims>,
    visibility: &str,
    resource: ResourceType,
    resource_id: Uuid,
) -> Result<bool, AppError> {
    match visibility {
        "public" => Ok(true),
        "private" => {
            let owner_id = match resource {
                ResourceType::Activity { owner_id } => owner_id,
                ResourceType::Segment { creator_id } => creator_id,
            };
            Ok(claims.is_some_and(|c| c.sub == owner_id))
        }
        "teams_only" => {
            if let Some(c) = claims {
                let owner_id = match resource {
                    ResourceType::Activity { owner_id } => owner_id,
                    ResourceType::Segment { creator_id } => creator_id,
                };

                // Owner always has access
                if c.sub == owner_id {
                    return Ok(true);
                }

                // Check team membership
                match resource {
                    ResourceType::Activity { .. } => {
                        db.user_has_activity_team_access(c.sub, resource_id).await
                    }
                    ResourceType::Segment { .. } => {
                        db.user_has_segment_team_access(c.sub, resource_id).await
                    }
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}

/// Check access for an activity, returning NotFound if no access.
///
/// This is a convenience function that combines fetching and access checking.
pub async fn require_activity_access(
    db: &Database,
    claims: Option<&Claims>,
    activity_id: Uuid,
) -> Result<crate::models::Activity, AppError> {
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let has_access = check_visibility_access(
        db,
        claims,
        &activity.visibility,
        ResourceType::Activity {
            owner_id: activity.user_id,
        },
        activity_id,
    )
    .await?;

    if has_access {
        Ok(activity)
    } else {
        Err(AppError::NotFound) // Don't leak existence
    }
}

/// Check access for a segment, returning NotFound if no access.
///
/// This is a convenience function that combines fetching and access checking.
pub async fn require_segment_access(
    db: &Database,
    claims: Option<&Claims>,
    segment_id: Uuid,
) -> Result<crate::models::Segment, AppError> {
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let has_access = check_visibility_access(
        db,
        claims,
        &segment.visibility,
        ResourceType::Segment {
            creator_id: segment.creator_id,
        },
        segment_id,
    )
    .await?;

    if has_access {
        Ok(segment)
    } else {
        Err(AppError::NotFound) // Don't leak existence
    }
}
