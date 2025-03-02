//! Module for sending [`Command`] requests over a TCP stream and receiving [`Response`](response::Response).
//!
//! This module provides functions to write command requests to a TCP stream and send them, waiting for the corresponding response.
//! It serializes commands to JSON with a length prefix and expects the response to include an identifier matching the request.

use std::{fmt::Debug, io, result};

use serde::{Serialize, de::DeserializeOwned};
use tokio::{io::AsyncWriteExt, net::TcpStream};
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
pub async fn write<C, D>(stream: &mut TcpStream, command: C, data: &D) -> Result<u32>
where
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
pub async fn send<C, D, T>(stream: &mut TcpStream, command: C, data: &D) -> Result<T>
where
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
