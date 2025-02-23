use axum::{body::Body, http::Request, routing::get, Router};
use color_eyre::eyre::Result;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{cli, routes, signal};

pub async fn start(settings: cli::PantinSettings) -> Result<()> {
    debug!(?settings, "Starting...");
    let listener = TcpListener::bind((settings.host.as_str(), settings.port)).await?;
    info!("Listening at http://{}:{}", settings.host, settings.port);

    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        tracing::debug_span!("request", uuid=?Uuid::new_v4(), method=?request.method(), uri=?request.uri(), version=?request.version())
    });

    let router = Router::new()
        .route("/ping", get(routes::ping))
        .route("/screenshot", get(routes::screenshot))
        .fallback(routes::not_found)
        .layer(trace_layer);

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
