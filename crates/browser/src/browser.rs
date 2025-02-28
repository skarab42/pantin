use std::{ffi::OsStr, fmt::Debug, result};

use base64::{DecodeError, Engine, prelude::BASE64_STANDARD};
use pantin_marionette::{Marionette, webdriver};
use pantin_process::{Process, Status};
use thiserror::Error;
use tracing::{debug, instrument};
use url::{ParseError, Url};
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
    #[error("parse url failed: {0}")]
    ParseUrl(#[source] ParseError),
    #[error("unsupported url protocol: only 'http://' and 'https://' are allowed")]
    UnsupportedUrlProtocol,
}

pub type Result<T, E = Error> = result::Result<T, E>;

pub type ScreenshotFindElementUsing = webdriver::FindElementUsing;
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
                url: parse_url(url.into().as_str())?,
            }))
            .await?;

        Ok(())
    }

    #[instrument(name = "Browser::execute_script", skip(self), fields(uuid = ?self.uuid))]
    pub async fn execute_script<S: Into<String> + Send + Debug>(
        &mut self,
        script: S,
        args: Option<Vec<String>>,
    ) -> Result<()> {
        self.marionette
            .send(&webdriver::ExecuteScript::new(
                webdriver::ExecuteScriptParameters {
                    script: script.into(),
                    args: args.unwrap_or_default(),
                },
            ))
            .await?;

        Ok(())
    }

    #[instrument(name = "Browser::inject_header_styles", skip(self), fields(uuid = ?self.uuid))]
    pub async fn inject_header_styles<S: Into<String> + Debug>(&mut self, styles: S) -> Result<()> {
        let script = "
            let style = document.createElement('style');
            style.innerHTML = arguments[0];
            document.head.appendChild(style);
        ";
        let args = Vec::from([styles.into()]);

        self.execute_script(script, Some(args)).await
    }

    #[instrument(name = "Browser::hide_body_scrollbar", skip(self), fields(uuid = ?self.uuid))]
    pub async fn hide_body_scrollbar(&mut self) -> Result<()> {
        self.inject_header_styles("html, body { scrollbar-width: none !important; }")
            .await
    }

    #[instrument(name = "Browser::find_element", skip(self), fields(uuid = ?self.uuid))]
    pub async fn find_element<V: Into<String> + Send + Debug>(
        &mut self,
        using: ScreenshotFindElementUsing,
        value: V,
    ) -> Result<webdriver::Element> {
        let element = self
            .marionette
            .send(&webdriver::FindElement::new(
                webdriver::FindElementParameters {
                    using,
                    value: value.into(),
                },
            ))
            .await?;

        Ok(element.value)
    }

    #[instrument(name = "Browser::screenshot_base64", skip(self), fields(uuid = ?self.uuid))]
    pub async fn screenshot_base64(&mut self, parameters: ScreenshotParameters) -> Result<String> {
        let webdriver::TakeScreenshotResponse { base64_png } = self
            .marionette
            .send(&webdriver::TakeScreenshot::new(parameters))
            .await?;

        Ok(base64_png)
    }

    #[instrument(name = "Browser::screenshot_bytes", skip(self), fields(uuid = ?self.uuid))]
    pub async fn screenshot_bytes(&mut self, parameters: ScreenshotParameters) -> Result<Vec<u8>> {
        BASE64_STANDARD
            .decode(self.screenshot_base64(parameters).await?)
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

fn parse_url(url: &str) -> Result<String> {
    match Url::parse(url) {
        Ok(parsed_url) => {
            if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                Ok(parsed_url.into())
            } else {
                Err(Error::UnsupportedUrlProtocol)
            }
        },
        Err(ParseError::RelativeUrlWithoutBase) => parse_url(format!("https://{url}").as_str()),
        Err(error) => Err(Error::ParseUrl(error)),
    }
}
