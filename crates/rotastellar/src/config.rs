//! RotaStellar SDK - Configuration
//!
//! SDK configuration and settings.

use crate::auth::Environment;

/// SDK configuration settings.
///
/// # Example
///
/// ```
/// use rotastellar::config::Config;
///
/// let config = Config::new("rs_live_xxx");
/// assert_eq!(config.base_url, "https://api.rotastellar.com/v1");
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// RotaStellar API key
    pub api_key: Option<String>,
    /// API base URL
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout: f64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries in seconds
    pub retry_delay: f64,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ROTASTELLAR_API_KEY").ok(),
            base_url: std::env::var("ROTASTELLAR_BASE_URL")
                .unwrap_or_else(|_| "https://api.rotastellar.com/v1".to_string()),
            timeout: 30.0,
            max_retries: 3,
            retry_delay: 1.0,
            debug: std::env::var("ROTASTELLAR_DEBUG")
                .map(|v| ["1", "true", "yes"].contains(&v.to_lowercase().as_str()))
                .unwrap_or(false),
        }
    }
}

impl Config {
    /// Create a new Config with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Create a Config builder for more options.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Check if using a test API key.
    pub fn is_test_key(&self) -> bool {
        self.api_key
            .as_ref()
            .map(|k| k.starts_with("rs_test_"))
            .unwrap_or(false)
    }

    /// Check if using a live API key.
    pub fn is_live_key(&self) -> bool {
        self.api_key
            .as_ref()
            .map(|k| k.starts_with("rs_live_"))
            .unwrap_or(false)
    }

    /// Get the API key environment (test or live).
    pub fn environment(&self) -> Option<Environment> {
        if self.is_test_key() {
            Some(Environment::Test)
        } else if self.is_live_key() {
            Some(Environment::Live)
        } else {
            None
        }
    }
}

/// Builder for Config.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<f64>,
    max_retries: Option<u32>,
    retry_delay: Option<f64>,
    debug: Option<bool>,
}

impl ConfigBuilder {
    /// Set the API key.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the timeout in seconds.
    pub fn timeout(mut self, timeout: f64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the retry delay in seconds.
    pub fn retry_delay(mut self, retry_delay: f64) -> Self {
        self.retry_delay = Some(retry_delay);
        self
    }

    /// Enable or disable debug mode.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = Some(debug);
        self
    }

    /// Build the Config.
    pub fn build(self) -> Config {
        let mut config = Config::default();

        if let Some(api_key) = self.api_key {
            config.api_key = Some(api_key);
        }
        if let Some(base_url) = self.base_url {
            config.base_url = base_url;
        }
        if let Some(timeout) = self.timeout {
            config.timeout = timeout;
        }
        if let Some(max_retries) = self.max_retries {
            config.max_retries = max_retries;
        }
        if let Some(retry_delay) = self.retry_delay {
            config.retry_delay = retry_delay;
        }
        if let Some(debug) = self.debug {
            config.debug = debug;
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("rs_live_test");
        assert_eq!(config.api_key, Some("rs_live_test".to_string()));
        assert!(config.is_live_key());
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .api_key("rs_test_key")
            .timeout(60.0)
            .debug(true)
            .build();

        assert_eq!(config.api_key, Some("rs_test_key".to_string()));
        assert_eq!(config.timeout, 60.0);
        assert!(config.debug);
        assert!(config.is_test_key());
    }
}
