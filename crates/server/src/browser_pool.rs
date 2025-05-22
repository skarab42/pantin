//! Module for managing a pool of browser instances.
//!
//! This module integrates with [deadpool](https://crates.io/crates/deadpool) to create a managed pool of
//! browser instances. The pool is built around a custom [`BrowserManager`] that implements the
//! [`managed::Manager`] trait. This manager is responsible for creating, recycling, and detaching
//! browser instances.
//!
//! # `BrowserManager`
//!
//! The [`BrowserManager`] struct holds the command or binary path needed to launch a browser. It
//! implements the manager trait for creating new browser instances using [`Browser::open`] from the
//! [`pantin_browser`] crate.
//!
//! # `BrowserPool`
//!
//! The [`BrowserPool`] type alias defines a pool of browsers managed by the [`BrowserManager`].
//!
//! ## Example
//!
//! ```no_run
//! use pantin_browser::browser::Browser;
//! use pantin_server::browser_pool::{BrowserManager, BrowserPool};
//! use deadpool::managed::Pool;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a browser manager with the browser program path.
//!     let manager = BrowserManager::new("firefox");
//!
//!     // Create a pool of browser instances.
//!     let pool: BrowserPool = Pool::builder(manager)
//!         .max_size(5)
//!         .build()?;
//!
//!     // Get a browser from the pool.
//!     let _browser = pool.get().await?;
//!
//!     // Use the browser instance...
//!
//!     Ok(())
//! }
//! ```

use deadpool::managed;
use pantin_browser::{Browser, browser};
use tracing::debug;

/// The browser manager responsible for creating and recycling [`Browser`] instances.
///
/// It holds the program path used to launch the browser.
#[derive(Debug)]
pub struct BrowserManager {
    program: String,
}

impl BrowserManager {
    /// Creates a new [`BrowserManager`] with the specified browser program.
    ///
    /// # Arguments
    ///
    /// * `program` - A value convertible to a `String` that represents the browser command or the path to the binary.
    pub fn new<P: Into<String>>(program: P) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl managed::Manager for BrowserManager {
    type Type = Browser;
    type Error = browser::Error;

    /// Creates a new [`Browser`] instance.
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let browser = Browser::open(self.program.clone()).await?;
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Create Browser instance in pool");

        Ok(browser)
    }

    /// Recycles an existing browser instance.
    ///
    /// This method is called by the pool when a browser instance is returned.
    /// If needed it can perform any necessary recycling steps.
    async fn recycle(
        &self,
        browser: &mut Self::Type,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Recycle Browser instance from pool");

        Ok(())
    }

    /// Detaches a browser instance from the pool.
    ///
    /// This method is called when a browser instance is permanently removed from the pool.
    fn detach(&self, browser: &mut Self::Type) {
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Detach Browser instance from pool");
    }
}

/// A type alias for a pool of browser instances managed by [`BrowserManager`].
pub type BrowserPool = managed::Pool<BrowserManager>;

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use deadpool::managed::Pool;

    use super::*;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_browser_manager() {
        let manager = BrowserManager::new("firefox");
        assert_eq!(manager.program, "firefox");

        let pool: BrowserPool = Pool::builder(manager)
            .max_size(1)
            .build()
            .expect("Failed to build pool");

        let browser_uuid: uuid::Uuid;

        {
            let browser = Box::pin(pool.get()).await.expect("Firefox browser");
            browser_uuid = browser.uuid();

            assert_eq!(
                browser_uuid.to_string().len(),
                36,
                "Browser should have a valid uuid"
            );
        }

        assert!(logs_contain(
            "pantin_server::browser_pool: Create Browser instance in pool"
        ));

        {
            let browser = Box::pin(pool.get()).await.expect("Firefox browser");

            assert_eq!(
                browser_uuid,
                browser.uuid(),
                "Browser should be recycled and have a same uuid as preview"
            );
        }

        assert!(logs_contain(
            "pantin_server::browser_pool: Recycle Browser instance from pool"
        ));

        for browser in pool.retain(|_, _| false).removed {
            browser.close().await.expect("Browser close");
        }

        assert!(logs_contain(
            "pantin_server::browser_pool: Detach Browser instance from pool"
        ));
    }
}
