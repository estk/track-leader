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
    file_parsers::{self, ParsedActivity},
    models::TrackPointData,
    object_store_service::FileType,
    scoring,
    segment_matching::{self, SegmentMatch},
};
use time::OffsetDateTime;

/// Submission data for processing an activity
pub struct ActivitySubmission {
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub file_type: FileType,
    pub bytes: Bytes,
    pub activity_type_id: Uuid,
    pub type_boundaries: Option<Vec<OffsetDateTime>>,
    pub segment_types: Option<Vec<Uuid>>,
}

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

    /// Reprocess an orphaned activity by loading its file from the object store.
    pub async fn reprocess_orphaned(
        &self,
        activity: crate::models::OrphanedActivity,
        store: &crate::object_store_service::ObjectStoreService,
    ) -> anyhow::Result<()> {
        let bytes = store.get_file(&activity.object_store_path).await?;
        let file_type = crate::object_store_service::FileType::detect_from_bytes(&bytes);

        tracing::info!(
            activity_id = %activity.id,
            "Reprocessing orphaned activity"
        );

        self.submit(ActivitySubmission {
            user_id: activity.user_id,
            activity_id: activity.id,
            file_type,
            bytes,
            activity_type_id: activity.activity_type_id,
            type_boundaries: activity.type_boundaries,
            segment_types: activity.segment_types,
        })
    }

    pub fn submit(&self, submission: ActivitySubmission) -> anyhow::Result<()> {
        let ActivitySubmission {
            user_id: uid,
            activity_id: id,
            file_type,
            bytes,
            activity_type_id,
            type_boundaries,
            segment_types,
        } = submission;

        self.activities.lock().unwrap().insert(id);
        let tx = self.done_tx.clone();
        let db = self.db.clone();
        let handle = self.handle.clone();
        self.pool.spawn(move || {
            // Parse the activity file using the unified parser
            let parsed = match file_parsers::parse_activity_file(file_type, bytes.clone()) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to parse activity file: {e}");
                    tx.send(id).unwrap();
                    return;
                }
            };

            let ParsedActivity {
                track_points,
                sensor_data,
                sport_segments: _,
            } = parsed;

            // For GPX files, also parse the raw GPX for segment timing extraction
            // (segment matching currently requires the gpx::Gpx structure)
            let gpx_data = if file_type == FileType::Gpx || file_type == FileType::Other {
                // Re-parse as GPX if it's a GPX file (or might be detected as GPX)
                if FileType::detect_from_bytes(&bytes) == FileType::Gpx {
                    gpx::read(bytes.reader()).ok()
                } else {
                    None
                }
            } else {
                None
            };

            // Calculate scores from track points
            let scores = scoring::score_track_points(&track_points);

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

                // Save sensor data if present
                if sensor_data.has_any_data() {
                    if let Err(e) = db.save_sensor_data(id, &sensor_data).await {
                        tracing::error!("Failed to save sensor data: {e}");
                    }
                }

                // Find and create segment efforts (only for GPX files currently)
                if track_saved {
                    if let Some(ref gpx) = gpx_data {
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
                            process_segment_match(&db, gpx, uid, id, segment_match).await;
                        }
                    }
                }

                // Detect and save stopped segments
                let stopped_segments = detect_stopped_segments(&track_points);
                if !stopped_segments.is_empty() {
                    tracing::info!(
                        "Detected {} stopped segments in activity {}",
                        stopped_segments.len(),
                        id
                    );
                    if let Err(e) = db.save_stopped_segments(id, &stopped_segments).await {
                        tracing::error!("Failed to save stopped segments: {e}");
                    }
                }

                // Extract dig segments from multi-sport activities with DIG activity type
                if let (Some(boundaries), Some(types)) = (&type_boundaries, &segment_types) {
                    let dig_parts =
                        extract_dig_parts_from_multi_sport(&track_points, boundaries, types);
                    if !dig_parts.is_empty() {
                        tracing::info!(
                            "Found {} dig segments in multi-sport activity {}",
                            dig_parts.len(),
                            id
                        );
                        if let Err(e) = db.save_dig_parts_batch(id, &dig_parts).await {
                            tracing::error!("Failed to save dig segments: {e}");
                        }
                    }
                }
            });
            tx.send(id).unwrap();
        });
        Ok(())
    }
}

/// Detected stopped segment with timestamps
pub struct DetectedStoppedSegment {
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub duration_seconds: f64,
}

/// Detect stopped segments in a track.
/// A stopped segment is defined as consecutive points where:
/// - Speed < 1 m/s (essentially not moving)
/// - Duration > 30 seconds
fn detect_stopped_segments(points: &[TrackPointData]) -> Vec<DetectedStoppedSegment> {
    const SPEED_THRESHOLD_MPS: f64 = 1.0; // 1 m/s = 3.6 km/h
    const MIN_STOPPED_DURATION_SECS: f64 = 30.0;

    let mut segments = Vec::new();

    // Need at least 2 points with timestamps to detect stopped segments
    if points.len() < 2 {
        return segments;
    }

    let mut stop_start: Option<(usize, OffsetDateTime)> = None;

    for i in 1..points.len() {
        let prev = &points[i - 1];
        let curr = &points[i];

        // Skip points without timestamps
        let (Some(prev_time), Some(curr_time)) = (&prev.timestamp, &curr.timestamp) else {
            // End any current stopped segment if we lose timestamp continuity
            if let Some((_, start_time)) = stop_start.take() {
                if let Some(end_time) = points[i - 1].timestamp {
                    let duration = (end_time - start_time).as_seconds_f64();
                    if duration >= MIN_STOPPED_DURATION_SECS {
                        segments.push(DetectedStoppedSegment {
                            start_time,
                            end_time,
                            duration_seconds: duration,
                        });
                    }
                }
            }
            continue;
        };

        // Calculate distance and speed
        let distance = haversine_distance(prev.lat, prev.lon, curr.lat, curr.lon);
        let time_diff = (*curr_time - *prev_time).as_seconds_f64();

        // Skip if time_diff is zero or negative (bad data)
        if time_diff <= 0.0 {
            continue;
        }

        let speed = distance / time_diff;

        if speed < SPEED_THRESHOLD_MPS {
            // Moving slowly or stopped
            if stop_start.is_none() {
                stop_start = Some((i - 1, *prev_time));
            }
        } else {
            // Moving again - end any current stopped segment
            if let Some((_, start_time)) = stop_start.take() {
                let duration = (*prev_time - start_time).as_seconds_f64();
                if duration >= MIN_STOPPED_DURATION_SECS {
                    segments.push(DetectedStoppedSegment {
                        start_time,
                        end_time: *prev_time,
                        duration_seconds: duration,
                    });
                }
            }
        }
    }

    // Handle case where track ends while stopped
    if let Some((_, start_time)) = stop_start {
        if let Some(end_time) = points.last().and_then(|p| p.timestamp) {
            let duration = (end_time - start_time).as_seconds_f64();
            if duration >= MIN_STOPPED_DURATION_SECS {
                segments.push(DetectedStoppedSegment {
                    start_time,
                    end_time,
                    duration_seconds: duration,
                });
            }
        }
    }

    segments
}

/// Extract dig segments from multi-sport activities where segment type is DIG.
/// Returns detected dig segments with their time ranges and durations.
fn extract_dig_parts_from_multi_sport(
    _points: &[TrackPointData],
    boundaries: &[OffsetDateTime],
    types: &[Uuid],
) -> Vec<DetectedStoppedSegment> {
    use crate::models::builtin_types;

    let mut dig_parts = Vec::new();

    // boundaries define edges: [start, b1, b2, ..., end] for n segments
    // types[i] is the activity type for segment between boundaries[i] and boundaries[i+1]
    if boundaries.len() < 2 || types.is_empty() {
        return dig_parts;
    }

    for (i, segment_type) in types.iter().enumerate() {
        // Check if this segment is a DIG type
        if *segment_type != builtin_types::DIG {
            continue;
        }

        // Get the boundary timestamps for this segment
        let Some(start_time) = boundaries.get(i).copied() else {
            continue;
        };
        let Some(end_time) = boundaries.get(i + 1).copied() else {
            continue;
        };

        let duration = (end_time - start_time).as_seconds_f64();
        if duration > 0.0 {
            dig_parts.push(DetectedStoppedSegment {
                start_time,
                end_time,
                duration_seconds: duration,
            });
        }
    }

    dig_parts
}

/// Calculate the distance in meters between two lat/lon points using the haversine formula
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_METERS * c
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
            // Check and award achievements (KOM/QOM)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::builtin_types;
    use crate::segment_matching::SegmentMatch;
    use time::Duration;

    /// Create a simple track with timestamps spanning the given duration.
    fn make_track_points(duration_minutes: i64) -> Vec<TrackPointData> {
        let start = OffsetDateTime::now_utc();
        let num_points = 100;
        (0..num_points)
            .map(|i| {
                let fraction = i as f64 / (num_points - 1) as f64;
                let offset = Duration::minutes((duration_minutes as f64 * fraction) as i64);
                TrackPointData {
                    lat: 40.0 + (i as f64 * 0.0001),
                    lon: -105.3 + (i as f64 * 0.0001),
                    elevation: Some(1650.0 + (i as f64 * 0.5)),
                    timestamp: Some(start + offset),
                }
            })
            .collect()
    }

    // ========================================================================
    // Tests for fraction_to_timestamp
    // ========================================================================

    #[test]
    fn test_fraction_to_timestamp_at_start() {
        let points = make_track_points(60);
        let result = fraction_to_timestamp(&points, 0.0);
        assert!(result.is_some());

        let first_ts = points.first().unwrap().timestamp.unwrap();
        assert_eq!(result.unwrap(), first_ts);
    }

    #[test]
    fn test_fraction_to_timestamp_at_end() {
        let points = make_track_points(60);
        let result = fraction_to_timestamp(&points, 1.0);
        assert!(result.is_some());

        let first_ts = points.first().unwrap().timestamp.unwrap();
        let last_ts = points.last().unwrap().timestamp.unwrap();
        let expected = first_ts + Duration::seconds((last_ts - first_ts).whole_seconds());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_fraction_to_timestamp_at_midpoint() {
        let points = make_track_points(60);
        let result = fraction_to_timestamp(&points, 0.5);
        assert!(result.is_some());

        let first_ts = points.first().unwrap().timestamp.unwrap();
        let last_ts = points.last().unwrap().timestamp.unwrap();
        let duration_secs = (last_ts - first_ts).whole_seconds();
        let expected = first_ts + Duration::seconds(duration_secs / 2);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_fraction_to_timestamp_empty_track() {
        let points: Vec<TrackPointData> = vec![];
        let result = fraction_to_timestamp(&points, 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn test_fraction_to_timestamp_no_timestamps() {
        let points = vec![
            TrackPointData {
                lat: 40.0,
                lon: -105.3,
                elevation: Some(1650.0),
                timestamp: None,
            },
            TrackPointData {
                lat: 40.001,
                lon: -105.299,
                elevation: Some(1660.0),
                timestamp: None,
            },
        ];
        let result = fraction_to_timestamp(&points, 0.5);
        assert!(result.is_none());
    }

    // ========================================================================
    // Tests for get_activity_type_at_timestamp
    // ========================================================================

    #[test]
    fn test_get_activity_type_at_timestamp_single_segment() {
        let start = OffsetDateTime::now_utc();
        let boundaries = vec![start, start + Duration::hours(1)];
        let types = vec![builtin_types::RUN];

        // At start
        let result = get_activity_type_at_timestamp(&boundaries, &types, start);
        assert_eq!(result, Some(builtin_types::RUN));

        // In middle
        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::minutes(30));
        assert_eq!(result, Some(builtin_types::RUN));

        // At end (should use last segment)
        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::hours(1));
        assert_eq!(result, Some(builtin_types::RUN));
    }

    #[test]
    fn test_get_activity_type_at_timestamp_multi_segment() {
        let start = OffsetDateTime::now_utc();
        // 3-segment activity: RUN (0-30min), MTB (30-60min), RUN (60-90min)
        let boundaries = vec![
            start,
            start + Duration::minutes(30),
            start + Duration::minutes(60),
            start + Duration::minutes(90),
        ];
        let types = vec![builtin_types::RUN, builtin_types::MTB, builtin_types::RUN];

        // First segment (0-30 min): RUN
        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::minutes(15));
        assert_eq!(result, Some(builtin_types::RUN));

        // Second segment (30-60 min): MTB
        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::minutes(45));
        assert_eq!(result, Some(builtin_types::MTB));

        // Third segment (60-90 min): RUN
        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::minutes(75));
        assert_eq!(result, Some(builtin_types::RUN));

        // Exactly at boundary between segments
        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::minutes(30));
        assert_eq!(result, Some(builtin_types::MTB)); // Should be in second segment
    }

    #[test]
    fn test_get_activity_type_at_timestamp_mismatched_arrays() {
        let start = OffsetDateTime::now_utc();
        let boundaries = vec![start, start + Duration::hours(1)];
        let types = vec![builtin_types::RUN, builtin_types::MTB]; // 2 types but only 1 segment

        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start + Duration::minutes(30));
        assert!(result.is_none()); // Should fail validation
    }

    #[test]
    fn test_get_activity_type_at_timestamp_before_start() {
        let start = OffsetDateTime::now_utc();
        let boundaries = vec![start, start + Duration::hours(1)];
        let types = vec![builtin_types::RUN];

        let result =
            get_activity_type_at_timestamp(&boundaries, &types, start - Duration::minutes(10));
        assert!(result.is_none());
    }

    // ========================================================================
    // Tests for filter_multi_sport_matches
    // ========================================================================

    #[test]
    fn test_filter_multi_sport_matches_matching_type() {
        let start = OffsetDateTime::now_utc();
        let points: Vec<TrackPointData> = (0..100)
            .map(|i| {
                let offset = Duration::minutes(i);
                TrackPointData {
                    lat: 40.0 + (i as f64 * 0.0001),
                    lon: -105.3 + (i as f64 * 0.0001),
                    elevation: Some(1650.0),
                    timestamp: Some(start + offset),
                }
            })
            .collect();

        // Activity: RUN (0-50 min), MTB (50-100 min)
        let boundaries = vec![
            start,
            start + Duration::minutes(50),
            start + Duration::minutes(99),
        ];
        let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

        // A RUN segment in the first half should match
        let matches = vec![(
            SegmentMatch {
                segment_id: Uuid::new_v4(),
                distance_meters: 1000.0,
                start_fraction: 0.1, // 10% into track = ~10 min in = RUN portion
                end_fraction: 0.3,   // 30% into track = ~30 min in = still RUN
            },
            builtin_types::RUN,
        )];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_multi_sport_matches_non_matching_type() {
        let start = OffsetDateTime::now_utc();
        let points: Vec<TrackPointData> = (0..100)
            .map(|i| {
                let offset = Duration::minutes(i);
                TrackPointData {
                    lat: 40.0 + (i as f64 * 0.0001),
                    lon: -105.3 + (i as f64 * 0.0001),
                    elevation: Some(1650.0),
                    timestamp: Some(start + offset),
                }
            })
            .collect();

        // Activity: RUN (0-50 min), MTB (50-100 min)
        let boundaries = vec![
            start,
            start + Duration::minutes(50),
            start + Duration::minutes(99),
        ];
        let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

        // An MTB segment in the first half (RUN portion) should NOT match
        let matches = vec![(
            SegmentMatch {
                segment_id: Uuid::new_v4(),
                distance_meters: 1000.0,
                start_fraction: 0.1, // 10% into track = RUN portion
                end_fraction: 0.3,   // 30% into track = still RUN portion
            },
            builtin_types::MTB, // But segment expects MTB
        )];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_multi_sport_matches_mixed_results() {
        let start = OffsetDateTime::now_utc();
        let points: Vec<TrackPointData> = (0..100)
            .map(|i| {
                let offset = Duration::minutes(i);
                TrackPointData {
                    lat: 40.0 + (i as f64 * 0.0001),
                    lon: -105.3 + (i as f64 * 0.0001),
                    elevation: Some(1650.0),
                    timestamp: Some(start + offset),
                }
            })
            .collect();

        // Activity: RUN (0-50 min), MTB (50-100 min)
        let boundaries = vec![
            start,
            start + Duration::minutes(50),
            start + Duration::minutes(99),
        ];
        let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

        let run_segment_id = Uuid::new_v4();
        let mtb_segment_id = Uuid::new_v4();
        let mtb_wrong_place_id = Uuid::new_v4();

        let matches = vec![
            // RUN segment in RUN portion - SHOULD MATCH
            (
                SegmentMatch {
                    segment_id: run_segment_id,
                    distance_meters: 1000.0,
                    start_fraction: 0.1,
                    end_fraction: 0.3,
                },
                builtin_types::RUN,
            ),
            // MTB segment in MTB portion - SHOULD MATCH
            (
                SegmentMatch {
                    segment_id: mtb_segment_id,
                    distance_meters: 1500.0,
                    start_fraction: 0.6, // 60% = MTB portion
                    end_fraction: 0.8,
                },
                builtin_types::MTB,
            ),
            // MTB segment in RUN portion - SHOULD NOT MATCH
            (
                SegmentMatch {
                    segment_id: mtb_wrong_place_id,
                    distance_meters: 800.0,
                    start_fraction: 0.1, // 10% = RUN portion
                    end_fraction: 0.2,
                },
                builtin_types::MTB,
            ),
        ];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|m| m.segment_id == run_segment_id));
        assert!(result.iter().any(|m| m.segment_id == mtb_segment_id));
        assert!(!result.iter().any(|m| m.segment_id == mtb_wrong_place_id));
    }

    #[test]
    fn test_filter_multi_sport_matches_empty_track() {
        let points: Vec<TrackPointData> = vec![];
        let boundaries = vec![OffsetDateTime::now_utc()];
        let segment_types = vec![];

        let matches = vec![(
            SegmentMatch {
                segment_id: Uuid::new_v4(),
                distance_meters: 1000.0,
                start_fraction: 0.1,
                end_fraction: 0.3,
            },
            builtin_types::RUN,
        )];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_multi_sport_matches_insufficient_boundaries() {
        let start = OffsetDateTime::now_utc();
        let points = make_track_points(60);
        let boundaries = vec![start]; // Only 1 boundary, need at least 2
        let segment_types = vec![];

        let matches = vec![(
            SegmentMatch {
                segment_id: Uuid::new_v4(),
                distance_meters: 1000.0,
                start_fraction: 0.1,
                end_fraction: 0.3,
            },
            builtin_types::RUN,
        )];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_multi_sport_segment_spanning_boundary() {
        let start = OffsetDateTime::now_utc();
        let points: Vec<TrackPointData> = (0..100)
            .map(|i| {
                let offset = Duration::minutes(i);
                TrackPointData {
                    lat: 40.0 + (i as f64 * 0.0001),
                    lon: -105.3 + (i as f64 * 0.0001),
                    elevation: Some(1650.0),
                    timestamp: Some(start + offset),
                }
            })
            .collect();

        // Activity: RUN (0-50 min), MTB (50-100 min)
        // Note: Track is 0-99 minutes, so 50% = 49.5 min which is in RUN portion
        let boundaries = vec![
            start,
            start + Duration::minutes(50),
            start + Duration::minutes(99),
        ];
        let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

        // Segment that spans the boundary (40% to 60%)
        // Midpoint = 50% of 99 min = 49.5 min, which is < 50 min boundary, so in RUN portion
        let matches = vec![(
            SegmentMatch {
                segment_id: Uuid::new_v4(),
                distance_meters: 2000.0,
                start_fraction: 0.4, // 40% into track
                end_fraction: 0.6,   // 60% into track
            },
            builtin_types::RUN, // Segment is RUN type (midpoint is in RUN portion)
        )];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        // Midpoint (50% of 99min = 49.5min) falls just before the 50min boundary, in RUN portion
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_multi_sport_segment_clearly_in_second_portion() {
        let start = OffsetDateTime::now_utc();
        let points: Vec<TrackPointData> = (0..100)
            .map(|i| {
                let offset = Duration::minutes(i);
                TrackPointData {
                    lat: 40.0 + (i as f64 * 0.0001),
                    lon: -105.3 + (i as f64 * 0.0001),
                    elevation: Some(1650.0),
                    timestamp: Some(start + offset),
                }
            })
            .collect();

        // Activity: RUN (0-50 min), MTB (50-100 min)
        let boundaries = vec![
            start,
            start + Duration::minutes(50),
            start + Duration::minutes(99),
        ];
        let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

        // Segment clearly in the MTB portion (60% to 80%)
        // Midpoint = 70% of 99 min = 69.3 min, which is > 50 min, so in MTB portion
        let matches = vec![(
            SegmentMatch {
                segment_id: Uuid::new_v4(),
                distance_meters: 2000.0,
                start_fraction: 0.6,
                end_fraction: 0.8,
            },
            builtin_types::MTB, // Segment is MTB type
        )];

        let result = filter_multi_sport_matches(matches, &points, &boundaries, &segment_types);
        assert_eq!(result.len(), 1);
    }
}
