mod common;

use axum::http::StatusCode;
use common::{app, content_type, get, send};

#[tokio::test]
async fn serves_index_html() {
    let app = app().await;
    let res = send(&app, get("/")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(content_type(&res).starts_with("text/html"));
}
