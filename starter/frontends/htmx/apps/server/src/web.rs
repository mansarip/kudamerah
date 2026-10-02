//! Frontend: server-rendered HTML (maud) enhanced with htmx.
//! Handlers return full pages or HTML fragments. Static assets under `/static` can be served
//! from `apps/web` or embedded at release build.

use std::path::Path;

use axum::{Router, routing::get};
#[cfg(all(feature = "embed-web", not(debug_assertions)))]
use axum::{
    body::Body,
    extract::Path as AxumPath,
    http::{StatusCode, header},
    response::Response,
};
#[cfg(all(feature = "embed-web", not(debug_assertions)))]
use include_dir::{Dir, include_dir};
use maud::{DOCTYPE, Markup, html};
#[cfg(any(not(feature = "embed-web"), debug_assertions))]
use tower_http::services::ServeDir;

use crate::AppState;

/// Static assets for development and separate-file releases. Override with `WEB_DIR`.
pub const DEFAULT_DIR: &str = "apps/web";

pub fn router(dir: &Path) -> Router<AppState> {
    let router = Router::new()
        .route("/", get(index))
        .merge(notes_ui::router()) // starter:example
        ;
    #[cfg(all(feature = "embed-web", not(debug_assertions)))]
    {
        let _ = dir;
        router.route("/static/{*path}", get(embedded_asset))
    }
    #[cfg(any(not(feature = "embed-web"), debug_assertions))]
    {
        router.nest_service("/static", ServeDir::new(dir))
    }
}

#[cfg(all(feature = "embed-web", not(debug_assertions)))]
static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../web");

#[cfg(all(feature = "embed-web", not(debug_assertions)))]
async fn embedded_asset(AxumPath(path): AxumPath<String>) -> Response {
    let Some(file) = WEB.get_file(&path) else {
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
