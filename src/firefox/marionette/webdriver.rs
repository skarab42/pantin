use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::net::TcpStream;

use crate::firefox::marionette::request;

pub type NewSessionCapabilities = Map<String, Value>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSession {
    pub session_id: String,
    pub capabilities: NewSessionCapabilities,
}

impl NewSession {
    pub async fn send(
        stream: &mut TcpStream,
        capabilities: Option<&NewSessionCapabilities>,
    ) -> request::Result<Self> {
        request::send(stream, "WebDriver:NewSession", capabilities).await
    }
}

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
