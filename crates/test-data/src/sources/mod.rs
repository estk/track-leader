//! Data acquisition sources for track generation.
//!
//! This module provides multiple ways to obtain track geometry:
//! - [`OsmClient`]: Fetch real-world routes from OpenStreetMap via Overpass API
//! - [`ProceduralGenerator`]: Generate synthetic tracks with configurable parameters
//! - [`GpxLoader`]: Load existing GPX files

mod gpx_files;
mod osm;
mod procedural;

pub use gpx_files::GpxLoader;
pub use osm::OsmClient;
pub use procedural::{ProceduralGenerator, RoutePattern};
