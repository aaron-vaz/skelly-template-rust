//! Configuration, read once at startup and passed down.

use std::net::SocketAddr;
use std::time::Duration;

use thiserror::Error;
use tracing::Level;

const DEFAULT_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Everything the service needs to start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The address the HTTP server listens on.
    pub addr: SocketAddr,
    /// The minimum level that is logged.
    pub log_level: Level,
    /// How long in-flight requests get to finish on shutdown.
    pub shutdown_timeout: Duration,
}

/// Why the configuration could not be loaded.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// `ADDR` is not a socket address.
    #[error("invalid ADDR {0:?}: want host:port such as 0.0.0.0:8080")]
    Addr(String),
    /// `LOG_LEVEL` is not a log level.
    #[error("invalid LOG_LEVEL {0:?}: want error, warn, info, debug or trace")]
    LogLevel(String),
    /// `SHUTDOWN_TIMEOUT` is not a positive number of seconds.
    #[error("invalid SHUTDOWN_TIMEOUT {0:?}: want a positive number of seconds such as 30")]
    ShutdownTimeout(String),
}

impl Config {
    /// Reads `ADDR`, `LOG_LEVEL` and `SHUTDOWN_TIMEOUT` (seconds) through
    /// `getenv`, falling back to defaults for anything that is unset or empty.
    /// Taking the lookup as a function keeps tests away from the real environment.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] naming the first variable with an invalid value.
    pub fn load(getenv: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |key: &str| getenv(key).filter(|value| !value.is_empty());

        let addr = get("ADDR").unwrap_or_else(|| DEFAULT_ADDR.to_owned());
        let addr = addr.parse().map_err(|_| ConfigError::Addr(addr))?;

        let log_level = get("LOG_LEVEL").unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_owned());
        let log_level = log_level
            .parse()
            .map_err(|_| ConfigError::LogLevel(log_level))?;

        let shutdown_timeout = match get("SHUTDOWN_TIMEOUT") {
            None => DEFAULT_SHUTDOWN_TIMEOUT,
            Some(raw) => match raw.parse::<u64>() {
                Ok(seconds) if seconds > 0 => Duration::from_secs(seconds),
                _ => return Err(ConfigError::ShutdownTimeout(raw)),
            },
        };

        Ok(Self {
            addr,
            log_level,
            shutdown_timeout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        Config::load(|key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn load_uses_defaults_when_nothing_is_set() {
        // Given an empty environment
        let env = [];

        // When the config is loaded
        let config = load(&env).unwrap();

        // Then the defaults apply
        assert_eq!(config.addr, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(config.log_level, Level::INFO);
        assert_eq!(config.shutdown_timeout, Duration::from_secs(10));
    }

    #[test]
    fn load_reads_every_variable() {
        // Given all three variables
        let env = [
            ("ADDR", "127.0.0.1:9000"),
            ("LOG_LEVEL", "DEBUG"),
            ("SHUTDOWN_TIMEOUT", "30"),
        ];

        // When the config is loaded
        let config = load(&env).unwrap();

        // Then they are used
        assert_eq!(config.addr, "127.0.0.1:9000".parse().unwrap());
        assert_eq!(config.log_level, Level::DEBUG);
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
    }

    #[test]
    fn load_treats_an_empty_value_as_unset() {
        // Given a variable that is set but empty
        let env = [("ADDR", "")];

        // When the config is loaded
        let config = load(&env).unwrap();

        // Then the default applies
        assert_eq!(config.addr, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn load_rejects_invalid_values() {
        // Given values that cannot be used
        let cases = [
            ("ADDR", "nowhere", ConfigError::Addr("nowhere".to_owned())),
            (
                "LOG_LEVEL",
                "loud",
                ConfigError::LogLevel("loud".to_owned()),
            ),
            (
                "SHUTDOWN_TIMEOUT",
                "0",
                ConfigError::ShutdownTimeout("0".to_owned()),
            ),
            (
                "SHUTDOWN_TIMEOUT",
                "soon",
                ConfigError::ShutdownTimeout("soon".to_owned()),
            ),
        ];

        for (key, value, expected) in cases {
            // When the config is loaded
            let result = load(&[(key, value)]);

            // Then the error names the variable
            assert_eq!(result, Err(expected), "{key}={value}");
        }
    }
}
