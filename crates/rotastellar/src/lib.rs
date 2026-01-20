//! # RotaStellar
//!
//! Rust SDK for RotaStellar - Space Computing Infrastructure.
//!
//! Plan, simulate, and operate orbital data centers and space intelligence systems.
//!
//! **🚀 Launching Q1 2026**
//!
//! ## Overview
//!
//! RotaStellar provides tools for:
//! - **Orbital Compute Suite** — Plan and simulate space-based data centers
//! - **Orbital Intelligence** — Track, analyze, and monitor orbital activity
//!
//! ## Example (Coming Soon)
//!
//! ```rust,ignore
//! use rotastellar::OrbitalIntel;
//!
//! let client = OrbitalIntel::new("api_key")?;
//! let iss = client.satellite("ISS").await?;
//! let pos = iss.position().await?;
//! println!("ISS: {}, {}", pos.lat, pos.lon);
//! ```
//!
//! ## Links
//!
//! - [Website](https://rotastellar.com)
//! - [Documentation](https://rotastellar.com/docs)
//! - [GitHub](https://github.com/rotastellar/rotastellar-rust)

#![doc(html_root_url = "https://docs.rs/rotastellar/0.0.1")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Current version of the crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Orbital compute planning and simulation.
///
/// **Coming Q1 2026**
pub mod compute {
    /// Feasibility analysis for orbital compute.
    pub fn feasibility() {
        unimplemented!("rotastellar::compute is launching Q1 2026. Visit https://rotastellar.com")
    }
}

/// Orbital intelligence and tracking.
///
/// **Coming Q1 2026**
pub mod intel {
    /// Satellite tracking and analysis.
    pub fn track() {
        unimplemented!("rotastellar::intel is launching Q1 2026. Visit https://rotastellar.com")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert_eq!(VERSION, "0.0.1");
    }
}
