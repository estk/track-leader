use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use bytes::{Buf as _, Bytes};
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::{
    database::Database, models::TrackScoringMetricTags, object_store_service::FileType, scoring,
};

#[derive(Clone)]
pub struct ActivityQueue {
    db: Database,
    trt: Arc<Runtime>,
    pool: Arc<rayon::ThreadPool>,
    activities: Arc<Mutex<HashSet<Uuid>>>,
    done_tx: std::sync::mpsc::Sender<Uuid>,
}

impl ActivityQueue {
    pub fn new(db: Database) -> Self {
        let rpool = rayon::ThreadPoolBuilder::new().build().unwrap();
        let trt = Arc::new(Runtime::new().unwrap());
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
            trt,
            activities,
            done_tx: tx,
        }
    }
    pub fn submit(
        &self,
        uid: Uuid,
        id: Uuid,
        scoring_metrics: TrackScoringMetricTags,
        ft: FileType,
        bytes: Bytes,
    ) -> anyhow::Result<()> {
        assert!(matches!(ft, FileType::Gpx));

        self.activities.lock().unwrap().insert(id);
        let tx = self.done_tx.clone();
        let db = self.db.clone();
        let trt = self.trt.clone();
        self.pool.spawn(move || {
            let parsed_track = gpx::read(bytes.reader()).unwrap();

            let scores = scoring::score_track(scoring_metrics, &parsed_track);

            trt.block_on(async move {
                db.save_scores(uid, id, scores).await.unwrap();
            });
            tx.send(id).unwrap();
        });
        Ok(())
    }
}
