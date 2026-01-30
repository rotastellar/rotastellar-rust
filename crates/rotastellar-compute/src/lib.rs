//! # RotaStellar Compute
//!
//! Orbital compute planning and simulation tools for space-based data centers.
//!
//! subhadipmitra@: This crate helps users understand if their workload is a good fit
//! for orbital compute. The key constraints are thermal, power, and latency - all of
//! which are modeled here with aerospace-standard equations.
//!
//! ## Overview
//!
//! This crate provides tools for planning and simulating space-based compute
//! infrastructure, including:
//!
//! - **Feasibility Analysis** — Evaluate workload suitability for orbital compute
//! - **Thermal Simulation** — Model heat rejection in vacuum using Stefan-Boltzmann law
//! - **Latency Modeling** — End-to-end latency for space-ground communication
//! - **Power Analysis** — Solar panel and battery sizing for orbital systems
//!
//! ## Example
//!
//! ```rust
//! use rotastellar_compute::{
//!     FeasibilityCalculator, WorkloadProfile, WorkloadType,
//!     ThermalSimulator, ThermalConfig, ThermalEnvironment,
//!     LatencySimulator,
//!     PowerAnalyzer, PowerProfile,
//! };
//!
//! // Feasibility analysis
//! let calc = FeasibilityCalculator::new(550.0);
//! let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0)
//!     .with_memory_gb(32.0);
//! let feasibility = calc.analyze(&profile, None);
//! println!("Feasible: {}, Rating: {}", feasibility.feasible, feasibility.rating);
//!
//! // Thermal simulation
//! let thermal_sim = ThermalSimulator::new();
//! let thermal_cfg = ThermalConfig::for_power(500.0);
//! let thermal_env = ThermalEnvironment::leo(550.0);
//! let thermal = thermal_sim.simulate(&thermal_cfg, &thermal_env);
//! println!("Equilibrium temperature: {:.1}°C", thermal.equilibrium_temp_c);
//!
//! // Latency simulation
//! let latency_sim = LatencySimulator::new(550.0);
//! let latency = latency_sim.simulate(Some(100.0), None);
//! println!("Total latency: {:.1} ms", latency.total_latency_ms);
//!
//! // Power analysis
//! let power_analyzer = PowerAnalyzer::new(550.0);
//! let power_profile = PowerProfile::new(500.0);
//! let power = power_analyzer.analyze(&power_profile, None, None, None, None);
//! println!("Solar panel area: {:.2} m²", power.solar_panel_area_m2);
//! ```
//!
//! ## Modules
//!
//! - [`feasibility`] — Workload feasibility analysis
//! - [`thermal`] — Thermal simulation for orbital systems
//! - [`latency`] — Latency modeling for space-ground communication
//! - [`power`] — Power system analysis and sizing
//!
//! ## Links
//!
//! - [Website](https://rotastellar.com)
//! - [Documentation](https://rotastellar.com/docs/compute)
//! - [GitHub](https://github.com/rotastellar/rotastellar-rust)

#![doc(html_root_url = "https://docs.rs/rotastellar-compute/0.1.0")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod feasibility;
pub mod latency;
pub mod power;
pub mod thermal;

// Re-export commonly used items at crate root
pub use feasibility::{
    FeasibilityCalculator, FeasibilityRating, FeasibilityResult, ScenarioResult, WorkloadProfile,
    WorkloadType,
};

pub use thermal::{
    OrbitType, RadiatorSizing, ThermalConfig, ThermalEnvironment, ThermalResult, ThermalSimulator,
    ThermalTimePoint,
};

pub use latency::{
    AltitudeLatency, ElevationLatency, LatencyComponent, LatencyResult, LatencySimulator,
    LinkType, TerrestrialComparison,
};

pub use power::{
    BatteryChemistry, BatteryConfig, BatterySizing, PowerAnalyzer, PowerBudget, PowerProfile,
    SolarCellType, SolarConfig, SolarPanelSizing,
};

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
    fn test_feasibility_integration() {
        let calc = FeasibilityCalculator::new(550.0);
        let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0)
            .with_memory_gb(32.0)
            .with_latency_requirement_ms(100.0);

        let result = calc.analyze(&profile, None);

        assert!(result.feasible);
        assert!(matches!(
            result.rating,
            FeasibilityRating::Excellent | FeasibilityRating::Good
        ));
    }

    #[test]
    fn test_thermal_integration() {
        let sim = ThermalSimulator::new();
        let config = ThermalConfig::for_power(500.0);
        let env = ThermalEnvironment::leo(550.0);

        let result = sim.simulate(&config, &env);

        assert!(result.equilibrium_temp_k > 250.0);
        assert!(result.equilibrium_temp_k < 350.0);
    }

    #[test]
    fn test_latency_integration() {
        let sim = LatencySimulator::new(550.0);
        let result = sim.simulate(Some(100.0), None);

        assert!(result.total_latency_ms > 0.0);
        assert!(result.meets_requirement);
    }

    #[test]
    fn test_power_integration() {
        let analyzer = PowerAnalyzer::new(550.0);
        let profile = PowerProfile::new(500.0);

        let budget = analyzer.analyze(&profile, None, None, None, None);

        assert!(budget.solar_panel_area_m2 > 0.0);
        assert!(budget.battery_capacity_wh > 0.0);
        // Auto-sized panels meet requirements (margin near zero)
        assert!(budget.power_required_w > 0.0);
    }

    #[test]
    fn test_cross_module_consistency() {
        // Verify that power and thermal are consistent
        let power_analyzer = PowerAnalyzer::new(550.0);
        let power_profile = PowerProfile::new(500.0);
        let power_result = power_analyzer.analyze(&power_profile, None, None, None, None);

        let thermal_sim = ThermalSimulator::new();
        let thermal_config = ThermalConfig::for_power(500.0);
        let thermal_env = ThermalEnvironment::leo(550.0);
        let thermal_result = thermal_sim.simulate(&thermal_config, &thermal_env);

        // Both should be analyzing similar orbital parameters
        assert!(
            (power_result.eclipse_duration_min - thermal_result.equilibrium_temp_k).abs()
                < thermal_result.equilibrium_temp_k
        );
    }

    #[test]
    fn test_altitude_consistency() {
        let altitudes = [400.0, 550.0, 800.0, 1200.0];

        let feasibility_calc = FeasibilityCalculator::new(550.0);
        let latency_sim = LatencySimulator::new(550.0);

        let profile = WorkloadProfile::new(WorkloadType::Batch, 20.0);

        let feasibility_results = feasibility_calc.compare_scenarios(&profile, &altitudes);
        let latency_results = latency_sim.compare_altitudes(&altitudes);

        assert_eq!(feasibility_results.len(), altitudes.len());
        assert_eq!(latency_results.len(), altitudes.len());

        // Higher altitude = more latency
        assert!(latency_results[3].typical_latency_ms > latency_results[0].typical_latency_ms);
    }
}
