//! RotaStellar Intel - Conjunction Analysis
//!
//! Space object collision probability and close approach analysis.
//!
//! subhadipmitra@: Conjunction assessment is critical for space safety.
//! We use the Pc (probability of collision) framework from NASA's CARA team.
//!
//! Risk thresholds (industry standard):
//! - 1e-4: Red threshold - maneuver decision required
//! - 1e-5: Yellow threshold - enhanced monitoring
//! - 1e-7: Green threshold - routine tracking
//!
//! Miss distance alone is NOT sufficient - a 1km miss with high covariance
//! uncertainty might be riskier than a 100m miss with low uncertainty.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// TODO(subhadipmitra): Add Monte Carlo Pc estimation
// TODO: Integrate with Space-Track CDM (Conjunction Data Messages)
// FIXME: Current Pc calculation assumes spherical covariance (simplification)

/// Conjunction risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Immediate action required (P > 1e-4)
    Critical,
    /// Close monitoring needed (P > 1e-5)
    High,
    /// Standard monitoring (P > 1e-6)
    Medium,
    /// Routine tracking (P > 1e-7)
    Low,
    /// No action needed (P <= 1e-7)
    Negligible,
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::Low
    }
}

impl FromStr for RiskLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            "negligible" => Ok(Self::Negligible),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
            Self::Negligible => write!(f, "negligible"),
        }
    }
}

/// A conjunction (close approach) between two space objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjunction {
    /// Unique conjunction ID
    pub id: String,
    /// Primary satellite ID
    pub primary_id: String,
    /// Primary satellite name
    pub primary_name: String,
    /// Secondary object ID (satellite or debris)
    pub secondary_id: String,
    /// Secondary object name
    pub secondary_name: String,
    /// Time of Closest Approach
    pub tca: DateTime<Utc>,
    /// Predicted miss distance in km
    pub miss_distance_km: f64,
    /// Radial component of miss distance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miss_distance_radial_km: Option<f64>,
    /// In-track component of miss distance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miss_distance_in_track_km: Option<f64>,
    /// Cross-track component of miss distance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miss_distance_cross_track_km: Option<f64>,
    /// Relative velocity at TCA in km/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_velocity_km_s: Option<f64>,
    /// Probability of collision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision_probability: Option<f64>,
    /// Risk classification
    #[serde(default)]
    pub risk_level: RiskLevel,
    /// When this conjunction was identified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Conjunction {
    /// Create a new conjunction.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        primary_id: impl Into<String>,
        primary_name: impl Into<String>,
        secondary_id: impl Into<String>,
        secondary_name: impl Into<String>,
        tca: DateTime<Utc>,
        miss_distance_km: f64,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            id: id.into(),
            primary_id: primary_id.into(),
            primary_name: primary_name.into(),
            secondary_id: secondary_id.into(),
            secondary_name: secondary_name.into(),
            tca,
            miss_distance_km,
            miss_distance_radial_km: None,
            miss_distance_in_track_km: None,
            miss_distance_cross_track_km: None,
            relative_velocity_km_s: None,
            collision_probability: None,
            risk_level,
            created_at: None,
            updated_at: None,
        }
    }

    /// Check if this conjunction is critical risk.
    pub fn is_critical(&self) -> bool {
        self.risk_level == RiskLevel::Critical
    }

    /// Check if this conjunction is high risk or above.
    pub fn is_high_risk(&self) -> bool {
        matches!(self.risk_level, RiskLevel::Critical | RiskLevel::High)
    }

    /// Get time to TCA in hours (negative if past).
    pub fn time_to_tca_hours(&self) -> f64 {
        let now = Utc::now();
        (self.tca - now).num_milliseconds() as f64 / (1000.0 * 3600.0)
    }
}

/// Recommended maneuver to avoid a conjunction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManeuverRecommendation {
    /// ID of the conjunction to avoid
    pub conjunction_id: String,
    /// Recommended maneuver execution time
    pub maneuver_time: DateTime<Utc>,
    /// Required delta-v in m/s
    pub delta_v_m_s: f64,
    /// Maneuver direction (radial, in-track, cross-track)
    pub direction: String,
    /// Expected miss distance after maneuver
    pub post_maneuver_miss_km: f64,
    /// Expected collision probability after maneuver
    pub post_maneuver_probability: f64,
    /// Estimated fuel required (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_required_kg: Option<f64>,
    /// Confidence level of the recommendation
    pub confidence: f64,
}

/// Conjunction analyzer for collision risk assessment.
///
/// # Example
///
/// ```ignore
/// use rotastellar_intel::{ConjunctionAnalyzer, RiskLevel};
///
/// let analyzer = ConjunctionAnalyzer::new();
///
/// // Analyze risk for a satellite
/// let analysis = analyzer.analyze_risk("starlink-1234", Some(168.0));
/// println!("Critical conjunctions: {}", analysis.critical_count);
/// ```
pub struct ConjunctionAnalyzer {
    /// Cached conjunctions
    conjunctions: Vec<Conjunction>,
}

impl Default for ConjunctionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConjunctionAnalyzer {
    /// Create a new conjunction analyzer.
    pub fn new() -> Self {
        Self {
            conjunctions: Vec::new(),
        }
    }

    /// Add a conjunction to the analyzer.
    pub fn add_conjunction(&mut self, conjunction: Conjunction) {
        self.conjunctions.push(conjunction);
    }

    /// Get all conjunctions.
    pub fn get_conjunctions(&self) -> &[Conjunction] {
        &self.conjunctions
    }

    /// Get conjunctions for a specific satellite.
    pub fn get_conjunctions_for_satellite(&self, satellite_id: &str) -> Vec<&Conjunction> {
        self.conjunctions
            .iter()
            .filter(|c| c.primary_id == satellite_id || c.secondary_id == satellite_id)
            .collect()
    }

    /// Get high-risk conjunctions.
    pub fn get_high_risk_conjunctions(&self) -> Vec<&Conjunction> {
        let mut high_risk: Vec<_> = self
            .conjunctions
            .iter()
            .filter(|c| c.is_high_risk())
            .collect();

        // Sort by: CRITICAL first, then by miss distance
        high_risk.sort_by(|a, b| {
            match (a.risk_level == RiskLevel::Critical, b.risk_level == RiskLevel::Critical) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.miss_distance_km.partial_cmp(&b.miss_distance_km).unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        high_risk
    }

    /// Analyze risk for a satellite.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Satellite to analyze
    /// * `hours` - Analysis window in hours (default: 168 = 7 days)
    ///
    /// # Returns
    ///
    /// Risk analysis summary.
    pub fn analyze_risk(&self, satellite_id: &str, _hours: Option<f64>) -> RiskAnalysis {
        let conjunctions = self.get_conjunctions_for_satellite(satellite_id);

        // Count by risk level
        let mut by_risk_level = std::collections::HashMap::new();
        for risk in [
            RiskLevel::Critical,
            RiskLevel::High,
            RiskLevel::Medium,
            RiskLevel::Low,
            RiskLevel::Negligible,
        ] {
            by_risk_level.insert(risk, 0usize);
        }

        for c in &conjunctions {
            *by_risk_level.entry(c.risk_level).or_insert(0) += 1;
        }

        // Find closest approach
        let closest = conjunctions.iter().min_by(|a, b| {
            a.miss_distance_km
                .partial_cmp(&b.miss_distance_km)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let critical_count = by_risk_level.get(&RiskLevel::Critical).copied().unwrap_or(0);
        let high_risk_count = by_risk_level.get(&RiskLevel::High).copied().unwrap_or(0);

        RiskAnalysis {
            satellite_id: satellite_id.to_string(),
            total_conjunctions: conjunctions.len(),
            critical_count,
            high_risk_count,
            closest_approach_km: closest.map(|c| c.miss_distance_km),
            closest_approach_tca: closest.map(|c| c.tca),
            requires_attention: critical_count > 0 || high_risk_count > 0,
        }
    }
}

/// Risk analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysis {
    /// Satellite ID
    pub satellite_id: String,
    /// Total number of conjunctions
    pub total_conjunctions: usize,
    /// Number of critical conjunctions
    pub critical_count: usize,
    /// Number of high-risk conjunctions
    pub high_risk_count: usize,
    /// Closest approach distance
    pub closest_approach_km: Option<f64>,
    /// Time of closest approach
    pub closest_approach_tca: Option<DateTime<Utc>>,
    /// Whether attention is required
    pub requires_attention: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_risk_level_from_str() {
        assert_eq!(RiskLevel::from_str("critical"), Ok(RiskLevel::Critical));
        assert_eq!(RiskLevel::from_str("HIGH"), Ok(RiskLevel::High));
        assert_eq!(RiskLevel::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_conjunction_is_high_risk() {
        let tca = Utc::now() + Duration::hours(24);

        let critical = Conjunction::new(
            "conj-1", "sat-1", "Satellite 1", "sat-2", "Satellite 2",
            tca, 0.5, RiskLevel::Critical,
        );
        assert!(critical.is_critical());
        assert!(critical.is_high_risk());

        let high = Conjunction::new(
            "conj-2", "sat-1", "Satellite 1", "sat-3", "Satellite 3",
            tca, 1.0, RiskLevel::High,
        );
        assert!(!high.is_critical());
        assert!(high.is_high_risk());

        let low = Conjunction::new(
            "conj-3", "sat-1", "Satellite 1", "sat-4", "Satellite 4",
            tca, 5.0, RiskLevel::Low,
        );
        assert!(!low.is_high_risk());
    }

    #[test]
    fn test_analyzer() {
        let mut analyzer = ConjunctionAnalyzer::new();
        let tca = Utc::now() + Duration::hours(24);

        analyzer.add_conjunction(Conjunction::new(
            "conj-1", "sat-1", "Satellite 1", "sat-2", "Satellite 2",
            tca, 0.5, RiskLevel::Critical,
        ));
        analyzer.add_conjunction(Conjunction::new(
            "conj-2", "sat-1", "Satellite 1", "sat-3", "Satellite 3",
            tca, 2.0, RiskLevel::Medium,
        ));

        let analysis = analyzer.analyze_risk("sat-1", None);
        assert_eq!(analysis.total_conjunctions, 2);
        assert_eq!(analysis.critical_count, 1);
        assert!(analysis.requires_attention);
        assert!((analysis.closest_approach_km.unwrap() - 0.5).abs() < 0.01);
    }
}
