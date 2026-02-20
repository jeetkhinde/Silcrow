// silcrow/src/response/mod.rs
// silcrow/crates/silcrow/src/response/mod.rs — Silcrow server-side response types and builders for HTML, JSON, redirects, and errors
//
// Docs default to plain Axum + Maud returns (`Markup`, `(StatusCode, Html<_>)`,
// and `Result<_, AppError>`). Use these Silcrow response builders when protocol
// ergonomics are helpful (e.g. `_toast` JSON payloads, fluent status/header setup,
// or explicit Silcrow-oriented branching between JSON and HTML).
pub mod base;
pub mod error;
pub mod html;
pub mod json;
pub mod macros;
pub mod redirect;
pub use error::ErrorResponse;
pub use html::HtmlOkResponse;
pub use json::JsonOkResponse;
pub use redirect::RedirectResponse;

// Clean public type aliases (convenience usage for Silcrow-oriented responses)
pub type Json = JsonOkResponse;
pub type Html = HtmlOkResponse;
pub type Redirect = RedirectResponse;
pub type Error = ErrorResponse;

/// Framework result type
pub type Result<T> = std::result::Result<T, ErrorResponse>;
