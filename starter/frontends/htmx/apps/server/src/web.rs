//! Frontend: server-rendered HTML (maud) enhanced with htmx.
//! Handlers return full pages or HTML fragments. `apps/web` holds static assets under `/static`.

use std::path::Path;

use axum::{Router, routing::get};
use maud::{DOCTYPE, Markup, html};
use tower_http::services::ServeDir;

use crate::AppState;

/// Static assets (CSS, vendored htmx). Override with `WEB_DIR`.
pub const DEFAULT_DIR: &str = "apps/web";

pub fn router(dir: &Path) -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .merge(notes_ui::router()) // starter:example
        .nest_service("/static", ServeDir::new(dir))
}

fn page(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="icon" href="data:,";
                link rel="stylesheet" href="/static/styles.css";
                script src="/static/htmx.min.js" defer {}
            }
            body { main { (content) } }
        }
    }
}

async fn index() -> Markup {
    page(
        "Kudamerah",
        html! {
            h1 { "Kudamerah" }
            p { "Server-rendered with maud, enhanced with htmx." }
            div hx-get="/ui/notes" hx-trigger="load" hx-swap="outerHTML" { "Loading notes…" } // starter:example
        },
    )
}

// starter:example:begin
/// Notes example: HTML fragments for htmx, reusing the queries in `modules::notes`.
mod notes_ui {
    use axum::{
        Form, Router,
        extract::{Path, State},
        routing::{delete, get},
    };
    use maud::{Markup, html};
    use serde::Deserialize;

    use crate::{
        AppState,
        error::AppError,
        modules::notes::{self, Note},
    };

    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/ui/notes", get(show).post(create))
            .route("/ui/notes/{id}", delete(remove))
    }

    #[derive(Deserialize)]
    struct NoteForm {
        body: String,
    }

    async fn show(State(state): State<AppState>) -> Result<Markup, AppError> {
        Ok(section(&notes::list(&state.db).await?))
    }

    /// Returns the whole section so the form comes back cleared.
    async fn create(
        State(state): State<AppState>,
        Form(form): Form<NoteForm>,
    ) -> Result<Markup, AppError> {
        notes::create(&state.db, &form.body).await?;
        show(State(state)).await
    }

    /// Empty 200 response: htmx swaps the `<li>` out.
    async fn remove(State(state): State<AppState>, Path(id): Path<i64>) -> Result<(), AppError> {
        notes::delete(&state.db, id).await
    }

    fn section(notes: &[Note]) -> Markup {
        html! {
            section id="notes" {
                h2 { "Notes" }
                form hx-post="/ui/notes" hx-target="#notes" hx-swap="outerHTML" {
                    input name="body" required maxlength="1000" placeholder="Write a note…" aria-label="Note";
                    button { "Add" }
                }
                ul {
                    @for note in notes {
                        li {
                            (note.body)
                            " "
                            button hx-delete={ "/ui/notes/" (note.id) } hx-target="closest li" hx-swap="outerHTML" { "Delete" }
                        }
                    }
                }
            }
        }
    }
}
// starter:example:end
