use std::result;

use serde::Deserialize;
use thiserror::Error;
use tokio::net::TcpStream;
use tracing::{debug, error};

use crate::response;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseResponse(#[from] response::Error),
    #[error("expected application type 'gecko', got: {0}")]
    UnexpectedApplicationType(String),
    #[error("expected marionette protocol version 3, got: {0}")]
    UnexpectedMarionetteProtocolVersion(u8),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::module_name_repetitions)]
pub struct HandshakeResponse {
    pub marionette_protocol: u8,
    pub application_type: String,
}

impl HandshakeResponse {
    pub async fn read(stream: &mut TcpStream) -> Result<Self> {
        debug!("Reading Handshake...");
        let json = response::read(stream).await?;
        let handshake: Self = response::parse_raw(json)?;
        debug!(response = ?handshake, "Got response");

        if handshake.application_type != "gecko" {
            return Err(Error::UnexpectedApplicationType(handshake.application_type));
        }

        if handshake.marionette_protocol != 3 {
            return Err(Error::UnexpectedMarionetteProtocolVersion(
                handshake.marionette_protocol,
            ));
        }

        Ok(handshake)
    }
}
