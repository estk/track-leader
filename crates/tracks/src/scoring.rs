use enumflags2::{bitflags, BitFlags};
use geo::{geometry::Point, Distance as _, Haversine};
use gpx::Gpx;

type TrackPoint = gpx::Waypoint;

pub trait TrackMetric {
    type Score;
    fn next_point(&mut self, point: &TrackPoint);
    fn finish(&mut self) -> Self::Score;
}

pub fn score_track(tags: BitFlags<TrackScoringMetricTag>, track: &Gpx) -> Scores {
    let mut acc = Metrics::new(tags);

    // todo: revisit
    for track in &track.tracks {
        for seg in &track.segments {
            for point in &seg.points {
                acc.next_point(&point);
            }
        }
    }

    acc.finish()
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum TrackScoringMetricTag {
    Distance,
    Duration,
    ElevationGain,
}

#[derive(Debug, Clone, Default)]
struct Scores {
    distance: f64,
    duration: f64,
    elevation_gain: f64,
}
#[derive(Debug, Clone, Default)]
struct Metrics {
    distance: Option<DistanceMetric>,
    duration: Option<DurationMetric>,
    elevation_gain: Option<ElevationGainMetric>,
}
impl Metrics {
    fn new(tags: BitFlags<TrackScoringMetricTag>) -> Self {
        let mut this = Self::default();
        for t in tags {
            match t {
                TrackScoringMetricTag::Distance => this.distance = Some(DistanceMetric::default()),
                TrackScoringMetricTag::Duration => this.duration = Some(DurationMetric::default()),
                TrackScoringMetricTag::ElevationGain => {
                    this.elevation_gain = Some(ElevationGainMetric::default())
                }
            };
        }
        this
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
