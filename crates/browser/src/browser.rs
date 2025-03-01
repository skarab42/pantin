//! Crate for controlling a Firefox browser instance with a temporary profile.
//!
//! This crate integrates the functionality provided by [`pantin_process`](https://github.com/skarab42/pantin/crates/process),
//! [`pantin_marionette`](https://github.com/skarab42/pantin/crates/marionette) and the profile management from the [`profile`] module.
//! It offers a unified interface to launch, control, and close a Firefox browser using a temporary profile,
//! automatically cleaning up resources on drop.

use std::{ffi::OsStr, fmt::Debug, result};

use base64::{DecodeError, Engine, prelude::BASE64_STANDARD};
use pantin_marionette::{Marionette, webdriver};
use pantin_process::{Process, Status};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, error, instrument};
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
    SerdeJson(#[from] serde_json::Error),
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

/// Alias for the element finding strategy used when taking a screenshot.
pub type ScreenshotFindElementUsing = webdriver::FindElementUsing;

/// Alias for the screenshot parameters.
pub type ScreenshotParameters = webdriver::TakeScreenshotParameters;

/// Represents a controlled Firefox browser instance with a temporary profile.
///
/// This struct wraps a temporary Firefox profile, a process managing the browser,
/// and a Marionette connection to control the browser remotely,
/// ensuring cleaning up resources on drop.
#[derive(Debug)]
pub struct Browser {
    uuid: Uuid,
    profile: Profile,
    process: Process,
    marionette: Marionette,
}

impl Browser {
    /// Creates a new Browser instance using a given UUID and program.
    ///
    /// This function launches Firefox in headless mode with the necessary flags, creates a temporary
    /// profile, and establishes a Marionette connection.
    ///
    /// # Arguments
    ///
    /// * `uuid` - Unique identifier for the browser instance.
    /// * `program` - The path to the Firefox executable.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if profile creation, process spawning or Marionette initialization fails.
    #[instrument(name = "Browser::new")]
    pub async fn new<P>(uuid: Uuid, program: P) -> Result<Self>
    where
        P: AsRef<OsStr> + Debug + Send,
    {
        debug!("Opening a new Browser instance...");
        let profile = Profile::new().await?;
        let process = Process::spawn(
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

    /// Opens a new Browser instance with a randomly generated UUID.
    ///
    /// # Arguments
    ///
    /// * `program` - The path to the Firefox executable.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if profile creation, process spawning or Marionette initialization fails.
    pub async fn open<P>(program: P) -> Result<Self>
    where
        P: AsRef<OsStr> + Debug + Send,
    {
        Self::new(Uuid::new_v4(), program).await
    }

    /// Returns the unique identifier of the browser instance.
    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the process ID of the Firefox process, if available.
    pub fn pid(&self) -> Option<u32> {
        self.process.id()
    }

    /// Returns the current Marionette session ID.
    pub fn sid(&self) -> &str {
        self.marionette.session_id()
    }

    /// Returns the current status of the Firefox process.
    pub fn status(&mut self) -> Status {
        self.process.status()
    }

    /// Set the browser window size.
    ///
    /// # Arguments
    ///
    /// * `width` - The desired window width.
    /// * `height` - The desired window height.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the resize operation fails.
    #[instrument(name = "Browser::set_window_size", skip(self), fields(uuid = ?self.uuid))]
    pub async fn set_window_size(&mut self, width: u16, height: u16) -> Result<(u16, u16)> {
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

    /// Navigates the browser to the specified URL.
    ///
    /// The URL is parsed and validated to ensure it uses either http or https.
    ///
    /// # Arguments
    ///
    /// * `url` - The target URL to navigate to.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if URL parsing or the navigation command fails.
    #[instrument(name = "Browser::navigate", skip(self), fields(uuid = ?self.uuid))]
    pub async fn navigate<U: Into<String> + Send + Debug>(&mut self, url: U) -> Result<()> {
        self.marionette
            .send(&webdriver::Navigate::new(webdriver::NavigateParameters {
                url: parse_url(url.into().as_str())?,
            }))
            .await?;

        Ok(())
    }

    /// Executes a JavaScript script in the context of the browser.
    ///
    /// # Arguments
    ///
    /// * `script` - The JavaScript code to execute.
    /// * `args` - Optional arguments to pass to the script.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the script execution fails.
    #[instrument(name = "Browser::execute_script", skip(self), fields(uuid = ?self.uuid))]
    pub async fn execute_script<S: Into<String> + Send + Debug>(
        &mut self,
        script: S,
        args: Option<Vec<Value>>,
    ) -> Result<Value> {
        let response = self
            .marionette
            .send(&webdriver::ExecuteScript::new(
                webdriver::ExecuteScriptParameters {
                    script: script.into(),
                    args: args.unwrap_or_default(),
                },
            ))
            .await?;

        Ok(response.value)
    }

    /// Injects CSS styles into the document header.
    ///
    /// Useful for modifying the appearance of the page (e.g., hiding scrollbars).
    ///
    /// # Arguments
    ///
    /// * `styles` - The CSS styles to inject.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the injection fails.
    #[instrument(name = "Browser::inject_header_styles", skip(self), fields(uuid = ?self.uuid))]
    pub async fn inject_header_styles<S: Into<String> + Debug>(
        &mut self,
        styles: S,
    ) -> Result<Value> {
        let script = "
            let style = document.createElement('style');
            style.innerHTML = arguments[0];
            document.head.appendChild(style);
        ";
        let args = Vec::from([Value::from(styles.into())]);

        self.execute_script(script, Some(args)).await
    }

    /// Get the browser window size to match viewport size.
    ///
    /// # Arguments
    ///
    /// * `width` - The desired window width.
    /// * `height` - The desired window height.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the resize operation fails.
    #[instrument(name = "Browser::get_window_to_viewport_size", skip(self), fields(uuid = ?self.uuid))]
    pub async fn get_window_to_viewport_size(
        &mut self,
        width: u16,
        height: u16,
    ) -> Result<(u16, u16)> {
        let script = "
            let [width, height] = arguments;
            return [
                width + (window.outerWidth - window.innerWidth),
                height + (window.outerHeight - window.innerHeight)
            ];
        ";
        let args = Vec::from([Value::from(width), Value::from(height)]);

        serde_json::from_value(self.execute_script(script, Some(args)).await?)
            .map_err(Error::SerdeJson)
    }

    /// Set the browser viewport size.
    ///
    /// # Arguments
    ///
    /// * `width` - The desired viewport width.
    /// * `height` - The desired viewport height.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the resize operation fails.
    #[instrument(name = "Browser::set_viewport_size", skip(self), fields(uuid = ?self.uuid))]
    pub async fn set_viewport_size(&mut self, width: u16, height: u16) -> Result<(u16, u16)> {
        let (window_width, window_height) = self.get_window_to_viewport_size(width, height).await?;

        self.set_window_size(window_width, window_height).await?;

        Ok((width, height))
    }

    /// Hides the browser's scrollbar by injecting custom CSS.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the operation fails.
    #[instrument(name = "Browser::hide_body_scrollbar", skip(self), fields(uuid = ?self.uuid))]
    pub async fn hide_body_scrollbar(&mut self) -> Result<Value> {
        self.inject_header_styles("html, body { scrollbar-width: none !important; }")
            .await
    }

    /// Finds an element on the page using the specified strategy and value.
    ///
    /// # Arguments
    ///
    /// * `using` - The element-finding strategy.
    /// * `value` - The value to search for.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the element cannot be found.
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

    /// Takes a screenshot and returns it as a Base64-encoded string.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameters to customize the screenshot.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the screenshot command fails.
    #[instrument(name = "Browser::screenshot_base64", skip(self), fields(uuid = ?self.uuid))]
    pub async fn screenshot_base64(&mut self, parameters: ScreenshotParameters) -> Result<String> {
        let webdriver::TakeScreenshotResponse { base64_png } = self
            .marionette
            .send(&webdriver::TakeScreenshot::new(parameters))
            .await?;

        Ok(base64_png)
    }

    /// Takes a screenshot and returns the image as a byte vector.
    ///
    /// This method decodes the Base64-encoded screenshot.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameters to customize the screenshot.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if decoding the screenshot fails.
    #[instrument(name = "Browser::screenshot_bytes", skip(self), fields(uuid = ?self.uuid))]
    pub async fn screenshot_bytes(&mut self, parameters: ScreenshotParameters) -> Result<Vec<u8>> {
        BASE64_STANDARD
            .decode(self.screenshot_base64(parameters).await?)
            .map_err(Error::DecodeScreenshot)
    }

    /// Closes the browser instance.
    ///
    /// This method attempts to kill the Firefox process if it is still alive,
    /// waits briefly (on Windows) for the process to terminate, and then removes the temporary profile.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the process termination or profile removal fails.
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

/// Parses and validates a URL string, ensuring that only HTTP and HTTPS protocols are allowed.
///
/// If the URL is relative (without a base), it prepends "https://" and retries parsing.
///
/// # Arguments
///
/// * `url` - The URL string to parse.
///
/// # Errors
///
/// Returns an [`Error`] if parsing fails or the URL protocol is unsupported.
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

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use image::GenericImageView;

    use super::*;

    #[test]
    fn test_parse_url_valid() {
        let url = parse_url("http://example.com").expect("Should parse http url");
        assert_eq!(url, "http://example.com/");

        let url = parse_url("https://example.com").expect("Should parse https url");
        assert_eq!(url, "https://example.com/");

        let url = parse_url("example.com").expect("Should parse relative url");
        assert_eq!(url, "https://example.com/");
    }

    #[test]
    fn test_parse_url_invalid() {
        let err = parse_url("about:config").unwrap_err();
        match err {
            Error::UnsupportedUrlProtocol => {},
            _ => panic!("Expected UnsupportedUrlProtocol error"),
        }

        let err = parse_url("file://filename.ext").unwrap_err();
        match err {
            Error::UnsupportedUrlProtocol => {},
            _ => panic!("Expected UnsupportedUrlProtocol error"),
        }

        let err = parse_url("not a valid url").unwrap_err();
        match err {
            Error::ParseUrl(_) => {},
            _ => panic!("Expected ParseUrl error"),
        }
    }

    #[tokio::test]
    async fn test_browser_open_and_close() {
        let mut browser = Browser::open("firefox").await.expect("Opening browser");

        assert!(!browser.uuid().is_nil(), "Browser UUID should not be nil");

        assert!(
            browser.pid().is_some(),
            "Browser process ID should be available"
        );

        assert!(
            !browser.sid().is_empty(),
            "Marionette session ID should not be empty"
        );

        assert!(
            matches!(browser.status(), Status::Alive),
            "Browser status should be alive before close"
        );

        let status = browser.close().await.expect("Closing browser");

        assert!(
            matches!(status, Status::Exited(_)),
            "Browser status should be exited after close"
        );
    }

    #[tokio::test]
    async fn test_browser_resize() {
        let mut browser = Browser::open("firefox").await.expect("Opening browser");

        let (width, height) = browser
            .set_window_size(800, 600)
            .await
            .expect("Resize failed");
        assert_eq!(width, 800);
        assert_eq!(height, 600);

        let (width, height) = browser
            .set_window_size(1024, 720)
            .await
            .expect("Resize failed");
        assert_eq!(width, 1024);
        assert_eq!(height, 720);

        browser.close().await.expect("Closing browser");
    }

    #[tokio::test]
    async fn test_browser_navigate_and_execute_script() {
        let mut browser = Browser::open("firefox").await.expect("Opening browser");

        browser
            .navigate("https://www.infomaniak.com")
            .await
            .expect("Navigation failed");

        let value = browser
            .execute_script("return document.location.hostname", None)
            .await
            .expect("Script execution failed");

        assert_eq!(value, "www.infomaniak.com");

        browser.close().await.expect("Closing browser");
    }

    #[tokio::test]
    async fn test_browser_screenshot() {
        let mut browser = Browser::open("firefox").await.expect("Opening browser");

        let screenshot_base64 = browser
            .screenshot_base64(webdriver::TakeScreenshotParameters::viewport())
            .await
            .expect("Screenshot base64 failed");
        assert!(
            !screenshot_base64.is_empty(),
            "Screenshot (base64) should not be empty"
        );

        let (expected_width, expected_height) = browser
            .set_viewport_size(1042, 742)
            .await
            .expect("Resize failed");
        assert_eq!(expected_width, 1042, "Viewport width mismatch");
        assert_eq!(expected_height, 742, "Viewport height mismatch");

        let screenshot_bytes = browser
            .screenshot_bytes(webdriver::TakeScreenshotParameters::viewport())
            .await
            .expect("Screenshot bytes failed");
        assert!(
            !screenshot_bytes.is_empty(),
            "Screenshot (bytes) should not be empty"
        );

        let format = image::guess_format(&screenshot_bytes).expect("Failed to guess image format");
        assert_eq!(
            format,
            image::ImageFormat::Png,
            "The screenshot is not in PNG format"
        );

        let img = image::load_from_memory(&screenshot_bytes).expect("Failed to decode PNG image");
        let (width, height) = img.dimensions();
        assert_eq!(width, 1042, "Image width mismatch");
        assert_eq!(height, 742, "Image height mismatch");

        browser.close().await.expect("Closing browser");
    }
}
