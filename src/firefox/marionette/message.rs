use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;

use crate::firefox::marionette::command;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSession {
    pub session_id: String,
}

impl NewSession {
    pub async fn send(stream: &mut TcpStream) -> command::Result<Self> {
        command::send(stream, "NewSession", json!({})).await
    }
}
