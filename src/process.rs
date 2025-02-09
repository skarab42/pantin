use std::{ffi::OsStr, io, process::Stdio, result};

use process_wrap::tokio::{KillOnDrop, TokioChildWrapper, TokioCommandWrap};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tracing::trace;

#[derive(Error, Debug)]
pub enum Error {
    #[error("spawn command failed: {1:?}")]
    SpawnCommand(#[source] io::Error, Command),
    #[error("kill child with pid = {1:?} failed")]
    KillChild(#[source] io::Error, Option<u32>),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub enum ChildStatus {
    Alive,
    Exiting,
    Exited(i32),
    Error(String),
}

#[derive(Debug)]
pub struct ChildWrapper {
    child: Box<dyn TokioChildWrapper>,
}

impl ChildWrapper {
    pub fn new<P, A, I>(program: P, args: A) -> Result<Self>
    where
        P: AsRef<OsStr>,
        A: IntoIterator<Item = I>,
        I: AsRef<OsStr>,
    {
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

        let mut child = command
            .spawn()
            .map_err(|error| Error::SpawnCommand(error, command.into_command()))?;

        if trace_enabled {
            child = trace_child_output(child);
        }

        Ok(Self { child })
    }

    #[must_use]
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    pub fn status(&mut self) -> ChildStatus {
        match self.child.try_wait() {
            Ok(None) => ChildStatus::Alive,
            Ok(Some(status)) => status
                .code()
                .map_or(ChildStatus::Exiting, ChildStatus::Exited),
            Err(error) => ChildStatus::Error(error.to_string()),
        }
    }

    pub async fn kill(&mut self) -> Result<()> {
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

fn trace_child_output(mut child: Box<dyn TokioChildWrapper>) -> Box<dyn TokioChildWrapper> {
    let pid = child.id();

    if let Some(stdout) = child.stdout().take() {
        let mut stdout_reader = BufReader::new(stdout).lines();

        tokio::spawn(async move {
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                trace!(?pid, "[stdout] {line}");
            }
        });
    }

    if let Some(stderr) = child.stderr().take() {
        let mut stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn(async move {
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                trace!(?pid, "[stderr] {line}");
            }
        });
    }

    child
}
