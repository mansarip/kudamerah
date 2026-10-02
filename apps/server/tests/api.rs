mod common;

use axum::http::StatusCode;
use common::{app, get, json, send};

#[tokio::test]
async fn health_reports_ok() {
    let app = app().await;
    let res = send(&app, get("/api/health")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await["status"], "ok");
}

#[tokio::test]
async fn unknown_api_route_is_json_404() {
    let app = app().await;
    let res = send(&app, get("/api/nope")).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(res).await["error"], "not found");
}
