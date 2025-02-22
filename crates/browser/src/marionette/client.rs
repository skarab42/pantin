use std::{fmt::Debug, io, net::SocketAddr, result, time::Duration};

use thiserror::Error;
use tokio::{
    net::TcpStream,
    time::{sleep, Instant},
};
use tracing::{debug, error};

use crate::marionette::{handshake, request, webdriver};

#[derive(Error, Debug)]
pub enum Error {
    #[error("connection timout: {address} - {timeout:?}")]
    ConnectionTimeout {
        address: String,
        timeout: Duration,
        source: io::Error,
    },
    #[error(transparent)]
    Handshake(#[from] handshake::Error),
    #[error(transparent)]
    Request(#[from] request::Error),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    handshake: handshake::HandshakeResponse,
    session: webdriver::NewSessionResponse,
}

impl Client {
    pub async fn new(address: &SocketAddr) -> Result<Self> {
        debug!("Creating a new Marionette Client instance...");
        let mut stream = connect(address, 2000, 100).await?;
        let handshake = read_handshake(&mut stream).await?;
        let session = new_session(&mut stream).await?;

        Ok(Self {
            stream,
            handshake,
            session,
        })
    }

    pub async fn send<C>(&mut self, command: &C) -> request::Result<C::Response>
    where
        C: webdriver::Command + Send + Sync,
    {
        request::send(&mut self.stream, command.name(), &command.parameters()).await
    }
}

async fn new_session(stream: &mut TcpStream) -> request::Result<webdriver::NewSessionResponse> {
    send(stream, &webdriver::NewSession::new(None)).await
}

async fn send<C>(stream: &mut TcpStream, command: &C) -> request::Result<C::Response>
where
    C: webdriver::Command + Send + Sync,
{
    request::send(stream, command.name(), &command.parameters()).await
}

async fn read_handshake(stream: &mut TcpStream) -> handshake::Result<handshake::HandshakeResponse> {
    handshake::HandshakeResponse::read(stream).await
}

async fn connect(address: &SocketAddr, timeout_ms: u64, interval_ms: u64) -> Result<TcpStream> {
    let interval = Duration::from_millis(interval_ms);
    let timeout = Duration::from_millis(timeout_ms);
    let now = Instant::now();

    debug!(
        ?address,
        ?timeout,
        ?interval,
        "Try to connect to Marionette..."
    );

    loop {
        let stream = TcpStream::connect(address).await;

        match stream {
            Ok(stream) => {
                debug!(?address, "Connected !");
                return Ok(stream);
            },
            Err(source) => {
                if now.elapsed() < timeout {
                    debug!(?address, "Retrying in {interval_ms}ms...",);
                    sleep(interval).await;
                } else {
                    return Err(Error::ConnectionTimeout {
                        address: address.to_string(),
                        timeout,
                        source,
                    });
                }
            },
        }
    }
}
