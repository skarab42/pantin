use std::{fmt::Debug, io, net::SocketAddr, result, time::Duration};

use thiserror::Error;
use tokio::{
    net::TcpStream,
    time::{sleep, Instant},
};
use tracing::{debug, error};

use crate::firefox::marionette::{handshake, request, webdriver};

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
    handshake: handshake::Handshake,
    session: webdriver::NewSession,
}

impl Client {
    pub async fn new(address: &SocketAddr) -> Result<Self> {
        debug!("Creating a new Marionette Client instance...");
        let mut stream = connect(address, 2000, 100).await?;
        let handshake = handshake::Handshake::read_response(&mut stream).await?;
        let session = webdriver::NewSession::send(&mut stream, None).await?;

        Ok(Self {
            stream,
            handshake,
            session,
        })
    }

    pub async fn set_window_rect(
        &mut self,
        window_rect: webdriver::WindowRect,
    ) -> request::Result<webdriver::SetWindowRect> {
        webdriver::SetWindowRect::send(&mut self.stream, window_rect).await
    }

    pub async fn set_window_size(
        &mut self,
        width: u16,
        height: u16,
    ) -> request::Result<webdriver::SetWindowRect> {
        self.set_window_rect(webdriver::WindowRect {
            x: None,
            y: None,
            width: Some(width),
            height: Some(height),
        })
        .await
    }

    pub async fn navigate(
        &mut self,
        location: webdriver::NavigateLocation,
    ) -> request::Result<webdriver::Navigate> {
        webdriver::Navigate::send(&mut self.stream, location).await
    }
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
