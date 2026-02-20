// silcrow/crates/silcrow/lib.rs — Silcrow server-side library for building Silcrow-compatible web applications
//
// Public docs/examples should default to plain Axum + Maud handler returns.
//
// Typical patterns:
// - `Markup` for full-page SSR templates.
// - `(StatusCode, axum::response::Html<String>)` for explicit HTML fragment responses.
// - `Result<T, AppError>` for domain/application error flow where `AppError: IntoResponse`.
//
// `JsonOkResponse` / `HtmlOkResponse` remain available as convenience builders when you
// specifically want Silcrow protocol-oriented helpers (for example `_toast` payloads,
// Silcrow-focused headers, or builder-style mutation of JSON/HTML responses).
pub mod assets;
pub mod config;
pub mod layout;
pub mod response;

pub use assets::{script_tag, serve_silcrow_js, SILCROW_JS_PATH};
pub use axum;
pub use axum::http::StatusCode;
pub use axum::response::Response;
pub use config::SilcrowConfig;
pub use layout::page;
pub use maud;
pub use response::*;

/// Parsed Silcrow request metadata from incoming headers.
pub struct SilcrowRequest {
    /// `true` when the `silcrow-target` header is present (request came from Silcrow navigation).
    pub is_silcrow: bool,
    /// `true` when `Accept` header contains `text/html` (element has `s-html` attribute).
    pub wants_html: bool,
    /// `true` when `Accept` header contains `application/json` (default for `s-action` without `s-html`).
    pub wants_json: bool,
}

impl SilcrowRequest {
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        let accept = headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        Self {
            is_silcrow: headers.get("silcrow-target").is_some(),
            wants_html: accept.contains("text/html"),
            wants_json: accept.contains("application/json"),
        }
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for SilcrowRequest
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Result::Ok(Self::from_headers(&parts.headers))
    }
}
