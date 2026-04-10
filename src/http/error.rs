use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use crate::metadata::openlibrary::OpenLibraryError;

pub enum AppError {
    Metadata(OpenLibraryError),
    Database(sqlx::Error),
    NotFound(String),
    BadRequest(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Metadata(OpenLibraryError::Timeout(_)) => (StatusCode::GATEWAY_TIMEOUT, "Metadata service timed out".to_string()),
            Self::Metadata(OpenLibraryError::NoMatch) => (StatusCode::NOT_FOUND, "No matching work found".to_string()),
            Self::Metadata(_) => (StatusCode::BAD_GATEWAY, "Error from metadata service".to_string()),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred".to_string()),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<OpenLibraryError> for AppError {
    fn from(inner: OpenLibraryError) -> Self { Self::Metadata(inner) }
}

impl From<sqlx::Error> for AppError {
    fn from(inner: sqlx::Error) -> Self { Self::Database(inner) }
}

impl From<anyhow::Error> for AppError {
    fn from(inner: anyhow::Error) -> Self { Self::Internal(inner) }
}
