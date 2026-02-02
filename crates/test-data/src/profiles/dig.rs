//! Dig (trail work) profile for stationary sections.

use super::AthleteProfile;

/// Athletic profile for dig (trail work) sections.
///
/// Represents essentially stationary activity with GPS drift simulation.
/// Used for multi-sport activities where athletes stop to do trail maintenance.
///
/// Characteristics:
/// - Very low base speed (~0.2 m/s, essentially GPS drift)
/// - No meaningful grade effect (stationary work)
/// - High variance to simulate GPS wander
#[derive(Debug, Clone)]
pub struct DigProfile {
    /// Base speed in m/s (represents GPS drift, not actual movement).
    base_speed: f64,
    /// Performance variance (high to simulate GPS wander).
    variance: f64,
}

impl Default for DigProfile {
    fn default() -> Self {
        Self {
            base_speed: 0.2, // Essentially stationary with GPS drift
            variance: 0.5,   // High variance for realistic GPS wander
        }
    }
}

impl AthleteProfile for DigProfile {
    fn base_speed_mps(&self) -> f64 {
        self.base_speed
    }

    fn grade_factor(&self, _grade: f64) -> f64 {
        // Stationary work - grade doesn't affect "speed"
        1.0
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
        let profile = DigProfile::default();
        assert!(
            profile.base_speed_mps() < 0.5,
            "Dig should be essentially stationary"
        );
        assert!(
            profile.variance() > 0.3,
            "Dig should have high variance for GPS drift"
        );
    }

    #[test]
    fn test_grade_has_no_effect() {
        let profile = DigProfile::default();
        // Grade shouldn't affect stationary work
        assert!((profile.grade_factor(0.0) - 1.0).abs() < 0.01);
        assert!((profile.grade_factor(0.10) - 1.0).abs() < 0.01);
        assert!((profile.grade_factor(-0.10) - 1.0).abs() < 0.01);
    }
}
