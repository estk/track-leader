//! Perlin noise-based elevation generation.

use noise::{NoiseFn, Perlin};
use rand::Rng;

/// Generates realistic elevation data using Perlin noise.
///
/// The generator uses multiple octaves of Perlin noise to create
/// natural-looking terrain with both large-scale features and
/// small-scale variation.
#[derive(Debug, Clone)]
pub struct ElevationGenerator {
    perlin: Perlin,
    /// Base elevation in meters (e.g., valley floor).
    base_elevation: f64,
    /// Scale factor for terrain height variation.
    height_scale: f64,
    /// Spatial frequency (controls terrain "wavelength").
    frequency: f64,
    /// Number of noise octaves for detail.
    octaves: u32,
}

impl ElevationGenerator {
    /// Creates a new elevation generator with default settings.
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            base_elevation: 1500.0, // Reasonable base for mountain terrain
            height_scale: 500.0,    // ±500m variation
            frequency: 0.0001,      // Large-scale features
            octaves: 4,
        }
    }

    /// Creates a generator configured for the Reno/Tahoe region.
    ///
    /// Higher base elevation and larger height scale for Sierra Nevada terrain.
    pub fn reno_tahoe(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            base_elevation: 1900.0, // Lake Tahoe elevation ~1900m
            height_scale: 800.0,    // Significant mountain terrain
            frequency: 0.00008,
            octaves: 5,
        }
    }

    /// Creates a generator configured for the Boulder, CO region.
    pub fn boulder(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            base_elevation: 1650.0, // Boulder elevation ~1650m
            height_scale: 600.0,    // Foothills terrain
            frequency: 0.0001,
            octaves: 4,
        }
    }

    /// Creates a generator for relatively flat terrain (rolling hills).
    pub fn flat(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            base_elevation: 300.0,
            height_scale: 50.0, // Minimal variation
            frequency: 0.0002,
            octaves: 2,
        }
    }

    /// Sets the base elevation.
    pub fn with_base_elevation(mut self, elevation: f64) -> Self {
        self.base_elevation = elevation;
        self
    }

    /// Sets the height scale (variation amplitude).
    pub fn with_height_scale(mut self, scale: f64) -> Self {
        self.height_scale = scale;
        self
    }

    /// Sets the spatial frequency.
    pub fn with_frequency(mut self, freq: f64) -> Self {
        self.frequency = freq;
        self
    }

    /// Gets elevation at a given lat/lon coordinate.
    ///
    /// Uses fractal Brownian motion (fBm) for natural terrain appearance.
    pub fn elevation_at(&self, lat: f64, lon: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency;
        let mut max_amplitude = 0.0;

        for _ in 0..self.octaves {
            let noise_val = self.perlin.get([lat * frequency, lon * frequency]);
            total += noise_val * amplitude;
            max_amplitude += amplitude;
            amplitude *= 0.5; // Each octave has half the amplitude
            frequency *= 2.0; // Each octave has double the frequency
        }

        // Normalize and scale
        let normalized = total / max_amplitude; // Range: -1 to 1
        self.base_elevation + (normalized * self.height_scale)
    }

    /// Generates elevation profile along a path defined by lat/lon points.
    ///
    /// Returns elevations for each input coordinate.
    pub fn elevation_profile(&self, coords: &[(f64, f64)]) -> Vec<f64> {
        coords
            .iter()
            .map(|(lat, lon)| self.elevation_at(*lat, *lon))
            .collect()
    }

    /// Generates elevation profile with interpolation for smoother results.
    ///
    /// This adds interpolated points between input coordinates to create
    /// a smoother elevation profile that better represents real terrain.
    pub fn smooth_elevation_profile(
        &self,
        coords: &[(f64, f64)],
        points_between: usize,
    ) -> Vec<f64> {
        if coords.len() < 2 {
            return coords
                .iter()
                .map(|(lat, lon)| self.elevation_at(*lat, *lon))
                .collect();
        }

        let mut result = Vec::with_capacity(coords.len() + (coords.len() - 1) * points_between);

        for window in coords.windows(2) {
            let (lat1, lon1) = window[0];
            let (lat2, lon2) = window[1];

            // Add start point
            result.push(self.elevation_at(lat1, lon1));

            // Add interpolated points
            for i in 1..=points_between {
                let t = i as f64 / (points_between + 1) as f64;
                let lat = lat1 + (lat2 - lat1) * t;
                let lon = lon1 + (lon2 - lon1) * t;
                result.push(self.elevation_at(lat, lon));
            }
        }

        // Add final point
        if let Some((lat, lon)) = coords.last() {
            result.push(self.elevation_at(*lat, *lon));
        }

        result
    }
}

/// Utility to add random GPS jitter to elevation readings.
///
/// Real GPS devices have elevation accuracy of ±3-20m depending on conditions.
pub fn add_elevation_jitter(elevation: f64, rng: &mut impl Rng, std_dev: f64) -> f64 {
    use rand_distr::{Distribution, Normal};
    let normal = Normal::new(0.0, std_dev).unwrap();
    elevation + normal.sample(rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_consistency() {
        let elev_gen = ElevationGenerator::new(42);
        let elev1 = elev_gen.elevation_at(39.5, -119.8);
        let elev2 = elev_gen.elevation_at(39.5, -119.8);
        assert!((elev1 - elev2).abs() < 0.001);
    }

    #[test]
    fn test_elevation_range() {
        let elev_gen = ElevationGenerator::new(42);
        let elev = elev_gen.elevation_at(39.5, -119.8);
        // Should be within base ± scale
        assert!(elev > elev_gen.base_elevation - elev_gen.height_scale);
        assert!(elev < elev_gen.base_elevation + elev_gen.height_scale);
    }

    #[test]
    fn test_profile_generation() {
        let elev_gen = ElevationGenerator::boulder(42);
        let coords = vec![(40.0, -105.3), (40.01, -105.29), (40.02, -105.28)];
        let profile = elev_gen.elevation_profile(&coords);
        assert_eq!(profile.len(), 3);
    }
}
