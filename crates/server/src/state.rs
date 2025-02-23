use deadpool::managed::Object;

use crate::browser_pool::{BrowserManager, BrowserPool};

#[derive(Clone)]
pub struct State {
    browser_pool: BrowserPool,
}

impl State {
    pub const fn new(browser_pool: BrowserPool) -> Self {
        Self { browser_pool }
    }

    pub async fn get_browser(&self) -> color_eyre::Result<Object<BrowserManager>> {
        Ok(Box::pin(self.browser_pool.get()).await?)
    }
}
