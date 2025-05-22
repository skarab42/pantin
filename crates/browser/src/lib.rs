#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![cfg_attr(coverage, feature(coverage_attribute))]

//! Crate for controlling a Firefox browser instance using a temporary profile.
//!
//! # Modules
//!
//! - [`profile`]: Contains functions for managing a temporary Firefox profile,
//!   ensuring that the profile directory is automatically cleaned up on drop.
//! - [`browser`]: Provides an interface to launch, control, and close a Firefox browser
//!   using a temporary profile and a Marionette connection,
//!   ensuring all resources is automatically cleaned up on drop.
//!
//! # Usage
//!
//! Elements from the [`browser`] module are re-exported at the crate root for easier access.
//! For example, to launch a new browser instance:
//!
//! ```no_run
//! use pantin_browser::Browser;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut browser = Browser::open("firefox").await?;
//!     // Use the browser instance...
//!     browser.close().await?;
//!     Ok(())
//! }
//! ```

pub mod browser;
pub mod profile;

pub use browser::*;
