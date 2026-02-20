// silcrow/src/response/error.rs
// silcrow/crates/silcrow/src/response/error.rs — Silcrow server-side error response builder
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub struct ErrorResponse {
    status: StatusCode,
    message: String,
}

impl ErrorResponse {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody {
            error: self.message,
        });

        (self.status, body).into_response()
    }
}
impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        ErrorResponse::internal(err.to_string())
    }
}
impl From<serde_json::Error> for ErrorResponse {
    fn from(err: serde_json::Error) -> Self {
        ErrorResponse::bad_request(err.to_string())
    }
}
impl From<std::io::Error> for ErrorResponse {
    fn from(err: std::io::Error) -> Self {
        ErrorResponse::internal(err.to_string())
    }
}
