use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use bytes::{Buf as _, Bytes};
use tokio::runtime::Handle;
use uuid::Uuid;

use crate::{
    achievements_service,
    database::Database,
    models::TrackPointData,
    object_store_service::FileType,
    scoring,
    segment_matching::{self, SegmentMatch},
};
use time::OffsetDateTime;

#[derive(Clone)]
pub struct ActivityQueue {
    db: Database,
    handle: Handle,
    pool: Arc<rayon::ThreadPool>,
    activities: Arc<Mutex<HashSet<Uuid>>>,
    done_tx: std::sync::mpsc::Sender<Uuid>,
}

impl ActivityQueue {
    pub fn new(db: Database) -> Self {
        let rpool = rayon::ThreadPoolBuilder::new().build().unwrap();
        let handle = Handle::current();
        let activities = Arc::new(Mutex::new(HashSet::new()));

        let (tx, rx) = std::sync::mpsc::channel::<Uuid>();
        let worker_activities = activities.clone();
        rpool.spawn(move || {
            while let Ok(id) = rx.recv() {
                let mut activities = worker_activities.lock().unwrap();
                activities.remove(&id);
            }
        });
        Self {
            db,
            pool: Arc::new(rpool),
            handle,
            activities,
            done_tx: tx,
        }
    }

    pub fn submit(
        &self,
        uid: Uuid,
        id: Uuid,
        ft: FileType,
        bytes: Bytes,
        activity_type_id: Uuid,
        type_boundaries: Option<Vec<OffsetDateTime>>,
        segment_types: Option<Vec<Uuid>>,
    ) -> anyhow::Result<()> {
        assert!(matches!(ft, FileType::Gpx));

        self.activities.lock().unwrap().insert(id);
        let tx = self.done_tx.clone();
        let db = self.db.clone();
        let handle = self.handle.clone();
        self.pool.spawn(move || {
            let parsed_track = gpx::read(bytes.reader()).unwrap();

            // Calculate scores
            let scores = scoring::score_track(&parsed_track);

            // Extract track points with elevation and timestamps
            let track_points = extract_track_points(&parsed_track);

            handle.block_on(async move {
                // Save scores
                db.save_scores(uid, id, scores).await.unwrap();

                // Save track geometry with elevation and timestamps
                let track_saved = if !track_points.is_empty() {
                    match db
                        .save_track_geometry_with_data(uid, id, &track_points)
                        .await
                    {
                        Ok(()) => true,
                        Err(e) => {
                            tracing::error!("Failed to save track geometry: {e}");
                            false
                        }
                    }
                } else {
                    false
                };

                // Find and create segment efforts
                if track_saved {
                    let matches = if let (Some(boundaries), Some(types)) =
                        (&type_boundaries, &segment_types)
                    {
                        // Multi-sport activity: find all geometric matches, then filter by type
                        match db.find_matching_segments_any_type(id).await {
                            Ok(all_matches) => filter_multi_sport_matches(
                                all_matches,
                                &track_points,
                                boundaries,
                                types,
                            ),
                            Err(e) => {
                                tracing::error!("Failed to find matching segments: {e}");
                                vec![]
                            }
                        }
                    } else {
                        // Single-sport activity: filter by activity_type_id directly
                        match db.find_matching_segments(id, activity_type_id).await {
                            Ok(m) => m,
                            Err(e) => {
                                tracing::error!("Failed to find matching segments: {e}");
                                vec![]
                            }
                        }
                    };

                    for segment_match in matches {
                        process_segment_match(&db, &parsed_track, uid, id, segment_match).await;
                    }
                }
            });
            tx.send(id).unwrap();
        });
        Ok(())
    }
}

/// Extract track points with elevation and timestamps from GPX
fn extract_track_points(gpx: &gpx::Gpx) -> Vec<TrackPointData> {
    let mut points = Vec::new();

    for track in &gpx.tracks {
        for seg in &track.segments {
            for pt in &seg.points {
                let lon = pt.point().x();
                let lat = pt.point().y();
                let elevation = pt.elevation;
                let timestamp = pt.time.as_ref().and_then(|t| {
                    // gpx::Time has a format() method that returns ISO 8601 string
                    // We need to parse it to OffsetDateTime
                    t.format().ok().and_then(|s| {
                        time::OffsetDateTime::parse(
                            &s,
                            &time::format_description::well_known::Rfc3339,
                        )
                        .ok()
                    })
                });

                points.push(TrackPointData {
                    lat,
                    lon,
                    elevation,
                    timestamp,
                });
            }
        }
    }

    points
}

/// Process a single segment match: extract timing and create effort
async fn process_segment_match(
    db: &Database,
    gpx: &gpx::Gpx,
    user_id: Uuid,
    activity_id: Uuid,
    segment_match: SegmentMatch,
) {
    // Check if effort already exists (idempotency)
    match db
        .segment_effort_exists(segment_match.segment_id, activity_id)
        .await
    {
        Ok(true) => {
            tracing::debug!(
                "Effort already exists for segment {} and activity {}",
                segment_match.segment_id,
                activity_id
            );
            return;
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!("Failed to check effort existence: {e}");
            return;
        }
    }

    // Extract timing from GPX
    let timing = match segment_matching::extract_timing_from_gpx(
        gpx,
        segment_match.start_fraction,
        segment_match.end_fraction,
    ) {
        Some(t) => t,
        None => {
            tracing::warn!(
                "Could not extract timing for segment {} on activity {}",
                segment_match.segment_id,
                activity_id
            );
            return;
        }
    };

    // Calculate average speed: distance / time
    let average_speed_mps = if timing.elapsed_time_seconds > 0.0 {
        Some(segment_match.distance_meters / timing.elapsed_time_seconds)
    } else {
        None
    };

    // Create the effort
    match db
        .create_segment_effort(
            segment_match.segment_id,
            activity_id,
            user_id,
            timing.started_at,
            timing.elapsed_time_seconds,
            Some(timing.moving_time_seconds),
            average_speed_mps,
            None, // max_speed_mps
            Some(segment_match.start_fraction),
            Some(segment_match.end_fraction),
        )
        .await
    {
        Ok(effort) => {
            tracing::info!(
                "Created segment effort {} for segment {} with time {:.1}s (moving: {:.1}s)",
                effort.id,
                segment_match.segment_id,
                timing.elapsed_time_seconds,
                timing.moving_time_seconds
            );
            // Update effort count
            if let Err(e) = db
                .increment_segment_effort_count(segment_match.segment_id)
                .await
            {
                tracing::error!("Failed to increment effort count: {e}");
            }
            // Update personal records
            if let Err(e) = db
                .update_personal_records(segment_match.segment_id, user_id)
                .await
            {
                tracing::error!("Failed to update personal records: {e}");
            }
            // Check and award achievements (KOM/QOM and Local Legend)
            if let Err(e) = achievements_service::process_achievements(
                db,
                segment_match.segment_id,
                user_id,
                effort.id,
                timing.elapsed_time_seconds,
            )
            .await
            {
                tracing::error!("Failed to process achievements: {e}");
            }
        }
        Err(e) => {
            tracing::error!("Failed to create segment effort: {e}");
        }
    }
}

/// For multi-sport activities, filter segment matches to only include those where
/// the segment's activity type matches the activity type at that position on the track.
fn filter_multi_sport_matches(
    all_matches: Vec<(SegmentMatch, Uuid)>,
    track_points: &[TrackPointData],
    type_boundaries: &[OffsetDateTime],
    segment_types: &[Uuid],
) -> Vec<SegmentMatch> {
    if track_points.is_empty() || type_boundaries.len() < 2 {
        return vec![];
    }

    all_matches
        .into_iter()
        .filter_map(|(segment_match, segment_type_id)| {
            // Get the midpoint fraction of the segment on the track
            let mid_fraction = (segment_match.start_fraction + segment_match.end_fraction) / 2.0;

            // Convert fraction to timestamp
            let timestamp = fraction_to_timestamp(track_points, mid_fraction)?;

            // Find which activity type applies at that timestamp
            let activity_type_at_pos =
                get_activity_type_at_timestamp(type_boundaries, segment_types, timestamp)?;

            // Only include if types match
            if activity_type_at_pos == segment_type_id {
                Some(segment_match)
            } else {
                None
            }
        })
        .collect()
}

/// Convert a fractional position (0.0 to 1.0) on the track to a timestamp.
fn fraction_to_timestamp(track_points: &[TrackPointData], fraction: f64) -> Option<OffsetDateTime> {
    if track_points.is_empty() {
        return None;
    }

    // Find the first and last timestamps in the track
    let first_ts = track_points.iter().find_map(|p| p.timestamp)?;
    let last_ts = track_points.iter().rev().find_map(|p| p.timestamp)?;

    // Interpolate the timestamp based on the fraction
    let duration_secs = (last_ts - first_ts).whole_seconds() as f64;
    let offset_secs = (duration_secs * fraction) as i64;

    Some(first_ts + time::Duration::seconds(offset_secs))
}

/// Get the activity type ID at a given timestamp based on type boundaries.
fn get_activity_type_at_timestamp(
    type_boundaries: &[OffsetDateTime],
    segment_types: &[Uuid],
    timestamp: OffsetDateTime,
) -> Option<Uuid> {
    // type_boundaries: [start, boundary1, boundary2, end]
    // segment_types: [type0, type1, type2] (one less than boundaries)
    if segment_types.len() != type_boundaries.len() - 1 {
        return None;
    }

    // Find which segment the timestamp falls into
    for (i, window) in type_boundaries.windows(2).enumerate() {
        if timestamp >= window[0] && timestamp < window[1] {
            return segment_types.get(i).copied();
        }
    }

    // If timestamp is exactly at or after the last boundary, use the last segment type
    if timestamp >= *type_boundaries.last()? {
        return segment_types.last().copied();
    }

    None
}
