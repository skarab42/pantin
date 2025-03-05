use std::time::Duration;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use pantin_browser::{Browser, ScreenshotFindElementUsing, ScreenshotParameters};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    api,
    api::{Failure, Query, Success},
    state,
};

pub async fn ping() -> Response {
    Json(Success::<String>::new("pong".into())).into_response()
}

pub async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, Json(Failure::new("not found"))).into_response()
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
#[serde(rename_all(deserialize = "kebab-case"))]
pub enum ScreenshotResponseType {
    Attachment,
    ImagePngBase64,
    ImagePngBytes,
    JsonPngBase64,
    JsonPngBytes,
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
    /// One of `'attachment'`, `'image-png-base64'`, `'image-png-bytes'`, `'json-png-base64'` or `'json-png-bytes'` (default: 'image-png-bytes').
    response_type: Option<ScreenshotResponseType>,
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
) -> api::Result {
    info!(?query, "Screenshot");

    let mut browser = state.get_browser().await?;

    browser.navigate(query.url).await?;

    let scrollbar = query.scrollbar.unwrap_or(false);
    if !scrollbar {
        browser.hide_body_scrollbar().await?;
    }

    let width = query.width.unwrap_or(800);
    let height = query.height.unwrap_or(600);
    browser.set_window_size(width, height).await?;

    let delay = query.delay.unwrap_or(0);
    if delay > 0 {
        tokio::time::sleep(Duration::from_millis(u64::from(delay))).await;
    }

    let mode = query.mode.unwrap_or(ScreenshotMode::Viewport);
    let parameters = match mode {
        ScreenshotMode::Full => ScreenshotParameters::full(),
        ScreenshotMode::Viewport => ScreenshotParameters::viewport(),
        ScreenshotMode::Selector => {
            let selector = query
                .selector
                .ok_or_else(|| api::Error::MissingField("selector".into()))?;
            let element = browser
                .find_element(ScreenshotFindElementUsing::CssSelector, selector)
                .await?;

            ScreenshotParameters::element(element.id)
        },
        ScreenshotMode::XPath => {
            let xpath = query
                .xpath
                .ok_or_else(|| api::Error::MissingField("xpath".into()))?;
            let element = browser
                .find_element(ScreenshotFindElementUsing::XPath, xpath)
                .await?;

            ScreenshotParameters::element(element.id)
        },
    };

    let response_type = query
        .response_type
        .unwrap_or(ScreenshotResponseType::ImagePngBytes);

    let response = match response_type {
        ScreenshotResponseType::ImagePngBytes => {
            screenshot_image_bytes(&mut browser, parameters).await?
        },
        ScreenshotResponseType::Attachment => {
            screenshot_attachment(&mut browser, parameters).await?
        },
        ScreenshotResponseType::ImagePngBase64 => {
            screenshot_image_base64(&mut browser, parameters).await?
        },
        ScreenshotResponseType::JsonPngBase64 => {
            screenshot_json_base64(&mut browser, parameters).await?
        },
        ScreenshotResponseType::JsonPngBytes => {
            screenshot_json_bytes(&mut browser, parameters).await?
        },
    };

    Ok(response)
}

async fn screenshot_image_bytes(
    browser: &mut Browser,
    parameters: ScreenshotParameters,
) -> api::Result {
    let bytes = browser.screenshot_bytes(parameters).await?;
    let headers = [(header::CONTENT_TYPE, "image/png")];

    Ok((StatusCode::OK, headers, bytes).into_response())
}

async fn screenshot_attachment(
    browser: &mut Browser,
    parameters: ScreenshotParameters,
) -> api::Result {
    let bytes = browser.screenshot_bytes(parameters).await?;
    let headers = [
        (header::CONTENT_TYPE, "image/png"),
        (
            header::CONTENT_DISPOSITION,
            // TODO: make `filename` configurable ?!
            "attachment; filename=\"screenshot.png\"",
        ),
    ];

    Ok((StatusCode::OK, headers, bytes).into_response())
}

async fn screenshot_image_base64(
    browser: &mut Browser,
    parameters: ScreenshotParameters,
) -> api::Result {
    let base64 = browser.screenshot_base64(parameters).await?;
    let headers = [(header::CONTENT_TYPE, "text/plain")];

    Ok((
        StatusCode::OK,
        headers,
        format!("data:image/png;base64,{base64}"),
    )
        .into_response())
}

#[derive(Debug, Serialize)]
struct JsonPngBase64 {
    base64: String,
}

async fn screenshot_json_base64(
    browser: &mut Browser,
    parameters: ScreenshotParameters,
) -> api::Result {
    let base64 = browser.screenshot_base64(parameters).await?;

    Ok((StatusCode::OK, Json(JsonPngBase64 { base64 })).into_response())
}

#[derive(Debug, Serialize)]
struct JsonPngBytes {
    bytes: Vec<u8>,
}

async fn screenshot_json_bytes(
    browser: &mut Browser,
    parameters: ScreenshotParameters,
) -> api::Result {
    let bytes = browser.screenshot_bytes(parameters).await?;

    Ok((StatusCode::OK, Json(JsonPngBytes { bytes })).into_response())
}
