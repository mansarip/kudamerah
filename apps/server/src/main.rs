use std::sync::Arc;

use kudamerah_server::{AppState, config::Config, db};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    let config = Arc::new(Config::from_env()?);
    let db = db::connect(&config.database_path).await?;
    let app = kudamerah_server::app(AppState {
        db: db.clone(),
        config: config.clone(),
    });

    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Flush the WAL and release file locks before exiting.
    db.close().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
