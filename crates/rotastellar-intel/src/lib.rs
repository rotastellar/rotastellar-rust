//! # RotaStellar Intel
//!
//! Orbital intelligence and space situational awareness.
//!
//! subhadipmitra@: This crate provides the SSA (Space Situational Awareness) capabilities.
//! It's designed for both real-time tracking and historical analysis. The TLE parser
//! is fully compatible with Space-Track format, and SGP4 propagation matches AFSPC results.
//!
//! This crate provides tools for tracking satellites, analyzing conjunctions,
//! and detecting behavioral patterns.
//!
//! ## Features
//!
//! - **TLE Parsing**: Parse Two-Line Element sets and propagate orbits
//! - **Satellite Tracking**: Track satellite positions over time
//! - **Conjunction Analysis**: Analyze collision probabilities (Pc framework)
//! - **Pattern Detection**: Detect maneuvers and anomalies from TLE history
//!
//! ## Example
//!
//! ```
//! use rotastellar_intel::{TLE, Tracker, PatternType};
//!
//! // Parse a TLE
//! let tle_lines = vec![
//!     "ISS (ZARYA)".to_string(),
//!     "1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999".to_string(),
//!     "2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818".to_string(),
//! ];
//! let tle = TLE::parse(&tle_lines).unwrap();
//! println!("ISS inclination: {}°", tle.inclination);
//!
//! // Add to tracker
//! let mut tracker = Tracker::new();
//! tracker.add_tle("ISS", tle);
//!
//! // Get position at epoch
//! let pos = tracker.get_position("ISS", None).unwrap();
//! println!("ISS at {:.2}, {:.2}", pos.latitude, pos.longitude);
//! ```
//!
//! ## Links
//!
//! - [Website](https://rotastellar.com)
//! - [Documentation](https://rotastellar.com/docs/intel)
//! - [GitHub](https://github.com/rotastellar/rotastellar-rust)

#![warn(missing_docs)]

pub mod conjunctions;
pub mod patterns;
pub mod tle;
pub mod tracker;

// Re-export commonly used items
pub use conjunctions::{
    Conjunction, ConjunctionAnalyzer, ManeuverRecommendation, RiskAnalysis, RiskLevel,
};
pub use patterns::{
    BehaviorAnalysis, ConfidenceLevel, DetectedPattern, PatternDetector, PatternType,
};
pub use tle::{parse_tle, TLE};
pub use tracker::{GroundStation, SatellitePass, TrackedSatelliteInfo, Tracker};

/// Current version of the crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_tle_parse() {
        let lines = vec![
            "ISS (ZARYA)".to_string(),
            "1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999".to_string(),
            "2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818".to_string(),
        ];
        let tle = TLE::parse(&lines).unwrap();
        assert_eq!(tle.norad_id, 25544);
    }

    #[test]
    fn test_tracker() {
        let lines = vec![
            "ISS (ZARYA)".to_string(),
            "1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999".to_string(),
            "2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818".to_string(),
        ];
        let tle = TLE::parse(&lines).unwrap();

        let mut tracker = Tracker::new();
        tracker.add_tle("ISS", tle);

        assert_eq!(tracker.list_satellites().len(), 1);
    }

    #[test]
    fn test_pattern_types() {
        assert!(PatternType::OrbitRaise.is_maneuver());
        assert!(!PatternType::Tumbling.is_maneuver());
        assert!(PatternType::Tumbling.is_anomaly());
    }

    #[test]
    fn test_risk_levels() {
        let critical = RiskLevel::Critical;
        let low = RiskLevel::Low;
        assert!(matches!(critical, RiskLevel::Critical));
        assert!(matches!(low, RiskLevel::Low));
    }
}
