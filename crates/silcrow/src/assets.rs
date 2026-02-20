// silcrow/crates/silcrow/src/assets.rs — Silcrow embedded assets and asset-serving utilities
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

/// The unified Silcrow client runtime, embedded at compile time.
pub const SILCROW_JS: &str = include_str!("../silcrow.js");

/// Canonical URL path for serving the Silcrow JS bundle.
pub const SILCROW_JS_PATH: &str = "/_silcrow/silcrow.js";

/// Axum handler that serves the embedded Silcrow JS with aggressive caching.
///
/// Wire into your router:
/// ```rust
/// use silcrow::assets;
/// let app = Router::new()
///     .route(assets::SILCROW_JS_PATH, get(assets::serve_silcrow_js));
/// ```
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

/// Returns a `<script>` tag pointing to the Silcrow JS bundle.
///
/// Use in Maud templates:
/// ```rust
/// use silcrow::assets::script_tag;
/// html! {
///     head { (script_tag()) }
/// }
/// ```
pub fn script_tag() -> maud::Markup {
    maud::html! {
        script src=(SILCROW_JS_PATH) defer {}
    }
}
