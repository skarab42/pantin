//! Crate for managing a Firefox profile for testing purposes.
//!
//! This crate wraps around the [`temp_dir`](https://docs.rs/temp_dir) crate to create a temporary
//! Firefox profile directory which is automatically removed when dropped. It also creates a `user.js` file
//! configured with a free Marionette address for remote control.

use std::{fmt::Display, io, net::SocketAddr, result};

use temp_dir::TempDir;
use thiserror::Error;
use tokio::{fs::write, net::TcpListener};
use tracing::debug;

#[derive(Error, Debug)]
pub enum Error {
    #[error("create temporary profile directory failed")]
    CreateDirectory(#[source] io::Error),
    #[error("remove temporary profile directory failed")]
    RemoveDirectory(#[source] io::Error),
    #[error("get a free local address failed")]
    GetFreeLocalAddress(#[source] io::Error),
    #[error("create temporary profile 'user.js' file failed")]
    CreateUserJsFile(#[source] io::Error),
    #[error("temporary profile directory path is undefined")]
    UndefinedPath,
}

pub type Result<T, E = Error> = result::Result<T, E>;

static USER_JS: [u8; include_bytes!("user.js").len()] = *include_bytes!("user.js");

/// Represents a temporary Firefox profile.
///
/// This structure wraps a temporary directory (provided by the [`temp_dir`](https://docs.rs/temp_dir) crate)
/// and ensures that the directory is removed when dropped. It also creates a `user.js` file containing a free
/// Marionette address used for controlling Firefox.
#[derive(Debug)]
pub struct Profile {
    directory: TempDir,
    marionette_address: SocketAddr,
}

impl Profile {
    /// Creates a new Firefox profile.
    ///
    /// This function creates a temporary directory for the profile and writes a `user.js` file inside it,
    /// which includes a free Marionette port for remote control.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if creating the directory, writing the file, or obtaining a free local address fails.
    pub async fn new() -> Result<Self> {
        debug!("Creating a new Profile instance...");
        let directory = create_directory()?;
        debug!("Created profile directory at: {:?}", directory.path());
        let marionette_address = crate_user_js_file(&directory).await?;

        Ok(Self {
            directory,
            marionette_address,
        })
    }

    /// Returns the Marionette address associated with this profile.
    #[must_use]
    pub const fn marionette_address(&self) -> SocketAddr {
        self.marionette_address
    }

    /// Checks whether the profile directory still exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.directory.path().exists()
    }

    /// Returns the path to the profile directory as a string slice.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the path cannot be retrieved.
    pub fn path(&self) -> Result<&str> {
        self.directory.path().to_str().ok_or(Error::UndefinedPath)
    }

    /// Explicitly removes the profile directory.
    ///
    /// Although the directory is automatically removed when the `Profile` instance is dropped,
    /// this function allows for manual cleanup.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the cleanup process fails.
    pub fn remove(self) -> Result<()> {
        debug!("Removing profile directory at: {:?}", self.directory.path());
        self.directory.cleanup().map_err(Error::RemoveDirectory)
    }
}

/// Creates the `user.js` file within the temporary profile directory.
///
/// This function writes the default `user.js` content concatenated with a configured Marionette port.
/// It returns the free Marionette address used in the file.
///
/// # Arguments
///
/// * `directory` - The temporary directory where the `user.js` file will be created.
///
/// # Errors
///
/// Returns an [`Error`] if writing the file or obtaining a free local address fails.
async fn crate_user_js_file(directory: &TempDir) -> Result<SocketAddr> {
    debug!("Creating 'user.js' inside the temporary profile directory.");
    let marionette_address = get_free_local_address().await?;
    debug!(
        "Get free local Marionette address: {:?}",
        marionette_address
    );
    let marionette_port_pref = user_pref("marionette.port", marionette_address.port());

    let user_js_path = directory.child("user.js");
    let user_js_data = [&USER_JS, marionette_port_pref.as_bytes()].concat();

    debug!("Write 'user.js' file at: {:?}", user_js_path);
    write(&user_js_path, user_js_data)
        .await
        .map_err(Error::CreateUserJsFile)?;

    Ok(marionette_address)
}

/// Creates a new temporary directory for the Firefox profile.
///
/// This function leverages the [`temp_dir`](https://docs.rs/temp_dir) crate to create a directory with a specific prefix.
/// The directory is set to panic on cleanup errors to ensure reliability.
///
/// # Errors
///
/// Returns an [`Error`] if creating the directory fail.
fn create_directory() -> Result<TempDir> {
    let directory = TempDir::with_prefix("pantin-moz-profile")
        .map_err(Error::CreateDirectory)?
        .panic_on_cleanup_error();

    Ok(directory)
}

/// Obtains a free local socket address.
///
/// This function binds a TCP listener to `127.0.0.1` with port `0`, letting the OS assign a free port,
/// and then returns the resulting local address.
///
/// # Errors
///
/// Returns an [`Error`] if binding or retrieving the local address fails.
async fn get_free_local_address() -> Result<SocketAddr> {
    TcpListener::bind(("127.0.0.1", 0))
        .await
        .and_then(|stream| stream.local_addr())
        .map_err(Error::GetFreeLocalAddress)
}

/// Formats a Firefox user preference.
///
/// # Arguments
///
/// * `key` - The preference key.
/// * `value` - The preference value.
///
/// # Returns
///
/// A string representing a Firefox user preference declaration.
fn user_pref(key: impl Display, value: impl Display) -> String {
    format!("user_pref(\"{key}\", {value});\n")
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[tokio::test]
    async fn test_profile_new() {
        let profile = Profile::new().await.expect("Failed to create profile");
        assert!(profile.exists());

        let path_str = profile.path().expect("Profile path should be valid");
        assert!(!path_str.is_empty());

        assert!(
            path_str.contains("pantin-moz-profile"),
            "Profile directory should contain the expected prefix"
        );

        let addr = profile.marionette_address();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_user_js_exists_and_contains_marionette_port() {
        let profile = Profile::new().await.expect("Failed to create profile");
        let path_str = profile.path().expect("Profile path should be valid");
        let user_js_path = std::path::Path::new(path_str).join("user.js");
        let metadata = tokio::fs::metadata(&user_js_path).await;

        assert!(
            metadata.is_ok(),
            "user.js should exist in the profile directory"
        );

        let content = tokio::fs::read_to_string(&user_js_path)
            .await
            .expect("Failed to read user.js file");

        let addr = profile.marionette_address();
        let user_pref = format!("user_pref(\"marionette.port\", {});", addr.port());

        assert!(
            content.contains(user_pref.as_str()),
            "user.js does not contain the expected Marionette port preference"
        );
    }

    #[tokio::test]
    async fn test_profile_remove() {
        let profile = Profile::new().await.expect("Failed to create profile");
        let path_str = profile
            .path()
            .expect("Profile path should be valid")
            .to_string();

        profile.remove().expect("Failed to remove profile");

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert!(
            !std::path::Path::new(&path_str).exists(),
            "Profile directory should be removed"
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_profile_tracing() {
        let profile = Profile::new().await.expect("Failed to create profile");

        assert!(logs_contain(
            " pantin_browser::profile: Creating a new Profile instance..."
        ));

        assert!(logs_contain(
            "pantin_browser::profile: Created profile directory at:"
        ));

        assert!(logs_contain(
            "pantin_browser::profile: Creating 'user.js' inside the temporary profile directory."
        ));

        let addr = profile.marionette_address();
        let address = format!(
            "pantin_browser::profile: Get free local Marionette address: {}:{}",
            addr.ip(),
            addr.port()
        );
        assert!(logs_contain(address.as_str()));

        assert!(logs_contain(
            "pantin_browser::profile: Write 'user.js' file at:"
        ));

        profile.remove().expect("Failed to remove profile");

        assert!(logs_contain(
            "pantin_browser::profile: Removing profile directory at: "
        ));
    }
}
