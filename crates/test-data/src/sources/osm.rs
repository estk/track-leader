//! OpenStreetMap Overpass API client for fetching route skeletons.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::BoundingBox;

#[derive(Debug, Error)]
pub enum OsmError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No ways found in response")]
    NoWays,
    #[error("Rate limited, try again later")]
    RateLimited,
}

/// Response from Overpass API.
#[derive(Debug, Deserialize)]
struct OverpassResponse {
    elements: Vec<OverpassElement>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum OverpassElement {
    #[serde(rename = "node")]
    Node {
        id: i64,
        lat: f64,
        lon: f64,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
    #[serde(rename = "way")]
    Way {
        id: i64,
        nodes: Vec<i64>,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
}

/// A route extracted from OSM data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmRoute {
    /// OSM way ID.
    pub id: i64,
    /// Route name from OSM tags.
    pub name: Option<String>,
    /// Highway/path type (e.g., "path", "footway", "cycleway").
    pub highway_type: Option<String>,
    /// Ordered coordinates (lat, lon).
    pub coords: Vec<(f64, f64)>,
}

/// Client for fetching route data from OpenStreetMap via Overpass API.
pub struct OsmClient {
    client: reqwest::Client,
    cache_dir: Option<PathBuf>,
    endpoint: String,
}

impl OsmClient {
    /// Creates a new OSM client with default endpoint.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache_dir: None,
            endpoint: "https://overpass-api.de/api/interpreter".to_string(),
        }
    }

    /// Enables file-based caching of API responses.
    ///
    /// Cached responses are stored as JSON files based on query hash.
    pub fn with_cache_dir(mut self, dir: impl AsRef<Path>) -> Self {
        let path = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&path).ok();
        self.cache_dir = Some(path);
        self
    }

    /// Sets a custom Overpass API endpoint.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Fetches hiking/walking trails within a bounding box.
    pub async fn fetch_trails(&self, bounds: BoundingBox) -> Result<Vec<OsmRoute>, OsmError> {
        let query = format!(
            r#"[out:json][timeout:60];
            (
              way["highway"="path"]({},{},{},{});
              way["highway"="footway"]({},{},{},{});
              way["highway"="track"]({},{},{},{});
            );
            out body;
            >;
            out skel qt;"#,
            bounds.min_lat,
            bounds.min_lon,
            bounds.max_lat,
            bounds.max_lon,
            bounds.min_lat,
            bounds.min_lon,
            bounds.max_lat,
            bounds.max_lon,
            bounds.min_lat,
            bounds.min_lon,
            bounds.max_lat,
            bounds.max_lon,
        );

        self.execute_query(&query).await
    }

    /// Fetches cycling routes within a bounding box.
    pub async fn fetch_cycle_routes(&self, bounds: BoundingBox) -> Result<Vec<OsmRoute>, OsmError> {
        let query = format!(
            r#"[out:json][timeout:60];
            (
              way["highway"="cycleway"]({},{},{},{});
              way["bicycle"="designated"]({},{},{},{});
            );
            out body;
            >;
            out skel qt;"#,
            bounds.min_lat,
            bounds.min_lon,
            bounds.max_lat,
            bounds.max_lon,
            bounds.min_lat,
            bounds.min_lon,
            bounds.max_lat,
            bounds.max_lon,
        );

        self.execute_query(&query).await
    }

    /// Fetches all paths (trails + cycling) within a bounding box.
    pub async fn fetch_all_paths(&self, bounds: BoundingBox) -> Result<Vec<OsmRoute>, OsmError> {
        let query = format!(
            r#"[out:json][timeout:60];
            (
              way["highway"~"path|footway|track|cycleway"]({},{},{},{});
            );
            out body;
            >;
            out skel qt;"#,
            bounds.min_lat, bounds.min_lon, bounds.max_lat, bounds.max_lon,
        );

        self.execute_query(&query).await
    }

    /// Executes an Overpass query and parses the results.
    async fn execute_query(&self, query: &str) -> Result<Vec<OsmRoute>, OsmError> {
        // Check cache first
        if let Some(cached) = self.check_cache(query)? {
            return Ok(cached);
        }

        // Make API request
        let response = self
            .client
            .post(&self.endpoint)
            .body(query.to_string())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(OsmError::RateLimited);
        }

        let text = response.text().await?;
        let parsed: OverpassResponse = serde_json::from_str(&text)?;

        let routes = self.parse_response(parsed)?;

        // Save to cache
        self.save_cache(query, &routes)?;

        Ok(routes)
    }

    /// Parses Overpass response into routes.
    fn parse_response(&self, response: OverpassResponse) -> Result<Vec<OsmRoute>, OsmError> {
        // Build node lookup
        let mut nodes: HashMap<i64, (f64, f64)> = HashMap::new();
        let mut ways: Vec<(i64, Vec<i64>, HashMap<String, String>)> = Vec::new();

        for element in response.elements {
            match element {
                OverpassElement::Node { id, lat, lon, .. } => {
                    nodes.insert(id, (lat, lon));
                }
                OverpassElement::Way {
                    id,
                    nodes: node_ids,
                    tags,
                } => {
                    ways.push((id, node_ids, tags));
                }
            }
        }

        if ways.is_empty() {
            return Err(OsmError::NoWays);
        }

        // Build routes
        let routes: Vec<OsmRoute> = ways
            .into_iter()
            .filter_map(|(id, node_ids, tags)| {
                let coords: Vec<(f64, f64)> = node_ids
                    .iter()
                    .filter_map(|nid| nodes.get(nid).copied())
                    .collect();

                if coords.len() < 2 {
                    return None;
                }

                Some(OsmRoute {
                    id,
                    name: tags.get("name").cloned(),
                    highway_type: tags.get("highway").cloned(),
                    coords,
                })
            })
            .collect();

        Ok(routes)
    }

    /// Checks cache for a previous response.
    fn check_cache(&self, query: &str) -> Result<Option<Vec<OsmRoute>>, OsmError> {
        let Some(ref cache_dir) = self.cache_dir else {
            return Ok(None);
        };

        let hash = Self::hash_query(query);
        let cache_path = cache_dir.join(format!("{hash}.json"));

        if cache_path.exists() {
            let data = std::fs::read_to_string(&cache_path)?;
            let routes: Vec<OsmRoute> = serde_json::from_str(&data)?;
            tracing::debug!("Cache hit for query hash {hash}");
            return Ok(Some(routes));
        }

        Ok(None)
    }

    /// Saves response to cache.
    fn save_cache(&self, query: &str, routes: &[OsmRoute]) -> Result<(), OsmError> {
        let Some(ref cache_dir) = self.cache_dir else {
            return Ok(());
        };

        let hash = Self::hash_query(query);
        let cache_path = cache_dir.join(format!("{hash}.json"));
        let data = serde_json::to_string_pretty(routes)?;
        std::fs::write(cache_path, data)?;
        tracing::debug!("Cached response for query hash {hash}");

        Ok(())
    }

    /// Simple hash of query string for cache key.
    fn hash_query(query: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

impl Default for OsmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let query = "some query";
        let hash1 = OsmClient::hash_query(query);
        let hash2 = OsmClient::hash_query(query);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different() {
        let hash1 = OsmClient::hash_query("query1");
        let hash2 = OsmClient::hash_query("query2");
        assert_ne!(hash1, hash2);
    }
}
