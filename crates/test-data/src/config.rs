//! Configuration types for test data generation.

use serde::{Deserialize, Serialize};

/// Geographic bounding box defined by southwest and northeast corners.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum latitude (south)
    pub min_lat: f64,
    /// Minimum longitude (west)
    pub min_lon: f64,
    /// Maximum latitude (north)
    pub max_lat: f64,
    /// Maximum longitude (east)
    pub max_lon: f64,
}

impl BoundingBox {
    pub const fn new(min_lat: f64, min_lon: f64, max_lat: f64, max_lon: f64) -> Self {
        Self {
            min_lat,
            min_lon,
            max_lat,
            max_lon,
        }
    }

    /// Returns a random point within the bounding box.
    pub fn random_point(&self, rng: &mut impl rand::Rng) -> (f64, f64) {
        let lat = rng.gen_range(self.min_lat..self.max_lat);
        let lon = rng.gen_range(self.min_lon..self.max_lon);
        (lat, lon)
    }

    /// Returns the center of the bounding box.
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lon + self.max_lon) / 2.0,
        )
    }
}

/// Pre-defined geographic regions for test data generation.
#[derive(Debug, Clone, Copy)]
pub struct Region;

impl Region {
    /// Reno/Tahoe area - mountain trails with significant elevation changes.
    pub const RENO_TAHOE: BoundingBox = BoundingBox::new(39.0, -120.5, 39.6, -119.5);

    /// Boulder, CO area - popular fitness trails with varied terrain.
    pub const BOULDER: BoundingBox = BoundingBox::new(39.9, -105.5, 40.1, -105.2);
}

/// Configuration for seeding operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedConfig {
    /// Number of users to generate.
    pub user_count: usize,

    /// Number of activities per user (range).
    pub activities_per_user: (usize, usize),

    /// Target region for track generation.
    pub region: BoundingBox,

    /// Whether to generate social relationships (follows, kudos, comments).
    pub generate_social: bool,

    /// Batch size for database insertions.
    pub batch_size: usize,
}

impl Default for SeedConfig {
    fn default() -> Self {
        Self {
            user_count: 100,
            activities_per_user: (1, 5),
            region: Region::BOULDER,
            generate_social: true,
            batch_size: 50,
        }
    }
}

/// Distribution for skill levels (used for effort time generation).
#[derive(Debug, Clone, Copy)]
pub enum SkillDistribution {
    /// Uniform distribution - equal probability across skill range.
    Uniform,
    /// Normal distribution with specified mean and std deviation.
    Normal { mean: f64, std_dev: f64 },
    /// Power-law distribution - few elite, many average.
    PowerLaw { alpha: f64 },
}

impl Default for SkillDistribution {
    fn default() -> Self {
        Self::PowerLaw { alpha: 2.0 }
    }
}

impl SkillDistribution {
    /// Creates a power-law distribution with default parameters.
    /// This produces realistic athletic performance: few elite performers, many average.
    pub fn power_law() -> Self {
        Self::PowerLaw { alpha: 2.0 }
    }
}

/// Controls how efforts are distributed across users and segments.
#[derive(Debug, Clone, Copy, Default)]
pub enum EffortCoverage {
    /// Every user gets efforts on every segment (original behavior).
    #[default]
    Full,
    /// Random fraction of users get efforts on each segment.
    Sparse {
        /// Probability (0.0-1.0) that a user has an effort on a segment.
        fraction: f64,
    },
    /// Power-law distribution: popular segments get more efforts.
    /// Simulates real-world where some segments are heavily trafficked.
    Zipf {
        /// Zipf exponent (higher = more skewed toward popular segments).
        alpha: f64,
    },
}

impl EffortCoverage {
    /// Creates a sparse distribution with 70% coverage (default for stress tests).
    pub fn sparse() -> Self {
        Self::Sparse { fraction: 0.7 }
    }

    /// Creates a Zipf distribution with default parameters.
    pub fn zipf() -> Self {
        Self::Zipf { alpha: 1.5 }
    }
}
