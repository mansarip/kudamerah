use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Error type for handlers. Internal details are logged, never sent to the client.
#[derive(Debug)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_owned()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_owned(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            other => Self::Internal(other.into()),
        }
    }
}
