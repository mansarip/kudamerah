//! Example module: a minimal notes CRUD showing the module pattern
//! (routes, handlers and SQL in one file). Copy it as a starting point, or delete it
//! together with its migration and `tests/notes.rs`.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{AppState, error::AppError};

const MAX_BODY_CHARS: usize = 1000;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .route("/notes/{id}", get(get_note).delete(delete_note))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub body: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct NewNote {
    pub body: String,
}

async fn list_notes(State(state): State<AppState>) -> Result<Json<Vec<Note>>, AppError> {
    Ok(Json(list(&state.db).await?))
}

async fn get_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Note>, AppError> {
    Ok(Json(find(&state.db, id).await?))
}

async fn create_note(
    State(state): State<AppState>,
    Json(input): Json<NewNote>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    let note = create(&state.db, &input.body).await?;
    Ok((StatusCode::CREATED, Json(note)))
}

async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Queries. Public so other modules (e.g. server-rendered pages) can reuse them.

pub async fn list(db: &SqlitePool) -> Result<Vec<Note>, AppError> {
    let notes = sqlx::query_as("SELECT id, body, created_at FROM notes ORDER BY id DESC")
        .fetch_all(db)
        .await?;
    Ok(notes)
}

pub async fn find(db: &SqlitePool, id: i64) -> Result<Note, AppError> {
    let note = sqlx::query_as("SELECT id, body, created_at FROM notes WHERE id = ?")
        .bind(id)
        .fetch_one(db)
        .await?;
    Ok(note)
}

pub async fn create(db: &SqlitePool, body: &str) -> Result<Note, AppError> {
    let body = body.trim();
    if body.is_empty() || body.chars().count() > MAX_BODY_CHARS {
        return Err(AppError::BadRequest(format!(
            "body must be 1-{MAX_BODY_CHARS} characters"
        )));
    }
    let note = sqlx::query_as("INSERT INTO notes (body) VALUES (?) RETURNING id, body, created_at")
        .bind(body)
        .fetch_one(db)
        .await?;
    Ok(note)
}

pub async fn delete(db: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
