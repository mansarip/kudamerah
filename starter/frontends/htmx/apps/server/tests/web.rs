mod common;

use axum::http::StatusCode;
use common::{app, content_type, get, send, text};

#[tokio::test]
async fn renders_index_page() {
    let app = app().await;
    let res = send(&app, get("/")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(content_type(&res).starts_with("text/html"));
    assert!(text(res).await.contains("/static/htmx.min.js"));
}

#[tokio::test]
async fn serves_static_assets() {
    let app = app().await;
    let res = send(&app, get("/static/htmx.min.js")).await;
    assert_eq!(res.status(), StatusCode::OK);
}

// starter:example:begin
#[tokio::test]
async fn form_post_returns_updated_section() {
    let app = app().await;
    let res = send(&app, common::post_form("/ui/notes", "body=%3Cb%3Ehi%3C%2Fb%3E")).await;
    assert_eq!(res.status(), StatusCode::OK);
    // User input is escaped by maud.
    assert!(text(res).await.contains("&lt;b&gt;hi&lt;/b&gt;"));
}
// starter:example:end
