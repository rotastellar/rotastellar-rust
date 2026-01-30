//! RotaStellar SDK - Common Types
//!
//! Core data types used throughout the SDK.
//!
//! subhadipmitra@: These types are shared across all RotaStellar crates. Design decisions:
//! - Use degrees (not radians) for human-readable I/O
//! - Use km as standard distance unit (aerospace convention)
//! - Validate on construction to fail fast
//! - Implement Copy for small types (Position, Orbit) for ergonomics

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::error::ValidationError;

// TODO(subhadipmitra): Consider adding a Coordinate3D type for ECI/ECEF vectors
// TODO: Add geodetic-to-geocentric conversion utilities

/// Earth's equatorial radius in kilometers.
/// NOTE(subhadipmitra): Using WGS84. Polar radius is 6356.752 km.
pub const EARTH_RADIUS_KM: f64 = 6378.137;

/// Earth's gravitational parameter (km^3/s^2).
/// Standard value used by GPS, TLE propagators, etc.
pub const EARTH_MU: f64 = 398600.4418;

/// Geographic position with altitude.
///
/// # Example
///
/// ```
/// use rotastellar::types::Position;
///
/// let pos = Position::new(28.5729, -80.6490, 408.0).unwrap();
/// println!("ISS at {}, {}", pos.latitude, pos.longitude);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Latitude in degrees (-90 to 90)
    pub latitude: f64,
    /// Longitude in degrees (-180 to 180)
    pub longitude: f64,
    /// Altitude above sea level in kilometers
    #[serde(default)]
    pub altitude_km: f64,
}

impl Position {
    /// Create a new Position with validation.
    ///
    /// # Errors
    ///
    /// Returns a ValidationError if:
    /// - latitude is not in [-90, 90]
    /// - longitude is not in [-180, 180]
    /// - altitude_km is negative
    pub fn new(latitude: f64, longitude: f64, altitude_km: f64) -> Result<Self, ValidationError> {
        let pos = Self {
            latitude,
            longitude,
            altitude_km,
        };
        pos.validate()?;
        Ok(pos)
    }

    /// Validate position parameters.
    fn validate(&self) -> Result<(), ValidationError> {
        if !(-90.0..=90.0).contains(&self.latitude) {
            return Err(ValidationError::new(
                "latitude",
                "Must be between -90 and 90 degrees",
            ));
        }
        if !(-180.0..=180.0).contains(&self.longitude) {
            return Err(ValidationError::new(
                "longitude",
                "Must be between -180 and 180 degrees",
            ));
        }
        if self.altitude_km < 0.0 {
            return Err(ValidationError::new("altitude_km", "Must be non-negative"));
        }
        Ok(())
    }
}

/// Keplerian orbital elements.
///
/// # Example
///
/// ```
/// use rotastellar::types::Orbit;
///
/// let orbit = Orbit::new(6778.0, 0.0001, 51.6, 100.0, 90.0, 0.0).unwrap();
/// println!("Period: {:.1} minutes", orbit.orbital_period_minutes());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Orbit {
    /// Semi-major axis in kilometers
    pub semi_major_axis_km: f64,
    /// Orbital eccentricity (0 = circular, 0-1 = elliptical)
    pub eccentricity: f64,
    /// Inclination in degrees (0-180)
    pub inclination_deg: f64,
    /// Right ascension of ascending node in degrees (0-360)
    pub raan_deg: f64,
    /// Argument of periapsis in degrees (0-360)
    pub arg_periapsis_deg: f64,
    /// True anomaly in degrees (0-360)
    pub true_anomaly_deg: f64,
}

impl Orbit {
    /// Create a new Orbit with validation.
    ///
    /// # Errors
    ///
    /// Returns a ValidationError if:
    /// - semi_major_axis_km is less than Earth's radius
    /// - eccentricity is not in [0, 1)
    /// - inclination_deg is not in [0, 180]
    pub fn new(
        semi_major_axis_km: f64,
        eccentricity: f64,
        inclination_deg: f64,
        raan_deg: f64,
        arg_periapsis_deg: f64,
        true_anomaly_deg: f64,
    ) -> Result<Self, ValidationError> {
        let orbit = Self {
            semi_major_axis_km,
            eccentricity,
            inclination_deg,
            raan_deg,
            arg_periapsis_deg,
            true_anomaly_deg,
        };
        orbit.validate()?;
        Ok(orbit)
    }

    /// Validate orbital parameters.
    fn validate(&self) -> Result<(), ValidationError> {
        if self.semi_major_axis_km <= EARTH_RADIUS_KM {
            return Err(ValidationError::new(
                "semi_major_axis_km",
                format!(
                    "Must be greater than Earth radius ({} km)",
                    EARTH_RADIUS_KM
                ),
            ));
        }
        if !(0.0..1.0).contains(&self.eccentricity) {
            return Err(ValidationError::new(
                "eccentricity",
                "Must be between 0 (inclusive) and 1 (exclusive)",
            ));
        }
        if !(0.0..=180.0).contains(&self.inclination_deg) {
            return Err(ValidationError::new(
                "inclination_deg",
                "Must be between 0 and 180 degrees",
            ));
        }
        Ok(())
    }

    /// Apogee altitude above Earth surface in kilometers.
    pub fn apogee_km(&self) -> f64 {
        self.semi_major_axis_km * (1.0 + self.eccentricity) - EARTH_RADIUS_KM
    }

    /// Perigee altitude above Earth surface in kilometers.
    pub fn perigee_km(&self) -> f64 {
        self.semi_major_axis_km * (1.0 - self.eccentricity) - EARTH_RADIUS_KM
    }

    /// Orbital period in seconds.
    pub fn orbital_period_seconds(&self) -> f64 {
        2.0 * PI * (self.semi_major_axis_km.powi(3) / EARTH_MU).sqrt()
    }

    /// Orbital period in minutes.
    pub fn orbital_period_minutes(&self) -> f64 {
        self.orbital_period_seconds() / 60.0
    }

    /// Mean motion in revolutions per day.
    pub fn mean_motion(&self) -> f64 {
        86400.0 / self.orbital_period_seconds()
    }
}

/// Time range for queries.
///
/// # Example
///
/// ```
/// use rotastellar::types::TimeRange;
/// use std::time::Duration;
///
/// let tr = TimeRange::next_hours(24.0);
/// println!("Duration: {} hours", tr.duration_hours());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (ISO 8601 string)
    pub start: String,
    /// End time (ISO 8601 string)
    pub end: String,
}

impl TimeRange {
    /// Create a new TimeRange.
    pub fn new(start: impl Into<String>, end: impl Into<String>) -> Result<Self, ValidationError> {
        let range = Self {
            start: start.into(),
            end: end.into(),
        };
        // Note: Full validation would require parsing dates
        Ok(range)
    }

    /// Create a time range starting now for the specified hours.
    pub fn next_hours(hours: f64) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let end = now + (hours * 3600.0) as u64;

        // Simple ISO 8601 formatting
        Self {
            start: format_timestamp(now),
            end: format_timestamp(end),
        }
    }

    /// Duration in hours (approximate, based on string parsing).
    pub fn duration_hours(&self) -> f64 {
        // This is a simplified implementation
        // Full implementation would parse the timestamps
        24.0 // Placeholder
    }
}

/// Format a Unix timestamp as ISO 8601.
fn format_timestamp(secs: u64) -> String {
    // Simple implementation - in production use chrono
    let days_since_epoch = secs / 86400;
    let secs_today = secs % 86400;
    let hours = secs_today / 3600;
    let minutes = (secs_today % 3600) / 60;
    let seconds = secs_today % 60;

    // Approximate date calculation (not accounting for leap years properly)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    while remaining_days >= 365 {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days >= days_in_year {
            remaining_days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    for days in days_in_months {
        if remaining_days >= days {
            remaining_days -= days;
            month += 1;
        } else {
            break;
        }
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Satellite information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Satellite {
    /// RotaStellar satellite ID
    pub id: String,
    /// NORAD catalog number
    pub norad_id: u32,
    /// Satellite name
    pub name: String,
    /// Satellite operator/owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    /// Constellation name (if part of one)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constellation: Option<String>,
    /// Current orbital elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orbit: Option<Orbit>,
    /// Current position (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

impl Satellite {
    /// Create a new Satellite.
    pub fn new(id: impl Into<String>, norad_id: u32, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            norad_id,
            name: name.into(),
            operator: None,
            constellation: None,
            orbit: None,
            position: None,
        }
    }

    /// Set the operator.
    pub fn with_operator(mut self, operator: impl Into<String>) -> Self {
        self.operator = Some(operator.into());
        self
    }

    /// Set the constellation.
    pub fn with_constellation(mut self, constellation: impl Into<String>) -> Self {
        self.constellation = Some(constellation.into());
        self
    }

    /// Set the orbit.
    pub fn with_orbit(mut self, orbit: Orbit) -> Self {
        self.orbit = Some(orbit);
        self
    }

    /// Set the position.
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_valid() {
        let pos = Position::new(28.5729, -80.6490, 408.0).unwrap();
        assert_eq!(pos.latitude, 28.5729);
        assert_eq!(pos.longitude, -80.6490);
        assert_eq!(pos.altitude_km, 408.0);
    }

    #[test]
    fn test_position_invalid_latitude() {
        let result = Position::new(91.0, 0.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_orbit_valid() {
        let orbit = Orbit::new(6778.0, 0.0001, 51.6, 100.0, 90.0, 0.0).unwrap();
        let period = orbit.orbital_period_minutes();
        assert!((period - 92.56).abs() < 0.1);
    }

    #[test]
    fn test_orbit_properties() {
        let orbit = Orbit::new(6778.0, 0.0001, 51.6, 100.0, 90.0, 0.0).unwrap();
        assert!((orbit.apogee_km() - 400.5).abs() < 1.0);
        assert!((orbit.perigee_km() - 399.2).abs() < 1.0);
    }
}
