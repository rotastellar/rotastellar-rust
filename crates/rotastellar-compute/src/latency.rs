//! Latency Simulation for Orbital Compute
//!
//! Model end-to-end latency for space-based data processing.

use serde::{Deserialize, Serialize};

/// Speed of light in km/s
const SPEED_OF_LIGHT_KM_S: f64 = 299792.458;

/// Type of communication link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Ground to satellite uplink
    Uplink,
    /// Satellite to ground downlink
    Downlink,
    /// Inter-satellite link (optical or RF)
    Isl,
    /// Ground relay station link
    GroundRelay,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkType::Uplink => write!(f, "uplink"),
            LinkType::Downlink => write!(f, "downlink"),
            LinkType::Isl => write!(f, "inter-satellite"),
            LinkType::GroundRelay => write!(f, "ground-relay"),
        }
    }
}

/// A component contributing to total latency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyComponent {
    /// Name of the component
    pub name: String,
    /// Type of link (if applicable)
    pub link_type: Option<LinkType>,
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// Description of the component
    pub description: String,
}

impl LatencyComponent {
    /// Create a new latency component.
    pub fn new(name: &str, latency_ms: f64, description: &str) -> Self {
        Self {
            name: name.to_string(),
            link_type: None,
            latency_ms,
            description: description.to_string(),
        }
    }

    /// Create a link latency component.
    pub fn link(name: &str, link_type: LinkType, latency_ms: f64) -> Self {
        let description = match link_type {
            LinkType::Uplink => "Ground to satellite propagation".to_string(),
            LinkType::Downlink => "Satellite to ground propagation".to_string(),
            LinkType::Isl => "Inter-satellite optical link".to_string(),
            LinkType::GroundRelay => "Ground relay station hop".to_string(),
        };
        Self {
            name: name.to_string(),
            link_type: Some(link_type),
            latency_ms,
            description,
        }
    }
}

/// Result of latency simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyResult {
    /// Total end-to-end latency in ms
    pub total_latency_ms: f64,
    /// Propagation latency (speed of light) in ms
    pub propagation_latency_ms: f64,
    /// Processing latency in ms
    pub processing_latency_ms: f64,
    /// Queueing latency in ms
    pub queueing_latency_ms: f64,
    /// Transmission latency in ms
    pub transmission_latency_ms: f64,
    /// Breakdown of latency components
    pub components: Vec<LatencyComponent>,
    /// One-way latency in ms
    pub one_way_latency_ms: f64,
    /// Round-trip latency in ms
    pub round_trip_latency_ms: f64,
    /// Comparison with terrestrial alternative
    pub terrestrial_comparison: TerrestrialComparison,
    /// Whether latency meets the requirement
    pub meets_requirement: bool,
    /// Latency requirement if specified
    pub requirement_ms: Option<f64>,
}

/// Comparison with terrestrial latency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrestrialComparison {
    /// Typical terrestrial latency in ms
    pub terrestrial_latency_ms: f64,
    /// Difference (orbital - terrestrial) in ms
    pub difference_ms: f64,
    /// Ratio (orbital / terrestrial)
    pub ratio: f64,
    /// Whether orbital is competitive
    pub competitive: bool,
}

/// Simulate latency for orbital compute scenarios.
///
/// # Example
///
/// ```rust
/// use rotastellar_compute::LatencySimulator;
///
/// let simulator = LatencySimulator::new(550.0);
/// let result = simulator.simulate(Some(100.0), Some(2));
/// println!("Total latency: {:.1} ms", result.total_latency_ms);
/// println!("Meets 100ms requirement: {}", result.meets_requirement);
/// ```
pub struct LatencySimulator {
    orbit_altitude_km: f64,
    processing_latency_ms: f64,
    ground_network_latency_ms: f64,
}

impl LatencySimulator {
    /// Create a new latency simulator.
    ///
    /// # Arguments
    ///
    /// * `orbit_altitude_km` - Orbit altitude in kilometers
    pub fn new(orbit_altitude_km: f64) -> Self {
        Self {
            orbit_altitude_km,
            processing_latency_ms: 5.0,
            ground_network_latency_ms: 10.0,
        }
    }

    /// Create a simulator with default altitude (550 km).
    pub fn default_altitude() -> Self {
        Self::new(550.0)
    }

    /// Set processing latency.
    pub fn with_processing_latency(mut self, latency_ms: f64) -> Self {
        self.processing_latency_ms = latency_ms;
        self
    }

    /// Set ground network latency.
    pub fn with_ground_network_latency(mut self, latency_ms: f64) -> Self {
        self.ground_network_latency_ms = latency_ms;
        self
    }

    /// Simulate end-to-end latency.
    ///
    /// # Arguments
    ///
    /// * `latency_requirement_ms` - Optional latency requirement to check against
    /// * `num_isl_hops` - Number of inter-satellite link hops (default: 0)
    pub fn simulate(
        &self,
        latency_requirement_ms: Option<f64>,
        num_isl_hops: Option<u32>,
    ) -> LatencyResult {
        let isl_hops = num_isl_hops.unwrap_or(0);
        let mut components = Vec::new();

        // Ground network latency
        components.push(LatencyComponent::new(
            "Ground Network",
            self.ground_network_latency_ms,
            "User to ground station network latency",
        ));

        // Uplink propagation
        let uplink_latency = self.propagation_delay_ms(self.orbit_altitude_km);
        components.push(LatencyComponent::link("Uplink", LinkType::Uplink, uplink_latency));

        // ISL hops
        if isl_hops > 0 {
            let isl_latency_per_hop = self.isl_propagation_delay_ms();
            for i in 0..isl_hops {
                components.push(LatencyComponent::link(
                    &format!("ISL Hop {}", i + 1),
                    LinkType::Isl,
                    isl_latency_per_hop,
                ));
            }
        }

        // Orbital processing
        components.push(LatencyComponent::new(
            "Orbital Processing",
            self.processing_latency_ms,
            "On-board compute processing time",
        ));

        // Downlink propagation
        let downlink_latency = self.propagation_delay_ms(self.orbit_altitude_km);
        components.push(LatencyComponent::link("Downlink", LinkType::Downlink, downlink_latency));

        // Return ground network
        components.push(LatencyComponent::new(
            "Return Ground Network",
            self.ground_network_latency_ms,
            "Ground station to user network latency",
        ));

        // Calculate totals
        let propagation_latency = uplink_latency + downlink_latency
            + (isl_hops as f64 * self.isl_propagation_delay_ms());
        let queueing_latency = 2.0; // Fixed small queueing delay
        let transmission_latency = 1.0; // Fixed transmission overhead

        let total_latency = components.iter().map(|c| c.latency_ms).sum::<f64>()
            + queueing_latency
            + transmission_latency;

        let one_way_latency = total_latency / 2.0;
        let round_trip_latency = total_latency;

        // Terrestrial comparison
        let terrestrial_latency = self.estimate_terrestrial_latency();
        let terrestrial_comparison = TerrestrialComparison {
            terrestrial_latency_ms: terrestrial_latency,
            difference_ms: (total_latency - terrestrial_latency * 10.0).round() / 10.0,
            ratio: (total_latency / terrestrial_latency * 100.0).round() / 100.0,
            competitive: total_latency < terrestrial_latency * 2.0,
        };

        // Check requirement
        let meets_requirement = latency_requirement_ms
            .map(|req| total_latency <= req)
            .unwrap_or(true);

        LatencyResult {
            total_latency_ms: (total_latency * 10.0).round() / 10.0,
            propagation_latency_ms: (propagation_latency * 10.0).round() / 10.0,
            processing_latency_ms: self.processing_latency_ms,
            queueing_latency_ms: queueing_latency,
            transmission_latency_ms: transmission_latency,
            components,
            one_way_latency_ms: (one_way_latency * 10.0).round() / 10.0,
            round_trip_latency_ms: (round_trip_latency * 10.0).round() / 10.0,
            terrestrial_comparison,
            meets_requirement,
            requirement_ms: latency_requirement_ms,
        }
    }

    /// Calculate minimum theoretical latency.
    pub fn min_latency_ms(&self) -> f64 {
        // Minimum is just the propagation delay (no processing, no queueing)
        let uplink = self.propagation_delay_ms(self.orbit_altitude_km);
        let downlink = self.propagation_delay_ms(self.orbit_altitude_km);
        uplink + downlink
    }

    /// Calculate latency for a specific elevation angle.
    pub fn latency_at_elevation(&self, elevation_deg: f64) -> ElevationLatency {
        let slant_range = self.slant_range_km(elevation_deg);
        let propagation_ms = (slant_range / SPEED_OF_LIGHT_KM_S) * 1000.0;

        // At low elevation, more atmospheric effects
        let atmospheric_delay = if elevation_deg < 10.0 {
            2.0
        } else if elevation_deg < 30.0 {
            1.0
        } else {
            0.5
        };

        let total_one_way = propagation_ms + atmospheric_delay;

        ElevationLatency {
            elevation_deg,
            slant_range_km: (slant_range * 10.0).round() / 10.0,
            propagation_ms: (propagation_ms * 100.0).round() / 100.0,
            atmospheric_delay_ms: atmospheric_delay,
            total_one_way_ms: (total_one_way * 100.0).round() / 100.0,
            total_round_trip_ms: (total_one_way * 2.0 * 100.0).round() / 100.0,
        }
    }

    /// Compare latency across different altitudes.
    pub fn compare_altitudes(&self, altitudes: &[f64]) -> Vec<AltitudeLatency> {
        altitudes
            .iter()
            .map(|&altitude| {
                let simulator = LatencySimulator::new(altitude);
                let result = simulator.simulate(None, None);
                AltitudeLatency {
                    altitude_km: altitude,
                    min_latency_ms: simulator.min_latency_ms(),
                    typical_latency_ms: result.total_latency_ms,
                    propagation_ms: result.propagation_latency_ms,
                }
            })
            .collect()
    }

    fn propagation_delay_ms(&self, altitude_km: f64) -> f64 {
        // Simplified: assume average elevation of 45 degrees
        let slant_range = self.slant_range_km(45.0);
        let base_delay = (slant_range / SPEED_OF_LIGHT_KM_S) * 1000.0;

        // Adjust for altitude (higher altitude = longer delay)
        let altitude_factor = altitude_km / 550.0;
        base_delay * altitude_factor.sqrt()
    }

    fn slant_range_km(&self, elevation_deg: f64) -> f64 {
        let earth_radius = 6371.0;
        let elevation_rad = elevation_deg.to_radians();
        let r = earth_radius + self.orbit_altitude_km;

        // Slant range formula
        let sin_el = elevation_rad.sin();
        let term1 = (r / earth_radius).powi(2) - (elevation_rad.cos()).powi(2);
        earth_radius * (term1.sqrt() - sin_el)
    }

    fn isl_propagation_delay_ms(&self) -> f64 {
        // Typical ISL distance of 2000 km
        let isl_distance_km = 2000.0;
        (isl_distance_km / SPEED_OF_LIGHT_KM_S) * 1000.0
    }

    fn estimate_terrestrial_latency(&self) -> f64 {
        // Typical terrestrial datacenter round-trip: 20-50ms
        35.0
    }
}

impl Default for LatencySimulator {
    fn default() -> Self {
        Self::default_altitude()
    }
}

/// Latency at a specific elevation angle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationLatency {
    /// Elevation angle in degrees
    pub elevation_deg: f64,
    /// Slant range in km
    pub slant_range_km: f64,
    /// Propagation delay in ms
    pub propagation_ms: f64,
    /// Atmospheric delay in ms
    pub atmospheric_delay_ms: f64,
    /// Total one-way latency in ms
    pub total_one_way_ms: f64,
    /// Total round-trip latency in ms
    pub total_round_trip_ms: f64,
}

/// Latency comparison for different altitudes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltitudeLatency {
    /// Orbit altitude in km
    pub altitude_km: f64,
    /// Minimum theoretical latency in ms
    pub min_latency_ms: f64,
    /// Typical end-to-end latency in ms
    pub typical_latency_ms: f64,
    /// Propagation component in ms
    pub propagation_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_simulator() {
        let simulator = LatencySimulator::new(550.0);
        let result = simulator.simulate(None, None);

        assert!(result.total_latency_ms > 0.0);
        assert!(result.propagation_latency_ms > 0.0);
        assert!(!result.components.is_empty());
    }

    #[test]
    fn test_meets_requirement() {
        let simulator = LatencySimulator::new(550.0);

        // Should meet 100ms requirement
        let result = simulator.simulate(Some(100.0), None);
        assert!(result.meets_requirement);

        // Should not meet 5ms requirement
        let result = simulator.simulate(Some(5.0), None);
        assert!(!result.meets_requirement);
    }

    #[test]
    fn test_isl_hops_increase_latency() {
        let simulator = LatencySimulator::new(550.0);

        let result_no_isl = simulator.simulate(None, Some(0));
        let result_with_isl = simulator.simulate(None, Some(3));

        assert!(result_with_isl.total_latency_ms > result_no_isl.total_latency_ms);
    }

    #[test]
    fn test_elevation_latency() {
        let simulator = LatencySimulator::new(550.0);

        let high_elevation = simulator.latency_at_elevation(90.0);
        let low_elevation = simulator.latency_at_elevation(10.0);

        // Lower elevation = longer slant range = more latency
        assert!(low_elevation.propagation_ms > high_elevation.propagation_ms);
    }

    #[test]
    fn test_compare_altitudes() {
        let simulator = LatencySimulator::new(550.0);
        let results = simulator.compare_altitudes(&[400.0, 550.0, 1200.0]);

        assert_eq!(results.len(), 3);
        // Higher altitude = more latency
        assert!(results[2].typical_latency_ms > results[0].typical_latency_ms);
    }

    #[test]
    fn test_terrestrial_comparison() {
        let simulator = LatencySimulator::new(550.0);
        let result = simulator.simulate(None, None);

        assert!(result.terrestrial_comparison.terrestrial_latency_ms > 0.0);
        assert!(result.terrestrial_comparison.ratio > 0.0);
    }

    #[test]
    fn test_min_latency() {
        let simulator = LatencySimulator::new(550.0);
        let min = simulator.min_latency_ms();

        // At 550km, min round-trip should be roughly 4-8ms
        assert!(min > 3.0);
        assert!(min < 20.0);
    }
}
