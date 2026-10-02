//! Frontend: serves the SolidJS build (`npm run build` in `apps/web` writes `apps/web/dist`).
//! Unknown paths fall back to `index.html` so client-side routing works.
//! During development, use `npm run dev` instead (Vite proxies `/api` to this server).

use std::path::Path;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::AppState;

/// Vite build output. Override with `WEB_DIR`.
pub const DEFAULT_DIR: &str = "apps/web/dist";

pub fn router(dir: &Path) -> Router<AppState> {
    let spa = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
    Router::new().fallback_service(spa)
}
