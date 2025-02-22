#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_errors_doc, clippy::multiple_crate_versions)]

pub mod browser;
pub mod marionette;
pub mod profile;

pub use browser::Browser;
pub use marionette::Marionette;
pub use profile::Profile;
