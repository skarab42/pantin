use std::{fmt::Debug, io, result};

use serde::{de::DeserializeOwned, Serialize};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::debug;

use crate::marionette::{command::Command, response};

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
