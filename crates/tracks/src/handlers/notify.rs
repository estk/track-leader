//! Notification builder for creating notifications with a fluent API.

use uuid::Uuid;

use crate::{database::Database, errors::AppError};

/// Builder for creating notifications with a fluent API.
pub struct NotificationBuilder {
    recipient_id: Uuid,
    notification_type: &'static str,
    actor_id: Option<Uuid>,
    target_type: Option<&'static str>,
    target_id: Option<Uuid>,
    message: Option<String>,
}

impl NotificationBuilder {
    /// Create a new notification builder.
    pub fn new(recipient_id: Uuid, notification_type: &'static str) -> Self {
        Self {
            recipient_id,
            notification_type,
            actor_id: None,
            target_type: None,
            target_id: None,
            message: None,
        }
    }

    /// Set the actor who triggered this notification.
    pub fn actor(mut self, actor_id: Uuid) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    /// Set the target entity for this notification.
    pub fn target(mut self, target_type: &'static str, target_id: Uuid) -> Self {
        self.target_type = Some(target_type);
        self.target_id = Some(target_id);
        self
    }

    /// Set an optional message for this notification.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Send the notification.
    pub async fn send(self, db: &Database) -> Result<(), AppError> {
        db.create_notification(
            self.recipient_id,
            self.notification_type,
            self.actor_id,
            self.target_type,
            self.target_id,
            self.message.as_deref(),
        )
        .await?;
        Ok(())
    }
}

/// Helper function to send a follow notification.
pub async fn notify_follow(
    db: &Database,
    follower_id: Uuid,
    followed_id: Uuid,
) -> Result<(), AppError> {
    NotificationBuilder::new(followed_id, "follow")
        .actor(follower_id)
        .target("user", follower_id)
        .send(db)
        .await
}

/// Helper function to send a kudos notification.
pub async fn notify_kudos(
    db: &Database,
    giver_id: Uuid,
    activity_owner_id: Uuid,
    activity_id: Uuid,
) -> Result<(), AppError> {
    NotificationBuilder::new(activity_owner_id, "kudos")
        .actor(giver_id)
        .target("activity", activity_id)
        .send(db)
        .await
}

/// Helper function to send a comment notification.
pub async fn notify_comment(
    db: &Database,
    commenter_id: Uuid,
    activity_owner_id: Uuid,
    activity_id: Uuid,
    comment_content: &str,
) -> Result<(), AppError> {
    NotificationBuilder::new(activity_owner_id, "comment")
        .actor(commenter_id)
        .target("activity", activity_id)
        .message(comment_content)
        .send(db)
        .await
}
