//! Achievement service for KOM/QOM tracking.
//!
//! This service handles checking and awarding achievements when segment efforts are created.
//! - KOM (King of the Mountain): Fastest male time on a segment
//! - QOM (Queen of the Mountain): Fastest female time on a segment

use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    database::Database,
    errors::AppError,
    models::{AchievementType, Gender},
};

/// Check and award KOM/QOM achievement if this effort is the fastest for its gender category.
///
/// This function:
/// 1. Gets the user's gender to determine KOM vs QOM
/// 2. Gets the current fastest effort for that gender category
/// 3. If this effort is faster, dethrones the current holder and awards the crown
pub async fn check_and_award_kom_qom(
    db: &Database,
    segment_id: Uuid,
    user_id: Uuid,
    effort_id: Uuid,
    elapsed_time_seconds: f64,
) -> Result<(), AppError> {
    // Get user's gender to determine achievement type
    let user = match db.get_user_with_demographics(user_id).await? {
        Some(u) => u,
        None => {
            warn!("User {user_id} not found when checking achievements");
            return Ok(());
        }
    };

    let achievement_type = match user.gender {
        Some(Gender::Male) => AchievementType::Kom,
        Some(Gender::Female) => AchievementType::Qom,
        // Users without gender or with "other"/"prefer not to say" are not eligible for KOM/QOM
        _ => {
            info!("User {user_id} has no gender set or is not male/female, skipping KOM/QOM check");
            return Ok(());
        }
    };

    // Get current achievement holder for this segment and type
    let current_holder = db
        .get_current_achievement_holder(segment_id, achievement_type)
        .await?;

    // Check if we should award the achievement
    let should_award = match &current_holder {
        None => {
            // No current holder, this is the first eligible effort
            info!(
                "No current {achievement_type} holder for segment {segment_id}, awarding to user {user_id}"
            );
            true
        }
        Some(holder) => {
            // Check if this effort is faster
            if let Some(holder_time) = holder.elapsed_time_seconds {
                if elapsed_time_seconds < holder_time {
                    info!(
                        "User {user_id} beat {achievement_type} on segment {segment_id}: {elapsed_time_seconds:.1}s < {holder_time:.1}s"
                    );
                    true
                } else {
                    false
                }
            } else {
                // Current holder has no time recorded (shouldn't happen), award to this user
                warn!(
                    "Current {achievement_type} holder has no elapsed time, awarding to user {user_id}"
                );
                true
            }
        }
    };

    if should_award {
        // Dethrone current holder if exists
        if current_holder.is_some() {
            db.dethrone_achievement(segment_id, achievement_type)
                .await?;
        }

        // Award achievement to this user
        db.create_achievement(user_id, segment_id, Some(effort_id), achievement_type, None)
            .await?;

        info!("Awarded {achievement_type} to user {user_id} on segment {segment_id}");
    }

    Ok(())
}

/// Process achievements after a segment effort is created.
///
/// This is the main entry point called from activity_queue after creating a segment effort.
/// It checks KOM/QOM achievements.
pub async fn process_achievements(
    db: &Database,
    segment_id: Uuid,
    user_id: Uuid,
    effort_id: Uuid,
    elapsed_time_seconds: f64,
) -> Result<(), AppError> {
    // Check KOM/QOM
    if let Err(e) =
        check_and_award_kom_qom(db, segment_id, user_id, effort_id, elapsed_time_seconds).await
    {
        warn!("Failed to check KOM/QOM for segment {segment_id}: {e}");
    }

    Ok(())
}
