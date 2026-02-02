//! API-based seeding for activities.
//!
//! Uploads activities via the HTTP API to ensure DIG part extraction
//! happens through the production code path in `activity_queue.rs`.

use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::generators::GeneratedActivity;
use crate::gpx::generate_gpx;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Login failed: {0}")]
    LoginFailed(String),
    #[error("Upload failed: {0}")]
    UploadFailed(String),
    #[error("Backend not reachable at {0}")]
    BackendNotReachable(String),
}

/// Response from the login endpoint.
#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
}

/// Response from the activity creation endpoint.
#[derive(Debug, Deserialize)]
struct ActivityResponse {
    id: Uuid,
}

/// API seeder that uploads activities via HTTP.
///
/// This ensures activities go through the same processing pipeline
/// as real uploads, including DIG part extraction.
pub struct ApiSeeder {
    client: Client,
    base_url: String,
}

impl ApiSeeder {
    /// Creates a new API seeder for the given backend URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Checks if the backend is reachable.
    pub async fn check_health(&self) -> Result<(), ApiError> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => Err(ApiError::BackendNotReachable(format!(
                "Health check returned status {}",
                resp.status()
            ))),
            Err(e) => Err(ApiError::BackendNotReachable(e.to_string())),
        }
    }

    /// Logs in a user and returns their auth token.
    pub async fn login(&self, email: &str, password: &str) -> Result<String, ApiError> {
        let url = format!("{}/auth/login", self.base_url);

        #[derive(Serialize)]
        struct LoginRequest<'a> {
            email: &'a str,
            password: &'a str,
        }

        let resp = self
            .client
            .post(&url)
            .json(&LoginRequest { email, password })
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::LoginFailed(format!(
                "Status {status}: {body}"
            )));
        }

        let login_resp: LoginResponse = resp.json().await?;
        Ok(login_resp.token)
    }

    /// Uploads an activity via the API.
    ///
    /// This triggers the activity queue processing which includes DIG part extraction.
    pub async fn upload_activity(
        &self,
        token: &str,
        activity: &GeneratedActivity,
    ) -> Result<Uuid, ApiError> {
        let url = format!("{}/activities/new", self.base_url);

        // Generate GPX from track points
        let gpx_bytes = generate_gpx(&activity.track_points, &activity.name);

        // Build multipart form
        let file_part = Part::bytes(gpx_bytes)
            .file_name("activity.gpx")
            .mime_str("application/gpx+xml")
            .map_err(|e| ApiError::UploadFailed(e.to_string()))?;

        let form = Form::new().part("file", file_part);

        // Build query params
        let mut query_params = vec![
            ("activity_type_id", activity.activity_type_id.to_string()),
            ("name", activity.name.clone()),
            ("visibility", activity.visibility.as_str().to_string()),
        ];

        // Add multi-sport params if present
        if let (Some(boundaries), Some(types)) =
            (&activity.type_boundaries, &activity.segment_types)
        {
            let boundaries_str = format_boundaries(boundaries);
            let types_str = format_types(types);
            query_params.push(("type_boundaries", boundaries_str));
            query_params.push(("segment_types", types_str));
        }

        debug!(
            "Uploading activity {} with {} track points",
            activity.name,
            activity.track_points.len()
        );

        let resp = self
            .client
            .post(&url)
            .bearer_auth(token)
            .query(&query_params)
            .multipart(form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::UploadFailed(format!(
                "Status {status}: {body}"
            )));
        }

        let activity_resp: ActivityResponse = resp.json().await?;
        Ok(activity_resp.id)
    }

    /// Uploads activities for a user, logging in first.
    ///
    /// Returns the number of successfully uploaded activities.
    pub async fn upload_user_activities(
        &self,
        email: &str,
        password: &str,
        activities: &[GeneratedActivity],
    ) -> Result<usize, ApiError> {
        if activities.is_empty() {
            return Ok(0);
        }

        let token = self.login(email, password).await?;
        let mut uploaded = 0;

        for activity in activities {
            match self.upload_activity(&token, activity).await {
                Ok(id) => {
                    debug!("Uploaded activity {}: {}", activity.name, id);
                    uploaded += 1;
                }
                Err(e) => {
                    warn!("Failed to upload activity {}: {}", activity.name, e);
                }
            }
        }

        Ok(uploaded)
    }
}

/// Formats type boundaries as comma-separated ISO-8601 timestamps.
fn format_boundaries(boundaries: &[OffsetDateTime]) -> String {
    boundaries
        .iter()
        .map(|ts| {
            ts.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Formats segment types as comma-separated UUIDs.
fn format_types(types: &[Uuid]) -> String {
    types
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Reads the backend port from .dev-ports file.
///
/// Returns the full backend URL or None if the file doesn't exist.
pub fn read_backend_url_from_dev_ports() -> Option<String> {
    use std::path::Path;

    let dev_ports_path = Path::new(".dev-ports");
    if !dev_ports_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(dev_ports_path).ok()?;
    for line in contents.lines() {
        if let Some(port_str) = line.strip_prefix("BACKEND_PORT=") {
            if let Ok(port) = port_str.trim().parse::<u16>() {
                return Some(format!("http://localhost:{}", port));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;
    use tracks::models::builtin_types;

    #[test]
    fn test_format_boundaries() {
        let now = OffsetDateTime::now_utc();
        let boundaries = vec![now, now + Duration::hours(1), now + Duration::hours(2)];
        let formatted = format_boundaries(&boundaries);

        // Should have 2 commas for 3 items
        assert_eq!(formatted.matches(',').count(), 2);
    }

    #[test]
    fn test_format_types() {
        let types = vec![builtin_types::MTB, builtin_types::DIG, builtin_types::MTB];
        let formatted = format_types(&types);

        // Should have 2 commas for 3 items
        assert_eq!(formatted.matches(',').count(), 2);
        assert!(formatted.contains(&builtin_types::MTB.to_string()));
        assert!(formatted.contains(&builtin_types::DIG.to_string()));
    }

    #[test]
    fn test_api_seeder_creation() {
        let seeder = ApiSeeder::new("http://localhost:3000");
        assert_eq!(seeder.base_url, "http://localhost:3000");
    }
}
