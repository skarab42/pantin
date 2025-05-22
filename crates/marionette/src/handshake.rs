//! This module implements the handshake procedure for Marionette.
//!
//! It reads a handshake message from a TCP stream using the Marionette JSON response format
//! and verifies that the response contains the expected values.

use std::result;

use serde::Deserialize;
use thiserror::Error;
use tokio::io::AsyncRead;
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

/// Represents a handshake message received from the Marionette server.
///
/// The handshake message is deserialized from JSON and contains:
///
/// - `marionette_protocol`: the version of the Marionette protocol, expected to be `3`.
/// - `application_type`: the type of the application, expected to be `"gecko"`.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Handshake {
    pub marionette_protocol: u8,
    pub application_type: String,
}

impl Handshake {
    /// Reads and parses a handshake message from the given stream.
    ///
    /// This function:
    /// 1. Reads a JSON-formatted response from the stream using [`response::read`].
    /// 2. Parses the JSON into a [`Handshake`] using [`response::parse_raw`].
    /// 3. Verifies that `application_type` is `"gecko"` and `marionette_protocol` equals 3.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to a [`TcpStream`] from which the handshake message is read.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - Parsing the response fails.
    /// - The `application_type` is not `"gecko"`.
    /// - The `marionette_protocol` is not 3.
    pub async fn read<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Self> {
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

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use tokio::io::{AsyncWriteExt, duplex};

    use super::*;

    fn format_message(body: &str) -> String {
        format!("{}:{}", body.len(), body)
    }

    #[tokio::test]
    async fn test_handshake_read_success() {
        let (mut client, mut server) = duplex(1024);
        let json = r#"{"marionetteProtocol":3,"applicationType":"gecko"}"#;
        let message = format_message(json);

        tokio::spawn(async move {
            server.write_all(message.as_bytes()).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let handshake = Handshake::read(&mut client)
            .await
            .expect("Expected valid handshake");
        assert_eq!(handshake.marionette_protocol, 3);
        assert_eq!(handshake.application_type, "gecko");
    }

    #[tokio::test]
    async fn test_handshake_unexpected_application_type() {
        let json = r#"{"marionetteProtocol":3,"applicationType":"not-gecko"}"#;
        let message = format_message(json);
        let (mut client, mut server) = duplex(1024);

        tokio::spawn(async move {
            server.write_all(message.as_bytes()).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let error = Handshake::read(&mut client)
            .await
            .expect_err("Expected an UnexpectedApplicationType error");
        match error {
            Error::UnexpectedApplicationType(app) => assert_eq!(app, "not-gecko"),
            _ => panic!("Expected UnexpectedApplicationType error, got {error:?}"),
        }
    }

    #[tokio::test]
    async fn test_handshake_unexpected_marionette_protocol() {
        let (mut client, mut server) = duplex(1024);
        let json = r#"{"marionetteProtocol":2,"applicationType":"gecko"}"#;
        let message = format_message(json);

        tokio::spawn(async move {
            server.write_all(message.as_bytes()).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let error = Handshake::read(&mut client)
            .await
            .expect_err("Expected an UnexpectedMarionetteProtocolVersion error");
        match error {
            Error::UnexpectedMarionetteProtocolVersion(version) => assert_eq!(version, 2),
            _ => panic!("Expected UnexpectedMarionetteProtocolVersion error, got {error:?}",),
        }
    }
}
