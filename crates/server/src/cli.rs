//! This module defines the configuration settings for the pantin server.

use clap::{Parser, ValueEnum};
use serde::Serialize;

/// Represents the log verbosity level.
///
/// The variants are serialized as lowercase strings.
#[derive(Debug, Copy, Clone, ValueEnum, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum LogLevel {
    Info,
    Debug,
    Trace,
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// Holds all configuration settings to start the pantin server.
///
/// Values can be provided via command-line arguments or through environment variables.
/// Default values are provided if none are specified.
#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct PantinSettings {
    /// Host of the API server
    #[arg(long, default_value = "localhost", env = "PANTIN_SERVER_HOST")]
    pub server_host: String,

    /// Port number of the API server
    #[arg(short, long, default_value_t = 4242, env = "PANTIN_SERVER_PORT")]
    pub server_port: u16,

    /// Request timeout in seconds
    #[arg(short, long, default_value_t = 30, env = "PANTIN_REQUEST_TIMEOUT")]
    pub request_timeout: u16,

    /// Number of active browser in the pool
    #[arg(long, default_value_t = 5, env = "PANTIN_BROWSER_POOL_MAX_SIZE")]
    pub browser_pool_max_size: u8,

    /// Maximum age in seconds of an unused browser session
    #[arg(long, default_value_t = 60, env = "PANTIN_BROWSER_MAX_AGE")]
    pub browser_max_age: u16,

    /// Maximum number of times to recycle a browser session
    #[arg(long, default_value_t = 10, env = "PANTIN_BROWSER_MAX_RECYCLE_COUNT")]
    pub browser_max_recycle_count: u16,

    /// Command or binary path to launch a gecko like browser
    #[arg(long, default_value = "firefox", env = "PANTIN_BROWSER_PROGRAM")]
    pub browser_program: String,

    /// Log level
    #[arg(value_enum, long, default_value = "info", env = "PANTIN_LOG_LEVEL")]
    pub log_level: LogLevel,
}

/// Parses the command-line arguments and environment variables to produce a [`PantinSettings`] instance.
pub fn parse() -> PantinSettings {
    PantinSettings::parse()
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {

    use super::*;

    #[test]
    fn test_log_level_as_ref() {
        assert_eq!(LogLevel::Info.as_ref(), "info");
        assert_eq!(LogLevel::Debug.as_ref(), "debug");
        assert_eq!(LogLevel::Trace.as_ref(), "trace");
    }

    #[test]
    fn test_default_settings() {
        let args = vec!["pantin"];
        let settings = PantinSettings::parse_from(args);

        assert_eq!(settings.server_host, "localhost");
        assert_eq!(settings.server_port, 4242);
        assert_eq!(settings.request_timeout, 30);
        assert_eq!(settings.browser_pool_max_size, 5);
        assert_eq!(settings.browser_max_age, 60);
        assert_eq!(settings.browser_max_recycle_count, 10);
        assert_eq!(settings.browser_program, "firefox");
        assert!(
            matches!(settings.log_level, LogLevel::Info),
            "Should have Info log level, got: {:?}",
            settings.log_level
        );
    }

    #[test]
    fn test_custom_settings() {
        // Provide custom CLI arguments.
        let args = vec![
            "pantin",
            "--server-host",
            "example.com",
            "--server-port",
            "8080",
            "--request-timeout",
            "60",
            "--browser-pool-max-size",
            "10",
            "--browser-max-age",
            "120",
            "--browser-max-recycle-count",
            "20",
            "--browser-program",
            "custom_browser",
            "--log-level",
            "debug",
        ];
        let settings = PantinSettings::parse_from(args);

        assert_eq!(settings.server_host, "example.com");
        assert_eq!(settings.server_port, 8080);
        assert_eq!(settings.request_timeout, 60);
        assert_eq!(settings.browser_pool_max_size, 10);
        assert_eq!(settings.browser_max_age, 120);
        assert_eq!(settings.browser_max_recycle_count, 20);
        assert_eq!(settings.browser_program, "custom_browser");
        assert!(matches!(settings.log_level, LogLevel::Debug));
    }
}
