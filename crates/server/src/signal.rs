//! [`tokio-graceful-shutdown`](https://github.com/Finomnis/tokio-graceful-shutdown) crate.
//!
//! ## Platform-Specific Behavior
//!
//! - **Unix:**  
//!   Listens for SIGTERM and SIGINT signals. When either signal is received, a debug message is logged and the
//!   shutdown process continues.
//!
//! - **Windows:**  
//!   Listens for various control signals: `CTRL_C`, `CTRL_BREAK`, `CTRL_CLOSE`, and `CTRL_SHUTDOWN`. When any of these signals
//!   is received, a corresponding debug message is logged.
//!
//! ## Usage
//!
//! Use the [`shutdown`] function to block asynchronously until a shutdown signal is received:
//!
//! ```no_run
//! use pantin_shutdown::shutdown;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Wait for a shutdown signal (e.g., SIGTERM or SIGINT on Unix, CTRL_C on Windows)
//!     shutdown().await?;
//!     println!("Shutdown signal received, exiting gracefully.");
//!     Ok(())
//! }
//! ```
//!
//! ## Errors
//!
//! The [`Error`] enum defined in this module is used to wrap any I/O errors encountered while setting up the signal handlers.

use std::{io, result};

use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum Error {
    #[error("register shutdown signal failed")]
    RegisterShutdown(#[source] io::Error),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[cfg(unix)]
async fn shutdown_impl() -> io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = terminate.recv() => debug!("Received SIGTERM."),
        _ = interrupt.recv() => debug!("Received SIGINT."),
    };

    Ok(())
}

#[cfg(windows)]
#[allow(clippy::cognitive_complexity)]
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

/// Asynchronously waits for a shutdown signal from the operating system.
///
/// This function blocks until a shutdown signal is received.
///
/// # Errors
///
/// Returns an [`Error::RegisterShutdown`] if setting up the signal handlers fails.
pub async fn shutdown() -> Result<()> {
    shutdown_impl().await.map_err(Error::RegisterShutdown)
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use tokio::time::{Duration, timeout};

    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn test_shutdown_unix() {
        use nix::{
            sys::signal::{Signal, kill},
            unistd::getpid,
        };

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            kill(getpid(), Signal::SIGINT).expect("Failed to send SIGINT");
        });

        let result = timeout(Duration::from_millis(500), shutdown()).await;
        assert!(
            result.is_ok(),
            "shutdown() did not complete within the timeout period"
        );
        assert!(
            result.as_ref().unwrap().is_ok(),
            "shutdown() returned an error: {:?}",
            result.unwrap_err()
        );
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_shutdown_windows_no_signal() {
        // On Windows, we cannot easily simulate a signal without affecting the process.
        // Instead, we check that shutdown() does not immediately complete if no signal is sent.
        let shutdown_future = shutdown();
        let result = timeout(Duration::from_millis(100), shutdown_future).await;
        assert!(
            result.is_err(),
            "Expected shutdown() to not complete without a signal on Windows"
        );
    }
}
