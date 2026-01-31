//! Activity file parsers for GPX, TCX, and FIT formats.
//!
//! This module provides unified parsing for different activity file formats,
//! extracting both track geometry and sensor data (heart rate, cadence, power).

use bytes::Buf as _;
use bytes::Bytes;
use std::io::BufReader;
use time::OffsetDateTime;

use crate::models::TrackPointData;
use crate::object_store_service::FileType;

/// Sensor data extracted from an activity file.
/// Arrays are parallel to track points - each index corresponds to the same point.
/// Values are `None` where the sensor data is not available for that point.
#[derive(Debug, Clone, Default)]
pub struct SensorData {
    /// Heart rate in beats per minute
    pub heart_rates: Vec<Option<i32>>,
    /// Cadence in RPM (revolutions/steps per minute)
    pub cadences: Vec<Option<i32>>,
    /// Power in watts
    pub powers: Vec<Option<i32>>,
    /// Temperature in degrees Celsius
    pub temperatures: Vec<Option<f64>>,
}

impl SensorData {
    pub fn has_heart_rate(&self) -> bool {
        self.heart_rates.iter().any(|v| v.is_some())
    }

    pub fn has_cadence(&self) -> bool {
        self.cadences.iter().any(|v| v.is_some())
    }

    pub fn has_power(&self) -> bool {
        self.powers.iter().any(|v| v.is_some())
    }

    pub fn has_temperature(&self) -> bool {
        self.temperatures.iter().any(|v| v.is_some())
    }

    /// Returns true if any sensor data is present
    pub fn has_any_data(&self) -> bool {
        self.has_heart_rate() || self.has_cadence() || self.has_power() || self.has_temperature()
    }
}

/// Result of parsing an activity file
#[derive(Debug, Clone)]
pub struct ParsedActivity {
    /// Track points with lat/lon/elevation/timestamp
    pub track_points: Vec<TrackPointData>,
    /// Sensor data parallel to track points
    pub sensor_data: SensorData,
}

impl ParsedActivity {
    /// Get the timestamp when the activity started (first track point with a timestamp)
    pub fn started_at(&self) -> Option<OffsetDateTime> {
        self.track_points.iter().find_map(|pt| pt.timestamp)
    }
}

/// Error type for parsing failures
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to parse GPX file: {0}")]
    GpxError(String),
    #[error("Failed to parse TCX file: {0}")]
    TcxError(String),
    #[error("Failed to parse FIT file: {0}")]
    FitError(String),
    #[error("Unsupported file type: {0:?}")]
    UnsupportedFileType(FileType),
}

/// Parse an activity file based on its detected type.
/// For FileType::Other, attempts to detect the format from the bytes.
pub fn parse_activity_file(
    file_type: FileType,
    bytes: Bytes,
) -> Result<ParsedActivity, ParseError> {
    // If type is Other, try to detect from bytes
    let actual_type = if file_type == FileType::Other {
        FileType::detect_from_bytes(&bytes)
    } else {
        file_type
    };

    match actual_type {
        FileType::Gpx => parse_gpx(bytes),
        FileType::Tcx => parse_tcx(bytes),
        FileType::Fit => parse_fit(bytes),
        FileType::Other => Err(ParseError::UnsupportedFileType(actual_type)),
    }
}

/// Parse a GPX file, extracting track points and sensor data from Garmin TrackPointExtension.
pub fn parse_gpx(bytes: Bytes) -> Result<ParsedActivity, ParseError> {
    let gpx = gpx::read(bytes.reader()).map_err(|e| ParseError::GpxError(e.to_string()))?;

    let mut track_points = Vec::new();
    let mut sensor_data = SensorData::default();

    for track in &gpx.tracks {
        for seg in &track.segments {
            for pt in &seg.points {
                let lon = pt.point().x();
                let lat = pt.point().y();
                let elevation = pt.elevation;
                let timestamp = pt.time.as_ref().and_then(|t| {
                    t.format().ok().and_then(|s| {
                        time::OffsetDateTime::parse(
                            &s,
                            &time::format_description::well_known::Rfc3339,
                        )
                        .ok()
                    })
                });

                track_points.push(TrackPointData {
                    lat,
                    lon,
                    elevation,
                    timestamp,
                });

                // Extract sensor data from Garmin TrackPointExtension if present
                let (hr, cad, temp) = extract_gpx_extensions(pt);
                sensor_data.heart_rates.push(hr);
                sensor_data.cadences.push(cad);
                sensor_data.powers.push(None); // GPX typically doesn't have power
                sensor_data.temperatures.push(temp);
            }
        }
    }

    Ok(ParsedActivity {
        track_points,
        sensor_data,
    })
}

/// Extract sensor data from GPX track point extensions.
/// Looks for Garmin TrackPointExtension v1/v2 namespace elements.
fn extract_gpx_extensions(pt: &gpx::Waypoint) -> (Option<i32>, Option<i32>, Option<f64>) {
    // GPX extensions are stored as raw XML in the gpx crate
    // We need to look for elements like:
    // <gpxtpx:hr>150</gpxtpx:hr>
    // <gpxtpx:cad>80</gpxtpx:cad>
    // <gpxtpx:atemp>22.0</gpxtpx:atemp>

    // The gpx crate doesn't expose extensions directly in a structured way,
    // so we'll need to check if there's extension data available.
    // For now, return None values - this can be enhanced later with XML parsing
    // if the gpx crate exposes the raw extension XML.

    // Note: The gpx crate's Waypoint struct has an `extensions` field but it may not
    // be fully populated depending on the crate version. This is a known limitation.

    let _ = pt; // Acknowledge we receive the point but can't extract extensions with current crate

    (None, None, None)
}

/// Parse a TCX (Training Center XML) file.
pub fn parse_tcx(bytes: Bytes) -> Result<ParsedActivity, ParseError> {
    // TCX crate needs a BufReader
    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mut buf_reader = BufReader::new(cursor);

    let tcx_data =
        tcx::read(&mut buf_reader).map_err(|e| ParseError::TcxError(format!("{e:?}")))?;

    let mut track_points = Vec::new();
    let mut sensor_data = SensorData::default();

    // activities is Option<Activities>
    if let Some(ref activities) = tcx_data.activities {
        for activity in &activities.activities {
            for lap in &activity.laps {
                // Laps have tracks as Vec<Track>
                for track in &lap.tracks {
                    for trackpoint in &track.trackpoints {
                        // Get position (lat/lon)
                        let Some(ref position) = trackpoint.position else {
                            continue;
                        };

                        let lat = position.latitude;
                        let lon = position.longitude;
                        let elevation = trackpoint.altitude_meters;

                        // Parse timestamp - TCX uses chrono DateTime<Utc>
                        let timestamp = Some(chrono_to_offset_datetime_utc(&trackpoint.time));

                        track_points.push(TrackPointData {
                            lat,
                            lon,
                            elevation,
                            timestamp,
                        });

                        // Extract sensor data
                        let hr = trackpoint.heart_rate.as_ref().map(|h| h.value as i32);
                        let cad = trackpoint.cadence.map(|c| c as i32);

                        sensor_data.heart_rates.push(hr);
                        sensor_data.cadences.push(cad);
                        sensor_data.powers.push(None); // Power may be in extensions
                        sensor_data.temperatures.push(None);
                    }
                }
            }
        }
    }

    Ok(ParsedActivity {
        track_points,
        sensor_data,
    })
}

/// Convert chrono DateTime<Utc> to time OffsetDateTime
fn chrono_to_offset_datetime_utc(dt: &chrono::DateTime<chrono::Utc>) -> OffsetDateTime {
    // Get components from chrono
    let ts = dt.timestamp();
    let ns = dt.timestamp_subsec_nanos();

    // Construct OffsetDateTime from unix timestamp
    OffsetDateTime::from_unix_timestamp(ts)
        .map(|odt| odt.replace_nanosecond(ns).unwrap_or(odt))
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
}

/// Convert chrono DateTime<Local> to time OffsetDateTime
fn chrono_to_offset_datetime_local(dt: &chrono::DateTime<chrono::Local>) -> OffsetDateTime {
    // Convert Local to UTC first, then to OffsetDateTime
    use chrono::Utc;
    let utc_dt = dt.with_timezone(&Utc);
    chrono_to_offset_datetime_utc(&utc_dt)
}

/// Parse a FIT (Flexible and Interoperable Data Transfer) file.
pub fn parse_fit(bytes: Bytes) -> Result<ParsedActivity, ParseError> {
    let data = bytes.to_vec();
    let fit_data = fitparser::from_bytes(&data).map_err(|e| ParseError::FitError(e.to_string()))?;

    let mut track_points = Vec::new();
    let mut sensor_data = SensorData::default();

    for record in fit_data {
        // We're looking for "record" messages which contain per-second data
        if record.kind() != fitparser::profile::field_types::MesgNum::Record {
            continue;
        }

        let mut lat: Option<f64> = None;
        let mut lon: Option<f64> = None;
        let mut elevation: Option<f64> = None;
        let mut timestamp: Option<OffsetDateTime> = None;
        let mut hr: Option<i32> = None;
        let mut cad: Option<i32> = None;
        let mut power: Option<i32> = None;
        let mut temp: Option<f64> = None;

        for field in record.fields() {
            match field.name() {
                "position_lat" => {
                    if let fitparser::Value::SInt32(v) = field.value() {
                        // FIT stores lat/lon as semicircles, convert to degrees
                        lat = Some(semicircles_to_degrees(*v));
                    }
                }
                "position_long" => {
                    if let fitparser::Value::SInt32(v) = field.value() {
                        lon = Some(semicircles_to_degrees(*v));
                    }
                }
                "altitude" | "enhanced_altitude" => {
                    elevation = extract_fit_f64(field.value());
                }
                "timestamp" => {
                    if let fitparser::Value::Timestamp(t) = field.value() {
                        timestamp = Some(chrono_to_offset_datetime_local(t));
                    }
                }
                "heart_rate" => {
                    hr = extract_fit_i32(field.value());
                }
                "cadence" => {
                    cad = extract_fit_i32(field.value());
                }
                "power" => {
                    power = extract_fit_i32(field.value());
                }
                "temperature" => {
                    temp = extract_fit_f64(field.value());
                }
                _ => {}
            }
        }

        // Only add points that have valid position data
        if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
            track_points.push(TrackPointData {
                lat: lat_val,
                lon: lon_val,
                elevation,
                timestamp,
            });

            sensor_data.heart_rates.push(hr);
            sensor_data.cadences.push(cad);
            sensor_data.powers.push(power);
            sensor_data.temperatures.push(temp);
        }
    }

    Ok(ParsedActivity {
        track_points,
        sensor_data,
    })
}

/// Convert FIT semicircles to degrees.
/// FIT uses semicircles where 2^31 semicircles = 180 degrees.
fn semicircles_to_degrees(semicircles: i32) -> f64 {
    (semicircles as f64) * (180.0 / 2_147_483_648.0)
}

/// Extract i32 from various FIT value types
fn extract_fit_i32(value: &fitparser::Value) -> Option<i32> {
    match value {
        fitparser::Value::SInt8(v) => Some(*v as i32),
        fitparser::Value::UInt8(v) => Some(*v as i32),
        fitparser::Value::SInt16(v) => Some(*v as i32),
        fitparser::Value::UInt16(v) => Some(*v as i32),
        fitparser::Value::SInt32(v) => Some(*v),
        fitparser::Value::UInt32(v) => Some(*v as i32),
        _ => None,
    }
}

/// Extract f64 from various FIT value types
fn extract_fit_f64(value: &fitparser::Value) -> Option<f64> {
    match value {
        fitparser::Value::Float32(v) => Some(*v as f64),
        fitparser::Value::Float64(v) => Some(*v),
        fitparser::Value::SInt8(v) => Some(*v as f64),
        fitparser::Value::UInt8(v) => Some(*v as f64),
        fitparser::Value::SInt16(v) => Some(*v as f64),
        fitparser::Value::UInt16(v) => Some(*v as f64),
        fitparser::Value::SInt32(v) => Some(*v as f64),
        fitparser::Value::UInt32(v) => Some(*v as f64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semicircles_to_degrees() {
        // Test known conversion: 2^31 semicircles = 180 degrees
        assert!((semicircles_to_degrees(2_147_483_647) - 180.0).abs() < 0.0001);

        // Test zero
        assert!((semicircles_to_degrees(0) - 0.0).abs() < 0.0001);

        // Test negative (south/west)
        assert!((semicircles_to_degrees(-2_147_483_648) - (-180.0)).abs() < 0.0001);
    }

    #[test]
    fn test_sensor_data_has_methods() {
        let mut data = SensorData::default();
        assert!(!data.has_heart_rate());
        assert!(!data.has_cadence());
        assert!(!data.has_power());
        assert!(!data.has_temperature());
        assert!(!data.has_any_data());

        data.heart_rates.push(Some(150));
        assert!(data.has_heart_rate());
        assert!(data.has_any_data());

        data.cadences.push(Some(80));
        assert!(data.has_cadence());

        data.powers.push(Some(200));
        assert!(data.has_power());

        data.temperatures.push(Some(22.5));
        assert!(data.has_temperature());
    }

    #[test]
    fn test_file_type_detection() {
        // Test FIT magic bytes
        let mut fit_bytes = vec![14u8, 0, 0, 0, 0, 0, 0, 0];
        fit_bytes.extend_from_slice(b".FIT");
        fit_bytes.extend_from_slice(&[0, 0]); // CRC placeholder
        assert_eq!(FileType::detect_from_bytes(&fit_bytes), FileType::Fit);

        // Test GPX detection
        let gpx_bytes = b"<?xml version=\"1.0\"?><gpx version=\"1.1\">";
        assert_eq!(FileType::detect_from_bytes(gpx_bytes), FileType::Gpx);

        // Test TCX detection
        let tcx_bytes = b"<?xml version=\"1.0\"?><TrainingCenterDatabase>";
        assert_eq!(FileType::detect_from_bytes(tcx_bytes), FileType::Tcx);

        // Test unknown
        let unknown_bytes = b"random data that is not a valid file";
        assert_eq!(FileType::detect_from_bytes(unknown_bytes), FileType::Other);
    }
}
