// RHTMX - Rust + HTMX Framework
// Single-crate web framework with Maud templates and Axum routing

pub mod html;
pub mod request_context;
pub mod config;

// Re-export Maud for templates
pub use maud::{html as maud, Markup, PreEscaped, DOCTYPE};

// Re-export core types and response builders
pub use html::{
    Html,
    OkResponse, ErrorResponse, RedirectResponse,
    Ok, Error, Redirect,
    ok, error, redirect,
    maud_to_html, MaudMarkup,
};

// Re-export framework types
pub use request_context::{RequestContext, QueryParams, FormData};
pub use config::Config;

// Re-export commonly used types from dependencies
pub use axum;
pub use axum::http::StatusCode;
