use std::{io, net::SocketAddr, result, time::Duration};

use thiserror::Error;
use tokio::{
    net::TcpStream,
    time::{sleep, Instant},
};
use tracing::debug;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to connect to {address} after {timeout_ms}ms.")]
    ConnectionTimeout {
        address: String,
        timeout_ms: u64,
        source: io::Error,
    },
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn new(address: &SocketAddr) -> Result<Self> {
        debug!("Creating a new Marionette Client instance...");
        let stream = try_connect(address, 1000, 100).await?;

        Ok(Self { stream })
    }
}

async fn try_connect(address: &SocketAddr, timeout_ms: u64, interval_ms: u64) -> Result<TcpStream> {
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
                        timeout_ms,
                        source,
                    });
                }
            },
        }
    }
}
