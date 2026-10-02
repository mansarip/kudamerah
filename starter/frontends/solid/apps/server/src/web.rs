//! Frontend: serves or embeds the SolidJS build (`npm run build` writes `apps/web/dist`).
//! Unknown paths fall back to `index.html` so client-side routing works.
//! During development, use `npm run dev` instead (Vite proxies `/api` to this server).

use std::path::Path;

use axum::Router;
#[cfg(all(feature = "embed-web", not(debug_assertions)))]
use axum::{
    body::Body,
    http::{StatusCode, Uri, header},
    response::Response,
};
#[cfg(all(feature = "embed-web", not(debug_assertions)))]
use include_dir::{Dir, include_dir};
#[cfg(any(not(feature = "embed-web"), debug_assertions))]
use tower_http::services::{ServeDir, ServeFile};

use crate::AppState;

/// Vite output for development and separate-file releases. Override with `WEB_DIR`.
pub const DEFAULT_DIR: &str = "apps/web/dist";

pub fn router(dir: &Path) -> Router<AppState> {
    #[cfg(all(feature = "embed-web", not(debug_assertions)))]
    {
        let _ = dir;
        Router::new().fallback(embedded_asset)
    }
    #[cfg(any(not(feature = "embed-web"), debug_assertions))]
    {
        let spa = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
        Router::new().fallback_service(spa)
    }
}

#[cfg(all(feature = "embed-web", not(debug_assertions)))]
static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../web/dist");

#[cfg(all(feature = "embed-web", not(debug_assertions)))]
async fn embedded_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let file = WEB.get_file(path).or_else(|| WEB.get_file("index.html"));
    let Some(file) = file else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .expect("valid response");
    };
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            mime_guess::from_path(file.path())
                .first_or_octet_stream()
                .as_ref(),
        )
        .body(Body::from(file.contents()))
        .expect("valid response")
}
