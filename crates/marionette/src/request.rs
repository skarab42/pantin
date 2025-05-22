//! Module for sending [`Command`] requests over a TCP stream and receiving [`Response`](response::Response).
//!
//! This module provides functions to write command requests to a TCP stream and send them, waiting for the corresponding response.
//! It serializes commands to JSON with a length prefix and expects the response to include an identifier matching the request.

use std::{fmt::Debug, io, result};

use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::debug;

use crate::{command::Command, response};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("convert message to json failed: {0}")]
    ConvertJson(#[source] serde_json::Error),
    #[error("write request to stream failed")]
    FailedToWriteRequest(#[source] io::Error),
    #[error("command id mismatch: expected {request_id}, got: {response_id}")]
    CommandIdMismatch { request_id: u32, response_id: u32 },
    #[error(transparent)]
    Response(#[from] response::Error),
}

pub type Result<T, E = Error> = result::Result<T, E>;

/// Writes a command request to the provided TCP stream.
///
/// This function creates a new request using [`Command::new_request`], serializes it to JSON,
/// and writes the message with a length prefix to the TCP stream. The message format is
/// `"length:JSON"`, where `length` is the number of bytes in the JSON payload.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the TCP stream.
/// * `command` - The command name, convertible into a [`String`].
/// * `data` - The data to be sent with the command, which must implement [`Serialize`].
///
/// # Returns
///
/// Returns the unique command ID as [`u32`] if the write operation succeeds.
///
/// # Errors
///
/// Returns an [`Error::ConvertJson`] if JSON conversion fails or an [`Error::FailedToWriteRequest`]
/// if writing to the stream fails.
pub async fn write<S, C, D>(stream: &mut S, command: C, data: &D) -> Result<u32>
where
    S: AsyncRead + AsyncWrite + Unpin,
    C: Into<String> + Send,
    D: Serialize + Send + Sync,
{
    let request = Command::new_request(command, data);
    let body = serde_json::to_string(&request).map_err(Error::ConvertJson)?;
    let data = format!("{}:{}", body.len(), body);

    debug!(?data, "Write request");

    stream
        .write(data.as_bytes())
        .await
        .map_err(Error::FailedToWriteRequest)?;

    Ok(request.id())
}

/// Sends a command request and waits for its corresponding response.
///
/// This function writes a command request using [`write`], then reads and parses the response
/// from the TCP stream using functions from the [`response`] module. It verifies that the response's
/// command ID matches the request's command ID before returning the parsed response.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the TCP stream.
/// * `command` - The command name, convertible into a [`String`].
/// * `data` - The data to be sent with the command, which must implement [`Serialize`].
///
/// # Returns
///
/// Returns the parsed response of type `T` if the request-response cycle succeeds.
///
/// # Errors
///
/// Returns an [`Error`] if writing the request fails, reading or parsing the response fails,
/// or if there is a mismatch between the command IDs in the request and response.
pub async fn send<S, C, D, T>(stream: &mut S, command: C, data: &D) -> Result<T>
where
    S: AsyncRead + AsyncWrite + Unpin,
    C: Into<String> + Send,
    D: Serialize + Send + Sync,
    T: DeserializeOwned + Debug,
{
    let request_id = write(stream, command, data).await?;
    let json_string = response::read(stream).await?;
    let (response_id, response) = response::parse(json_string)?;

    if request_id == response_id {
        Ok(response)
    } else {
        Err(Error::CommandIdMismatch {
            request_id,
            response_id,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use serde_json::Value;
    use tokio::io::{AsyncReadExt, duplex};

    use super::*;

    #[tokio::test]
    async fn test_write() {
        let (mut client, mut server) = duplex(1024);

        let command_id = write(&mut client, "test-write", &42)
            .await
            .expect("Write should succeed");

        client.shutdown().await.expect("Client shutdown");

        let mut buf = Vec::new();
        server.read_to_end(&mut buf).await.expect("Server read");
        let msg = String::from_utf8(buf).expect("Valid utf8");

        let parts: Vec<&str> = msg.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2, "Message should contain a colon separator");

        let length: usize = parts[0].parse().expect("Length prefix should be a number");
        assert_eq!(
            length,
            parts[1].len(),
            "Length prefix must match JSON body length"
        );

        let expected_json = format!("[0,{command_id},\"test-write\",42]",);
        assert_eq!(parts[1], expected_json);
    }

    #[tokio::test]
    async fn test_write_error() {
        let (mut client, _server) = duplex(1024);

        client.shutdown().await.expect("Client shutdown");

        let command_id = write(&mut client, "test-write-error", &42).await;

        assert!(
            matches!(command_id, Err(Error::FailedToWriteRequest(_))),
            "Expected FailedToWriteRequest"
        );
    }

    #[tokio::test]
    async fn test_send_success() {
        let (mut client, mut server) = duplex(1024);

        tokio::spawn(async move {
            let req_msg = response::read(&mut server).await.expect("Server read");
            let req_json: Value = serde_json::from_str(&req_msg).expect("Valid JSON");
            let req_id = req_json[1].as_u64().expect("Valid id");

            let response_json = serde_json::json!([0, req_id, null, "ok"]);
            let body = serde_json::to_string(&response_json).expect("Serialize response");
            let response_msg = format!("{}:{}", body.len(), body);

            server
                .write_all(response_msg.as_bytes())
                .await
                .expect("Server write");
            server.shutdown().await.expect("Server shutdown");
        });

        let result: String = send(&mut client, "dummy_cmd", &123)
            .await
            .expect("send should succeed");
        assert_eq!(result, "ok", "Response should be 'ok'");
    }

    #[tokio::test]
    async fn test_send_command_id_mismatch() {
        let (mut client, mut server) = duplex(1024);

        tokio::spawn(async move {
            let req_msg = response::read(&mut server).await.expect("Server read");
            let req_json: Value = serde_json::from_str(&req_msg).expect("Valid JSON");
            let req_id = req_json[1].as_u64().expect("Valid id");

            let response_json = serde_json::json!([0, req_id + 1, null, "mismatch"]);
            let body = serde_json::to_string(&response_json).expect("Serialize response");
            let response_msg = format!("{}:{}", body.len(), body);

            server
                .write_all(response_msg.as_bytes())
                .await
                .expect("Server write");
            server.shutdown().await.expect("Server shutdown");
        });

        let result: Result<String> = send(&mut client, "dummy_cmd", &123).await;
        match result {
            Err(Error::CommandIdMismatch {
                request_id,
                response_id,
            }) => {
                assert_eq!(
                    response_id,
                    request_id + 1,
                    "Response id should be request id + 1"
                );
            },
            _ => panic!("Expected CommandIdMismatch error"),
        }
    }
}
