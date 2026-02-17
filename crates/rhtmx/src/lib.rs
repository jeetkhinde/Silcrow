// RHTMX - Rust + HTMX Framework
// Type-safe web framework with Maud templates and zero runtime overhead

pub mod html;

// Framework modules
pub mod template_loader;
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
};

// Re-export framework types
pub use template_loader::{TemplateLoader, HandlerFn, PageRoute};
pub use request_context::{RequestContext, QueryParams, FormData};
pub use config::Config;

// Re-export commonly used types from dependencies
pub use axum;
pub use axum::http::StatusCode;
