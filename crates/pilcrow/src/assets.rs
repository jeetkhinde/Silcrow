// ./crates/pilcrow/src/assets.rs

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

/// The unified Silcrow client runtime, embedded at compile time.
pub const SILCROW_JS: &str = include_str!("../public/silcrow.js");

/// Canonical URL path for serving the Silcrow JS bundle.
pub const SILCROW_JS_PATH: &str = "/_silcrow/silcrow.js";

/// Axum handler that serves the embedded Silcrow JS with aggressive caching.
pub async fn serve_silcrow_js() -> Response {
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            ),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        SILCROW_JS,
    )
        .into_response()
}

/// Returns a raw HTML `<script>` tag pointing to the Silcrow JS bundle.
pub fn script_tag() -> &'static str {
    "<script src=\"/_silcrow/silcrow.js\" defer></script>"
}
