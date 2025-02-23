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

#[derive(Debug)]
pub struct Profile {
    directory: TempDir,
    marionette_address: SocketAddr,
}

impl Profile {
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

    #[must_use]
    pub const fn marionette_address(&self) -> SocketAddr {
        self.marionette_address
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.directory.path().exists()
    }

    pub fn path(&self) -> Result<&str> {
        self.directory.path().to_str().ok_or(Error::UndefinedPath)
    }

    pub fn remove(self) -> Result<()> {
        debug!("Removing profile directory at: {:?}", self.directory.path());
        self.directory.cleanup().map_err(Error::RemoveDirectory)
    }
}

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

fn create_directory() -> Result<TempDir> {
    let directory = TempDir::with_prefix("pantin-moz-profile")
        .map_err(Error::CreateDirectory)?
        .panic_on_cleanup_error();

    Ok(directory)
}

async fn get_free_local_address() -> Result<SocketAddr> {
    TcpListener::bind(("127.0.0.1", 0))
        .await
        .and_then(|stream| stream.local_addr())
        .map_err(Error::GetFreeLocalAddress)
}

fn user_pref(key: impl Display, value: impl Display) -> String {
    format!("user_pref(\"{key}\", {value});\n")
}
