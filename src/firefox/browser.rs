use std::result;

use thiserror::Error;

use crate::{
    firefox::Profile,
    process::{ChildStatus, ChildWrapper},
};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Profile(#[from] crate::firefox::profile::Error),
    #[error(transparent)]
    ChildWrapper(#[from] crate::process::Error),
    #[error("get child status failed: {0}")]
    ChildStatus(String),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug)]
pub struct Browser {
    profile: Profile,
    process: ChildWrapper,
}

impl Browser {
    pub async fn open() -> Result<Self> {
        let profile = Profile::new().await?;
        let process = ChildWrapper::new(
            "firefox",
            [
                "--private",
                "--headless",
                "--no-remote",
                "--marionette",
                "--new-instance",
                "--profile",
                profile.path()?,
            ],
        )?;

        Ok(Self { profile, process })
    }

    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.process.id()
    }

    pub fn status(&mut self) -> ChildStatus {
        self.process.status()
    }

    pub async fn close(mut self) -> Result<ChildStatus> {
        let status = match self.process.status() {
            ChildStatus::Alive => {
                self.process.kill().await?;

                Ok(self.process.status())
            },
            ChildStatus::Error(error) => Err(Error::ChildStatus(error)),
            status => Ok(status),
        };

        #[cfg(windows)]
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        if self.profile.exists() {
            self.profile.remove()?;
        }

        status
    }
}
