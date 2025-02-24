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

pub async fn read(stream: &mut TcpStream) -> Result<String> {
    let bytes = read_length(stream).await?;

    read_string(stream, bytes).await
}

#[derive(Debug, Deserialize)]
pub struct Failure {
    pub error: String,
    pub message: String,
    pub stacktrace: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response<T> {
    Success(#[allow(unused)] u8, u32, (), T),
    Failure(#[allow(unused)] u8, u32, Failure, ()),
}

pub fn parse_raw<J: AsRef<str> + Debug, T: DeserializeOwned + Debug>(json: J) -> Result<T> {
    serde_json::from_str(json.as_ref()).map_err(|error| {
        error!(?json, "Raw JSON response");
        Error::ParseResponse(error)
    })
}

pub fn parse<J: AsRef<str> + Debug, T: DeserializeOwned + Debug>(json: J) -> Result<(u32, T)> {
    let response = parse_raw(json)?;
    debug!(?response, "Got response");

    match response {
        Response::Success(_, id, (), success) => Ok((id, success)),
        Response::Failure(_, id, failure, ()) => Err(Error::CommandFailure(id, failure)),
    }
}
