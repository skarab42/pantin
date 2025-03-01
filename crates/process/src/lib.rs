#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

//! Crate for launching and managing asynchronous processes.
//!
//! This crate provides an abstraction to spawn a process, monitor its status,
//! and kill it asynchronously, using [`tokio`](https://docs.rs/tokio) and [`process_wrap`](https://docs.rs/process-wrap).
//!
//! The spawned process is configured with the `kill on drop` feature to ensure that the process is terminated when dropped,
//! and, depending on the operating system, it leverages [`ProcessGroup`](process_wrap::tokio::ProcessGroup) on Unix or [`JobObject`](process_wrap::tokio::JobObject) on Windows
//! to also kill all its child processes.

use std::{ffi::OsStr, io, process::Stdio, result};

use process_wrap::tokio::{KillOnDrop, TokioChildWrapper, TokioCommandWrap};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tracing::{Instrument, debug, trace};

#[derive(Error, Debug)]
pub enum Error {
    #[error("spawn command failed: {1:?}")]
    SpawnCommand(#[source] io::Error, Command),
    #[error("kill child with pid = {1:?} failed")]
    KillChild(#[source] io::Error, Option<u32>),
}

pub type Result<T, E = Error> = result::Result<T, E>;

/// Represents the status of the managed process.
#[derive(Debug, Eq, PartialEq)]
pub enum Status {
    Alive,
    Exiting,
    Exited(i32),
    Error(String),
}

/// Represents an asynchronously spawned process.
///
/// This structure wraps a child process (provided by the[`process_wrap`](https://docs.rs/process-wrap) crate)
/// and ensures that the process and its child is terminated when dropped.
#[derive(Debug)]
pub struct Process {
    child: Box<dyn TokioChildWrapper>,
}

impl Process {
    /// Creates and spawns a new process.
    ///
    /// # Arguments
    ///
    /// * `program` - The command or path to the program to execute.
    /// * `args` - An iterable of arguments to pass to the program.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the process cannot be spawned.
    pub fn spawn<P, A, I>(program: P, args: A) -> Result<Self>
    where
        P: AsRef<OsStr>,
        A: IntoIterator<Item = I>,
        I: AsRef<OsStr>,
    {
        debug!("Creating a new Command instance...");
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);

        let mut command = TokioCommandWrap::with_new(program.as_ref(), |command| {
            command
                .args(args)
                .stdout(pipe_or_null(trace_enabled))
                .stderr(pipe_or_null(trace_enabled));
        });

        #[cfg(windows)]
        command.wrap(process_wrap::tokio::JobObject);

        #[cfg(unix)]
        command.wrap(process_wrap::tokio::ProcessGroup::leader());

        command.wrap(KillOnDrop);

        debug!(stdout = ?command.command().as_std(), "Spawning command child...");
        let mut child = command
            .spawn()
            .map_err(|error| Error::SpawnCommand(error, command.into_command()))?;

        if trace_enabled {
            child = trace_child_output(child);
        }

        Ok(Self { child })
    }

    /// Returns the process identifier, if available.
    #[must_use]
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Returns the current status of the process.
    pub fn status(&mut self) -> Status {
        match self.child.try_wait() {
            Ok(None) => Status::Alive,
            Ok(Some(status)) => status.code().map_or(Status::Exiting, Status::Exited),
            Err(error) => Status::Error(error.to_string()),
        }
    }

    /// Attempts to kill the process asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if killing the process fails.
    pub async fn kill(&mut self) -> Result<()> {
        debug!("Killing child with process id: {:?}", self.child.id());
        Box::into_pin(self.child.kill())
            .await
            .map_err(|error| Error::KillChild(error, self.id()))
    }
}

fn pipe_or_null(condition: bool) -> Stdio {
    if condition {
        Stdio::piped()
    } else {
        Stdio::null()
    }
}

/// Spawns tasks to trace the child process output.
///
/// This function creates asynchronous tasks that read and log the standard output and error of the child process,
/// which is useful for debugging.
///
/// # Arguments
///
/// * `child` - The child process whose output will be traced.
///
/// # Returns
///
/// The modified child process with output tracing enabled.
fn trace_child_output(mut child: Box<dyn TokioChildWrapper>) -> Box<dyn TokioChildWrapper> {
    let pid = child.id();

    if let Some(stdout) = child.stdout().take() {
        let mut stdout_reader = BufReader::new(stdout).lines();

        tokio::spawn(
            async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    trace!(?pid, "[stdout] {line}");
                }
            }
            .in_current_span(),
        );
    }

    if let Some(stderr) = child.stderr().take() {
        let mut stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn(
            async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    trace!(?pid, "[stderr] {line}");
                }
            }
            .in_current_span(),
        );
    }

    child
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    fn spawn_sleep_process() -> Process {
        #[cfg(unix)]
        let process = Process::spawn("sleep", &["1"]);
        #[cfg(windows)]
        let process = Process::spawn("timeout", ["1"]);

        process.expect("Failed to spawn process")
    }

    #[tokio::test]
    async fn test_process_exit() {
        let mut process = spawn_sleep_process();

        assert!(
            matches!(process.status(), Status::Alive),
            "Should have alive status"
        );

        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        match process.status() {
            Status::Exited(actual_code) => assert_eq!(actual_code, 0),
            status => panic!("Unexpected status: {status:?}"),
        }
    }

    #[tokio::test]
    async fn test_process_kill() {
        let mut process = spawn_sleep_process();

        assert!(
            matches!(process.status(), Status::Alive),
            "Should have alive status"
        );

        process.kill().await.expect("Should kill");

        match process.status() {
            Status::Exited(actual_code) => assert_eq!(actual_code, 1),
            status => panic!("Unexpected status: {status:?}"),
        }
    }

    #[tokio::test]
    async fn test_process_id() {
        let process = spawn_sleep_process();

        assert!(process.id().is_some(), "Should have an id");
    }

    #[tokio::test]
    #[traced_test]
    async fn test_process_tracing() {
        let mut process = spawn_sleep_process();

        assert!(logs_contain(
            "pantin_process: Creating a new Command instance..."
        ));
        assert!(logs_contain("pantin_process: Spawning command child..."));

        process.kill().await.expect("Should kill");

        assert!(logs_contain(
            "pantin_process: Killing child with process id:"
        ));
    }
}
