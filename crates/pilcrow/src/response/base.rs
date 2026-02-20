// silcrow/src/response/base.rs
// silcrow/crates/silcrow/src/response/base.rs — Silcrow server-side base response type and shared finalizer for HTML, JSON, redirect, and error responses
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

pub struct BaseResponse {
    headers: HeaderMap,
    status: StatusCode,
}

impl BaseResponse {
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            status: StatusCode::OK,
        }
    }

    pub fn header(&mut self, key: &str, value: &str) {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            self.headers.insert(name, val);
        }
    }

    pub fn status(&mut self, status: StatusCode) {
        self.status = status;
    }
    pub fn insert_header(&mut self, key: HeaderName, value: HeaderValue) {
        self.headers.insert(key, value);
    }
    pub fn no_cache(&mut self) {
        self.insert_header(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-store"),
        );
    }
    pub fn status_code(&self) -> StatusCode {
        self.status
    }
}

/*
pub trait SilcrowResponse {
    fn into_silcrow_response(self) -> Response;
}
*/
// shared finalizer
pub fn finalize_response(base: BaseResponse, body: impl IntoResponse) -> Response {
    (base.status, base.headers, body).into_response()
}
impl Default for BaseResponse {
    fn default() -> Self {
        Self::new()
    }
}
