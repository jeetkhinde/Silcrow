// silcrow/src/response/redirect.rs
// silcrow/crates/silcrow/src/response/redirect.rs — Silcrow server-side HTTP redirect response builder
use super::base::{finalize_response, BaseResponse};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
pub struct RedirectResponse {
    base: BaseResponse,
    location: Option<String>,
}

impl RedirectResponse {
    pub fn new() -> Self {
        let mut base = BaseResponse::new();
        base.status(StatusCode::SEE_OTHER);

        Self {
            base,
            location: None,
        }
    }
    pub fn to(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.base.status(status);
        self
    }
}

impl Default for RedirectResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for RedirectResponse {
    fn into_response(mut self) -> Response {
        if let Some(loc) = self.location {
            let val = HeaderValue::from_str(&loc).expect("Invalid redirect location");

            self.base.insert_header(header::LOCATION, val);
        }

        finalize_response(self.base, "")
    }
}
