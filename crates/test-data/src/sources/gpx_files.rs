//! GPX file loading utilities.

use std::path::Path;

use gpx::{Gpx, read};
use thiserror::Error;
use time::OffsetDateTime;
use tracks::models::TrackPointData;

#[derive(Debug, Error)]
pub enum GpxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GPX parse error: {0}")]
    Parse(#[from] gpx::errors::GpxError),
    #[error("No tracks found in GPX file")]
    NoTracks,
    #[error("No track segments found")]
    NoSegments,
}

/// Loads GPS track data from GPX files.
pub struct GpxLoader;

impl GpxLoader {
    /// Loads track points from a GPX file.
    ///
    /// Returns all points from all tracks and segments in the file,
    /// flattened into a single vector.
    pub fn load_file(path: impl AsRef<Path>) -> Result<Vec<TrackPointData>, GpxError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let gpx: Gpx = read(reader)?;

        Self::extract_points(&gpx)
    }

    /// Loads track points from GPX data in memory.
    pub fn load_bytes(data: &[u8]) -> Result<Vec<TrackPointData>, GpxError> {
        let reader = std::io::Cursor::new(data);
        let gpx: Gpx = read(reader)?;

        Self::extract_points(&gpx)
    }

    /// Extracts track points from a parsed GPX structure.
    fn extract_points(gpx: &Gpx) -> Result<Vec<TrackPointData>, GpxError> {
        if gpx.tracks.is_empty() {
            return Err(GpxError::NoTracks);
        }

        let mut points = Vec::new();

        for track in &gpx.tracks {
            for segment in &track.segments {
                for waypoint in &segment.points {
                    let point = waypoint.point();
                    // gpx::Time wraps time::OffsetDateTime and implements From
                    let timestamp = waypoint.time.map(|t| OffsetDateTime::from(t));

                    points.push(TrackPointData {
                        lat: point.y(),
                        lon: point.x(),
                        elevation: waypoint.elevation,
                        timestamp,
                    });
                }
            }
        }

        if points.is_empty() {
            return Err(GpxError::NoSegments);
        }

        Ok(points)
    }

    /// Writes track points to a GPX file.
    ///
    /// Useful for exporting generated tracks for visualization in other tools.
    pub fn write_file(
        path: impl AsRef<Path>,
        points: &[TrackPointData],
        name: Option<&str>,
    ) -> Result<(), GpxError> {
        use geo::Point;
        use gpx::{Gpx, GpxVersion, Track, TrackSegment, Waypoint};

        let waypoints: Vec<Waypoint> = points
            .iter()
            .map(|p| {
                let mut wp = Waypoint::new(Point::new(p.lon, p.lat));
                wp.elevation = p.elevation;
                wp.time = p.timestamp.map(|t| {
                    gpx::Time::from(
                        time::OffsetDateTime::from_unix_timestamp(t.unix_timestamp())
                            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH),
                    )
                });
                wp
            })
            .collect();

        let segment = TrackSegment { points: waypoints };
        let mut track = Track::new();
        track.name = name.map(String::from);
        track.segments = vec![segment];

        let gpx = Gpx {
            version: GpxVersion::Gpx11,
            tracks: vec![track],
            ..Default::default()
        };

        let file = std::fs::File::create(path)?;
        gpx::write(&gpx, file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let points = vec![
            TrackPointData {
                lat: 40.0,
                lon: -105.3,
                elevation: Some(1650.0),
                timestamp: Some(OffsetDateTime::now_utc()),
            },
            TrackPointData {
                lat: 40.01,
                lon: -105.29,
                elevation: Some(1680.0),
                timestamp: Some(OffsetDateTime::now_utc()),
            },
        ];

        let temp_path = std::env::temp_dir().join("test_track.gpx");
        GpxLoader::write_file(&temp_path, &points, Some("Test Track")).unwrap();

        let loaded = GpxLoader::load_file(&temp_path).unwrap();
        assert_eq!(loaded.len(), 2);

        // Clean up
        std::fs::remove_file(temp_path).ok();
    }
}
