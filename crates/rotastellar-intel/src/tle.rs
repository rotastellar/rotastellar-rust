//! RotaStellar Intel - TLE Parsing and Propagation
//!
//! Two-Line Element set parsing and orbit propagation.
//!
//! subhadipmitra@: TLEs are the de facto standard for satellite orbit data,
//! published by Space-Track (18th Space Defense Squadron). Key caveats:
//! - TLEs degrade over time (~1km error after a few days for LEO)
//! - They're mean elements, not osculating - direct Keplerian conversion is wrong
//! - Must use SGP4/SDP4 for propagation, not simple two-body mechanics
//!
//! For precision work (rendezvous, formation flying), use ephemeris data instead.

use chrono::{DateTime, TimeZone, Utc};
use rotastellar::{Orbit, Position, ValidationError, EARTH_MU, EARTH_RADIUS_KM};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

// TODO(subhadipmitra): Add support for OMM (CCSDS Orbit Mean-elements Message) format
// TODO: Implement SDP4 for deep space objects (period > 225 min)
// NOTE: Using AFSPC compatibility mode for SGP4 constants

/// Minutes per day
const MINUTES_PER_DAY: f64 = 1440.0;
/// Seconds per day
const SECONDS_PER_DAY: f64 = 86400.0;

/// Two-Line Element set for satellite orbit determination.
///
/// A TLE contains orbital elements that describe a satellite's orbit at a
/// specific epoch time.
///
/// # Example
///
/// ```
/// use rotastellar_intel::TLE;
///
/// let tle_lines = vec![
///     "ISS (ZARYA)".to_string(),
///     "1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999".to_string(),
///     "2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818".to_string(),
/// ];
/// let tle = TLE::parse(&tle_lines).unwrap();
/// println!("ISS inclination: {}°", tle.inclination);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TLE {
    /// Satellite name (line 0)
    pub name: String,
    /// NORAD catalog number
    pub norad_id: u32,
    /// Classification (U=unclassified, C=classified, S=secret)
    pub classification: char,
    /// International designator (launch year, number, piece)
    pub intl_designator: String,
    /// Epoch year (2-digit)
    pub epoch_year: u16,
    /// Epoch day of year (fractional)
    pub epoch_day: f64,
    /// First derivative of mean motion (rev/day^2)
    pub mean_motion_dot: f64,
    /// Second derivative of mean motion (rev/day^3)
    pub mean_motion_ddot: f64,
    /// BSTAR drag term
    pub bstar: f64,
    /// Element set type
    pub element_set_type: u8,
    /// Element set number
    pub element_number: u16,
    /// Inclination in degrees
    pub inclination: f64,
    /// Right ascension of ascending node in degrees
    pub raan: f64,
    /// Eccentricity
    pub eccentricity: f64,
    /// Argument of perigee in degrees
    pub arg_perigee: f64,
    /// Mean anomaly in degrees
    pub mean_anomaly: f64,
    /// Mean motion in revolutions per day
    pub mean_motion: f64,
    /// Revolution number at epoch
    pub rev_number: u32,
}

impl TLE {
    /// Parse a TLE from its text representation.
    ///
    /// # Arguments
    ///
    /// * `lines` - Slice of 2 or 3 strings (name optional, then line 1, line 2)
    ///
    /// # Errors
    ///
    /// Returns a ValidationError if the TLE format is invalid.
    pub fn parse(lines: &[String]) -> Result<Self, ValidationError> {
        let (name, line1, line2) = match lines.len() {
            2 => ("UNKNOWN".to_string(), &lines[0], &lines[1]),
            3 => (lines[0].trim().to_string(), &lines[1], &lines[2]),
            _ => {
                return Err(ValidationError::new(
                    "lines",
                    "TLE must have 2 or 3 lines",
                ))
            }
        };

        // Validate line numbers
        if !line1.starts_with("1 ") {
            return Err(ValidationError::new("line1", "Line 1 must start with '1 '"));
        }
        if !line2.starts_with("2 ") {
            return Err(ValidationError::new("line2", "Line 2 must start with '2 '"));
        }

        // Ensure lines are long enough
        if line1.len() < 69 || line2.len() < 69 {
            return Err(ValidationError::new("tle", "TLE lines too short"));
        }

        // Parse line 1
        let norad_id = line1[2..7]
            .trim()
            .parse::<u32>()
            .map_err(|_| ValidationError::new("norad_id", "Invalid NORAD ID"))?;
        let classification = line1.chars().nth(7).unwrap_or('U');
        let intl_designator = line1[9..17].trim().to_string();
        let epoch_year = line1[18..20]
            .trim()
            .parse::<u16>()
            .map_err(|_| ValidationError::new("epoch_year", "Invalid epoch year"))?;
        let epoch_day = line1[20..32]
            .trim()
            .parse::<f64>()
            .map_err(|_| ValidationError::new("epoch_day", "Invalid epoch day"))?;
        let mean_motion_dot = line1[33..43]
            .trim()
            .parse::<f64>()
            .unwrap_or(0.0);

        // Parse mean_motion_ddot (scientific notation without 'E')
        let mean_motion_ddot = parse_tle_scientific(&line1[44..52]);

        // Parse BSTAR (scientific notation without 'E')
        let bstar = parse_tle_scientific(&line1[53..61]);

        let element_set_type = line1
            .chars()
            .nth(62)
            .and_then(|c| c.to_digit(10))
            .unwrap_or(0) as u8;
        let element_number = line1[64..68]
            .trim()
            .parse::<u16>()
            .unwrap_or(0);

        // Parse line 2
        let inclination = line2[8..16]
            .trim()
            .parse::<f64>()
            .map_err(|_| ValidationError::new("inclination", "Invalid inclination"))?;
        let raan = line2[17..25]
            .trim()
            .parse::<f64>()
            .map_err(|_| ValidationError::new("raan", "Invalid RAAN"))?;
        let eccentricity = format!("0.{}", line2[26..33].trim())
            .parse::<f64>()
            .map_err(|_| ValidationError::new("eccentricity", "Invalid eccentricity"))?;
        let arg_perigee = line2[34..42]
            .trim()
            .parse::<f64>()
            .map_err(|_| ValidationError::new("arg_perigee", "Invalid argument of perigee"))?;
        let mean_anomaly = line2[43..51]
            .trim()
            .parse::<f64>()
            .map_err(|_| ValidationError::new("mean_anomaly", "Invalid mean anomaly"))?;
        let mean_motion = line2[52..63]
            .trim()
            .parse::<f64>()
            .map_err(|_| ValidationError::new("mean_motion", "Invalid mean motion"))?;
        let rev_number = line2[63..68]
            .trim()
            .parse::<u32>()
            .unwrap_or(0);

        Ok(TLE {
            name,
            norad_id,
            classification,
            intl_designator,
            epoch_year,
            epoch_day,
            mean_motion_dot,
            mean_motion_ddot,
            bstar,
            element_set_type,
            element_number,
            inclination,
            raan,
            eccentricity,
            arg_perigee,
            mean_anomaly,
            mean_motion,
            rev_number,
        })
    }

    /// Get the epoch as a DateTime<Utc>.
    pub fn epoch(&self) -> DateTime<Utc> {
        // Convert 2-digit year to 4-digit
        let year = if self.epoch_year < 57 {
            2000 + self.epoch_year as i32
        } else {
            1900 + self.epoch_year as i32
        };

        // Convert day of year to datetime
        let jan1 = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
        let days_offset = chrono::Duration::milliseconds(
            ((self.epoch_day - 1.0) * 24.0 * 60.0 * 60.0 * 1000.0) as i64,
        );
        jan1 + days_offset
    }

    /// Calculate semi-major axis from mean motion.
    pub fn semi_major_axis_km(&self) -> f64 {
        // n = sqrt(mu / a^3), so a = (mu / n^2)^(1/3)
        let n_rad_per_sec = self.mean_motion * 2.0 * PI / SECONDS_PER_DAY;
        (EARTH_MU / (n_rad_per_sec * n_rad_per_sec)).powf(1.0 / 3.0)
    }

    /// Calculate orbital period in minutes.
    pub fn orbital_period_minutes(&self) -> f64 {
        MINUTES_PER_DAY / self.mean_motion
    }

    /// Calculate apogee altitude in km.
    pub fn apogee_km(&self) -> f64 {
        let a = self.semi_major_axis_km();
        a * (1.0 + self.eccentricity) - EARTH_RADIUS_KM
    }

    /// Calculate perigee altitude in km.
    pub fn perigee_km(&self) -> f64 {
        let a = self.semi_major_axis_km();
        a * (1.0 - self.eccentricity) - EARTH_RADIUS_KM
    }

    /// Convert TLE to Orbit object.
    ///
    /// Note: This uses osculating elements at epoch. For accurate
    /// propagation, use SGP4/SDP4.
    pub fn to_orbit(&self) -> Result<Orbit, ValidationError> {
        Orbit::new(
            self.semi_major_axis_km(),
            self.eccentricity,
            self.inclination,
            self.raan,
            self.arg_perigee,
            self.mean_anomaly, // Approximation
        )
    }

    /// Propagate the orbit to a given time.
    ///
    /// This is a simplified propagation. For accurate results,
    /// use the `sgp4` feature.
    ///
    /// # Arguments
    ///
    /// * `dt` - Target datetime (UTC)
    ///
    /// # Returns
    ///
    /// Estimated position at the given time.
    pub fn propagate(&self, dt: DateTime<Utc>) -> Result<Position, ValidationError> {
        // Simplified propagation - just use mean motion
        let minutes_since_epoch = (dt - self.epoch()).num_milliseconds() as f64 / 60000.0;
        let revolutions = minutes_since_epoch / self.orbital_period_minutes();

        // Simple circular orbit approximation
        let mean_anomaly_rad = self.mean_anomaly.to_radians();
        let new_anomaly = mean_anomaly_rad + revolutions * 2.0 * PI;

        // Convert to lat/lon (very simplified)
        let lat = (self.inclination.to_radians().sin() * new_anomaly.sin())
            .asin()
            .to_degrees();
        let mut lon = new_anomaly.to_degrees() - 180.0;
        while lon < -180.0 {
            lon += 360.0;
        }
        while lon > 180.0 {
            lon -= 360.0;
        }

        let alt = (self.apogee_km() + self.perigee_km()) / 2.0;

        Position::new(lat, lon, alt)
    }
}

/// Parse TLE scientific notation (without 'E').
/// e.g., " 12345-6" means 0.12345 * 10^-6
fn parse_tle_scientific(s: &str) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return 0.0;
    }

    // Handle sign at the beginning
    let (sign, s) = if trimmed.starts_with('-') {
        (-1.0, &trimmed[1..])
    } else if trimmed.starts_with('+') {
        (1.0, &trimmed[1..])
    } else {
        (1.0, trimmed)
    };

    if s.len() < 2 {
        return 0.0;
    }

    // Extract mantissa and exponent
    // Format: "NNNNN-E" or "NNNNN+E" where second-to-last char is exponent sign
    let exp_sign_char = s.chars().nth(s.len() - 2).unwrap_or('0');
    let exp_sign = if exp_sign_char == '-' { -1 } else { 1 };
    let exponent = s
        .chars()
        .last()
        .and_then(|c| c.to_digit(10))
        .unwrap_or(0) as i32
        * exp_sign;

    let mantissa_str = &s[..s.len().saturating_sub(2)];
    let mantissa_digits: String = mantissa_str
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let mantissa = format!("0.{}", mantissa_digits)
        .parse::<f64>()
        .unwrap_or(0.0);

    sign * mantissa * 10_f64.powi(exponent)
}

/// Parse multiple TLEs from text.
///
/// # Arguments
///
/// * `text` - Text containing one or more TLEs
///
/// # Returns
///
/// Vector of parsed TLE objects.
pub fn parse_tle(text: &str) -> Vec<TLE> {
    let lines: Vec<String> = text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    let mut tles = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("1 ") {
            // No name, just lines 1 and 2
            if i + 1 < lines.len() && lines[i + 1].starts_with("2 ") {
                if let Ok(tle) = TLE::parse(&lines[i..i + 2].to_vec()) {
                    tles.push(tle);
                }
                i += 2;
            } else {
                i += 1;
            }
        } else if i + 2 < lines.len() && lines[i + 1].starts_with("1 ") {
            // Name + line 1 + line 2
            if let Ok(tle) = TLE::parse(&lines[i..i + 3].to_vec()) {
                tles.push(tle);
            }
            i += 3;
        } else {
            i += 1;
        }
    }

    tles
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISS_TLE: &str = r#"ISS (ZARYA)
1 25544U 98067A   21275.52243902  .00001082  00000-0  27450-4 0  9999
2 25544  51.6443 208.5943 0003631 355.3422 144.3824 15.48919755304818"#;

    #[test]
    fn test_parse_tle() {
        let lines: Vec<String> = ISS_TLE.lines().map(|s| s.to_string()).collect();
        let tle = TLE::parse(&lines).unwrap();
        assert_eq!(tle.name, "ISS (ZARYA)");
        assert_eq!(tle.norad_id, 25544);
        assert!((tle.inclination - 51.6443).abs() < 0.001);
        assert!((tle.mean_motion - 15.4891975).abs() < 0.001);
    }

    #[test]
    fn test_orbital_calculations() {
        let lines: Vec<String> = ISS_TLE.lines().map(|s| s.to_string()).collect();
        let tle = TLE::parse(&lines).unwrap();

        // ISS should have ~92 minute period
        let period = tle.orbital_period_minutes();
        assert!((period - 92.9).abs() < 1.0, "Period was {}", period);

        // ISS should be at ~400 km altitude
        let apogee = tle.apogee_km();
        let perigee = tle.perigee_km();
        assert!(apogee > 350.0 && apogee < 450.0, "Apogee was {}", apogee);
        assert!(perigee > 350.0 && perigee < 450.0, "Perigee was {}", perigee);
    }

    #[test]
    fn test_parse_multiple() {
        let tles = parse_tle(ISS_TLE);
        assert_eq!(tles.len(), 1);
        assert_eq!(tles[0].name, "ISS (ZARYA)");
    }
}
