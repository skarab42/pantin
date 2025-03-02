#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_errors_doc, clippy::multiple_crate_versions)]
#![cfg_attr(coverage, feature(coverage_attribute))]

//! Crate for controlling a Marionette client that communicates with a Marionette server.
//!
//! This crate provides a high-level interface for interacting with a Marionette server over a TCP connection.
//! It performs the initial handshake, starts a new session, and sends `WebDriver` commands to the server.
//!
//! # Example
//!
//! ```rust
//! use std::net::SocketAddr;
//! use pantin_marionette::Marionette;
//! use pantin_marionette::webdriver::{Navigate, NavigateParameters};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Define the Marionette server address.
//!     let addr: SocketAddr = "127.0.0.1:2828".parse()?;
//!     
//!     // Create a new Marionette client instance.
//!     let mut client = Marionette::new(&addr).await?;
//!     println!("Connected with session id: {}", client.session_id());
//!     
//!     // Send a Navigate command to open a URL.
//!     let navigate_cmd = Navigate::new(NavigateParameters {
//!         url: "https://example.com".to_string(),
//!     });
//!     let response = client.send(&navigate_cmd).await?;
//!     println!("Navigate response: {:?}", response);
//!     
//!     Ok(())
//! }
//! ```

pub mod command;
pub mod handshake;
pub mod marionette;
pub mod request;
pub mod response;
pub mod webdriver;

pub use marionette::*;
