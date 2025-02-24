use clap::{Parser, ValueEnum};
use serde::Serialize;

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

pub fn parse() -> PantinSettings {
    PantinSettings::parse()
}
