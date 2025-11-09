pub mod action_executor;
pub mod action_handlers;
pub mod actions;
pub mod component;
pub mod config;
pub mod database;
pub mod example_actions;
pub mod form_context;
pub mod hot_reload;
pub mod html;
pub mod renderer;
pub mod request_context;
pub mod template_loader;
pub mod validation;
pub mod validation_pipeline;

// Re-export router from rhtml-router crate
pub use rhtml_router::{Route, RouteMatch, Router};

// Re-export html! macro from rhtml-macro crate
pub use rhtml_macro::html;

pub use action_executor::{deserialize_form, ActionResult, form_to_json};
pub use action_handlers::{ActionHandler, ActionHandlerRegistry, register_built_in_handlers};
pub use actions::{ActionInfo, ActionMethod, ActionRegistry, ActionResponse, Empty, ResultExt};
pub use component::{Component, ComponentRegistry, get_component, register_component};
pub use config::Config;
pub use form_context::FormContext;
pub use html::{Html, OkResponse, ErrorResponse, RedirectResponse, Ok, Error, Redirect, ok, error, redirect};
pub use renderer::{LayoutDirective, Renderer};
pub use request_context::{FormData, QueryParams, RequestContext};
pub use rhtml_parser::{DirectiveParser, ExpressionEvaluator};
pub use template_loader::{Template, TemplateLoader};
pub use validation::{Validate, ValidationResult};
pub use validation_pipeline::{validate_request, ValidationPipelineResult};
