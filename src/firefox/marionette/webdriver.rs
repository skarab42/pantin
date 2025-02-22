use std::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::net::TcpStream;

use crate::firefox::marionette::request;

pub trait Command {
    type Parameters: Serialize + Sync;
    type Response: DeserializeOwned + Debug;

    fn name(&self) -> &'static str;
    fn parameters(&self) -> &Self::Parameters;
}

// ---

pub type NewSessionCapabilities = Map<String, Value>;
pub type NewSessionParameters = Option<NewSessionCapabilities>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionResponse {
    pub session_id: String,
    pub capabilities: Map<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct NewSession {
    parameters: NewSessionParameters,
}

impl NewSession {
    #[must_use]
    pub const fn new(parameters: NewSessionParameters) -> Self {
        Self { parameters }
    }
}

impl Command for NewSession {
    type Parameters = NewSessionParameters;
    type Response = NewSessionResponse;

    fn name(&self) -> &'static str {
        "WebDriver:NewSession"
    }

    fn parameters(&self) -> &Self::Parameters {
        &self.parameters
    }
}

// ---

#[derive(Debug, Serialize)]
pub struct WindowRect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct SetWindowRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl SetWindowRect {
    pub async fn send(stream: &mut TcpStream, window_rect: &WindowRect) -> request::Result<Self> {
        request::send(stream, "WebDriver:SetWindowRect", window_rect).await
    }
}

#[derive(Debug, Serialize)]
pub struct NavigateLocation {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Navigate {
    pub value: (),
}

impl Navigate {
    pub async fn send(
        stream: &mut TcpStream,
        location: &NavigateLocation,
    ) -> request::Result<Self> {
        request::send(stream, "WebDriver:Navigate", location).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FindElementUsing {
    CssSelector,
    XPath,
}

impl From<FindElementUsing> for String {
    fn from(value: FindElementUsing) -> Self {
        match value {
            FindElementUsing::CssSelector => "css selector".to_string(),
            FindElementUsing::XPath => "xpath".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Element {
    #[serde(rename = "element-6066-11e4-a52e-4f735466cecf")]
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct FindElementSettings {
    pub using: FindElementUsing,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct FindElement {
    pub value: Element,
}

impl FindElement {
    pub async fn send(
        stream: &mut TcpStream,
        settings: &FindElementSettings,
    ) -> request::Result<Self> {
        request::send(stream, "WebDriver:FindElement", settings).await
    }
}

#[must_use]
#[derive(Debug, Serialize, Deserialize)]
pub struct TakeScreenshotOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    full: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scroll: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    element_id: Option<String>,
}

impl TakeScreenshotOptions {
    pub const fn new(full: Option<bool>, scroll: Option<bool>, element_id: Option<String>) -> Self {
        Self {
            full,
            scroll,
            element_id,
        }
    }

    pub const fn full() -> Self {
        Self::new(Some(true), Some(false), None)
    }

    pub const fn viewport() -> Self {
        Self::new(Some(false), Some(false), None)
    }

    pub const fn element(element_id: String, scroll: Option<bool>) -> Self {
        Self::new(Some(false), scroll, Some(element_id))
    }
}

#[derive(Debug, Deserialize)]
pub struct TakeScreenshot {
    #[serde(rename = "value")]
    pub base64_png: String,
}

impl TakeScreenshot {
    pub async fn send(
        stream: &mut TcpStream,
        options: &TakeScreenshotOptions,
    ) -> request::Result<Self> {
        request::send(stream, "WebDriver:TakeScreenshot", options).await
    }
}
