use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use tracing::info;

use crate::{
    api::{Error, Failure, Query, Success},
    state,
};

pub async fn ping() -> impl IntoResponse {
    Json(Success::<String>::new("pong".into()))
}

pub async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(Failure::new("NOT_FOUND".into(), "not found".into())),
    )
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum ScreenshotMode {
    Full,
    Viewport,
    Selector,
    XPath,
}

#[derive(Debug, Deserialize)]
pub struct ScreenshotQuery {
    /// Url of the page to take a screenshot.
    url: String,
    /// Delay in milliseconds to take the screenshot,
    /// after the `DOMContentLoaded` event occurs (default: 0).
    delay: Option<u16>,
    /// Screenshot with (default: 800).
    width: Option<u16>,
    /// Screenshot height (default: 600).
    height: Option<u16>,
    /// Should show the scrollbar on `html` and `body` elements (default: false).
    scrollbar: Option<bool>,
    /// Should be displayed as an attachment, that is downloaded and saved locally (default: false).
    attachment: Option<bool>,
    /// One of `'full'`, `'viewport'`, `'selector'` or `'xpath'` (default: 'viewport').
    mode: Option<ScreenshotMode>,
    /// CSS selector, only applied and required if `mode = 'selector'` (default: None).
    selector: Option<String>,
    /// `XPath`, only applied and required if `mode = 'xpath'` (default: None).
    xpath: Option<String>,
}

pub async fn screenshot(
    state: State<state::State>,
    query: Query<ScreenshotQuery>,
) -> Result<impl IntoResponse, Error> {
    info!(?query, "Screenshot");

    let browser = state.get_browser().await?;
    info!(uuid=?browser.uuid(), pid=?browser.pid(), sid=?browser.sid(), "Browser");

    Ok(())
}
