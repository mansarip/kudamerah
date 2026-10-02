mod common;

use axum::http::StatusCode;
use common::{app, delete, get, json, post_json, send};
use serde_json::json;

#[tokio::test]
async fn create_list_get_delete() {
    let app = app().await;

    let res = send(
        &app,
        post_json("/api/notes", json!({ "body": "  hello  " })),
    )
    .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let note = json(res).await;
    assert_eq!(note["body"], "hello");
    let id = note["id"].as_i64().unwrap();

    let list = json(send(&app, get("/api/notes")).await).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    let res = send(&app, get(&format!("/api/notes/{id}"))).await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = send(&app, delete(&format!("/api/notes/{id}"))).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = send(&app, delete(&format!("/api/notes/{id}"))).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_blank_body() {
    let app = app().await;
    let res = send(&app, post_json("/api/notes", json!({ "body": "   " }))).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(json(res).await["error"].is_string());
}
