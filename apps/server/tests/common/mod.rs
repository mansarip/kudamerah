// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, header},
    response::Response,
};
use http_body_util::BodyExt;
use kudamerah_server::{AppState, config::Config, db};
use tower::ServiceExt;

/// A fresh app backed by its own in-memory database.
pub async fn app() -> Router {
    let config = Config {
        database_path: ":memory:".into(),
        web_dir: "../web".into(), // starter:web
        ..Config::default()
    };
    let db = db::connect(&config.database_path).await.unwrap();
    kudamerah_server::app(AppState {
        db,
        config: Arc::new(config),
    })
}

pub async fn send(app: &Router, req: Request<Body>) -> Response {
    app.clone().oneshot(req).await.unwrap()
}

pub fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

pub fn delete(uri: &str) -> Request<Body> {
    Request::delete(uri).body(Body::empty()).unwrap()
}

pub fn post_json(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub fn post_form(uri: &str, body: &str) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

pub fn content_type(res: &Response) -> &str {
    res.headers()[header::CONTENT_TYPE].to_str().unwrap()
}

pub async fn text(res: Response) -> String {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

pub async fn json(res: Response) -> serde_json::Value {
    serde_json::from_str(&text(res).await).unwrap()
}
