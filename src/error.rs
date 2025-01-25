use axum::{http::StatusCode, response::IntoResponse};
use displaydoc::Display;
use thiserror::Error;

/// An error that can occur in the application.
#[derive(Debug, Display, Error)]
pub enum AppError {
    /// Internal server error: {0}
    EyreError(#[from] eyre::Report),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
