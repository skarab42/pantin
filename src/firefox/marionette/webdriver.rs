use serde::Deserialize;
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
    pub async fn send(stream: &mut TcpStream) -> request::Result<Self> {
        request::send(stream, "WebDriver:NewSession", ()).await
    }
}
