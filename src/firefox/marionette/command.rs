use std::{
    fmt::Debug,
    io, result,
    sync::atomic::{AtomicU32, Ordering},
};

use serde::{de::DeserializeOwned, Serialize, Serializer};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::debug;

use crate::firefox::marionette::response;

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

static MESSAGE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub enum Direction {
    Request = 0,
    Response = 1,
}

impl Serialize for Direction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            Self::Request => 0,
            Self::Response => 1,
        };

        value.serialize(serializer)
    }
}

type Id = u32;
type Name = String;

#[derive(Debug, Serialize)]
pub struct Command<T>(Direction, Id, Name, T);

impl<T> Command<T> {
    pub fn new<C>(command: C, data: T) -> Self
    where
        C: Into<String>,
    {
        Self(
            Direction::Request,
            MESSAGE_ID.fetch_add(1, Ordering::SeqCst),
            format!("WebDriver:{}", command.into()),
            data,
        )
    }

    #[must_use]
    pub const fn id(&self) -> Id {
        self.1
    }
}

pub async fn send<C, D, T>(stream: &mut TcpStream, command: C, data: D) -> Result<T>
where
    C: Into<String> + Send,
    D: Serialize + Send,
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

pub async fn write<C, D>(stream: &mut TcpStream, command: C, data: D) -> Result<u32>
where
    C: Into<String> + Send,
    D: Serialize + Send,
{
    let request = Command::new(command, data);
    let body = serde_json::to_string(&request).map_err(Error::ConvertJson)?;
    let data = format!("{}:{}", body.len(), body);

    debug!(?data, "Send command");

    stream
        .write(data.as_bytes())
        .await
        .map_err(Error::FailedToWriteRequest)?;

    Ok(request.id())
}
