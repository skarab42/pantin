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

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct PantinSettings {
    /// Host of the API server
    #[arg(long, default_value = "localhost", env = "PANTIN_SERVER_HOST")]
    pub host: String,

    /// Port number of the API server
    #[arg(short, long, default_value_t = 4242, env = "PANTIN_SERVER_PORT")]
    pub port: u16,

    /// Log level
    #[arg(value_enum, long, default_value = "info", env = "PANTIN_LOG_LEVEL")]
    pub log_level: LogLevel,
}

pub fn parse() -> PantinSettings {
    PantinSettings::parse()
}
