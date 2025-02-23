use axum::{
    extract::FromRequestParts,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::eyre;
use serde::Deserialize;
use tracing::info;

pub struct Error(eyre::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E: Into<eyre::Error>> From<E> for Error {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// ---

#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Error))]
pub struct Query<T>(pub T);

// ---

pub async fn ping() -> impl IntoResponse {
    "pong"
}

// ---

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

// ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
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

pub async fn screenshot(query: Query<ScreenshotQuery>) -> Result<impl IntoResponse, Error> {
    info!(?query, "Screenshot");

    Ok(())
}
