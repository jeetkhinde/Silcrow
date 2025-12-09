// RHTMX - Rust + HTMX Framework
// Type-safe web framework with Maud templates and zero runtime overhead

pub mod html;
pub mod validation;
pub mod form_field;
pub mod value;

// Framework modules
pub mod renderer;
pub mod template_loader;
pub mod request_context;
pub mod config;
pub mod database;
pub mod action_executor;
pub mod actions;

// Re-export HTTP handler macros from rhtmx-macro
pub use rhtmx_macro::{get, post, put, patch, delete};

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
pub use renderer::Renderer;
pub use template_loader::TemplateLoader;
pub use request_context::{RequestContext, QueryParams, FormData};
pub use config::Config;
pub use actions::ActionRegistry;
pub use value::Value;

// Re-export validation trait (backwards compatibility)
pub use validation::Validate as ValidateTrait;

// Re-export form field types
pub use form_field::{FormField, FieldAttrs};

// Re-export commonly used types from dependencies
pub use axum;
pub use axum::http::StatusCode;
