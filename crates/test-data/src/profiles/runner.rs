//! Runner athletic profile.

use super::AthleteProfile;

/// Athletic profile for running activities.
///
/// Based on typical recreational to competitive runner performance:
/// - Base pace: ~5:00/km (3.5 m/s)
/// - Uphill: ~15% slower per 1% grade
/// - Downhill: ~8% faster per 1% grade (limited by safety)
#[derive(Debug, Clone)]
pub struct RunnerProfile {
    /// Base speed in m/s on flat terrain.
    base_speed: f64,
    /// Performance variance (coefficient of variation).
    variance: f64,
}

impl Default for RunnerProfile {
    fn default() -> Self {
        Self {
            base_speed: 3.5, // ~5:00/km
            variance: 0.08,
        }
    }
}

impl RunnerProfile {
    /// Creates a new runner profile with specified base pace.
    ///
    /// # Arguments
    /// * `pace_min_per_km` - Base pace in minutes per kilometer (e.g., 5.0 for 5:00/km)
    pub fn with_pace(pace_min_per_km: f64) -> Self {
        let base_speed = 1000.0 / (pace_min_per_km * 60.0);
        Self {
            base_speed,
            ..Default::default()
        }
    }

    /// Creates an elite runner profile (~3:30/km base pace).
    pub fn elite() -> Self {
        Self::with_pace(3.5)
    }

    /// Creates a recreational runner profile (~6:00/km base pace).
    pub fn recreational() -> Self {
        Self::with_pace(6.0)
    }
}

impl AthleteProfile for RunnerProfile {
    fn base_speed_mps(&self) -> f64 {
        self.base_speed
    }

    fn grade_factor(&self, grade: f64) -> f64 {
        // Empirical grade adjustment for running
        // Uphill: lose ~15% per 1% grade
        // Downhill: gain ~8% per 1% grade (capped for safety)
        if grade >= 0.0 {
            // Uphill
            let factor = 1.0 - (grade * 15.0);
            factor.max(0.2) // Minimum 20% of base speed on steep climbs
        } else {
            // Downhill
            let factor = 1.0 - (grade * 8.0); // Note: grade is negative, so this adds
            factor.min(1.5) // Cap at 150% of base speed for safety
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
        let profile = RunnerProfile::default();
        assert!((profile.base_speed_mps() - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_grade_factors() {
        let profile = RunnerProfile::default();

        // Flat
        assert!((profile.grade_factor(0.0) - 1.0).abs() < 0.01);

        // 5% uphill should be slower
        assert!(profile.grade_factor(0.05) < 1.0);

        // 5% downhill should be faster
        assert!(profile.grade_factor(-0.05) > 1.0);
    }
}
