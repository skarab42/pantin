//! Module for reading and parsing responses from the Marionette server over a TCP stream.
//!
//! This module provides functions to read raw responses asynchronously,
//! convert them into UTF-8 strings, and parse them into strongly-typed responses.
//!
//! It also defines error types and structures to handle command failures.

use std::{fmt::Debug, io, result, string};

use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum Error {
    #[error("read byte count failed")]
    ReadByteCount(#[source] io::Error),
    #[error("expected one byte, got: {count}")]
    ReadByteCountLength { count: usize },
    #[error("expected byte: {byte}")]
    UnexpectedByte { byte: char },
    #[error("unexpected end of response")]
    UnexpectedEndOfResponse,
    #[error("read byte failed")]
    ReadByte(#[source] io::Error),
    #[error("convert to UTF-8 string failed")]
    ResponseToString(#[source] string::FromUtf8Error),
    #[error("parse response failed")]
    ParseResponse(#[source] serde_json::Error),
    #[error("command (id={0}) failed: {1:?}")]
    CommandFailure(u32, Failure),
}

pub type Result<T, E = Error> = result::Result<T, E>;

/// Reads the length prefix of a message from the TCP stream.
///
/// The length is encoded as ASCII digits terminated by a colon (`:`). This function reads bytes
/// until it encounters the colon and returns the accumulated length as a [`usize`].
///
/// # Errors
///
/// Returns an [`Error`] if the reading fails or an unexpected byte is encountered.
async fn read_length(stream: &mut TcpStream) -> Result<usize> {
    let mut bytes = 0usize;

    loop {
        let buffer = &mut [0u8];
        let byte_count = stream.read(buffer).await.map_err(Error::ReadByteCount)?;
        let byte = match byte_count {
            1 => Ok(buffer[0]),
            0 => Err(Error::UnexpectedEndOfResponse),
            count => Err(Error::ReadByteCountLength { count }),
        }? as char;

        match byte {
            '0'..='9' => {
                bytes *= 10;
                bytes += byte as usize - '0' as usize;
            },
            ':' => break,
            byte => return Err(Error::UnexpectedByte { byte }),
        }
    }

    Ok(bytes)
}

/// Reads a string of a given length from the TCP stream.
///
/// This function continuously reads from the stream until it has read the specified number of bytes,
/// then converts the byte vector into a UTF-8 string.
///
/// # Arguments
///
/// * `bytes` - The number of bytes to read.
///
/// # Errors
///
/// Returns an [`Error`] if reading fails or the conversion to UTF-8 fails.
async fn read_string(stream: &mut TcpStream, bytes: usize) -> Result<String> {
    let mut total_byte_read = 0;
    let buffer = &mut [0u8; 8192];
    let mut payload = Vec::with_capacity(bytes);

    while total_byte_read < bytes {
        let byte_read = stream.read(buffer).await.map_err(Error::ReadByte)?;

        if byte_read == 0 {
            return Err(Error::UnexpectedEndOfResponse);
        }

        total_byte_read += byte_read;

        for x in &buffer[..byte_read] {
            payload.push(*x);
        }
    }

    String::from_utf8(payload).map_err(Error::ResponseToString)
}

/// Reads a complete response from the TCP stream.
///
/// This function first reads the length prefix using [`read_length`],
/// then reads the corresponding string using [`read_string`].
///
/// # Errors
///
/// Returns an [`Error`] if reading fails or the conversion to UTF-8 fails.
pub async fn read(stream: &mut TcpStream) -> Result<String> {
    let bytes = read_length(stream).await?;

    read_string(stream, bytes).await
}

/// Represents a command failure response from the Marionette server.
#[derive(Debug, Deserialize)]
pub struct Failure {
    pub error: String,
    pub message: String,
    pub stacktrace: String,
}

/// Internal enum used to deserialize Marionette responses.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response<T> {
    Success(#[allow(unused)] u8, u32, (), T),
    Failure(#[allow(unused)] u8, u32, Failure, ()),
}

/// Parses a raw JSON string into the expected type.
///
/// # Arguments
///
/// * `json` - A raw JSON string reference.
///
/// # Errors
///
/// Returns an [`Error`] if JSON deserialization fails.
pub fn parse_raw<J: AsRef<str> + Debug, T: DeserializeOwned + Debug>(json: J) -> Result<T> {
    serde_json::from_str(json.as_ref()).map_err(|error| {
        error!(?json, "Raw JSON response");
        Error::ParseResponse(error)
    })
}

/// Parses a JSON response from Marionette into a tuple containing the command ID and the result data.
///
/// The JSON response is expected to conform to one of the variants in the [`Response`] enum.
/// If the response indicates a failure, an [`Error::CommandFailure`] is returned.
///
/// # Arguments
///
/// * `json` - A raw JSON string reference.
///
/// # Errors
///
/// Returns an [`Error`] if JSON deserialization fails or the response represents a failed command.
pub fn parse<J: AsRef<str> + Debug, T: DeserializeOwned + Debug>(json: J) -> Result<(u32, T)> {
    let response = parse_raw(json)?;
    debug!(?response, "Got response");

    match response {
        Response::Success(_, id, (), success) => Ok((id, success)),
        Response::Failure(_, id, failure, ()) => Err(Error::CommandFailure(id, failure)),
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_raw() {
        let json = "42";
        let value: i32 = parse_raw(json).expect("Failed to parse raw JSON");
        assert_eq!(value, 42);
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct DummyResponse {
        result: String,
    }

    #[test]
    fn test_parse_success() {
        let json = r#"[1, 123, null, {"result": "ok"}]"#;
        let (id, dummy): (u32, DummyResponse) =
            parse(json).expect("Failed to parse success response");
        assert_eq!(id, 123);
        assert_eq!(
            dummy,
            DummyResponse {
                result: "ok".to_string()
            }
        );
    }

    #[test]
    fn test_parse_failure() {
        let json =
            r#"[1, 456, {"error": "err", "message": "failed", "stacktrace": "trace"}, null]"#;
        let err = parse::<_, DummyResponse>(json).expect_err("Expected a failure response");
        match err {
            Error::CommandFailure(id, failure) => {
                assert_eq!(id, 456);
                assert_eq!(failure.error, "err");
                assert_eq!(failure.message, "failed");
                assert_eq!(failure.stacktrace, "trace");
            },
            _ => panic!("Expected CommandFailure error"),
        }
    }
}
