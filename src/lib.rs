pub mod action_executor;
pub mod action_handlers;
pub mod actions;
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

// Re-export router from rhtmx-router crate
pub use rhtmx_router::{Route, RouteMatch, Router};

// Re-export macros from rhtmx-macro crate
pub use rhtmx_macro::{html, get, post, put, patch, delete};

pub use action_executor::{deserialize_form, form_to_json, ActionResult};
pub use action_handlers::{register_built_in_handlers, ActionHandler, ActionHandlerRegistry};
pub use actions::{ActionInfo, ActionMethod, ActionRegistry, ActionResponse, Empty, ResultExt};
// pub use component::{get_component, register_component, Component, ComponentRegistry};  // TODO: Re-enable when component module is available
pub use config::Config;
pub use form_context::FormContext;
pub use html::{
    error, ok, redirect, Error, ErrorResponse, Html, Ok, OkResponse, Redirect, RedirectResponse,
};
pub use renderer::{LayoutDirective, Renderer};
pub use request_context::{FormData, QueryParams, RequestContext};
pub use rhtmx_parser::{DirectiveParser, ExpressionEvaluator};
pub use template_loader::{Template, TemplateLoader};
pub use validation::{Validate, ValidationResult};
pub use validation_pipeline::{validate_request, ValidationPipelineResult};
