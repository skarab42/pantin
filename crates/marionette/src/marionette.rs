//! Module for the Marionette client.
//!
//! This module implements a Marionette client that connects to a Marionette server over a TCP stream,
//! performs the handshake procedure, starts a new Marionette session, and sends commands.
//!
//! It integrates functionality from the [`handshake`], [`request`], and [`webdriver`] modules to provide
//! a unified interface for interacting with the Marionette protocol.

use std::{fmt::Debug, io, net::SocketAddr, result, time::Duration};

use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    time::{Instant, sleep},
};
use tracing::{debug, error};

use crate::{handshake, request, webdriver};

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

/// Represents a Marionette client connected to a Marionette server.
///
/// The client holds a TCP stream, the result of the handshake, and the session information
/// obtained from starting a new Marionette session.
#[derive(Debug)]
pub struct Marionette {
    stream: TcpStream,
    handshake: handshake::Handshake,
    session: webdriver::NewSessionResponse,
}

impl Marionette {
    /// Creates a new Marionette client by connecting to the server at the given address.
    ///
    /// The connection is attempted with a specified timeout and retry interval.
    /// After connecting, the client performs a handshake and starts a new session.
    ///
    /// # Arguments
    ///
    /// * `address` - The socket address of the Marionette server.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The connection to the server times out.
    /// - The handshake fails.
    /// - The new session request fails.
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

    /// Returns the Marionette protocol version obtained during the handshake.
    pub const fn protocol(&self) -> u8 {
        self.handshake.marionette_protocol
    }

    /// Returns the current session identifier.
    pub fn session_id(&self) -> &str {
        self.session.session_id.as_str()
    }

    /// Sends a command to the Marionette server.
    ///
    /// This method delegates to the [`request::send`] function to send the command
    /// and receive the corresponding response.
    ///
    /// # Type Parameters
    ///
    /// * `C`: A type that implements the [`webdriver::Command`] trait.
    ///
    /// # Arguments
    ///
    /// * `command` - A reference to the command to be sent.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Request`] if the request fails.
    pub async fn send<C>(&mut self, command: &C) -> Result<C::Response>
    where
        C: webdriver::Command + Send + Sync,
    {
        request::send(&mut self.stream, command.name(), &command.parameters())
            .await
            .map_err(Error::Request)
    }
}

/// Reads the handshake message from the provided stream.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the TCP stream.
///
/// # Errors
///
/// Returns an [`Error::Handshake`] if the handshake fails.
async fn read_handshake<S: AsyncRead + Unpin>(stream: &mut S) -> Result<handshake::Handshake> {
    handshake::Handshake::read(stream)
        .await
        .map_err(Error::Handshake)
}

/// Sends a command over the provided stream and returns its response.
///
/// # Type Parameters
///
/// * `C`: A type that implements the [`webdriver::Command`] trait.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the stream.
/// * `command` - A reference to the command to be sent.
///
/// # Errors
///
/// Returns an [`Error::Request`] if sending the command fails.
async fn send<S, C>(stream: &mut S, command: &C) -> Result<C::Response>
where
    S: AsyncRead + AsyncWrite + Unpin,
    C: webdriver::Command + Send + Sync,
{
    request::send(stream, command.name(), &command.parameters())
        .await
        .map_err(Error::Request)
}

/// Sends a new session request over the provided stream.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the stream.
///
/// # Errors
///
/// Returns an [`Error::Request`] if the request fails.
async fn new_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
) -> Result<webdriver::NewSessionResponse> {
    send(stream, &webdriver::NewSession::new(None)).await
}

/// Attempts to connect to the given address with a timeout and retry interval.
///
/// The function continuously retries to connect until the specified timeout is reached.
///
/// # Arguments
///
/// * `address` - The socket address of the Marionette server.
/// * `timeout_ms` - The total timeout in milliseconds.
/// * `interval_ms` - The retry interval in milliseconds.
///
/// # Errors
///
/// Returns an [`Error::ConnectionTimeout`] if the connection cannot be established within the timeout.
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
