use geo::{geometry::Point, Distance as _, Haversine};
use gpx::Gpx;

use crate::models::{Scores, };

type TrackPoint = gpx::Waypoint;

pub trait TrackMetric {
    type Score;
    fn next_point(&mut self, point: &TrackPoint);
    fn finish(&mut self) -> Self::Score;
}

pub fn score_track( track: &Gpx) -> Scores {
    let mut acc = Metrics::new();

    // todo: revisit
    for track in &track.tracks {
        for seg in &track.segments {
            for point in &seg.points {
                acc.next_point(point);
            }
        }
    }

    acc.finish()
}

#[derive(Debug, Clone, Default)]
struct Metrics {
    distance: Option<DistanceMetric>,
    duration: Option<DurationMetric>,
    elevation_gain: Option<ElevationGainMetric>,
}
impl Metrics {
    fn new() -> Self {
        Self {
            distance: Some(DistanceMetric::default()),
            duration: Some(DurationMetric::default()),
            elevation_gain: Some(ElevationGainMetric::default()),
        }
    }
}
impl TrackMetric for Metrics {
    type Score = Scores;
    fn next_point(&mut self, point: &TrackPoint) {
        if let Some(distance) = &mut self.distance {
            distance.next_point(point);
        }
        if let Some(duration) = &mut self.duration {
            duration.next_point(point);
        }
        if let Some(elevation_gain) = &mut self.elevation_gain {
            elevation_gain.next_point(point);
        }
    }

    fn finish(&mut self) -> Scores {
        let mut scores = Scores::default();
        if let Some(distance) = &mut self.distance {
            scores.distance = distance.finish();
        }
        if let Some(duration) = &mut self.duration {
            scores.duration = duration.finish();
        }
        if let Some(elevation_gain) = &mut self.elevation_gain {
            scores.elevation_gain = elevation_gain.finish();
        }
        scores
    }
}

#[derive(Debug, Clone, Default)]
struct DistanceMetric {
    total_distance: f64,
    last_point: Option<Point>,
}

impl TrackMetric for DistanceMetric {
    type Score = f64;
    fn next_point(&mut self, wpt: &TrackPoint) {
        self.total_distance += self
            .last_point
            .map_or(0.0, |prev| Haversine.distance(prev, wpt.point()));
        // self.total_distance += distance_between(self.last_point, wpt.point());
        self.last_point = Some(wpt.point());
    }

    fn finish(&mut self) -> f64 {
        self.total_distance
    }
}

#[derive(Debug, Clone, Default)]
struct DurationMetric {
    start_time: Option<gpx::Time>,
    end_time: Option<gpx::Time>,
}

impl TrackMetric for DurationMetric {
    type Score = f64;
    fn next_point(&mut self, wpt: &TrackPoint) {
        if let Some(time) = wpt.time {
            if self.start_time.is_none() {
                self.start_time = Some(time);
            }
            self.end_time = Some(time);
        }
    }

    fn finish(&mut self) -> f64 {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => {
                let start_dt: time::OffsetDateTime = start.into();
                let end_dt: time::OffsetDateTime = end.into();
                (end_dt - start_dt).as_seconds_f64()
            }
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ElevationGainMetric {
    total_gain: f64,
    last_elevation: Option<f64>,
}

impl TrackMetric for ElevationGainMetric {
    type Score = f64;
    fn next_point(&mut self, wpt: &TrackPoint) {
        if let Some(elevation) = wpt.elevation {
            if let Some(last_elev) = self.last_elevation {
                let gain = elevation - last_elev;
                if gain > 0.0 {
                    self.total_gain += gain;
                }
            }
            self.last_elevation = Some(elevation);
        }
    }

    fn finish(&mut self) -> f64 {
        self.total_gain
    }
}
