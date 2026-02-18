pub mod response;

pub use axum;
pub use axum::http::StatusCode;
pub use maud;

pub use response::{Error, ErrorResponse, Ok, OkResponse, Redirect, RedirectResponse};

pub struct HtmxRequest {
    pub is_htmx: bool,
    pub target: Option<String>,
    pub trigger: Option<String>,
}

impl HtmxRequest {
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        Self {
            is_htmx: headers.get("hx-request").is_some(),
            target: headers
                .get("hx-target")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            trigger: headers
                .get("hx-trigger")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        }
    }
}
