//! Frontend: plain HTML/CSS/JS from `apps/web`, served from disk or embedded at release build.

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
use tower_http::services::ServeDir;

use crate::AppState;

/// Asset directory for development and separate-file releases. Override with `WEB_DIR`.
pub const DEFAULT_DIR: &str = "apps/web";

pub fn router(dir: &Path) -> Router<AppState> {
    #[cfg(all(feature = "embed-web", not(debug_assertions)))]
    {
        let _ = dir;
        Router::new().fallback(embedded_asset)
    }
    #[cfg(any(not(feature = "embed-web"), debug_assertions))]
    {
        Router::new().fallback_service(ServeDir::new(dir))
    }
}

#[cfg(all(feature = "embed-web", not(debug_assertions)))]
static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../web");

#[cfg(all(feature = "embed-web", not(debug_assertions)))]
async fn embedded_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let Some(file) = WEB.get_file(path) else {
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
