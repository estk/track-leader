//! Fluent builder APIs for test scenarios.
//!
//! The [`ScenarioBuilder`] provides a convenient way to construct
//! complete test scenarios with users, tracks, segments, and efforts.

mod scenario;

pub use scenario::{MultiSportSegment, ScenarioBuilder, ScenarioMetrics, ScenarioResult};
