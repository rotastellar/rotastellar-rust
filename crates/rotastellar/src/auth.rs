//! RotaStellar SDK - Authentication
//!
//! API key validation and authentication handling.

use crate::error::AuthenticationError;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref API_KEY_PATTERN: Regex =
        Regex::new(r"^rs_(live|test)_[a-zA-Z0-9]{16,}$").unwrap();
}

const API_KEY_PREFIX_LIVE: &str = "rs_live_";
const API_KEY_PREFIX_TEST: &str = "rs_test_";

/// API key environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Live/production environment
    Live,
    /// Test/sandbox environment
    Test,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live => write!(f, "live"),
            Self::Test => write!(f, "test"),
        }
    }
}

/// Validate an API key and return its environment.
///
/// # Arguments
///
/// * `api_key` - The API key to validate
///
/// # Returns
///
/// The environment (Live or Test) if valid.
///
/// # Errors
///
/// Returns an error if:
/// - api_key is None or empty
/// - api_key format is invalid
pub fn validate_api_key(api_key: Option<&str>) -> Result<Environment, AuthenticationError> {
    let key = match api_key {
        None => return Err(AuthenticationError::MissingApiKey),
        Some(k) if k.trim().is_empty() => return Err(AuthenticationError::MissingApiKey),
        Some(k) => k.trim(),
    };

    // Check prefix and determine environment
    let environment = if key.starts_with(API_KEY_PREFIX_LIVE) {
        Environment::Live
    } else if key.starts_with(API_KEY_PREFIX_TEST) {
        Environment::Test
    } else {
        return Err(AuthenticationError::invalid_api_key(key));
    };

    // Validate full pattern
    if !API_KEY_PATTERN.is_match(key) {
        return Err(AuthenticationError::invalid_api_key(key));
    }

    Ok(environment)
}

/// Mask an API key for safe logging.
///
/// # Arguments
///
/// * `api_key` - The API key to mask
///
/// # Returns
///
/// Masked API key showing only prefix and last 4 characters.
///
/// # Example
///
/// ```
/// use rotastellar::auth::mask_api_key;
///
/// let masked = mask_api_key("rs_live_abc123def456xyz789");
/// assert!(masked.contains("****"));
/// ```
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 12 {
        return format!("{}****", &api_key[..api_key.len().min(8)]);
    }

    let prefix_end = 8; // "rs_live_" or "rs_test_"
    format!(
        "{}****{}",
        &api_key[..prefix_end],
        &api_key[api_key.len() - 4..]
    )
}

/// Get the authorization header value for API requests.
///
/// # Arguments
///
/// * `api_key` - The API key
///
/// # Returns
///
/// The "Bearer {api_key}" string for the Authorization header.
pub fn get_auth_header(api_key: &str) -> String {
    format!("Bearer {}", api_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_live_key() {
        let result = validate_api_key(Some("rs_live_abc123def456xyz789"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Environment::Live);
    }

    #[test]
    fn test_validate_test_key() {
        let result = validate_api_key(Some("rs_test_abc123def456xyz789"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Environment::Test);
    }

    #[test]
    fn test_validate_missing_key() {
        let result = validate_api_key(None);
        assert!(matches!(result, Err(AuthenticationError::MissingApiKey)));
    }

    #[test]
    fn test_validate_invalid_prefix() {
        let result = validate_api_key(Some("sk_live_abc123def456xyz789"));
        assert!(matches!(
            result,
            Err(AuthenticationError::InvalidApiKey { .. })
        ));
    }

    #[test]
    fn test_mask_api_key() {
        let masked = mask_api_key("rs_live_abc123def456xyz789");
        assert_eq!(masked, "rs_live_****z789");
    }
}
