use axum::{routing::get, Router};
use color_eyre::eyre::Result;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use crate::{api, cli, signal};

pub async fn start(settings: cli::PantinSettings) -> Result<()> {
    debug!(?settings, "Starting...");
    let listener = TcpListener::bind((settings.host.as_str(), settings.port)).await?;
    info!("Listening at http://{}:{}", settings.host, settings.port);

    let app = Router::new()
        .route("/ping", get(api::ping))
        .route("/screenshot", get(api::screenshot))
        .fallback(api::not_found);

    info!("Press [CTRL+C] to exit gracefully.");
    axum::serve(listener, app)
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
