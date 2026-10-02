pub mod config;
pub mod db;
pub mod error;
mod modules;
mod web; // starter:web

use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// Shared state, cloned into every handler. Keep it cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
}

/// Builds the application router: JSON API under `/api`, plus the frontend.
pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/api", modules::router())
        .merge(web::router(&state.config.web_dir)) // starter:web
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
