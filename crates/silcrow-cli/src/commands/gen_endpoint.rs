use crate::util::{
    append_to_mod, find_project_root, insert_at_marker, to_snake_case, validate_pascal_case,
    write_if_new,
};

pub fn run(name: &str, method: &str, path: &str) -> Result<(), String> {
    validate_pascal_case(name)?;
    let root = find_project_root()?;
    let snake = to_snake_case(name);

    let method_lower = method.to_lowercase();
    let valid_methods = ["get", "post", "put", "patch", "delete"];
    if !valid_methods.contains(&method_lower.as_str()) {
        return Err(format!("Invalid HTTP method: '{}'. Use one of: GET, POST, PUT, PATCH, DELETE", method));
    }

    println!("Generating endpoint: {name} ({method} {path})");

    // 1. Handler file
    write_if_new(
        &root.join(format!("src/presentation/handlers/{snake}.rs")),
        &handler_template(name, &snake),
    )?;

    // 2. Update handlers/mod.rs
    append_to_mod(
        &root.join("src/presentation/handlers/mod.rs"),
        &format!("pub(crate) mod {snake};"),
    )
    .map_err(|e| e.to_string())?;
    append_to_mod(
        &root.join("src/presentation/prelude.rs"),
        &format!("pub use crate::presentation::handlers::{snake}::{snake};"),
    )
    .map_err(|e| e.to_string())?;

    // 3. Register route in routes.rs
    let route_line = format!(
        "        .route(\"{path}\", axum::routing::{method_lower}(presentation::handlers::{snake}::{snake}))"
    );
    insert_at_marker(
        &root.join("src/routes.rs"),
        "ROUTES",
        &route_line,
    )?;

    println!("\nEndpoint '{}' created.", name);
    println!("  handler:  src/presentation/handlers/{snake}.rs");
    println!("  route:    {method} {path}");
    Ok(())
}

fn handler_template(name: &str, snake: &str) -> String {
    format!(
        r#"use crate::prelude::*;
use std::sync::Arc;
use axum::extract::State;
use axum::response::Response;
use silcrow::{{SilcrowRequest, JsonOk, Respond}};
use tracing::instrument;
use crate::AppState;

/// Handler for {name}
#[instrument(skip(state, _req), name = "handler::{snake}")]
pub async fn {snake}(
    State(state): State<Arc<AppState>>,
    _req: SilcrowRequest,
) -> Result<Response, ApplicationError> {{
    // TODO: construct usecase from state, call execute
    Ok(JsonOk().set("status", "ok").ok())
}}
"#
    )
}
