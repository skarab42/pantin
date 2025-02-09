use std::result;

use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

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
    uuid: Uuid,
    profile: Profile,
    process: ChildWrapper,
}

impl Browser {
    #[instrument(name = "Browser::new")]
    pub async fn new(uuid: Uuid) -> Result<Self> {
        debug!("Opening a new Browser instance...");
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

        debug!("Browser opened!");

        // TODO: connect to Marionette...

        debug!(
            "Marionette listening at http://{}",
            profile.marionette_address()
        );

        Ok(Self {
            uuid,
            profile,
            process,
        })
    }

    pub async fn open() -> Result<Self> {
        Self::new(Uuid::new_v4()).await
    }

    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.process.id()
    }

    pub fn status(&mut self) -> ChildStatus {
        self.process.status()
    }

    #[instrument(name = "Browser::close",skip(self), fields(uuid = ?self.uuid))]
    pub async fn close(mut self) -> Result<ChildStatus> {
        debug!("Closing browser instance...");
        let status = match self.process.status() {
            ChildStatus::Alive => {
                self.process.kill().await?;

                Ok(self.process.status())
            },
            ChildStatus::Error(error) => Err(Error::ChildStatus(error)),
            status => Ok(status),
        };
        debug!("Browser instance closed with status: {status:?}");

        #[cfg(windows)]
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        if self.profile.exists() {
            self.profile.remove()?;
        }

        status
    }
}
