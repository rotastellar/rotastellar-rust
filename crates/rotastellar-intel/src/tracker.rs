//! RotaStellar Intel - Satellite Tracker
//!
//! Real-time satellite tracking and position calculations.

use chrono::{DateTime, Duration, Utc};
use rotastellar::{Position, ValidationError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::TLE;

/// Ground station for satellite pass calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundStation {
    /// Station name/identifier
    pub name: String,
    /// Geographic position of the station
    pub position: Position,
    /// Minimum elevation angle for visibility (default: 10°)
    pub min_elevation_deg: f64,
}

impl GroundStation {
    /// Create a new ground station.
    ///
    /// # Arguments
    ///
    /// * `name` - Station name/identifier
    /// * `position` - Geographic position
    /// * `min_elevation_deg` - Minimum elevation angle (default: 10°)
    pub fn new(name: impl Into<String>, position: Position, min_elevation_deg: Option<f64>) -> Self {
        Self {
            name: name.into(),
            position,
            min_elevation_deg: min_elevation_deg.unwrap_or(10.0),
        }
    }
}

/// A satellite pass over a ground station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatellitePass {
    /// Satellite identifier
    pub satellite_id: String,
    /// Ground station name
    pub ground_station: String,
    /// Acquisition of Signal (rise time)
    pub aos: DateTime<Utc>,
    /// Loss of Signal (set time)
    pub los: DateTime<Utc>,
    /// Time of Closest Approach (max elevation)
    pub tca: DateTime<Utc>,
    /// Maximum elevation angle
    pub max_elevation_deg: f64,
    /// Azimuth at AOS
    pub aos_azimuth_deg: f64,
    /// Azimuth at LOS
    pub los_azimuth_deg: f64,
}

impl SatellitePass {
    /// Duration of the pass in seconds.
    pub fn duration_seconds(&self) -> f64 {
        (self.los - self.aos).num_milliseconds() as f64 / 1000.0
    }

    /// Duration of the pass in minutes.
    pub fn duration_minutes(&self) -> f64 {
        self.duration_seconds() / 60.0
    }
}

/// Satellite information (simplified version for tracking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedSatelliteInfo {
    /// Satellite ID
    pub id: String,
    /// NORAD catalog number
    pub norad_id: u32,
    /// Satellite name
    pub name: String,
    /// Operator/owner
    pub operator: Option<String>,
    /// Constellation name
    pub constellation: Option<String>,
}

/// Real-time satellite tracker.
///
/// Track satellites, calculate positions, and predict passes over ground stations.
///
/// # Example
///
/// ```ignore
/// use rotastellar_intel::{Tracker, GroundStation, TLE};
/// use rotastellar::Position;
///
/// let tracker = Tracker::new();
///
/// // Parse a TLE
/// let tle_lines = vec![
///     "ISS (ZARYA)".to_string(),
///     "1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999".to_string(),
///     "2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818".to_string(),
/// ];
/// let tle = TLE::parse(&tle_lines).unwrap();
///
/// // Get current position
/// let pos = tle.propagate(chrono::Utc::now()).unwrap();
/// println!("ISS at {:.2}, {:.2}", pos.latitude, pos.longitude);
/// ```
pub struct Tracker {
    /// Satellite info cache
    satellite_cache: HashMap<String, TrackedSatelliteInfo>,
    /// TLE cache
    tle_cache: HashMap<String, TLE>,
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            satellite_cache: HashMap::new(),
            tle_cache: HashMap::new(),
        }
    }

    /// Add a TLE to the cache for tracking.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Identifier for the satellite
    /// * `tle` - TLE data
    pub fn add_tle(&mut self, satellite_id: impl Into<String>, tle: TLE) {
        let id = satellite_id.into();
        self.satellite_cache.insert(
            id.clone(),
            TrackedSatelliteInfo {
                id: id.clone(),
                norad_id: tle.norad_id,
                name: tle.name.clone(),
                operator: None,
                constellation: None,
            },
        );
        self.tle_cache.insert(id, tle);
    }

    /// Get the TLE for a satellite.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Satellite identifier
    ///
    /// # Returns
    ///
    /// TLE if found in cache.
    pub fn get_tle(&self, satellite_id: &str) -> Option<&TLE> {
        self.tle_cache.get(satellite_id)
    }

    /// Get satellite position at a specific time.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Satellite identifier
    /// * `at_time` - Target time (default: now)
    ///
    /// # Returns
    ///
    /// Position at the specified time.
    ///
    /// # Errors
    ///
    /// Returns an error if the satellite is not found or propagation fails.
    pub fn get_position(
        &self,
        satellite_id: &str,
        at_time: Option<DateTime<Utc>>,
    ) -> Result<Position, ValidationError> {
        let tle = self.tle_cache.get(satellite_id).ok_or_else(|| {
            ValidationError::new("satellite_id", format!("Satellite not found: {}", satellite_id))
        })?;

        let time = at_time.unwrap_or_else(Utc::now);
        tle.propagate(time)
    }

    /// Get satellite positions over a time range.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Satellite identifier
    /// * `start` - Start time
    /// * `end` - End time
    /// * `step_seconds` - Time step between positions (default: 60)
    ///
    /// # Returns
    ///
    /// Vector of (time, position) tuples.
    pub fn get_positions(
        &self,
        satellite_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        step_seconds: Option<i64>,
    ) -> Result<Vec<(DateTime<Utc>, Position)>, ValidationError> {
        let tle = self.tle_cache.get(satellite_id).ok_or_else(|| {
            ValidationError::new("satellite_id", format!("Satellite not found: {}", satellite_id))
        })?;

        let step = Duration::seconds(step_seconds.unwrap_or(60));
        let mut positions = Vec::new();
        let mut current = start;

        while current <= end {
            if let Ok(pos) = tle.propagate(current) {
                positions.push((current, pos));
            }
            current = current + step;
        }

        Ok(positions)
    }

    /// Predict satellite passes over a ground station.
    ///
    /// Note: This is a placeholder. Real implementation would use
    /// SGP4 propagation for accurate pass predictions.
    ///
    /// # Arguments
    ///
    /// * `satellite_id` - Satellite identifier
    /// * `ground_station` - Ground station
    /// * `hours` - Time window in hours (default: 24)
    ///
    /// # Returns
    ///
    /// Vector of predicted passes.
    pub fn predict_passes(
        &self,
        _satellite_id: &str,
        _ground_station: &GroundStation,
        _hours: Option<f64>,
    ) -> Vec<SatellitePass> {
        // Placeholder - would need SGP4 propagation for real implementation
        Vec::new()
    }

    /// List all tracked satellites.
    pub fn list_satellites(&self) -> Vec<&TrackedSatelliteInfo> {
        self.satellite_cache.values().collect()
    }

    /// Get satellite info.
    pub fn get_satellite_info(&self, satellite_id: &str) -> Option<&TrackedSatelliteInfo> {
        self.satellite_cache.get(satellite_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISS_TLE: &str = r#"ISS (ZARYA)
1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999
2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818"#;

    #[test]
    fn test_tracker_add_tle() {
        let mut tracker = Tracker::new();
        let lines: Vec<String> = ISS_TLE.lines().map(|s| s.to_string()).collect();
        let tle = TLE::parse(&lines).unwrap();

        tracker.add_tle("ISS", tle);

        assert!(tracker.get_tle("ISS").is_some());
        assert!(tracker.get_satellite_info("ISS").is_some());
    }

    #[test]
    fn test_tracker_get_position() {
        let mut tracker = Tracker::new();
        let lines: Vec<String> = ISS_TLE.lines().map(|s| s.to_string()).collect();
        let tle = TLE::parse(&lines).unwrap();
        let epoch = tle.epoch();

        tracker.add_tle("ISS", tle);

        let pos = tracker.get_position("ISS", Some(epoch)).unwrap();
        assert!(pos.latitude.abs() <= 90.0);
        assert!(pos.longitude.abs() <= 180.0);
    }

    #[test]
    fn test_ground_station() {
        let station = GroundStation::new(
            "Test Station",
            Position::new(40.7128, -74.0060, 0.0).unwrap(),
            Some(5.0),
        );
        assert_eq!(station.name, "Test Station");
        assert_eq!(station.min_elevation_deg, 5.0);
    }
}
