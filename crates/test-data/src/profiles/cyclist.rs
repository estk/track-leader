//! Cyclist athletic profile.

use super::AthleteProfile;

/// Athletic profile for cycling activities.
///
/// Based on typical recreational to competitive cyclist performance:
/// - Base speed: ~28 km/h (8.0 m/s) on flat terrain
/// - Uphill: ~25% slower per 1% grade (significant impact)
/// - Downhill: ~15% faster per 1% grade (drafting effect, momentum)
#[derive(Debug, Clone)]
pub struct CyclistProfile {
    /// Base speed in m/s on flat terrain.
    base_speed: f64,
    /// Performance variance (coefficient of variation).
    variance: f64,
}

impl Default for CyclistProfile {
    fn default() -> Self {
        Self {
            base_speed: 8.0, // ~28 km/h
            variance: 0.10,
        }
    }
}

impl CyclistProfile {
    /// Creates a new cyclist profile with specified base speed.
    ///
    /// # Arguments
    /// * `speed_kmh` - Base speed in km/h on flat terrain
    pub fn with_speed(speed_kmh: f64) -> Self {
        let base_speed = speed_kmh / 3.6;
        Self {
            base_speed,
            ..Default::default()
        }
    }

    /// Creates an elite cyclist profile (~35 km/h base).
    pub fn elite() -> Self {
        Self::with_speed(35.0)
    }

    /// Creates a recreational cyclist profile (~22 km/h base).
    pub fn recreational() -> Self {
        Self::with_speed(22.0)
    }

    /// Creates a mountain biker profile (~18 km/h base, more variance).
    pub fn mountain_biker() -> Self {
        Self {
            base_speed: 5.0, // ~18 km/h
            variance: 0.15,  // More technical terrain = more variance
        }
    }
}

impl AthleteProfile for CyclistProfile {
    fn base_speed_mps(&self) -> f64 {
        self.base_speed
    }

    fn grade_factor(&self, grade: f64) -> f64 {
        // Cycling is heavily affected by grade due to gearing and momentum
        // Uphill: lose ~25% per 1% grade
        // Downhill: gain ~15% per 1% grade
        if grade >= 0.0 {
            // Uphill - cycling is heavily penalized
            let factor = 1.0 - (grade * 25.0);
            factor.max(0.15) // Minimum 15% on very steep climbs (walking speed)
        } else {
            // Downhill - momentum helps significantly
            let factor = 1.0 - (grade * 15.0); // grade is negative
            factor.min(2.5) // Cap at 250% for safety (descending limits)
        }
    }

    fn variance(&self) -> f64 {
        self.variance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = CyclistProfile::default();
        assert!((profile.base_speed_mps() - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_steep_climb() {
        let profile = CyclistProfile::default();
        // 10% grade should really slow things down
        let factor = profile.grade_factor(0.10);
        assert!(factor < 0.5);
    }

    #[test]
    fn test_downhill_boost() {
        let profile = CyclistProfile::default();
        // 5% descent should be significantly faster
        let factor = profile.grade_factor(-0.05);
        assert!(factor > 1.5);
    }
}
