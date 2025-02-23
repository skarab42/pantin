use deadpool::managed;
use pantin_browser::{browser, Browser};
use tracing::debug;

#[derive(Debug)]
pub struct BrowserManager;

impl managed::Manager for BrowserManager {
    type Type = Browser;
    type Error = browser::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let browser = Browser::open().await?;
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Add new Browser instance in pool");

        Ok(browser)
    }

    async fn recycle(
        &self,
        browser: &mut Self::Type,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Recycle Browser instance from pool");

        Ok(())
    }
}

pub type BrowserPool = managed::Pool<BrowserManager>;
