mod common;

use axum::http::StatusCode;
use common::{app, content_type, get, send};

// The tests serve `apps/web` (the Vite source); its index.html stands in for the build.

#[tokio::test]
async fn client_routes_fall_back_to_index_html() {
    let app = app().await;
    let res = send(&app, get("/some/client/route")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(content_type(&res).starts_with("text/html"));
}

#[tokio::test]
async fn api_404s_are_not_swallowed_by_the_fallback() {
    let app = app().await;
    let res = send(&app, get("/api/nope")).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
