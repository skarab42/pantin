use std::{ffi::OsStr, fmt::Debug, result};

use base64::{DecodeError, Engine, prelude::BASE64_STANDARD};
use pantin_marionette::{Marionette, webdriver};
use pantin_process::{Process, Status};
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{profile, profile::Profile};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Profile(#[from] profile::Error),
    #[error(transparent)]
    Process(#[from] pantin_process::Error),
    #[error(transparent)]
    Marionette(#[from] pantin_marionette::Error),
    #[error(transparent)]
    MarionetteRequest(#[from] pantin_marionette::request::Error),
    #[error("get child status failed: {0}")]
    ChildStatus(String),
    #[error("decode screenshot failed: {0}")]
    DecodeScreenshot(#[source] DecodeError),
}

pub type Result<T, E = Error> = result::Result<T, E>;

pub type ScreenshotParameters = webdriver::TakeScreenshotParameters;

#[derive(Debug)]
pub struct Browser {
    uuid: Uuid,
    profile: Profile,
    process: Process,
    marionette: Marionette,
}

impl Browser {
    #[instrument(name = "Browser::new")]
    pub async fn new<P>(uuid: Uuid, program: P) -> Result<Self>
    where
        P: AsRef<OsStr> + Debug + Send,
    {
        debug!("Opening a new Browser instance...");
        let profile = Profile::new().await?;
        let process = Process::new(
            program,
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
        let marionette = Marionette::new(&profile.marionette_address()).await?;
        debug!(
            "Marionette listening at http://{}",
            profile.marionette_address()
        );

        Ok(Self {
            uuid,
            profile,
            process,
            marionette,
        })
    }

    pub async fn open<P>(program: P) -> Result<Self>
    where
        P: AsRef<OsStr> + Debug + Send,
    {
        Self::new(Uuid::new_v4(), program).await
    }

    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn pid(&self) -> Option<u32> {
        self.process.id()
    }

    pub fn sid(&self) -> &str {
        self.marionette.session_id()
    }

    pub fn status(&mut self) -> Status {
        self.process.status()
    }

    #[instrument(name = "Browser::resize", skip(self), fields(uuid = ?self.uuid))]
    pub async fn resize(&mut self, width: u16, height: u16) -> Result<(u16, u16)> {
        let rect = self
            .marionette
            .send(&webdriver::SetWindowRect::new(
                webdriver::SetWindowRectParameters {
                    x: None,
                    y: None,
                    width: Some(width),
                    height: Some(height),
                },
            ))
            .await?;

        Ok((rect.width, rect.height))
    }

    #[instrument(name = "Browser::navigate", skip(self), fields(uuid = ?self.uuid))]
    pub async fn navigate<U: Into<String> + Send + Debug>(&mut self, url: U) -> Result<()> {
        self.marionette
            .send(&webdriver::Navigate::new(webdriver::NavigateParameters {
                url: url.into(),
            }))
            .await?;

        Ok(())
    }

    #[instrument(name = "Browser::screenshot", skip(self), fields(uuid = ?self.uuid))]
    pub async fn screenshot(
        &mut self,
        parameters: webdriver::TakeScreenshotParameters,
    ) -> Result<Vec<u8>> {
        let webdriver::TakeScreenshotResponse { base64_png } = self
            .marionette
            .send(&webdriver::TakeScreenshot::new(parameters))
            .await?;

        BASE64_STANDARD
            .decode(base64_png)
            .map_err(Error::DecodeScreenshot)
    }

    #[instrument(name = "Browser::close", skip(self), fields(uuid = ?self.uuid))]
    pub async fn close(mut self) -> Result<Status> {
        debug!("Closing browser instance...");
        let status = match self.process.status() {
            Status::Alive => {
                self.process.kill().await?;

                Ok(self.process.status())
            },
            Status::Error(error) => Err(Error::ChildStatus(error)),
            status => Ok(status),
        };
        debug!("Browser instance closed with status: {status:?}");

        #[cfg(windows)]
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if self.profile.exists() {
            self.profile.remove()?;
        }

        status
    }
}
