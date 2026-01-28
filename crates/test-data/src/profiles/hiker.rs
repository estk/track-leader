//! Hiker athletic profile.

use super::AthleteProfile;

/// Athletic profile for hiking activities.
///
/// Based on typical recreational hiker performance:
/// - Base speed: ~5.5 km/h (1.5 m/s) on flat terrain
/// - Uphill: ~12% slower per 1% grade
/// - Downhill: ~5% faster per 1% grade (conservative due to terrain)
#[derive(Debug, Clone)]
pub struct HikerProfile {
    /// Base speed in m/s on flat terrain.
    base_speed: f64,
    /// Performance variance (coefficient of variation).
    variance: f64,
}

impl Default for HikerProfile {
    fn default() -> Self {
        Self {
            base_speed: 1.5, // ~5.5 km/h
            variance: 0.12,
        }
    }
}

impl HikerProfile {
    /// Creates a new hiker profile with specified base speed.
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

    /// Creates a fast hiker profile (~6.5 km/h base).
    pub fn fast() -> Self {
        Self::with_speed(6.5)
    }

    /// Creates a leisurely hiker profile (~4.0 km/h base).
    pub fn leisurely() -> Self {
        Self::with_speed(4.0)
    }

    /// Creates a backpacker profile (slower due to pack weight).
    pub fn backpacker() -> Self {
        Self {
            base_speed: 1.2,  // ~4.3 km/h
            variance: 0.15,  // More fatigue variance
        }
    }
}

impl AthleteProfile for HikerProfile {
    fn base_speed_mps(&self) -> f64 {
        self.base_speed
    }

    fn grade_factor(&self, grade: f64) -> f64 {
        // Hiking is less affected than running due to lower speeds
        // and ability to adjust pace more easily
        // Uphill: lose ~12% per 1% grade
        // Downhill: gain ~5% per 1% grade (conservative - technical terrain)
        if grade >= 0.0 {
            // Uphill
            let factor = 1.0 - (grade * 12.0);
            factor.max(0.25) // Minimum 25% on steep terrain
        } else {
            // Downhill - be conservative
            let factor = 1.0 - (grade * 5.0); // grade is negative
            factor.min(1.3) // Cap at 130% - careful on descents
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
        let profile = HikerProfile::default();
        assert!((profile.base_speed_mps() - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_moderate_climb() {
        let profile = HikerProfile::default();
        // 5% grade should slow somewhat but not dramatically
        let factor = profile.grade_factor(0.05);
        assert!(factor > 0.3 && factor < 0.8);
    }

    #[test]
    fn test_conservative_descent() {
        let profile = HikerProfile::default();
        // Downhill should be faster but not too much
        let factor = profile.grade_factor(-0.05);
        assert!(factor > 1.0 && factor < 1.3);
    }
}
