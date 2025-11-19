// RHTMX - Rust + HTMX Framework
// Compile-time HTML generation with type safety and zero runtime overhead

pub mod html;
pub mod validation;
pub mod form_field;

// Framework modules
pub mod renderer;
pub mod template_loader;
pub mod request_context;
pub mod config;
pub mod database;
pub mod action_executor;
pub mod actions;

// Re-export the html! macro from rhtmx-macro
pub use rhtmx_macro::{html, css, get, post, put, patch, delete};

// Re-export the Validate and FormField derive macros from rhtmx-form
pub use rhtmx_form::{Validate, FormField};

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

// Re-export validation trait
pub use validation::Validate as ValidateTrait;

// Re-export form field types
pub use form_field::{FormField, FieldAttrs};

// Re-export commonly used types from dependencies
pub use axum;
pub use axum::http::StatusCode;
