// ./crates/pilcrow/src/extract.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

// ════════════════════════════════════════════════════════════
// 1. The Unified Mode Enum
// ════════════════════════════════════════════════════════════
#[derive(Debug, PartialEq, Eq)]
pub enum RequestMode {
    Html,
    Json,
    Navigate,
}

// ════════════════════════════════════════════════════════════
// 2. The Extractor Struct
// ════════════════════════════════════════════════════════════
#[derive(Debug, Clone)]
pub struct SilcrowRequest {
    pub is_silcrow: bool,
    pub accepts_html: bool,
    pub accepts_json: bool,
}

#[async_trait]
impl<S> FromRequestParts<S> for SilcrowRequest
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Did silcrow.js send this request?
        let is_silcrow = parts.headers.contains_key("silcrow-target");

        // What data format does the client want?
        let accept = parts
            .headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let accepts_html = accept.contains("text/html");
        let accepts_json = accept.contains("application/json");

        Ok(SilcrowRequest {
            is_silcrow,
            accepts_html,
            accepts_json,
        })
    }
}

// ════════════════════════════════════════════════════════════
// 3. Content Negotiation Logic
// ════════════════════════════════════════════════════════════
impl SilcrowRequest {
    /// Determines the exact format the handler should return based on headers.
    pub fn preferred_mode(&self) -> RequestMode {
        // If it's a Silcrow AJAX request, respect the Accept header strictly
        if self.is_silcrow {
            if self.accepts_html {
                return RequestMode::Html;
            }
            if self.accepts_json {
                return RequestMode::Json;
            }
        }

        // If it's a standard browser hard-refresh, default to HTML
        if self.accepts_html {
            return RequestMode::Html;
        }

        // Ultimate fallback for API clients
        RequestMode::Json
    }
}
