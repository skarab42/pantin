use std::{env::var, result};

use thiserror::Error;
use tracing::subscriber::{set_global_default, SetGlobalDefaultError};
use tracing_subscriber::{fmt::layer, layer::SubscriberExt, EnvFilter, Registry};

#[derive(Error, Debug)]
pub enum Error {
    #[error("setup tracing failed")]
    SetGlobalDefault(#[from] SetGlobalDefaultError),
}

pub type Result<T, E = Error> = result::Result<T, E>;

pub fn setup() -> Result<()> {
    let log_level = var("PANTIN_LOG_LEVEL").unwrap_or_else(|_| "info".into());
    let env_filter = EnvFilter::new(format!("none,pantin={log_level}"));
    let format_layer = layer().with_file(true).with_line_number(true);

    set_global_default(Registry::default().with(env_filter).with(format_layer))
        .map_err(Error::SetGlobalDefault)
}
