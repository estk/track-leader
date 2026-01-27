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
    models::ActivityType,
    object_store_service::FileType,
    scoring,
    segment_matching::{self, SegmentMatch},
};

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
        activity_type: ActivityType,
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

            // Build WKT LINESTRING from track points
            let geo_wkt = build_track_wkt(&parsed_track);

            handle.block_on(async move {
                // Save scores
                db.save_scores(uid, id, scores).await.unwrap();

                // Save track geometry and find segment matches
                if let Some(ref wkt) = geo_wkt
                    && let Err(e) = db.save_track_geometry(uid, id, wkt).await
                {
                    tracing::error!("Failed to save track geometry: {e}");
                }

                // Find and create segment efforts
                if geo_wkt.is_some() {
                    match db.find_matching_segments(id, &activity_type).await {
                        Ok(matches) => {
                            for segment_match in matches {
                                process_segment_match(&db, &parsed_track, uid, id, segment_match)
                                    .await;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to find matching segments: {e}");
                        }
                    }
                }
            });
            tx.send(id).unwrap();
        });
        Ok(())
    }
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
