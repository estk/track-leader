//! Generic toggle operations for star/follow/kudos patterns.
//!
//! This module provides a trait-based abstraction for toggle operations
//! that follow a common pattern: enable, disable, and check status.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{database::Database, errors::AppError};

/// Configuration for optional notification when a toggle is enabled.
pub struct NotificationConfig {
    pub notification_type: &'static str,
    pub target_type: &'static str,
}

/// A toggle operation that can be enabled/disabled for a user on a target entity.
#[async_trait]
pub trait ToggleOperation: Send + Sync {
    /// Check if the toggle is currently enabled.
    async fn is_enabled(db: &Database, user_id: Uuid, target_id: Uuid) -> Result<bool, AppError>;

    /// Enable the toggle (create relationship). Returns true if newly created.
    async fn enable(db: &Database, user_id: Uuid, target_id: Uuid) -> Result<bool, AppError>;

    /// Disable the toggle (remove relationship). Returns true if was enabled.
    async fn disable(db: &Database, user_id: Uuid, target_id: Uuid) -> Result<bool, AppError>;

    /// Validate before enabling. Returns Ok(()) if valid, Err otherwise.
    /// Default implementation allows all operations.
    async fn validate_enable(
        _db: &Database,
        _user_id: Uuid,
        _target_id: Uuid,
    ) -> Result<(), AppError> {
        Ok(())
    }

    /// Get the target entity owner's user ID for notifications.
    /// Returns None if no notification should be sent.
    async fn get_notification_recipient(
        _db: &Database,
        _target_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        Ok(None)
    }

    /// Notification configuration. Returns None if no notification should be sent.
    fn notification_config() -> Option<NotificationConfig> {
        None
    }
}

/// Segment starring toggle.
pub struct SegmentStar;

#[async_trait]
impl ToggleOperation for SegmentStar {
    async fn is_enabled(db: &Database, user_id: Uuid, segment_id: Uuid) -> Result<bool, AppError> {
        db.is_segment_starred(user_id, segment_id).await
    }

    async fn enable(db: &Database, user_id: Uuid, segment_id: Uuid) -> Result<bool, AppError> {
        // Verify segment exists
        db.get_segment(segment_id)
            .await?
            .ok_or(AppError::NotFound)?;

        db.star_segment(user_id, segment_id).await?;
        Ok(true)
    }

    async fn disable(db: &Database, user_id: Uuid, segment_id: Uuid) -> Result<bool, AppError> {
        db.unstar_segment(user_id, segment_id).await?;
        Ok(true)
    }
}

/// User following toggle.
pub struct UserFollow;

#[async_trait]
impl ToggleOperation for UserFollow {
    async fn is_enabled(
        db: &Database,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<bool, AppError> {
        db.is_following(follower_id, followed_id).await
    }

    async fn enable(db: &Database, follower_id: Uuid, followed_id: Uuid) -> Result<bool, AppError> {
        // Verify target user exists
        db.get_user(followed_id).await?.ok_or(AppError::NotFound)?;

        // Check if already following
        if db.is_following(follower_id, followed_id).await? {
            return Ok(false); // Already following, not newly created
        }

        db.follow_user(follower_id, followed_id).await?;
        Ok(true)
    }

    async fn disable(
        db: &Database,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<bool, AppError> {
        db.unfollow_user(follower_id, followed_id).await
    }

    async fn validate_enable(
        _db: &Database,
        user_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), AppError> {
        if user_id == target_id {
            return Err(AppError::InvalidInput("Cannot follow yourself".to_string()));
        }
        Ok(())
    }

    async fn get_notification_recipient(
        _db: &Database,
        target_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        // The followed user receives the notification
        Ok(Some(target_id))
    }

    fn notification_config() -> Option<NotificationConfig> {
        Some(NotificationConfig {
            notification_type: "follow",
            target_type: "user",
        })
    }
}

/// Activity kudos toggle.
pub struct ActivityKudos;

#[async_trait]
impl ToggleOperation for ActivityKudos {
    async fn is_enabled(db: &Database, user_id: Uuid, activity_id: Uuid) -> Result<bool, AppError> {
        db.has_given_kudos(user_id, activity_id).await
    }

    async fn enable(db: &Database, user_id: Uuid, activity_id: Uuid) -> Result<bool, AppError> {
        db.give_kudos(user_id, activity_id).await
    }

    async fn disable(db: &Database, user_id: Uuid, activity_id: Uuid) -> Result<bool, AppError> {
        db.remove_kudos(user_id, activity_id).await?;
        Ok(true)
    }

    async fn validate_enable(
        db: &Database,
        user_id: Uuid,
        activity_id: Uuid,
    ) -> Result<(), AppError> {
        let activity = db
            .get_activity(activity_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if activity.user_id == user_id {
            return Err(AppError::InvalidInput(
                "Cannot give kudos to your own activity".to_string(),
            ));
        }
        Ok(())
    }

    async fn get_notification_recipient(
        db: &Database,
        activity_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let activity = db
            .get_activity(activity_id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(Some(activity.user_id))
    }

    fn notification_config() -> Option<NotificationConfig> {
        Some(NotificationConfig {
            notification_type: "kudos",
            target_type: "activity",
        })
    }
}

/// Helper to send a toggle notification if configured.
pub async fn send_toggle_notification<T: ToggleOperation>(
    db: &Database,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<(), AppError> {
    if let Some(config) = T::notification_config() {
        if let Some(recipient_id) = T::get_notification_recipient(db, target_id).await? {
            // Don't notify yourself
            if recipient_id != actor_id {
                db.create_notification(
                    recipient_id,
                    config.notification_type,
                    Some(actor_id),
                    Some(config.target_type),
                    Some(target_id),
                    None,
                )
                .await?;
            }
        }
    }
    Ok(())
}
