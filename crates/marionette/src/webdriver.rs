//! Module for [WebDriver](https://www.w3.org/TR/webdriver2/) commands used to interact with a browser session.
//!
//! This module defines a trait for `WebDriver` commands and several concrete command types,
//! including commands to create a new session, execute scripts, set the window rectangle,
//! navigate to a URL, find an element, and take a screenshot.
//!
//! Each command is annotated with the [`WebDriverCommand`] derive macro,
//! which automates boilerplate code for serializing and deserializing the command messages.

use std::fmt::Debug;

use pantin_derive::WebDriverCommand;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

/// A trait representing a `WebDriver` command.
///
/// - Each command has associated parameters and an expected response type.
/// - The command name is defined as a static string and the parameters are serializable.
pub trait Command {
    type Parameters: Serialize + Sync;
    type Response: DeserializeOwned + Debug;

    fn name(&self) -> &'static str;
    fn parameters(&self) -> &Self::Parameters;
}

// --- NewSession command types ---

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

// --- ExecuteScript command types ---

#[derive(Debug, Serialize)]
pub struct ExecuteScriptParameters {
    pub script: String,
    pub args: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteScriptResponse {
    pub value: Value,
}

#[derive(Debug, WebDriverCommand)]
pub struct ExecuteScript {
    parameters: ExecuteScriptParameters,
}

// --- SetWindowRect command types ---

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

// --- Navigate command types ---

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

// --- FindElement command types ---

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

// --- TakeScreenshot command types ---

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

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let command = NewSession::new(Some(Map::new()));

        assert_eq!(command.name(), "WebDriver:NewSession");
        assert_eq!(command.parameters(), &Some(Map::new()));

        let json_data = r#"
        {
            "sessionId": "abc123",
            "capabilities": {
                "browserName": "firefox",
                "version": "85.0"
            }
        }
        "#;
        let response: NewSessionResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");
        let capabilities = response.capabilities;

        assert_eq!(response.session_id, "abc123");
        assert_eq!(
            capabilities.get("browserName").unwrap(),
            &Value::String("firefox".to_string())
        );
        assert_eq!(
            capabilities.get("version").unwrap(),
            &Value::String("85.0".to_string())
        );
    }

    #[test]
    fn test_execute_script() {
        let command = ExecuteScript::new(ExecuteScriptParameters {
            script: "return 42;".to_string(),
            args: vec![],
        });

        assert_eq!(command.name(), "WebDriver:ExecuteScript");
        assert_eq!(command.parameters().script, "return 42;");
        assert!(command.parameters().args.is_empty());

        let json_data = r#"{"value":42}"#;
        let response: ExecuteScriptResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");

        assert_eq!(response.value, 42);
    }

    #[test]
    fn test_execute_set_window_rect() {
        let command = SetWindowRect::new(SetWindowRectParameters {
            x: Some(100),
            y: Some(200),
            width: Some(800),
            height: Some(600),
        });

        assert_eq!(command.name(), "WebDriver:SetWindowRect");
        assert_eq!(command.parameters().x, Some(100));
        assert_eq!(command.parameters().y, Some(200));
        assert_eq!(command.parameters().width, Some(800));
        assert_eq!(command.parameters().height, Some(600));

        let json_data = r#"
        {
            "x": 100,
            "y": 200,
            "width": 800,
            "height": 600
        }
        "#;
        let response: SetWindowRectResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");

        assert_eq!(response.x, 100);
        assert_eq!(response.y, 200);
        assert_eq!(response.width, 800);
        assert_eq!(response.height, 600);
    }

    #[test]
    #[allow(clippy::unit_cmp)]
    fn test_navigate() {
        let command = Navigate::new(NavigateParameters {
            url: "http://example.com".into(),
        });

        assert_eq!(command.name(), "WebDriver:Navigate");
        assert_eq!(command.parameters().url, "http://example.com");

        let json_data = r#"{"value":null}"#;
        let response: NavigateResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");

        assert_eq!(response.value, ());
    }

    #[test]
    fn test_find_element_using_css_selector() {
        let command = FindElement::new(FindElementParameters {
            using: FindElementUsing::CssSelector,
            value: "#my-element".into(),
        });

        assert_eq!(command.name(), "WebDriver:FindElement");
        assert!(
            matches!(command.parameters().using, FindElementUsing::CssSelector),
            "Expected FindElementUsing::CssSelector, got: {:?}",
            command.parameters().using
        );
        assert_eq!(command.parameters().value, "#my-element".to_string());

        let json_data = r#"{"value":{"element-6066-11e4-a52e-4f735466cecf":"element-id-test"}}"#;
        let response: FindElementResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");

        assert_eq!(response.value.id, "element-id-test");
    }

    #[test]
    fn test_find_element_using_xpath() {
        let command = FindElement::new(FindElementParameters {
            using: FindElementUsing::XPath,
            value: "//div".into(),
        });

        assert_eq!(command.name(), "WebDriver:FindElement");
        assert!(
            matches!(command.parameters().using, FindElementUsing::XPath),
            "Expected FindElementUsing::CssSelector, got: {:?}",
            command.parameters().using
        );
        assert_eq!(command.parameters().value, "//div".to_string());

        let json_data = r#"{"value":{"element-6066-11e4-a52e-4f735466cecf":"element-id-test"}}"#;
        let response: FindElementResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");

        assert_eq!(response.value.id, "element-id-test");
    }

    #[test]
    fn test_take_screenshot_full() {
        let command = TakeScreenshot::new(TakeScreenshotParameters::full());

        assert_eq!(command.name(), "WebDriver:TakeScreenshot");
        assert_eq!(command.parameters().full, Some(true));
        assert!(command.parameters().element_id.is_none());
    }

    #[test]
    fn test_take_screenshot_viewport() {
        let command = TakeScreenshot::new(TakeScreenshotParameters::viewport());

        assert_eq!(command.name(), "WebDriver:TakeScreenshot");
        assert_eq!(command.parameters().full, Some(false));
        assert!(command.parameters().element_id.is_none());
    }

    #[test]
    fn test_take_screenshot_element() {
        let command = TakeScreenshot::new(TakeScreenshotParameters::element("element-42".into()));

        assert_eq!(command.name(), "WebDriver:TakeScreenshot");
        assert_eq!(command.parameters().full, Some(false));
        assert_eq!(command.parameters().element_id, Some("element-42".into()));
    }

    #[test]
    fn test_take_screenshot_response() {
        let json_data = r#"{"value":"some-base64-encoded-string..."}"#;
        let response: TakeScreenshotResponse =
            serde_json::from_str(json_data).expect("Deserialization should succeed");

        assert_eq!(response.base64_png, "some-base64-encoded-string...");
    }
}
