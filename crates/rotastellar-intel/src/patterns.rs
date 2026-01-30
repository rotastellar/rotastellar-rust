//! RotaStellar Intel - Pattern Detection
//!
//! Satellite behavior analysis, anomaly detection, and pattern recognition.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Types of detected patterns/anomalies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// Orbital maneuver detected
    Maneuver,
    /// Altitude increase
    OrbitRaise,
    /// Altitude decrease
    OrbitLower,
    /// Inclination change
    PlaneChange,
    /// Deorbit maneuver
    Deorbit,
    /// Station-keeping burn
    StationKeeping,
    /// Close approach to another object
    ProximityOps,
    /// Docking/berthing approach
    Rendezvous,
    /// Collision avoidance maneuver
    DebrisAvoidance,
    /// Unexpected behavior
    Anomaly,
    /// Loss of attitude control
    Tumbling,
    /// Breakup event
    Fragmentation,
    /// Satellite deployment
    Deployment,
    /// Atmospheric reentry
    Reentry,
}

impl Default for PatternType {
    fn default() -> Self {
        Self::Anomaly
    }
}

impl FromStr for PatternType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "maneuver" => Ok(Self::Maneuver),
            "orbit_raise" => Ok(Self::OrbitRaise),
            "orbit_lower" => Ok(Self::OrbitLower),
            "plane_change" => Ok(Self::PlaneChange),
            "deorbit" => Ok(Self::Deorbit),
            "station_keeping" => Ok(Self::StationKeeping),
            "proximity_ops" => Ok(Self::ProximityOps),
            "rendezvous" => Ok(Self::Rendezvous),
            "debris_avoidance" => Ok(Self::DebrisAvoidance),
            "anomaly" => Ok(Self::Anomaly),
            "tumbling" => Ok(Self::Tumbling),
            "fragmentation" => Ok(Self::Fragmentation),
            "deployment" => Ok(Self::Deployment),
            "reentry" => Ok(Self::Reentry),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Maneuver => "maneuver",
            Self::OrbitRaise => "orbit_raise",
            Self::OrbitLower => "orbit_lower",
            Self::PlaneChange => "plane_change",
            Self::Deorbit => "deorbit",
            Self::StationKeeping => "station_keeping",
            Self::ProximityOps => "proximity_ops",
            Self::Rendezvous => "rendezvous",
            Self::DebrisAvoidance => "debris_avoidance",
            Self::Anomaly => "anomaly",
            Self::Tumbling => "tumbling",
            Self::Fragmentation => "fragmentation",
            Self::Deployment => "deployment",
            Self::Reentry => "reentry",
        };
        write!(f, "{}", s)
    }
}

impl PatternType {
    /// Check if this pattern type is a maneuver.
    pub fn is_maneuver(&self) -> bool {
        matches!(
            self,
            Self::Maneuver
                | Self::OrbitRaise
                | Self::OrbitLower
                | Self::PlaneChange
                | Self::Deorbit
                | Self::StationKeeping
                | Self::DebrisAvoidance
        )
    }

    /// Check if this pattern type is an anomaly.
    pub fn is_anomaly(&self) -> bool {
        matches!(self, Self::Anomaly | Self::Tumbling | Self::Fragmentation)
    }
}

/// Confidence level of pattern detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// Low confidence, needs more data
    Uncertain,
    /// Moderate confidence
    Possible,
    /// Good confidence
    Likely,
    /// High confidence, multiple data sources
    Confirmed,
}

impl Default for ConfidenceLevel {
    fn default() -> Self {
        Self::Uncertain
    }
}

impl FromStr for ConfidenceLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "confirmed" => Ok(Self::Confirmed),
            "likely" => Ok(Self::Likely),
            "possible" => Ok(Self::Possible),
            "uncertain" => Ok(Self::Uncertain),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Confirmed => "confirmed",
            Self::Likely => "likely",
            Self::Possible => "possible",
            Self::Uncertain => "uncertain",
        };
        write!(f, "{}", s)
    }
}

/// A detected pattern or anomaly in satellite behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern ID
    pub id: String,
    /// Satellite that exhibited the pattern
    pub satellite_id: String,
    /// Satellite name
    pub satellite_name: String,
    /// Type of pattern detected
    pub pattern_type: PatternType,
    /// When the pattern was detected
    pub detected_at: DateTime<Utc>,
    /// When the pattern/event started
    pub start_time: DateTime<Utc>,
    /// When the pattern/event ended (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    /// Detection confidence level
    #[serde(default)]
    pub confidence: ConfidenceLevel,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Estimated delta-v if maneuver (m/s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_v_m_s: Option<f64>,
    /// Change in altitude (km)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude_change_km: Option<f64>,
    /// Change in inclination (degrees)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inclination_change_deg: Option<f64>,
    /// Additional pattern-specific details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl DetectedPattern {
    /// Create a new detected pattern.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        satellite_id: impl Into<String>,
        satellite_name: impl Into<String>,
        pattern_type: PatternType,
        detected_at: DateTime<Utc>,
        start_time: DateTime<Utc>,
        confidence: ConfidenceLevel,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            satellite_id: satellite_id.into(),
            satellite_name: satellite_name.into(),
            pattern_type,
            detected_at,
            start_time,
            end_time: None,
            confidence,
            description: description.into(),
            delta_v_m_s: None,
            altitude_change_km: None,
            inclination_change_deg: None,
            details: None,
        }
    }

    /// Check if this pattern is a maneuver.
    pub fn is_maneuver(&self) -> bool {
        self.pattern_type.is_maneuver()
    }

    /// Check if this pattern is an anomaly.
    pub fn is_anomaly(&self) -> bool {
        self.pattern_type.is_anomaly()
    }

    /// Set the delta-v value.
    pub fn with_delta_v(mut self, delta_v_m_s: f64) -> Self {
        self.delta_v_m_s = Some(delta_v_m_s);
        self
    }

    /// Set the altitude change.
    pub fn with_altitude_change(mut self, altitude_change_km: f64) -> Self {
        self.altitude_change_km = Some(altitude_change_km);
        self
    }

    /// Set the end time.
    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = Some(end_time);
        self
    }
}

/// Pattern detector for satellite behavior analysis.
///
/// # Example
///
/// ```ignore
/// use rotastellar_intel::{PatternDetector, PatternType, ConfidenceLevel};
///
/// let detector = PatternDetector::new();
///
/// // Get maneuvers
/// let maneuvers = detector.get_maneuvers(Some("starlink-1234"), Some(168.0));
///
/// // Get anomalies
/// let anomalies = detector.get_anomalies(None, Some(24.0));
/// ```
pub struct PatternDetector {
    /// Detected patterns
    patterns: Vec<DetectedPattern>,
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector {
    /// Create a new pattern detector.
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Add a detected pattern.
    pub fn add_pattern(&mut self, pattern: DetectedPattern) {
        self.patterns.push(pattern);
    }

    /// Get all patterns.
    pub fn get_patterns(&self) -> &[DetectedPattern] {
        &self.patterns
    }

    /// Get patterns with filtering.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Filter by satellite
    /// * `pattern_types` - Filter by pattern types
    /// * `confidence_min` - Minimum confidence level
    pub fn get_filtered_patterns(
        &self,
        satellite_id: Option<&str>,
        pattern_types: Option<&[PatternType]>,
        confidence_min: Option<ConfidenceLevel>,
    ) -> Vec<&DetectedPattern> {
        self.patterns
            .iter()
            .filter(|p| {
                // Filter by satellite
                if let Some(sat_id) = satellite_id {
                    if p.satellite_id != sat_id {
                        return false;
                    }
                }

                // Filter by pattern types
                if let Some(types) = pattern_types {
                    if !types.contains(&p.pattern_type) {
                        return false;
                    }
                }

                // Filter by confidence
                if let Some(min_conf) = confidence_min {
                    if p.confidence < min_conf {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get maneuvers.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Filter by satellite
    /// * `_hours` - Time window in hours (default: 168 = 7 days)
    pub fn get_maneuvers(
        &self,
        satellite_id: Option<&str>,
        _hours: Option<f64>,
    ) -> Vec<&DetectedPattern> {
        let maneuver_types = [
            PatternType::Maneuver,
            PatternType::OrbitRaise,
            PatternType::OrbitLower,
            PatternType::PlaneChange,
            PatternType::Deorbit,
            PatternType::StationKeeping,
            PatternType::DebrisAvoidance,
        ];

        self.get_filtered_patterns(satellite_id, Some(&maneuver_types), None)
    }

    /// Get anomalies.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Filter by satellite
    /// * `_hours` - Time window in hours (default: 24)
    pub fn get_anomalies(
        &self,
        satellite_id: Option<&str>,
        _hours: Option<f64>,
    ) -> Vec<&DetectedPattern> {
        let anomaly_types = [
            PatternType::Anomaly,
            PatternType::Tumbling,
            PatternType::Fragmentation,
        ];

        self.get_filtered_patterns(satellite_id, Some(&anomaly_types), None)
    }

    /// Get proximity events.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Filter by satellite
    /// * `_hours` - Time window in hours (default: 168 = 7 days)
    pub fn get_proximity_events(
        &self,
        satellite_id: Option<&str>,
        _hours: Option<f64>,
    ) -> Vec<&DetectedPattern> {
        let proximity_types = [PatternType::ProximityOps, PatternType::Rendezvous];

        self.get_filtered_patterns(satellite_id, Some(&proximity_types), None)
    }

    /// Analyze satellite behavior.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Satellite to analyze
    /// * `_hours` - Analysis window in hours (default: 720 = 30 days)
    pub fn analyze_behavior(&self, satellite_id: &str, _hours: Option<f64>) -> BehaviorAnalysis {
        let patterns: Vec<_> = self
            .patterns
            .iter()
            .filter(|p| p.satellite_id == satellite_id)
            .collect();

        // Categorize patterns
        let mut by_type: HashMap<PatternType, Vec<&DetectedPattern>> = HashMap::new();
        for p in &patterns {
            by_type.entry(p.pattern_type).or_default().push(p);
        }

        // Count maneuvers and anomalies
        let maneuvers: Vec<_> = patterns.iter().filter(|p| p.is_maneuver()).collect();
        let anomalies: Vec<_> = patterns.iter().filter(|p| p.is_anomaly()).collect();

        // Calculate total delta-v
        let total_delta_v: f64 = maneuvers
            .iter()
            .filter_map(|p| p.delta_v_m_s)
            .sum();

        BehaviorAnalysis {
            satellite_id: satellite_id.to_string(),
            total_patterns: patterns.len(),
            patterns_by_type: by_type
                .iter()
                .map(|(k, v)| (k.to_string(), v.len()))
                .collect(),
            maneuver_count: maneuvers.len(),
            anomaly_count: anomalies.len(),
            total_delta_v_m_s: total_delta_v,
            has_anomalies: !anomalies.is_empty(),
        }
    }
}

/// Behavior analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAnalysis {
    /// Satellite ID
    pub satellite_id: String,
    /// Total number of patterns
    pub total_patterns: usize,
    /// Patterns by type
    pub patterns_by_type: HashMap<String, usize>,
    /// Number of maneuvers
    pub maneuver_count: usize,
    /// Number of anomalies
    pub anomaly_count: usize,
    /// Total delta-v in m/s
    pub total_delta_v_m_s: f64,
    /// Whether anomalies were detected
    pub has_anomalies: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_pattern_type_from_str() {
        assert_eq!(PatternType::from_str("maneuver"), Ok(PatternType::Maneuver));
        assert_eq!(PatternType::from_str("ORBIT_RAISE"), Ok(PatternType::OrbitRaise));
        assert_eq!(PatternType::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_pattern_type_is_maneuver() {
        assert!(PatternType::Maneuver.is_maneuver());
        assert!(PatternType::OrbitRaise.is_maneuver());
        assert!(!PatternType::Anomaly.is_maneuver());
        assert!(!PatternType::Tumbling.is_maneuver());
    }

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::Uncertain < ConfidenceLevel::Possible);
        assert!(ConfidenceLevel::Possible < ConfidenceLevel::Likely);
        assert!(ConfidenceLevel::Likely < ConfidenceLevel::Confirmed);
    }

    #[test]
    fn test_detector() {
        let mut detector = PatternDetector::new();
        let now = Utc::now();

        detector.add_pattern(
            DetectedPattern::new(
                "pattern-1",
                "sat-1",
                "Satellite 1",
                PatternType::OrbitRaise,
                now,
                now - Duration::hours(1),
                ConfidenceLevel::Confirmed,
                "Orbit raising maneuver detected",
            )
            .with_delta_v(10.5),
        );

        detector.add_pattern(DetectedPattern::new(
            "pattern-2",
            "sat-1",
            "Satellite 1",
            PatternType::Anomaly,
            now,
            now - Duration::hours(2),
            ConfidenceLevel::Likely,
            "Anomalous behavior detected",
        ));

        let analysis = detector.analyze_behavior("sat-1", None);
        assert_eq!(analysis.total_patterns, 2);
        assert_eq!(analysis.maneuver_count, 1);
        assert_eq!(analysis.anomaly_count, 1);
        assert!((analysis.total_delta_v_m_s - 10.5).abs() < 0.01);
        assert!(analysis.has_anomalies);
    }
}
