//! This module starts the Pantin Server.
//!
//! It builds the Axum router with middleware (request IDs, tracing, timeouts), initializes the browser pool,
//! and runs the server with graceful shutdown support. Background tasks are spawned to recycle and clean up
//! browser instances.

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    routing::get,
};
use color_eyre::Result;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{debug, debug_span, error, info};
use uuid::Uuid;

use crate::{
    browser_pool::{BrowserManager, BrowserPool},
    cli, routes, signal,
    state::State,
};

#[derive(Clone)]
struct MakeRequestId;

impl request_id::MakeRequestId for MakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<request_id::RequestId> {
        let request_id = Uuid::new_v4().to_string();

        #[allow(clippy::expect_used)]
        Some(request_id::RequestId::new(
            HeaderValue::from_str(request_id.as_str()).expect("Request id"),
        ))
    }
}

/// Starts the Pantin Server with the given configuration settings.
///
/// This function:
/// 1. Configures middleware layers for request IDs, tracing, and timeouts.
/// 2. Initializes the browser pool and shared state.
/// 3. Builds the Axum router with routes (e.g. `/ping`, `/screenshot`) and fallback handling.
/// 4. Spawns background tasks to recycle and clean up browser instances.
/// 5. Binds a TCP listener to the configured host and port and serves the router with graceful shutdown.
///
/// # Arguments
///
/// * `settings` - The configuration settings parsed from CLI and environment variables.
///
/// # Errors
///
/// Returns an [`Error`] if binding to the address fails, or if any initialization step
/// (e.g. setting up middleware or the browser pool) encounters an error.
pub async fn start(settings: cli::PantinSettings) -> Result<()> {
    debug!(?settings, "Starting...");

    let x_request_id = HeaderName::from_static("x-request-id");
    let request_id_layer = SetRequestIdLayer::new(x_request_id.clone(), MakeRequestId);
    let propagate_request_id_layer = PropagateRequestIdLayer::new(x_request_id.clone());

    let trace_layer = TraceLayer::new_for_http().make_span_with(move |request: &Request<Body>| {
        let default_value = HeaderValue::from_static("none");
        let uuid = request.headers().get(&x_request_id).unwrap_or(&default_value);
        debug_span!("request", ?uuid, method=?request.method(), uri=?request.uri(), version=?request.version())
    });

    let timeout_layer = TimeoutLayer::new(Duration::from_secs(u64::from(settings.request_timeout)));

    let service_builder = ServiceBuilder::new()
        .layer(request_id_layer)
        .layer(propagate_request_id_layer)
        .layer(trace_layer)
        .layer(timeout_layer);

    let browser_pool = BrowserPool::builder(BrowserManager::new(settings.browser_program.clone()))
        .max_size(usize::from(settings.browser_pool_max_size))
        .build()?;

    let state = State::new(browser_pool.clone());

    let router = Router::new()
        .route("/ping", get(routes::ping))
        .route("/screenshot", get(routes::screenshot))
        .fallback(routes::not_found)
        .layer(service_builder)
        .with_state(state);

    tokio::spawn(retain_loop(settings.clone(), browser_pool.clone()));

    let listener = TcpListener::bind((settings.server_host.clone(), settings.server_port)).await?;
    info!(
        "Listening at http://{}:{}",
        settings.server_host, settings.server_port
    );

    info!("Press [CTRL+C] to exit gracefully.");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    debug!("Cleaning browser pool...");
    cleaning_loop(browser_pool).await?;

    info!("Exited gracefully !");

    Ok(())
}

async fn shutdown_signal() {
    match signal::shutdown().await {
        Ok(()) => info!("Exiting..."),
        Err(error) => error!(?error, "Failed to setup graceful shutdown !"),
    }
}

async fn retain_loop(settings: cli::PantinSettings, browser_pool: BrowserPool) -> Result<()> {
    let browser_max_age = Duration::from_secs(u64::from(settings.browser_max_age));
    let browser_max_recycle_count = usize::from(settings.browser_max_recycle_count);

    loop {
        tokio::time::sleep(browser_max_age).await;

        let retain_result = browser_pool.retain(|_, metrics| {
            metrics.recycle_count < browser_max_recycle_count
                && metrics.last_used() < browser_max_age
        });

        for browser in retain_result.removed {
            browser.close().await?;
        }
    }
}

async fn cleaning_loop(browser_pool: BrowserPool) -> Result<()> {
    let retain_result = browser_pool.retain(|_, _| false);

    for browser in retain_result.removed {
        browser.close().await?;
    }

    Ok(())
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use std::sync::atomic::{AtomicU16, Ordering};

    use tokio::task::JoinHandle;
    use url::Url;

    use super::*;
    use crate::cli::{LogLevel, PantinSettings};

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(3000);

    fn get_next_port() -> u16 {
        PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    struct ServerAssert {
        settings: PantinSettings,
        base_url: Url,
    }

    impl ServerAssert {
        fn new() -> Self {
            let settings = PantinSettings {
                server_host: "127.0.0.1".into(),
                server_port: get_next_port(),
                request_timeout: 1,
                browser_pool_max_size: 1,
                browser_max_age: 1,
                browser_max_recycle_count: 1,
                browser_program: "firefox".into(),
                log_level: LogLevel::Trace,
            };
            let url_string = format!("http://{}:{}", settings.server_host, settings.server_port);
            let base_url = Url::parse(url_string.as_str()).expect("Parse base url");

            Self { settings, base_url }
        }

        async fn spawn(&self) -> JoinHandle<Result<()>> {
            let settings = self.settings.clone();
            let handle = tokio::spawn(async move { start(settings).await });
            tokio::time::sleep(Duration::from_millis(100)).await;
            handle
        }

        fn url<P: AsRef<str>>(&self, path: P) -> Url {
            self.base_url
                .join(path.as_ref())
                .expect("Join path to base url")
        }
    }

    #[tokio::test]
    async fn test_server_ping() {
        let server_assert = ServerAssert::new();
        let server_handle = server_assert.spawn().await;

        let response = reqwest::get(server_assert.url("ping"))
            .await
            .expect("Failed to send GET request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let body = response.text().await.expect("Failed to read response body");
        assert_eq!(
            body, r#"{"data":"pong"}"#,
            "Expected pong JSON response, got: {body}"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_server_not_found() {
        let server_assert = ServerAssert::new();
        let server_handle = server_assert.spawn().await;

        let response = reqwest::get(server_assert.url("this-route-does-not-exists"))
            .await
            .expect("Failed to send GET request");
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

        let body = response.text().await.expect("Failed to read response body");
        assert_eq!(
            body, r#"{"cause":"not found"}"#,
            "Expected not found JSON response, got: {body}"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_server_screenshot() {
        let server_assert = ServerAssert::new();
        let server_handle = server_assert.spawn().await;

        let response = reqwest::get(server_assert.url("screenshot"))
            .await
            .expect("Failed to send GET request");
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

        let body = response.text().await.expect("Failed to read response body");
        assert_eq!(
            body, r#"{"cause":"Failed to deserialize query string: missing field `url`"}"#,
            "Expected not found JSON response, got: {body}"
        );

        server_handle.abort();
    }
}
