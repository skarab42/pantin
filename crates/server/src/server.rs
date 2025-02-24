use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    routing::get,
};
use color_eyre::eyre::Result;
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
