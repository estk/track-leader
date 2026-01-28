//! Athletic performance profiles.
//!
//! Profiles define realistic speeds and grade factors for different activity types.
//! They are used by track generators to produce realistic timestamps.

mod cyclist;
mod hiker;
mod runner;

pub use cyclist::CyclistProfile;
pub use hiker::HikerProfile;
pub use runner::RunnerProfile;

/// Trait for athletic performance profiles.
///
/// Profiles determine realistic speeds based on activity type and terrain.
/// Implementations should provide:
/// - Base speed on flat terrain
/// - Grade factor (speed multiplier based on slope)
/// - Day-to-day variance
pub trait AthleteProfile: Send + Sync {
    /// Base speed on flat terrain in meters per second.
    fn base_speed_mps(&self) -> f64;

    /// Speed multiplier for a given grade (expressed as a fraction, e.g., 0.05 = 5% grade).
    ///
    /// Returns a value between 0 and 2+:
    /// - < 1.0 means slower than base (uphill)
    /// - > 1.0 means faster than base (downhill)
    fn grade_factor(&self, grade: f64) -> f64;

    /// Day-to-day performance variance as a coefficient of variation (0.0 - 1.0).
    ///
    /// A value of 0.1 means typical day-to-day variation of ±10%.
    fn variance(&self) -> f64;
}

/// Extension functions for AthleteProfile that can be used with concrete types.
/// These are not dyn-compatible but provide convenient functionality.
pub fn speed_at_grade(profile: &dyn AthleteProfile, grade: f64, variance_factor: f64) -> f64 {
    let base = profile.base_speed_mps();
    let factor = profile.grade_factor(grade);
    let target = base * factor;

    // Apply variance factor (should be sampled by caller)
    (target * variance_factor).max(0.5) // Minimum 0.5 m/s to avoid division issues
}

/// Samples a variance factor from normal distribution.
/// Returns a multiplier around 1.0.
pub fn sample_variance(profile: &dyn AthleteProfile, rng: &mut impl rand::Rng) -> f64 {
    use rand_distr::{Distribution, Normal};

    let std_dev = profile.variance();
    if std_dev > 0.0 {
        let normal = Normal::new(1.0, std_dev).unwrap();
        let sample: f64 = normal.sample(rng);
        sample.clamp(0.7, 1.4)
    } else {
        1.0
    }
}
