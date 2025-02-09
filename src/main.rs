#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use color_eyre::eyre::Result;
use pantin_lib::process::ChildWrapper;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pantin_lib::tracing::setup()?;

    info!("Starting...");

    let mut firefox = ChildWrapper::new("firefox", ["--headless"])?;
    info!(?firefox);

    info!("Press [CTRL+C] to exit gracefully.");
    pantin_lib::signal::shutdown().await?;

    firefox.kill().await?;

    info!("Exited gracefully !");

    Ok(())
}
