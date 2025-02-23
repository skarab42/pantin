use std::result;

use thiserror::Error;
use tracing::subscriber::{set_global_default, SetGlobalDefaultError};
use tracing_subscriber::{fmt::layer, layer::SubscriberExt, EnvFilter, Registry};

#[derive(Error, Debug)]
pub enum Error {
    #[error("setup tracing failed")]
    SetGlobalDefault(#[source] SetGlobalDefaultError),
}

pub type Result<T, E = Error> = result::Result<T, E>;

pub fn install<L: AsRef<str>>(log_level: L) -> Result<()> {
    let env_filter = EnvFilter::new(format!("none,pantin={}", log_level.as_ref()));
    let format_layer = layer().with_file(true).with_line_number(true);

    set_global_default(Registry::default().with(env_filter).with(format_layer))
        .map_err(Error::SetGlobalDefault)
}
