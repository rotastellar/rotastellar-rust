//! # RotaStellar
//!
//! Rust SDK for RotaStellar - Space Computing Infrastructure.
//!
//! Plan, simulate, and operate orbital data centers and space intelligence systems.
//!
//! subhadipmitra@: This is the core SDK crate that all other rotastellar-* crates depend on.
//! It provides common types (Position, Orbit), error handling, and authentication.
//! We prioritized simplicity and zero-copy where possible for performance.
//!
//! ## Overview
//!
//! RotaStellar provides tools for:
//! - **Orbital Compute Suite** — Plan and simulate space-based data centers
//! - **Orbital Intelligence** — Track, analyze, and monitor orbital activity
//!
//! ## Example
//!
//! ```rust
//! use rotastellar::types::{Position, Orbit};
//!
//! // Create a position (e.g., Kennedy Space Center)
//! let pos = Position::new(28.5729, -80.6490, 0.0).unwrap();
//! println!("Position: {}, {}", pos.latitude, pos.longitude);
//!
//! // Create an ISS-like orbit
//! let orbit = Orbit::new(6778.0, 0.0001, 51.6, 100.0, 90.0, 0.0).unwrap();
//! println!("Orbital period: {:.1} minutes", orbit.orbital_period_minutes());
//! println!("Apogee: {:.1} km, Perigee: {:.1} km", orbit.apogee_km(), orbit.perigee_km());
//! ```
//!
//! ## Modules
//!
//! - [`types`] - Core data types (Position, Orbit, Satellite, TimeRange)
//! - [`error`] - Error types and Result alias
//! - [`auth`] - Authentication utilities
//! - [`config`] - SDK configuration
//!
//! ## Links
//!
//! - [Website](https://rotastellar.com)
//! - [Documentation](https://rotastellar.com/docs)
//! - [GitHub](https://github.com/rotastellar/rotastellar-rust)

#![doc(html_root_url = "https://docs.rs/rotastellar/0.1.0")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod auth;
pub mod config;
pub mod error;
pub mod types;

// Re-export commonly used items at crate root
pub use auth::{mask_api_key, validate_api_key, Environment};
pub use config::{Config, ConfigBuilder};
pub use error::{
    ApiError, AuthenticationError, NetworkError, Result, RotaStellarError, ValidationError,
};
pub use types::{Orbit, Position, Satellite, TimeRange, EARTH_MU, EARTH_RADIUS_KM};

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
    fn test_position() {
        let pos = Position::new(28.5729, -80.6490, 408.0).unwrap();
        assert_eq!(pos.latitude, 28.5729);
        assert_eq!(pos.altitude_km, 408.0);
    }

    #[test]
    fn test_orbit() {
        let orbit = Orbit::new(6778.0, 0.0001, 51.6, 100.0, 90.0, 0.0).unwrap();
        let period = orbit.orbital_period_minutes();
        assert!((period - 92.56).abs() < 0.1, "Period was {}", period);
    }

    #[test]
    fn test_validation_error() {
        let result = Position::new(91.0, 0.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_api_key_validation() {
        let result = validate_api_key(None);
        assert!(result.is_err());
    }
}
