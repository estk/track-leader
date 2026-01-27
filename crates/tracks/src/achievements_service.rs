//! Achievement service for KOM/QOM and Local Legend tracking.
//!
//! This service handles checking and awarding achievements when segment efforts are created.
//! - KOM (King of the Mountain): Fastest male time on a segment
//! - QOM (Queen of the Mountain): Fastest female time on a segment
//! - Local Legend: Most efforts on a segment in the last 90 days

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

/// Check and award Local Legend achievement based on effort counts in the last 90 days.
///
/// This function:
/// 1. Gets the top effort counts for this segment in the last 90 days
/// 2. If the top user is different from the current holder (or there's no holder), awards Local Legend
/// 3. If the current holder's count changed, updates the achievement
pub async fn check_local_legend(
    db: &Database,
    segment_id: Uuid,
    _user_id: Uuid,
) -> Result<(), AppError> {
    // Get top effort counts for this segment
    let top_counts = db.get_top_recent_effort_counts(segment_id, 5).await?;

    if top_counts.is_empty() {
        // No efforts yet, nothing to do
        return Ok(());
    }

    // Find who has the most efforts
    let (leader_id, leader_count) = top_counts[0];

    // Get current Local Legend holder
    let current_holder = db
        .get_current_achievement_holder(segment_id, AchievementType::LocalLegend)
        .await?;

    // Determine if we need to make changes
    let should_change = match &current_holder {
        None => {
            // No current holder, award to the leader
            info!(
                "No current Local Legend for segment {segment_id}, awarding to user {leader_id} with {leader_count} efforts"
            );
            true
        }
        Some(holder) => {
            // Check if leader is different from current holder
            if leader_id != holder.user_id {
                // Check if leader has more efforts than the current holder's recorded count
                let holder_count = holder.effort_count.unwrap_or(0) as i64;
                if leader_count > holder_count {
                    info!(
                        "User {leader_id} dethroned Local Legend on segment {segment_id}: {leader_count} > {holder_count}"
                    );
                    true
                } else {
                    false
                }
            } else if leader_id == holder.user_id {
                // Same user, but update their effort count if it changed
                let holder_count = holder.effort_count.unwrap_or(0) as i64;
                if leader_count != holder_count {
                    info!(
                        "Updating Local Legend effort count for user {leader_id} on segment {segment_id}: {holder_count} -> {leader_count}"
                    );
                    // Dethrone and re-award with updated count
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    };

    if should_change {
        // Dethrone current holder if exists
        if current_holder.is_some() {
            db.dethrone_achievement(segment_id, AchievementType::LocalLegend)
                .await?;
        }

        // Award Local Legend to the leader
        db.create_achievement(
            leader_id,
            segment_id,
            None, // Local Legend is not tied to a specific effort
            AchievementType::LocalLegend,
            Some(leader_count as i32),
        )
        .await?;

        info!(
            "Awarded Local Legend to user {leader_id} on segment {segment_id} with {leader_count} efforts"
        );
    }

    Ok(())
}

/// Process achievements after a segment effort is created.
///
/// This is the main entry point called from activity_queue after creating a segment effort.
/// It checks both KOM/QOM and Local Legend achievements.
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

    // Check Local Legend
    if let Err(e) = check_local_legend(db, segment_id, user_id).await {
        warn!("Failed to check Local Legend for segment {segment_id}: {e}");
    }

    Ok(())
}
