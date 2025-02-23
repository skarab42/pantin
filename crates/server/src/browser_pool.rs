use deadpool::managed;
use pantin_browser::{browser, Browser};
use tracing::debug;

#[derive(Debug)]
pub struct BrowserManager {
    program: String,
}

impl BrowserManager {
    pub fn new<P: Into<String>>(program: P) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl managed::Manager for BrowserManager {
    type Type = Browser;
    type Error = browser::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let browser = Browser::open(self.program.clone()).await?;
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Create Browser instance in pool");

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

    fn detach(&self, browser: &mut Self::Type) {
        debug!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Detach Browser instance from pool");
    }
}

pub type BrowserPool = managed::Pool<BrowserManager>;
