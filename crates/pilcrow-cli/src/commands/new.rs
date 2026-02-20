use crate::util::write_if_new;
use std::path::Path;

pub fn run(name: &str) -> Result<(), String> {
    let root = Path::new(name);
    if root.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }

    println!("Creating service: {name}");

    // Cargo.toml
    write_if_new(
        &root.join("Cargo.toml"),
        &cargo_toml(name),
    )?;

    // silcrow.toml
    write_if_new(
        &root.join("silcrow.toml"),
        &silcrow_toml(name),
    )?;

    // migrations/.gitkeep
    write_if_new(&root.join("migrations/.gitkeep"), "")?;

    // src/main.rs
    write_if_new(&root.join("src/main.rs"), MAIN_RS)?;

    // domain layer
    write_if_new(&root.join("src/domain/mod.rs"), DOMAIN_MOD)?;
    write_if_new(&root.join("src/domain/entities/mod.rs"), "")?;
    write_if_new(&root.join("src/domain/repositories/mod.rs"), "")?;
    write_if_new(&root.join("src/domain/errors.rs"), DOMAIN_ERRORS)?;

    // application layer
    write_if_new(&root.join("src/application/mod.rs"), APPLICATION_MOD)?;
    write_if_new(&root.join("src/application/usecases/mod.rs"), "")?;
    write_if_new(&root.join("src/application/errors.rs"), APPLICATION_ERRORS)?;

    // infrastructure layer
    write_if_new(&root.join("src/infrastructure/mod.rs"), INFRASTRUCTURE_MOD)?;
    write_if_new(&root.join("src/infrastructure/repositories/mod.rs"), "")?;

    // presentation layer
    write_if_new(&root.join("src/presentation/mod.rs"), PRESENTATION_MOD)?;
    write_if_new(&root.join("src/presentation/handlers/mod.rs"), "")?;
    write_if_new(&root.join("src/presentation/templates/mod.rs"), "")?;

    // preludes
    write_if_new(&root.join("src/prelude.rs"), ROOT_PRELUDE)?;
    write_if_new(&root.join("src/domain/prelude.rs"), DOMAIN_PRELUDE)?;
    write_if_new(&root.join("src/application/prelude.rs"), APPLICATION_PRELUDE)?;
    write_if_new(&root.join("src/infrastructure/prelude.rs"), INFRASTRUCTURE_PRELUDE)?;
    write_if_new(&root.join("src/presentation/prelude.rs"), PRESENTATION_PRELUDE)?;

    // routes
    write_if_new(&root.join("src/routes.rs"), ROUTES_RS)?;

    println!("\nService '{}' created successfully.", name);
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  export DATABASE_URL=postgres://user:pass@localhost/db");
    println!("  cargo run");
    Ok(())
}

fn cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
silcrow = "0.1"
axum = "0.7"
sqlx = {{ version = "0.8", features = ["runtime-tokio", "postgres"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
maud = "0.26"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
tokio = {{ version = "1", features = ["full"] }}
"#
    )
}

fn silcrow_toml(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"

[server]
host = "127.0.0.1"
port = 3000
"#
    )
}

const MAIN_RS: &str = r#"mod domain;
mod application;
mod infrastructure;
mod presentation;
mod routes;
mod prelude;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
    // SC:REPO_FIELDS
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = silcrow::SilcrowConfig::load();
    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&db).await?;

    // SC:REPO_INIT
    let state = Arc::new(AppState {
        db,
        // SC:STATE_FIELDS
    });

    let app = Router::new()
        .route(silcrow::SILCROW_JS_PATH, get(silcrow::serve_silcrow_js))
        .merge(routes::routes(state.clone()))
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
"#;

const DOMAIN_MOD: &str = r#"pub mod entities;
pub mod repositories;
pub mod errors;
pub mod prelude;
"#;

const DOMAIN_ERRORS: &str = r#"#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}
"#;

const APPLICATION_MOD: &str = r#"pub mod usecases;
pub mod errors;
pub mod prelude;
"#;

const APPLICATION_ERRORS: &str = r#"use crate::domain::errors::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DomainError> for ApplicationError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => ApplicationError::NotFound(msg),
            DomainError::Validation(msg) => ApplicationError::Validation(msg),
            DomainError::Internal(msg) => ApplicationError::Internal(msg),
        }
    }
}

impl From<sqlx::Error> for ApplicationError {
    fn from(err: sqlx::Error) -> Self {
        ApplicationError::Internal(err.to_string())
    }
}
"#;

const INFRASTRUCTURE_MOD: &str = r#"pub(crate) mod repositories;
pub mod prelude;
"#;

const PRESENTATION_MOD: &str = r#"pub(crate) mod handlers;
pub(crate) mod templates;
pub mod prelude;

use axum::response::{IntoResponse, Response};
use silcrow::{Error, StatusCode, Respond};
use crate::application::errors::ApplicationError;

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        match self {
            ApplicationError::NotFound(msg) => {
                Error().message(msg).status(StatusCode::NOT_FOUND).ok()
            }
            ApplicationError::Validation(msg) => {
                Error().message(msg).status(StatusCode::UNPROCESSABLE_ENTITY).ok()
            }
            ApplicationError::Internal(msg) => {
                Error().message(msg).status(StatusCode::INTERNAL_SERVER_ERROR).ok()
            }
        }
    }
}
"#;

const ROOT_PRELUDE: &str = r#"pub use crate::domain::prelude::*;
pub use crate::application::prelude::*;
pub use crate::infrastructure::prelude::*;
pub use crate::presentation::prelude::*;
"#;

const DOMAIN_PRELUDE: &str = r#"// Domain types — re-export entities, repository traits, and errors here.
pub use crate::domain::errors::DomainError;
"#;

const APPLICATION_PRELUDE: &str = r#"// Application types — re-export usecases and errors here.
pub use crate::application::errors::ApplicationError;
"#;

const INFRASTRUCTURE_PRELUDE: &str = r#"// Infrastructure types — re-export repository implementations here.
"#;

const PRESENTATION_PRELUDE: &str = r#"// Presentation types — re-export handlers and templates here.
"#;

const ROUTES_RS: &str = r#"use std::sync::Arc;
use axum::Router;
use crate::AppState;

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
    // SC:ROUTES
}
"#;
