//! Frontend: serves `apps/web` as-is (plain HTML/CSS/JS, no build step).

use std::path::Path;

use axum::Router;
use tower_http::services::ServeDir;

use crate::AppState;

/// Override with `WEB_DIR`.
pub const DEFAULT_DIR: &str = "apps/web";

pub fn router(dir: &Path) -> Router<AppState> {
    Router::new().fallback_service(ServeDir::new(dir))
}
