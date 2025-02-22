#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_errors_doc, clippy::multiple_crate_versions)]

pub mod command;
pub mod handshake;
pub mod marionette;
pub mod request;
pub mod response;
pub mod webdriver;

pub use marionette::*;
