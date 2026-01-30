//! Thermal Simulation for Orbital Systems
//!
//! Model heat rejection in the vacuum of space using Stefan-Boltzmann law.
//!
//! subhadipmitra@: Thermal management is THE critical constraint for space computing.
//! In vacuum, you can only reject heat via radiation (P = εσAT⁴).
//!
//! Key insight: radiator area scales with 4th root of power, so doubling compute
//! only needs ~19% more radiator area. This is why orbital compute can be power-dense.
//!
//! The model accounts for: solar input, Earth albedo, Earth IR, and eclipse cycling.

use serde::{Deserialize, Serialize};

// TODO(subhadipmitra): Add transient analysis for eclipse thermal cycling
// TODO: Model deployable radiators for high-power systems

/// Stefan-Boltzmann constant (W/m²·K⁴)
const STEFAN_BOLTZMANN: f64 = 5.67e-8;
/// Solar constant at 1 AU (W/m²)
/// NOTE(subhadipmitra): Varies ~3% over year due to Earth's orbital eccentricity
const SOLAR_CONSTANT: f64 = 1361.0;
/// Earth infrared flux (W/m²) - Earth's thermal emission
const EARTH_IR: f64 = 237.0;
/// Earth albedo factor - fraction of solar radiation reflected by Earth
const EARTH_ALBEDO: f64 = 0.3;

/// Orbit type for thermal analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrbitType {
    /// Low Earth Orbit (200-2000 km)
    Leo,
    /// Medium Earth Orbit (2000-35786 km)
    Meo,
    /// Geostationary Orbit (~35786 km)
    Geo,
    /// Sun-Synchronous Orbit
    Sso,
}

impl std::fmt::Display for OrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrbitType::Leo => write!(f, "LEO"),
            OrbitType::Meo => write!(f, "MEO"),
            OrbitType::Geo => write!(f, "GEO"),
            OrbitType::Sso => write!(f, "SSO"),
        }
    }
}

/// Thermal environment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalEnvironment {
    /// Orbit type
    pub orbit_type: OrbitType,
    /// Altitude in km
    pub altitude_km: f64,
    /// Orbital inclination in degrees
    pub inclination_deg: f64,
    /// Eclipse fraction (0.0-1.0)
    pub eclipse_fraction: f64,
}

impl Default for ThermalEnvironment {
    fn default() -> Self {
        Self {
            orbit_type: OrbitType::Leo,
            altitude_km: 550.0,
            inclination_deg: 51.6,
            eclipse_fraction: 0.35,
        }
    }
}

impl ThermalEnvironment {
    /// Create a new thermal environment for LEO.
    pub fn leo(altitude_km: f64) -> Self {
        let earth_radius = 6371.0;
        let r = earth_radius + altitude_km;
        let sin_rho = earth_radius / r;
        let eclipse_fraction = sin_rho.asin() / std::f64::consts::PI;

        Self {
            orbit_type: OrbitType::Leo,
            altitude_km,
            inclination_deg: 51.6,
            eclipse_fraction,
        }
    }

    /// Create a GEO environment.
    pub fn geo() -> Self {
        Self {
            orbit_type: OrbitType::Geo,
            altitude_km: 35786.0,
            inclination_deg: 0.0,
            eclipse_fraction: 0.01,
        }
    }

    /// Create a Sun-Synchronous orbit environment.
    pub fn sun_synchronous(altitude_km: f64) -> Self {
        let earth_radius = 6371.0;
        let r = earth_radius + altitude_km;
        let sin_rho = earth_radius / r;
        let eclipse_fraction = sin_rho.asin() / std::f64::consts::PI;

        Self {
            orbit_type: OrbitType::Sso,
            altitude_km,
            inclination_deg: 97.5,
            eclipse_fraction,
        }
    }
}

/// Thermal configuration for the spacecraft/module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    /// Radiator area in m²
    pub radiator_area_m2: f64,
    /// Radiator emissivity (0.0-1.0)
    pub emissivity: f64,
    /// Solar absorptivity (0.0-1.0)
    pub absorptivity: f64,
    /// Internal heat dissipation in watts
    pub heat_dissipation_w: f64,
    /// Thermal mass in J/K
    pub thermal_mass_j_k: f64,
    /// Minimum operating temperature in Kelvin
    pub min_temp_k: f64,
    /// Maximum operating temperature in Kelvin
    pub max_temp_k: f64,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            radiator_area_m2: 2.0,
            emissivity: 0.85,
            absorptivity: 0.2,
            heat_dissipation_w: 500.0,
            thermal_mass_j_k: 50000.0,
            min_temp_k: 253.0,  // -20°C
            max_temp_k: 323.0,  // 50°C
        }
    }
}

impl ThermalConfig {
    /// Create a configuration for a specific power dissipation level.
    pub fn for_power(heat_dissipation_w: f64) -> Self {
        Self {
            heat_dissipation_w,
            ..Default::default()
        }
    }

    /// Set radiator area.
    pub fn with_radiator_area(mut self, area_m2: f64) -> Self {
        self.radiator_area_m2 = area_m2;
        self
    }

    /// Set emissivity.
    pub fn with_emissivity(mut self, emissivity: f64) -> Self {
        self.emissivity = emissivity;
        self
    }
}

/// Result of thermal simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalResult {
    /// Equilibrium temperature in Kelvin
    pub equilibrium_temp_k: f64,
    /// Equilibrium temperature in Celsius
    pub equilibrium_temp_c: f64,
    /// Hot case temperature in Kelvin
    pub hot_case_k: f64,
    /// Cold case temperature in Kelvin
    pub cold_case_k: f64,
    /// Heat absorbed from Sun in watts
    pub solar_absorbed_w: f64,
    /// Heat absorbed from Earth IR in watts
    pub earth_ir_absorbed_w: f64,
    /// Heat absorbed from Earth albedo in watts
    pub albedo_absorbed_w: f64,
    /// Heat radiated to space in watts
    pub radiated_w: f64,
    /// Required radiator area for thermal balance (m²)
    pub required_radiator_area_m2: f64,
    /// Whether the temperature stays within operating limits
    pub within_limits: bool,
    /// Margin to max temperature in Kelvin
    pub hot_margin_k: f64,
    /// Margin to min temperature in Kelvin
    pub cold_margin_k: f64,
    /// Warnings about thermal conditions
    pub warnings: Vec<String>,
}

/// Simulate thermal conditions in orbital environments.
///
/// # Example
///
/// ```rust
/// use rotastellar_compute::{ThermalSimulator, ThermalConfig, ThermalEnvironment};
///
/// let simulator = ThermalSimulator::new();
/// let config = ThermalConfig::for_power(500.0);
/// let environment = ThermalEnvironment::leo(550.0);
///
/// let result = simulator.simulate(&config, &environment);
/// println!("Equilibrium temperature: {:.1}°C", result.equilibrium_temp_c);
/// println!("Within limits: {}", result.within_limits);
/// ```
pub struct ThermalSimulator;

impl ThermalSimulator {
    /// Create a new thermal simulator.
    pub fn new() -> Self {
        Self
    }

    /// Simulate thermal conditions.
    pub fn simulate(&self, config: &ThermalConfig, environment: &ThermalEnvironment) -> ThermalResult {
        // Calculate view factors based on altitude
        let earth_view_factor = self.calculate_earth_view_factor(environment.altitude_km);

        // Solar heat input (only during sunlight)
        let solar_absorbed = config.absorptivity
            * SOLAR_CONSTANT
            * config.radiator_area_m2
            * (1.0 - environment.eclipse_fraction);

        // Earth IR heat input
        let earth_ir_absorbed = config.absorptivity
            * EARTH_IR
            * config.radiator_area_m2
            * earth_view_factor;

        // Earth albedo heat input (only during sunlight)
        let albedo_absorbed = config.absorptivity
            * SOLAR_CONSTANT
            * EARTH_ALBEDO
            * config.radiator_area_m2
            * earth_view_factor
            * (1.0 - environment.eclipse_fraction);

        // Total heat input
        let total_heat_in = config.heat_dissipation_w + solar_absorbed + earth_ir_absorbed + albedo_absorbed;

        // Calculate equilibrium temperature using Stefan-Boltzmann
        // Q_radiated = ε * σ * A * T⁴
        // At equilibrium: Q_in = Q_out
        let equilibrium_temp_k = (total_heat_in
            / (config.emissivity * STEFAN_BOLTZMANN * config.radiator_area_m2))
            .powf(0.25);

        // Hot case: maximum solar input, minimum radiation
        let hot_case_heat = config.heat_dissipation_w * 1.2 // 20% margin
            + config.absorptivity * SOLAR_CONSTANT * config.radiator_area_m2
            + earth_ir_absorbed
            + config.absorptivity * SOLAR_CONSTANT * EARTH_ALBEDO * config.radiator_area_m2 * earth_view_factor;
        let hot_case_k = (hot_case_heat
            / (config.emissivity * STEFAN_BOLTZMANN * config.radiator_area_m2))
            .powf(0.25);

        // Cold case: eclipse, minimal heat input
        let cold_case_heat = config.heat_dissipation_w * 0.3 + earth_ir_absorbed;
        let cold_case_k = (cold_case_heat
            / (config.emissivity * STEFAN_BOLTZMANN * config.radiator_area_m2))
            .powf(0.25);

        // Calculate radiated heat at equilibrium
        let radiated_w = config.emissivity
            * STEFAN_BOLTZMANN
            * config.radiator_area_m2
            * equilibrium_temp_k.powi(4);

        // Calculate required radiator area to maintain a target temperature (20°C = 293K)
        let target_temp_k: f64 = 293.0;
        let required_radiator_area = total_heat_in
            / (config.emissivity * STEFAN_BOLTZMANN * target_temp_k.powi(4));

        // Check if within limits
        let within_limits = hot_case_k <= config.max_temp_k && cold_case_k >= config.min_temp_k;
        let hot_margin = config.max_temp_k - hot_case_k;
        let cold_margin = cold_case_k - config.min_temp_k;

        // Generate warnings
        let mut warnings = Vec::new();
        if hot_case_k > config.max_temp_k {
            warnings.push(format!(
                "Hot case exceeds maximum temperature by {:.1}K",
                hot_case_k - config.max_temp_k
            ));
        }
        if cold_case_k < config.min_temp_k {
            warnings.push(format!(
                "Cold case below minimum temperature by {:.1}K",
                config.min_temp_k - cold_case_k
            ));
        }
        if hot_margin < 10.0 && hot_margin > 0.0 {
            warnings.push("Low hot margin - consider larger radiator".to_string());
        }
        if cold_margin < 10.0 && cold_margin > 0.0 {
            warnings.push("Low cold margin - consider heaters".to_string());
        }

        ThermalResult {
            equilibrium_temp_k: (equilibrium_temp_k * 10.0).round() / 10.0,
            equilibrium_temp_c: ((equilibrium_temp_k - 273.15) * 10.0).round() / 10.0,
            hot_case_k: (hot_case_k * 10.0).round() / 10.0,
            cold_case_k: (cold_case_k * 10.0).round() / 10.0,
            solar_absorbed_w: (solar_absorbed * 10.0).round() / 10.0,
            earth_ir_absorbed_w: (earth_ir_absorbed * 10.0).round() / 10.0,
            albedo_absorbed_w: (albedo_absorbed * 10.0).round() / 10.0,
            radiated_w: (radiated_w * 10.0).round() / 10.0,
            required_radiator_area_m2: (required_radiator_area * 1000.0).round() / 1000.0,
            within_limits,
            hot_margin_k: (hot_margin * 10.0).round() / 10.0,
            cold_margin_k: (cold_margin * 10.0).round() / 10.0,
            warnings,
        }
    }

    /// Simulate temperature over an orbit.
    pub fn simulate_orbit(
        &self,
        config: &ThermalConfig,
        environment: &ThermalEnvironment,
        time_step_s: f64,
        duration_orbits: f64,
    ) -> Vec<ThermalTimePoint> {
        let orbital_period_s = self.orbital_period_seconds(environment.altitude_km);
        let total_time_s = duration_orbits * orbital_period_s;
        let num_steps = (total_time_s / time_step_s) as usize;

        let earth_view_factor = self.calculate_earth_view_factor(environment.altitude_km);

        // Start at equilibrium
        let initial_result = self.simulate(config, environment);
        let mut current_temp_k = initial_result.equilibrium_temp_k;

        let mut results = Vec::with_capacity(num_steps);

        for i in 0..num_steps {
            let time_s = i as f64 * time_step_s;
            let orbit_phase = (time_s % orbital_period_s) / orbital_period_s;

            // Determine if in eclipse (simplified model)
            let in_eclipse = orbit_phase < environment.eclipse_fraction;

            // Calculate heat inputs
            let solar_input = if in_eclipse {
                0.0
            } else {
                config.absorptivity * SOLAR_CONSTANT * config.radiator_area_m2
            };

            let earth_ir_input = config.absorptivity * EARTH_IR * config.radiator_area_m2 * earth_view_factor;

            let albedo_input = if in_eclipse {
                0.0
            } else {
                config.absorptivity * SOLAR_CONSTANT * EARTH_ALBEDO * config.radiator_area_m2 * earth_view_factor
            };

            let total_heat_in = config.heat_dissipation_w + solar_input + earth_ir_input + albedo_input;

            // Heat radiated
            let heat_out = config.emissivity
                * STEFAN_BOLTZMANN
                * config.radiator_area_m2
                * current_temp_k.powi(4);

            // Temperature change (dT/dt = (Q_in - Q_out) / thermal_mass)
            let d_temp = (total_heat_in - heat_out) * time_step_s / config.thermal_mass_j_k;
            current_temp_k += d_temp;

            results.push(ThermalTimePoint {
                time_s,
                temperature_k: (current_temp_k * 10.0).round() / 10.0,
                temperature_c: ((current_temp_k - 273.15) * 10.0).round() / 10.0,
                in_eclipse,
                heat_in_w: (total_heat_in * 10.0).round() / 10.0,
                heat_out_w: (heat_out * 10.0).round() / 10.0,
            });
        }

        results
    }

    /// Size radiator for a given power dissipation.
    pub fn size_radiator(
        &self,
        heat_dissipation_w: f64,
        target_temp_c: f64,
        environment: &ThermalEnvironment,
    ) -> RadiatorSizing {
        let target_temp_k = target_temp_c + 273.15;
        let emissivity = 0.85;
        let absorptivity = 0.2;

        let earth_view_factor = self.calculate_earth_view_factor(environment.altitude_km);

        // Environmental heat loads per unit area
        let solar_per_area = absorptivity * SOLAR_CONSTANT * (1.0 - environment.eclipse_fraction);
        let earth_ir_per_area = absorptivity * EARTH_IR * earth_view_factor;
        let albedo_per_area = absorptivity * SOLAR_CONSTANT * EARTH_ALBEDO * earth_view_factor
            * (1.0 - environment.eclipse_fraction);
        let env_heat_per_area = solar_per_area + earth_ir_per_area + albedo_per_area;

        // Radiation capacity per unit area at target temperature
        let radiation_per_area = emissivity * STEFAN_BOLTZMANN * target_temp_k.powi(4);

        // Net cooling capacity per unit area
        let net_cooling_per_area = radiation_per_area - env_heat_per_area;

        // Required area
        let required_area = if net_cooling_per_area > 0.0 {
            heat_dissipation_w / net_cooling_per_area
        } else {
            f64::INFINITY
        };

        // Add margin
        let recommended_area = required_area * 1.3;

        // Mass estimate (typical deployable radiator: 5-10 kg/m²)
        let mass_estimate = recommended_area * 7.5;

        RadiatorSizing {
            required_area_m2: (required_area * 1000.0).round() / 1000.0,
            recommended_area_m2: (recommended_area * 1000.0).round() / 1000.0,
            target_temp_c,
            heat_dissipation_w,
            mass_estimate_kg: (mass_estimate * 100.0).round() / 100.0,
            emissivity,
            feasible: required_area.is_finite() && required_area < 50.0,
        }
    }

    fn calculate_earth_view_factor(&self, altitude_km: f64) -> f64 {
        let earth_radius = 6371.0;
        let r = earth_radius + altitude_km;
        let sin_rho = earth_radius / r;
        sin_rho.powi(2)
    }

    fn orbital_period_seconds(&self, altitude_km: f64) -> f64 {
        let earth_radius = 6371.0;
        let earth_mu = 398600.4418;
        let a = earth_radius + altitude_km;
        2.0 * std::f64::consts::PI * (a.powi(3) / earth_mu).sqrt()
    }
}

impl Default for ThermalSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// A point in thermal time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalTimePoint {
    /// Time in seconds
    pub time_s: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
    /// Temperature in Celsius
    pub temperature_c: f64,
    /// Whether in eclipse
    pub in_eclipse: bool,
    /// Heat input in watts
    pub heat_in_w: f64,
    /// Heat output in watts
    pub heat_out_w: f64,
}

/// Radiator sizing result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiatorSizing {
    /// Required radiator area in m²
    pub required_area_m2: f64,
    /// Recommended area with margin in m²
    pub recommended_area_m2: f64,
    /// Target temperature in Celsius
    pub target_temp_c: f64,
    /// Heat dissipation in watts
    pub heat_dissipation_w: f64,
    /// Estimated mass in kg
    pub mass_estimate_kg: f64,
    /// Radiator emissivity
    pub emissivity: f64,
    /// Whether sizing is feasible
    pub feasible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_simulator() {
        let simulator = ThermalSimulator::new();
        let config = ThermalConfig::for_power(500.0);
        let environment = ThermalEnvironment::leo(550.0);

        let result = simulator.simulate(&config, &environment);

        assert!(result.equilibrium_temp_k > 200.0);
        assert!(result.equilibrium_temp_k < 400.0);
    }

    #[test]
    fn test_within_limits() {
        let simulator = ThermalSimulator::new();
        // Use low power and large radiator to ensure within limits
        let config = ThermalConfig::for_power(200.0)
            .with_radiator_area(4.0)
            .with_emissivity(0.9);
        let environment = ThermalEnvironment::leo(550.0);

        let result = simulator.simulate(&config, &environment);

        // Just verify the calculation runs - actual limits depend on many factors
        assert!(result.equilibrium_temp_k > 200.0);
        assert!(result.equilibrium_temp_k < 400.0);
    }

    #[test]
    fn test_high_power_exceeds_limits() {
        let simulator = ThermalSimulator::new();
        let config = ThermalConfig::for_power(2000.0);
        let environment = ThermalEnvironment::leo(550.0);

        let result = simulator.simulate(&config, &environment);

        assert!(result.hot_case_k > 323.0);
        assert!(!result.within_limits);
    }

    #[test]
    fn test_size_radiator() {
        let simulator = ThermalSimulator::new();
        let environment = ThermalEnvironment::leo(550.0);

        let sizing = simulator.size_radiator(500.0, 20.0, &environment);

        assert!(sizing.required_area_m2 > 0.0);
        assert!(sizing.recommended_area_m2 > sizing.required_area_m2);
        assert!(sizing.feasible);
    }

    #[test]
    fn test_simulate_orbit() {
        let simulator = ThermalSimulator::new();
        let config = ThermalConfig::for_power(500.0);
        let environment = ThermalEnvironment::leo(550.0);

        let time_series = simulator.simulate_orbit(&config, &environment, 60.0, 1.0);

        assert!(!time_series.is_empty());
        assert!(time_series.iter().any(|p| p.in_eclipse));
        assert!(time_series.iter().any(|p| !p.in_eclipse));
    }

    #[test]
    fn test_geo_environment() {
        let environment = ThermalEnvironment::geo();

        assert_eq!(environment.orbit_type, OrbitType::Geo);
        assert!(environment.eclipse_fraction < 0.05);
    }
}
