use deadpool::managed::{Object, PoolError};
use pantin_browser::browser;

use crate::browser_pool::{BrowserManager, BrowserPool};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    PoolError(#[from] PoolError<browser::Error>),
}

#[derive(Clone)]
pub struct State {
    browser_pool: BrowserPool,
}

impl State {
    pub const fn new(browser_pool: BrowserPool) -> Self {
        Self { browser_pool }
    }

    pub async fn get_browser(&self) -> Result<Object<BrowserManager>, Error> {
        Ok(Box::pin(self.browser_pool.get()).await?)
    }
}
