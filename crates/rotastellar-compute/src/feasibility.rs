//! Feasibility Analysis for Orbital Compute
//!
//! Analyze workload suitability for orbital compute environments.
//!
//! subhadipmitra@: Not all workloads belong in space. Good candidates:
//! - Batch processing (tolerant of intermittent connectivity)
//! - Large-scale ML training (compute-bound, can checkpoint)
//! - Rendering farms (embarrassingly parallel)
//! - Scientific simulation (high compute:data ratio)
//!
//! Poor candidates:
//! - Real-time trading (latency-critical)
//! - Interactive apps (user-facing latency)
//! - Database OLTP (requires persistent connections)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO(subhadipmitra): Add cost estimation to feasibility report
// TODO: Factor in constellation coverage for latency-sensitive workloads

/// Types of compute workloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadType {
    /// ML inference workloads
    Inference,
    /// ML training workloads
    Training,
    /// Batch processing
    Batch,
    /// Real-time streaming
    Streaming,
    /// 3D rendering
    Render,
    /// Scientific simulation
    Simulation,
    /// Data analytics
    Analytics,
}

/// Feasibility assessment rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeasibilityRating {
    /// Score >= 85
    Excellent,
    /// Score >= 70
    Good,
    /// Score >= 50
    Moderate,
    /// Score >= 30
    Poor,
    /// Score < 30
    Unsuitable,
}

impl std::fmt::Display for FeasibilityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeasibilityRating::Excellent => write!(f, "excellent"),
            FeasibilityRating::Good => write!(f, "good"),
            FeasibilityRating::Moderate => write!(f, "moderate"),
            FeasibilityRating::Poor => write!(f, "poor"),
            FeasibilityRating::Unsuitable => write!(f, "unsuitable"),
        }
    }
}

/// Profile of a compute workload for feasibility analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    /// Type of workload
    pub workload_type: WorkloadType,
    /// Required compute in TFLOPS
    pub compute_tflops: f64,
    /// Required memory in GB
    pub memory_gb: Option<f64>,
    /// Required storage in GB
    pub storage_gb: Option<f64>,
    /// Data to transfer per day in GB
    pub data_transfer_gb: Option<f64>,
    /// Maximum acceptable latency in ms
    pub latency_requirement_ms: Option<f64>,
    /// For batch workloads, typical job duration in hours
    pub batch_duration_hours: Option<f64>,
    /// Required uptime percentage (0-100)
    pub availability_requirement: Option<f64>,
}

impl WorkloadProfile {
    /// Create a new workload profile with the given type and compute requirement.
    pub fn new(workload_type: WorkloadType, compute_tflops: f64) -> Self {
        Self {
            workload_type,
            compute_tflops,
            memory_gb: None,
            storage_gb: None,
            data_transfer_gb: None,
            latency_requirement_ms: None,
            batch_duration_hours: None,
            availability_requirement: None,
        }
    }

    /// Set memory requirement.
    pub fn with_memory_gb(mut self, memory_gb: f64) -> Self {
        self.memory_gb = Some(memory_gb);
        self
    }

    /// Set data transfer requirement.
    pub fn with_data_transfer_gb(mut self, data_transfer_gb: f64) -> Self {
        self.data_transfer_gb = Some(data_transfer_gb);
        self
    }

    /// Set latency requirement.
    pub fn with_latency_requirement_ms(mut self, latency_ms: f64) -> Self {
        self.latency_requirement_ms = Some(latency_ms);
        self
    }
}

/// Result of feasibility analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityResult {
    /// Whether the workload is feasible for orbital compute
    pub feasible: bool,
    /// Overall feasibility rating
    pub rating: FeasibilityRating,
    /// Numeric score (0-100)
    pub score: f64,
    /// Whether compute requirements can be met
    pub compute_feasible: bool,
    /// Whether thermal constraints can be satisfied
    pub thermal_feasible: bool,
    /// Whether power requirements can be met
    pub power_feasible: bool,
    /// Whether latency requirements can be met
    pub latency_feasible: bool,
    /// Whether data transfer requirements can be met
    pub data_transfer_feasible: bool,
    /// List of recommendations
    pub recommendations: Vec<String>,
    /// Key constraints identified
    pub constraints: HashMap<String, f64>,
    /// Cost factor relative to terrestrial (1.0 = same)
    pub estimated_cost_factor: f64,
}

#[derive(Clone, Copy)]
struct WorkloadCharacteristics {
    thermal_factor: f64,
    power_factor: f64,
    latency_sensitive: bool,
    batch_friendly: bool,
}

impl WorkloadCharacteristics {
    fn for_workload(workload_type: WorkloadType) -> Self {
        match workload_type {
            WorkloadType::Inference => Self {
                thermal_factor: 0.7,
                power_factor: 0.6,
                latency_sensitive: true,
                batch_friendly: true,
            },
            WorkloadType::Training => Self {
                thermal_factor: 1.0,
                power_factor: 1.0,
                latency_sensitive: false,
                batch_friendly: true,
            },
            WorkloadType::Batch => Self {
                thermal_factor: 0.8,
                power_factor: 0.7,
                latency_sensitive: false,
                batch_friendly: true,
            },
            WorkloadType::Streaming => Self {
                thermal_factor: 0.5,
                power_factor: 0.5,
                latency_sensitive: true,
                batch_friendly: false,
            },
            WorkloadType::Render => Self {
                thermal_factor: 1.0,
                power_factor: 0.9,
                latency_sensitive: false,
                batch_friendly: true,
            },
            WorkloadType::Simulation => Self {
                thermal_factor: 0.9,
                power_factor: 0.8,
                latency_sensitive: false,
                batch_friendly: true,
            },
            WorkloadType::Analytics => Self {
                thermal_factor: 0.6,
                power_factor: 0.5,
                latency_sensitive: false,
                batch_friendly: true,
            },
        }
    }
}

/// Analyze workload feasibility for orbital compute.
///
/// # Example
///
/// ```rust
/// use rotastellar_compute::{FeasibilityCalculator, WorkloadProfile, WorkloadType};
///
/// let calculator = FeasibilityCalculator::new(550.0);
/// let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0)
///     .with_memory_gb(32.0)
///     .with_latency_requirement_ms(100.0);
/// let result = calculator.analyze(&profile, None);
/// println!("Feasible: {}, Rating: {}", result.feasible, result.rating);
/// ```
pub struct FeasibilityCalculator {
    orbit_altitude_km: f64,
}

impl FeasibilityCalculator {
    /// Maximum compute capacity in TFLOPS
    pub const MAX_COMPUTE_TFLOPS: f64 = 100.0;
    /// Maximum memory capacity in GB
    pub const MAX_MEMORY_GB: f64 = 256.0;
    /// Maximum power in watts
    pub const MAX_POWER_WATTS: f64 = 2000.0;
    /// Maximum data transfer per day in GB
    pub const MAX_DATA_TRANSFER_GB_DAY: f64 = 1000.0;

    /// Create a new feasibility calculator.
    ///
    /// # Arguments
    ///
    /// * `orbit_altitude_km` - Default orbit altitude in kilometers
    pub fn new(orbit_altitude_km: f64) -> Self {
        Self { orbit_altitude_km }
    }

    /// Create a calculator with default altitude (550 km).
    pub fn default_altitude() -> Self {
        Self::new(550.0)
    }

    /// Analyze workload feasibility.
    ///
    /// # Arguments
    ///
    /// * `profile` - The workload profile to analyze
    /// * `orbit_altitude_km` - Optional override for orbit altitude
    pub fn analyze(&self, profile: &WorkloadProfile, orbit_altitude_km: Option<f64>) -> FeasibilityResult {
        let altitude = orbit_altitude_km.unwrap_or(self.orbit_altitude_km);
        let characteristics = WorkloadCharacteristics::for_workload(profile.workload_type);

        let memory_gb = profile.memory_gb.unwrap_or(16.0);
        let data_transfer_gb = profile.data_transfer_gb.unwrap_or(10.0);

        // Check individual constraints
        let (compute_ok, compute_score) = self.check_compute(profile.compute_tflops, memory_gb);
        let (thermal_ok, thermal_score) = self.check_thermal(profile.compute_tflops, &characteristics);
        let (power_ok, power_score) = self.check_power(profile.compute_tflops, &characteristics);
        let (latency_ok, latency_score) =
            self.check_latency(profile.latency_requirement_ms, altitude, &characteristics);
        let (data_ok, data_score) = self.check_data_transfer(data_transfer_gb);

        // Calculate overall score
        let scores = [compute_score, thermal_score, power_score, latency_score, data_score];
        let overall_score = scores.iter().sum::<f64>() / scores.len() as f64;

        // Determine feasibility
        let feasible = compute_ok && thermal_ok && power_ok && latency_ok && data_ok;

        // Determine rating
        let rating = if overall_score >= 85.0 {
            FeasibilityRating::Excellent
        } else if overall_score >= 70.0 {
            FeasibilityRating::Good
        } else if overall_score >= 50.0 {
            FeasibilityRating::Moderate
        } else if overall_score >= 30.0 {
            FeasibilityRating::Poor
        } else {
            FeasibilityRating::Unsuitable
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            profile,
            compute_ok,
            thermal_ok,
            power_ok,
            latency_ok,
            data_ok,
        );

        // Estimate cost factor
        let cost_factor = self.estimate_cost_factor(profile, &characteristics);

        // Build constraints map
        let mut constraints = HashMap::new();
        constraints.insert("compute_score".to_string(), compute_score);
        constraints.insert("thermal_score".to_string(), thermal_score);
        constraints.insert("power_score".to_string(), power_score);
        constraints.insert("latency_score".to_string(), latency_score);
        constraints.insert("data_transfer_score".to_string(), data_score);
        constraints.insert("orbit_altitude_km".to_string(), altitude);

        FeasibilityResult {
            feasible,
            rating,
            score: (overall_score * 100.0).round() / 100.0,
            compute_feasible: compute_ok,
            thermal_feasible: thermal_ok,
            power_feasible: power_ok,
            latency_feasible: latency_ok,
            data_transfer_feasible: data_ok,
            recommendations,
            constraints,
            estimated_cost_factor: cost_factor,
        }
    }

    /// Compare feasibility across different orbit scenarios.
    pub fn compare_scenarios(
        &self,
        profile: &WorkloadProfile,
        altitudes: &[f64],
    ) -> Vec<ScenarioResult> {
        altitudes
            .iter()
            .map(|&altitude| {
                let result = self.analyze(profile, Some(altitude));
                ScenarioResult {
                    altitude_km: altitude,
                    feasible: result.feasible,
                    rating: result.rating,
                    score: result.score,
                }
            })
            .collect()
    }

    fn check_compute(&self, compute_tflops: f64, memory_gb: f64) -> (bool, f64) {
        if compute_tflops > Self::MAX_COMPUTE_TFLOPS {
            return (false, 20.0);
        }
        if memory_gb > Self::MAX_MEMORY_GB {
            return (false, 30.0);
        }

        let compute_util = compute_tflops / Self::MAX_COMPUTE_TFLOPS;
        let memory_util = memory_gb / Self::MAX_MEMORY_GB;

        if compute_util <= 0.5 && memory_util <= 0.5 {
            (true, 100.0)
        } else if compute_util <= 0.8 && memory_util <= 0.8 {
            (true, 80.0)
        } else {
            (true, 60.0)
        }
    }

    fn check_thermal(&self, compute_tflops: f64, characteristics: &WorkloadCharacteristics) -> (bool, f64) {
        let thermal_load = compute_tflops * characteristics.thermal_factor;
        let max_thermal_load = 70.0;

        if thermal_load > max_thermal_load {
            return (false, 20.0);
        }
        let score = 100.0 * (1.0 - thermal_load / max_thermal_load);
        (true, score.max(40.0))
    }

    fn check_power(&self, compute_tflops: f64, characteristics: &WorkloadCharacteristics) -> (bool, f64) {
        let estimated_power = compute_tflops * 20.0 * characteristics.power_factor;

        if estimated_power > Self::MAX_POWER_WATTS {
            return (false, 20.0);
        }
        let score = 100.0 * (1.0 - estimated_power / Self::MAX_POWER_WATTS);
        (true, score.max(40.0))
    }

    fn check_latency(
        &self,
        latency_requirement_ms: Option<f64>,
        altitude_km: f64,
        characteristics: &WorkloadCharacteristics,
    ) -> (bool, f64) {
        let latency_ms = match latency_requirement_ms {
            Some(l) => l,
            None => return (true, 100.0),
        };

        let speed_of_light_km_s = 299792.458;
        let min_latency = (2.0 * altitude_km / speed_of_light_km_s) * 1000.0 + 5.0;

        if !characteristics.latency_sensitive {
            return (true, 90.0);
        }
        if latency_ms < min_latency {
            return (false, 10.0);
        }

        let margin = latency_ms - min_latency;
        if margin > 50.0 {
            (true, 100.0)
        } else if margin > 20.0 {
            (true, 80.0)
        } else {
            (true, 60.0)
        }
    }

    fn check_data_transfer(&self, data_transfer_gb: f64) -> (bool, f64) {
        if data_transfer_gb > Self::MAX_DATA_TRANSFER_GB_DAY {
            return (false, 20.0);
        }

        let util = data_transfer_gb / Self::MAX_DATA_TRANSFER_GB_DAY;
        if util <= 0.3 {
            (true, 100.0)
        } else if util <= 0.6 {
            (true, 80.0)
        } else {
            (true, 60.0)
        }
    }

    fn generate_recommendations(
        &self,
        profile: &WorkloadProfile,
        compute_ok: bool,
        thermal_ok: bool,
        power_ok: bool,
        latency_ok: bool,
        data_ok: bool,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !compute_ok {
            recommendations.push("Consider partitioning workload across multiple orbital nodes".to_string());
        }
        if !thermal_ok {
            recommendations.push("Implement duty cycling to manage thermal constraints".to_string());
        }
        if !power_ok {
            recommendations.push("Schedule compute-intensive tasks during solar exposure windows".to_string());
        }
        if !latency_ok {
            recommendations.push("Consider edge caching or predictive pre-computation".to_string());
        }
        if !data_ok {
            recommendations.push("Implement data compression or delta-sync strategies".to_string());
        }

        if matches!(profile.workload_type, WorkloadType::Batch | WorkloadType::Training) {
            recommendations.push("Batch workloads are well-suited for orbital compute".to_string());
        }

        recommendations
    }

    fn estimate_cost_factor(&self, profile: &WorkloadProfile, characteristics: &WorkloadCharacteristics) -> f64 {
        let mut base_factor: f64 = 2.5;

        if characteristics.batch_friendly {
            base_factor *= 0.8;
        }
        if profile.compute_tflops > 50.0 {
            base_factor *= 1.2;
        }
        if profile.data_transfer_gb.unwrap_or(10.0) > 500.0 {
            base_factor *= 1.3;
        }

        (base_factor * 100.0).round() / 100.0
    }
}

impl Default for FeasibilityCalculator {
    fn default() -> Self {
        Self::default_altitude()
    }
}

/// Result of a scenario comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// Orbit altitude in km
    pub altitude_km: f64,
    /// Whether feasible at this altitude
    pub feasible: bool,
    /// Feasibility rating
    pub rating: FeasibilityRating,
    /// Numeric score
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feasibility_calculator() {
        let calculator = FeasibilityCalculator::new(550.0);
        let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0)
            .with_memory_gb(32.0)
            .with_latency_requirement_ms(100.0);

        let result = calculator.analyze(&profile, None);

        assert!(result.feasible);
        assert!(result.score > 50.0);
    }

    #[test]
    fn test_high_compute_infeasible() {
        let calculator = FeasibilityCalculator::new(550.0);
        let profile = WorkloadProfile::new(WorkloadType::Training, 150.0);

        let result = calculator.analyze(&profile, None);

        assert!(!result.compute_feasible);
        assert!(!result.feasible);
    }

    #[test]
    fn test_batch_workload_favorable() {
        let calculator = FeasibilityCalculator::new(550.0);
        let profile = WorkloadProfile::new(WorkloadType::Batch, 20.0);

        let result = calculator.analyze(&profile, None);

        assert!(result.feasible);
        assert!(result.recommendations.iter().any(|r| r.contains("well-suited")));
    }

    #[test]
    fn test_compare_scenarios() {
        let calculator = FeasibilityCalculator::new(550.0);
        let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0);

        let scenarios = calculator.compare_scenarios(&profile, &[400.0, 550.0, 800.0, 1200.0]);

        assert_eq!(scenarios.len(), 4);
        assert_eq!(scenarios[0].altitude_km, 400.0);
    }

    #[test]
    fn test_latency_sensitive_workload() {
        let calculator = FeasibilityCalculator::new(550.0);
        let profile = WorkloadProfile::new(WorkloadType::Streaming, 5.0)
            .with_latency_requirement_ms(2.0); // Too low for orbital

        let result = calculator.analyze(&profile, None);

        assert!(!result.latency_feasible);
    }

    #[test]
    fn test_default_constructor() {
        let calculator = FeasibilityCalculator::default();
        let profile = WorkloadProfile::new(WorkloadType::Analytics, 5.0);

        let result = calculator.analyze(&profile, None);

        assert!(result.feasible);
        assert_eq!(result.constraints.get("orbit_altitude_km"), Some(&550.0));
    }
}
