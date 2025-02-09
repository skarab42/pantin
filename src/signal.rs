// Mostly taken from `tokio-graceful-shutdown` crate
// https://github.com/Finomnis/tokio-graceful-shutdown/blob/a02c1b474034419d6ad9d30c66b1cafd7045773b/src/signal_handling.rs

use std::{io, result};

use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum Error {
    #[error("register shutdown signal failed")]
    RegisterShutdown(#[from] io::Error),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[cfg(unix)]
async fn shutdown_impl() -> io::Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = terminate.recv() => debug!("Received SIGTERM."),
        _ = interrupt.recv() => debug!("Received SIGINT."),
    };

    Ok(())
}

#[cfg(windows)]
async fn shutdown_impl() -> io::Result<()> {
    use tokio::signal::windows;

    let mut ctrl_c = windows::ctrl_c()?;
    let mut ctrl_break = windows::ctrl_break()?;
    let mut ctrl_close = windows::ctrl_close()?;
    let mut ctrl_shutdown = windows::ctrl_shutdown()?;

    tokio::select! {
        _ = ctrl_c.recv() => debug!("Received CTRL_C."),
        _ = ctrl_break.recv() => debug!("Received CTRL_BREAK."),
        _ = ctrl_close.recv() => debug!("Received CTRL_CLOSE."),
        _ = ctrl_shutdown.recv() => debug!("Received CTRL_SHUTDOWN."),
    };

    Ok(())
}

pub async fn shutdown() -> Result<()> {
    shutdown_impl().await.map_err(Error::RegisterShutdown)
}
