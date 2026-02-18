//! RHTMX — HTMX response builders for Axum + Maud.
//!
//! ```ignore
//! use rhtmx::{Ok, Error, Redirect, HtmxRequest};
//!
//! async fn index(hx: HtmxRequest) -> impl axum::response::IntoResponse {
//!     Ok().html(maud::html! { h1 { "Hello" } }).toast("Welcome!")
//! }
//! ```

pub mod response;

pub use axum;
pub use maud;
pub use axum::http::StatusCode;

pub use response::{Ok, Error, Redirect, OkResponse, ErrorResponse, RedirectResponse};

/// HTMX request metadata, extracted from headers.
///
/// ```ignore
/// async fn handler(headers: axum::http::HeaderMap) -> impl IntoResponse {
///     let hx = HtmxRequest::from_headers(&headers);
///     if hx.is_htmx {
///         Ok().html(maud::html! { div { "partial" } })
///     } else {
///         Ok().html(maud::html! { html { body { div { "full page" } } } })
///     }
/// }
/// ```
pub struct HtmxRequest {
    pub is_htmx: bool,
    pub target: Option<String>,
    pub trigger: Option<String>,
}

impl HtmxRequest {
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        Self {
            is_htmx: headers.get("hx-request").is_some(),
            target: headers.get("hx-target").and_then(|v| v.to_str().ok()).map(String::from),
            trigger: headers.get("hx-trigger").and_then(|v| v.to_str().ok()).map(String::from),
        }
    }
}
