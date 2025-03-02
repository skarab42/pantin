#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![cfg_attr(coverage, feature(coverage_attribute))]

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

    Ok(())
}
