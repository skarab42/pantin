use std::fmt::Debug;

use pantin_derive::WebDriverCommand;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

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
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NewSessionResponse {
    pub session_id: String,
    pub capabilities: Map<String, Value>,
}

#[derive(Debug, WebDriverCommand)]
pub struct NewSession {
    parameters: NewSessionParameters,
}

// ---

#[derive(Debug, Serialize)]
pub struct ExecuteScriptParameters {
    pub script: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteScriptResponse {
    pub value: (),
}

#[derive(Debug, WebDriverCommand)]
pub struct ExecuteScript {
    parameters: ExecuteScriptParameters,
}

// ---

#[derive(Debug, Serialize)]
pub struct SetWindowRectParameters {
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
pub struct SetWindowRectResponse {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, WebDriverCommand)]
pub struct SetWindowRect {
    parameters: SetWindowRectParameters,
}

// ---

#[derive(Debug, Serialize)]
pub struct NavigateParameters {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct NavigateResponse {
    pub value: (),
}

#[derive(Debug, WebDriverCommand)]
pub struct Navigate {
    parameters: NavigateParameters,
}

// ---

#[derive(Debug, Serialize)]
pub enum FindElementUsing {
    #[serde(rename = "css selector")]
    CssSelector,
    #[serde(rename = "xpath")]
    XPath,
}

#[derive(Debug, Serialize)]
pub struct FindElementParameters {
    pub using: FindElementUsing,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Element {
    #[serde(rename = "element-6066-11e4-a52e-4f735466cecf")]
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct FindElementResponse {
    pub value: Element,
}

#[derive(Debug, WebDriverCommand)]
pub struct FindElement {
    pub parameters: FindElementParameters,
}

// ---

#[must_use]
#[derive(Debug, Serialize)]
pub struct TakeScreenshotParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    full: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "id")]
    element_id: Option<String>,
}

impl TakeScreenshotParameters {
    pub const fn new(full: Option<bool>, element_id: Option<String>) -> Self {
        Self { full, element_id }
    }

    pub const fn full() -> Self {
        Self::new(Some(true), None)
    }

    pub const fn viewport() -> Self {
        Self::new(Some(false), None)
    }

    pub const fn element(id: String) -> Self {
        Self::new(Some(false), Some(id))
    }
}

#[derive(Debug, Deserialize)]
pub struct TakeScreenshotResponse {
    #[serde(rename = "value")]
    pub base64_png: String,
}

#[derive(Debug, WebDriverCommand)]
pub struct TakeScreenshot {
    pub parameters: TakeScreenshotParameters,
}
