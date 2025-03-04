//! Module for configuring and installing the global tracing subscriber for the pantin server.
//!
//! This module provides a simple function to install a global tracing subscriber configured
//! with an environment filter and a formatting layer. The subscriber logs messages at the specified
//! log level and includes file names and line numbers in its output.
//!
//! # Example
//!
//! ```no_run
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     pantin_tracing::install("debug")?;
//!     // Your application logic here...
//!     Ok(())
//! }
//! ```

use std::result;

use thiserror::Error;
use tracing::subscriber::{SetGlobalDefaultError, set_global_default};
use tracing_subscriber::{EnvFilter, Registry, fmt::layer, layer::SubscriberExt};

#[derive(Error, Debug)]
pub enum Error {
    #[error("setup tracing failed")]
    SetGlobalDefault(#[source] SetGlobalDefaultError),
}

pub type Result<T, E = Error> = result::Result<T, E>;

/// Installs a global tracing subscriber with the given log level.
///
/// This function configures an environment filter and a formatting layer that includes file names and
/// line numbers, then sets the global default subscriber. The log level is applied to the `"pantin"` target.
///
/// # Arguments
///
/// * `log_level` - A value convertible to a string representing the desired log level (e.g. "info", "debug", "trace").
///
/// # Errors
///
/// Returns an [`Error::SetGlobalDefault`] if the installation fails.
pub fn install<L: AsRef<str>>(log_level: L) -> Result<()> {
    let env_filter = EnvFilter::new(format!("pantin={}", log_level.as_ref()));
    let format_layer = layer().with_file(true).with_line_number(true);

    set_global_default(Registry::default().with(env_filter).with(format_layer))
        .map_err(Error::SetGlobalDefault)
}
