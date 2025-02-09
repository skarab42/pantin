use std::result;

use thiserror::Error;

use crate::process::{ChildStatus, ChildWrapper};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ChildWrapper(#[from] crate::process::Error),
    #[error("get child status failed: {0}")]
    ChildStatus(String),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug)]
pub struct Browser {
    process: ChildWrapper,
}

impl Browser {
    pub fn open() -> Result<Self> {
        let process = ChildWrapper::new(
            "firefox",
            [
                "--private",
                "--headless",
                "--no-remote",
                "--marionette",
                "--new-instance",
            ],
        )?;

        Ok(Self { process })
    }

    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.process.id()
    }

    pub fn status(&mut self) -> ChildStatus {
        self.process.status()
    }

    pub async fn close(mut self) -> Result<ChildStatus> {
        match self.process.status() {
            ChildStatus::Alive => {
                self.process.kill().await?;

                Ok(self.process.status())
            },
            ChildStatus::Error(error) => Err(Error::ChildStatus(error)),
            status => Ok(status),
        }
    }
}
