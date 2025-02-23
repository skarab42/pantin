#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

mod api;
mod browser_pool;
mod cli;
mod logger;
mod routes;
mod server;
mod signal;
mod state;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let settings = cli::parse();
    logger::install(settings.log_level)?;

    server::start(settings).await?;

    // info!("Starting...");
    //
    // let mut browser = Browser::open().await?;
    //
    // let size = browser.resize(800, 600).await?;
    // info!("Resized to {size:?}");
    //
    // browser.navigate("https://www.infomaniak.ch").await?;
    //
    // let parameters = ScreenshotParameters::viewport();
    // let bytes = browser.screenshot(parameters).await?;
    // info!("PNG size {}", bytes.len());

    // signal::shutdown().await?;

    // browser.close().await?;

    Ok(())
}
