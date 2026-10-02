use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{AppState, error::AppError};

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

async fn health(State(state): State<AppState>) -> Result<Json<Health>, AppError> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(Health { status: "ok" }))
}
