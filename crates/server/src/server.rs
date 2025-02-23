use axum::{body::Body, http::Request, routing::get, Router};
use color_eyre::eyre::Result;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, debug_span, error, info};
use uuid::Uuid;

use crate::{
    browser_pool::{BrowserManager, BrowserPool},
    cli, routes, signal,
    state::State,
};

pub async fn start(settings: cli::PantinSettings) -> Result<()> {
    debug!(?settings, "Starting...");
    let listener = TcpListener::bind((settings.host.as_str(), settings.port)).await?;
    info!("Listening at http://{}:{}", settings.host, settings.port);

    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        debug_span!("request", uuid=?Uuid::new_v4(), method=?request.method(), uri=?request.uri(), version=?request.version())
    });

    let browser_pool = BrowserPool::builder(BrowserManager)
        // .max_size(usize::from(settings.pool_size))
        .build()?;

    let state = State::new(browser_pool);
    let router = Router::new()
        .route("/ping", get(routes::ping))
        .route("/screenshot", get(routes::screenshot))
        .fallback(routes::not_found)
        .layer(trace_layer)
        .with_state(state);

    info!("Press [CTRL+C] to exit gracefully.");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(|| {
            debug!("Doing some cleaning...");
        }))
        .await?;

    Ok(())
}

async fn shutdown_signal<F: FnOnce() + Send>(cleanup: F) {
    match signal::shutdown().await {
        Ok(()) => {
            cleanup();
            info!("Exited gracefully !");
        },
        Err(error) => {
            error!(?error, "Failed to setup graceful shutdown !");
        },
    }
}
