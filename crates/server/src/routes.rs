//! This module provides HTTP handlers for screenshot functionality in the Pantin Server API.
//!
//! It allows clients to request screenshots of web pages using a headless browser.
//! The API supports various screenshot modes and response formats.

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

/// Health-check endpoint that returns a JSON response with "pong".
pub async fn ping() -> Response {
    Json(Success::<String>::new("pong".into())).into_response()
}

/// Fallback endpoint that returns a 404 Not Found error as a JSON response.
pub async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, Json(Failure::new("not found"))).into_response()
}

/// Specifies the mode used to capture a screenshot.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum ScreenshotMode {
    /// Capture the full page.
    Full,
    /// Capture only the visible (viewport) area.
    Viewport,
    /// Capture a specific element identified by a CSS selector.
    Selector,
    /// Capture a specific element identified by an `XPath` expression.
    XPath,
}

/// Specifies the response type for the screenshot.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub enum ScreenshotResponseType {
    /// Returns the screenshot as an attachment (with a filename).
    Attachment,
    /// Returns the screenshot as a Base64-encoded PNG string.
    ImagePngBase64,
    /// Returns the screenshot as raw PNG bytes.
    ImagePngBytes,
    /// Returns a JSON containing a Base64-encoded PNG string.
    JsonPngBase64,
    /// Returns a JSON containing raw PNG bytes.
    JsonPngBytes,
}

/// Represents the query parameters for a screenshot request.
///
/// This structure is deserialized from the URL query string.
#[derive(Debug, Deserialize)]
pub struct ScreenshotQuery {
    /// URL of the page to take a screenshot.
    url: String,
    /// Delay in milliseconds after `DOMContentLoaded` before taking the screenshot (default: 0).
    delay: Option<u16>,
    /// Screenshot width (default: 800).
    width: Option<u16>,
    /// Screenshot height (default: 600).
    height: Option<u16>,
    /// Whether to show the scrollbar on `html` and `body` elements (default: false).
    scrollbar: Option<bool>,
    /// Response type: one of 'attachment', 'image-png-base64', 'image-png-bytes', 'json-png-base64' or 'json-png-bytes' (default: 'image-png-bytes').
    response_type: Option<ScreenshotResponseType>,
    /// Screenshot mode: one of 'full', 'viewport', 'selector' or 'xpath' (default: 'viewport').
    mode: Option<ScreenshotMode>,
    /// CSS selector (required if `mode` is 'selector').
    selector: Option<String>,
    /// `XPath` expression (required if `mode` is 'xpath').
    xpath: Option<String>,
}

/// Handles screenshot requests by processing query parameters, interacting with a browser,
/// and returning the screenshot in the requested format.
///
/// This endpoint performs the following steps:
/// 1. Retrieves a browser instance from the shared state.
/// 2. Navigates the browser to the specified URL.
/// 3. Optionally hides scrollbars, sets the window size, and waits for a delay.
/// 4. Determines the screenshot mode and captures the screenshot.
/// 5. Returns the screenshot as raw bytes, Base64-encoded data, an attachment, or JSON-wrapped data.
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
