//! Module for managing the browser pool state.
//!
//! This module provides an integration with [deadpool](https://crates.io/crates/deadpool)
//! to manage a pool of browser instances. The [`State`] struct wraps a [`BrowserPool`] and
//! provides an asynchronous method to retrieve a browser from the pool.

use deadpool::managed::{Object, PoolError};
use pantin_browser::browser;

use crate::browser_pool::{BrowserManager, BrowserPool};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    PoolError(#[from] PoolError<browser::Error>),
}

/// Represents the application state that holds the browser pool.
///
/// This state encapsulates a [`BrowserPool`] and provides methods to retrieve browser instances.
#[derive(Clone)]
pub struct State {
    browser_pool: BrowserPool,
}

impl State {
    /// Creates a new state instance with the given browser pool.
    ///
    /// # Arguments
    ///
    /// * `browser_pool` - A [`BrowserPool`] instance to be managed.
    ///
    /// # Returns
    ///
    /// A new [`State`] instance.
    pub const fn new(browser_pool: BrowserPool) -> Self {
        Self { browser_pool }
    }

    /// Asynchronously retrieves a browser instance from the pool.
    ///
    /// This method returns an [`BrowserManager`] which represents a managed browser instance.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::PoolError`] if the browser pool fails to provide a browser instance.
    pub async fn get_browser(&self) -> Result<Object<BrowserManager>, Error> {
        Ok(Box::pin(self.browser_pool.get()).await?)
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use deadpool::managed::Pool;

    use super::*;

    #[tokio::test]
    async fn test_state() {
        let manager = BrowserManager::new("firefox");

        let pool: BrowserPool = Pool::builder(manager)
            .max_size(1)
            .build()
            .expect("Failed to build pool");

        let state = State::new(pool);

        {
            let browser = state.get_browser().await.expect("Firefox browser");

            assert_eq!(
                browser.uuid().to_string().len(),
                36,
                "Browser should have a valid uuid"
            );
        }

        for browser in state.browser_pool.retain(|_, _| false).removed {
            browser.close().await.expect("Browser close");
        }
    }

    #[tokio::test]
    async fn test_state_get_browser_error() {
        let manager = BrowserManager::new("invalid-browser-command");

        let pool: BrowserPool = Pool::builder(manager)
            .max_size(1)
            .build()
            .expect("Failed to build pool");

        let state = State::new(pool);
        let browser = state.get_browser().await;

        assert!(matches!(browser, Err(Error::PoolError(_))));
    }
}
