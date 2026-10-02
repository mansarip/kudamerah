//! Feature modules. Each module owns its routes, handlers and queries,
//! and exposes a `router()` that is merged here and mounted under `/api`.

mod health;
pub mod notes; // starter:example

use axum::Router;

use crate::{AppState, error::AppError};

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(notes::router()) // starter:example
        .fallback(|| async { AppError::NotFound })
}
