#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use color_eyre::eyre::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    pantin_lib::tracing::setup()?;

    info!("Starting...");

    info!("Press [CTRL+C] to exit gracefully.");
    pantin_lib::signal::shutdown().await?;

    info!("Exited gracefully !");

    Ok(())
}
