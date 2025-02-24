use std::time::Duration;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use color_eyre::eyre::eyre;
use pantin_browser::{Browser, ScreenshotFindElementUsing, ScreenshotParameters};
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
    Query(query): Query<ScreenshotQuery>,
) -> Result<impl IntoResponse, Error> {
    info!(?query, "Screenshot");

    let attachment = query.attachment.unwrap_or(false);
    let mut browser = state.get_browser().await?;
    let screenshot = take_screenshot(&mut browser, query).await?;
    let content_type = (header::CONTENT_TYPE, "image/png");
    let body = Body::from(screenshot);

    if attachment {
        let headers = [
            content_type,
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"screenshot.png\"",
            ),
        ];

        Ok((headers, body).into_response())
    } else {
        Ok(([content_type], body).into_response())
    }
}

async fn take_screenshot(browser: &mut Browser, query: ScreenshotQuery) -> Result<Vec<u8>, Error> {
    browser.navigate(query.url).await?;

    // let scrollbar = query.scrollbar.unwrap_or(false);
    // if !settings.scrollbar {
    //     browser.hide_body_scrollbar()?;
    // }

    let width = query.width.unwrap_or(800);
    let height = query.height.unwrap_or(600);
    browser.resize(width, height).await?;

    let delay = query.delay.unwrap_or(0);
    if delay > 0 {
        tokio::time::sleep(Duration::from_millis(u64::from(delay))).await;
    }

    let mode = query.mode.unwrap_or(ScreenshotMode::Viewport);
    let screenshot = match mode {
        ScreenshotMode::Full => ScreenshotParameters::full(),
        ScreenshotMode::Viewport => ScreenshotParameters::viewport(),
        ScreenshotMode::Selector => {
            let selector = query
                .selector
                .ok_or_else(|| eyre!("missing field `selector`"))?;
            let element = browser
                .find_element(ScreenshotFindElementUsing::CssSelector, selector)
                .await?;

            ScreenshotParameters::element(element.id)
        },
        ScreenshotMode::XPath => {
            let xpath = query.xpath.ok_or_else(|| eyre!("missing field `xpath`"))?;
            let element = browser
                .find_element(ScreenshotFindElementUsing::XPath, xpath)
                .await?;

            ScreenshotParameters::element(element.id)
        },
    };

    let png_bytes = browser.screenshot(screenshot).await?;
    // TODO: handle base_64 string too

    Ok(png_bytes)
}
