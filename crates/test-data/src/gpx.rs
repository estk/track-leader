//! GPX file generation from track points.
//!
//! Generates valid GPX 1.1 XML format for uploading activities via the API.

use tracks::models::TrackPointData;

/// Generates a GPX 1.1 XML string from track points.
///
/// The generated GPX includes:
/// - Standard GPX 1.1 header with schema declarations
/// - Single track with a single track segment
/// - Each point includes lat, lon, elevation, and timestamp
pub fn generate_gpx(points: &[TrackPointData], activity_name: &str) -> Vec<u8> {
    let mut gpx = String::new();

    // GPX 1.1 header
    gpx.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    gpx.push('\n');
    gpx.push_str(r#"<gpx version="1.1" creator="track-leader-test-data""#);
    gpx.push_str(r#" xmlns="http://www.topografix.com/GPX/1/1""#);
    gpx.push_str(r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#);
    gpx.push_str(r#" xsi:schemaLocation="http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd">"#);
    gpx.push('\n');

    // Metadata with activity name
    gpx.push_str("  <metadata>\n");
    gpx.push_str(&format!("    <name>{}</name>\n", escape_xml(activity_name)));
    gpx.push_str("  </metadata>\n");

    // Track
    gpx.push_str("  <trk>\n");
    gpx.push_str(&format!("    <name>{}</name>\n", escape_xml(activity_name)));
    gpx.push_str("    <trkseg>\n");

    for point in points {
        gpx.push_str(&format!(
            r#"      <trkpt lat="{:.7}" lon="{:.7}">"#,
            point.lat, point.lon
        ));
        gpx.push('\n');

        if let Some(ele) = point.elevation {
            gpx.push_str(&format!("        <ele>{:.2}</ele>\n", ele));
        }

        if let Some(ts) = point.timestamp {
            // Format as ISO 8601 / RFC 3339
            let formatted = ts
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            gpx.push_str(&format!("        <time>{}</time>\n", formatted));
        }

        gpx.push_str("      </trkpt>\n");
    }

    gpx.push_str("    </trkseg>\n");
    gpx.push_str("  </trk>\n");
    gpx.push_str("</gpx>\n");

    gpx.into_bytes()
}

/// Escapes XML special characters in a string.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn test_generate_gpx_basic() {
        let now = OffsetDateTime::now_utc();
        let points = vec![
            TrackPointData {
                lat: 40.0150,
                lon: -105.2705,
                elevation: Some(1650.0),
                timestamp: Some(now),
            },
            TrackPointData {
                lat: 40.0160,
                lon: -105.2695,
                elevation: Some(1660.0),
                timestamp: Some(now + time::Duration::seconds(60)),
            },
        ];

        let gpx = generate_gpx(&points, "Test Activity");
        let gpx_str = String::from_utf8(gpx).unwrap();

        assert!(gpx_str.contains(r#"version="1.1""#));
        assert!(gpx_str.contains("<name>Test Activity</name>"));
        assert!(gpx_str.contains(r#"lat="40.0150000""#));
        assert!(gpx_str.contains(r#"lon="-105.2705000""#));
        assert!(gpx_str.contains("<ele>1650.00</ele>"));
        assert!(gpx_str.contains("<time>"));
    }

    #[test]
    fn test_generate_gpx_escapes_special_chars() {
        let points = vec![TrackPointData {
            lat: 40.0,
            lon: -105.0,
            elevation: None,
            timestamp: None,
        }];

        let gpx = generate_gpx(&points, "Test & <Activity> \"Name\"");
        let gpx_str = String::from_utf8(gpx).unwrap();

        assert!(gpx_str.contains("Test &amp; &lt;Activity&gt; &quot;Name&quot;"));
    }

    #[test]
    fn test_generate_gpx_without_optional_fields() {
        let points = vec![TrackPointData {
            lat: 40.0,
            lon: -105.0,
            elevation: None,
            timestamp: None,
        }];

        let gpx = generate_gpx(&points, "Simple Track");
        let gpx_str = String::from_utf8(gpx).unwrap();

        assert!(!gpx_str.contains("<ele>"));
        assert!(!gpx_str.contains("<time>"));
        assert!(gpx_str.contains(r#"lat="40.0000000""#));
    }
}
