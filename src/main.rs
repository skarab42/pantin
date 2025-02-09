#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use color_eyre::eyre::Result;
use tracing::info;

fn main() -> Result<()> {
    pantin_lib::tracing::setup()?;

    info!("Starting...");

    pantin_lib::hello_world();

    info!("Exited gracefully !");

    Ok(())
}
