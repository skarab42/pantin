#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

mod logger;
mod signal;

use color_eyre::eyre::Result;
use pantin_browser::{Browser, ScreenshotParameters};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    logger::install()?;

    info!("Starting...");

    let mut browser = Browser::open().await?;

    let size = browser.resize(800, 600).await?;
    info!("Resized to {size:?}");

    browser.navigate("https://www.infomaniak.ch").await?;

    let parameters = ScreenshotParameters::viewport();
    let bytes = browser.screenshot(parameters).await?;
    info!("PNG size {}", bytes.len());

    info!("Press [CTRL+C] to exit gracefully.");
    signal::shutdown().await?;

    browser.close().await?;

    info!("Exited gracefully !");

    Ok(())
}
