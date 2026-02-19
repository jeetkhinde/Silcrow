use crate::util::{append_to_mod, find_project_root, to_snake_case, validate_pascal_case, write_if_new};

pub fn run(name: &str) -> Result<(), String> {
    validate_pascal_case(name)?;
    let root = find_project_root()?;
    let snake = to_snake_case(name);

    println!("Generating usecase: {name}");

    write_if_new(
        &root.join(format!("src/application/usecases/{snake}.rs")),
        &usecase_template(name, &snake),
    )?;

    append_to_mod(
        &root.join("src/application/usecases/mod.rs"),
        &format!("pub mod {snake};"),
    ).map_err(|e| e.to_string())?;
    append_to_mod(
        &root.join("src/application/prelude.rs"),
        &format!("pub use crate::application::usecases::{snake}::{name};"),
    ).map_err(|e| e.to_string())?;

    println!("\nUsecase '{}' created.", name);
    println!("  file: src/application/usecases/{snake}.rs");
    println!("\nWire it in your handler by constructing it with the appropriate repository from AppState.");
    Ok(())
}

fn usecase_template(name: &str, _snake: &str) -> String {
    format!(
        r#"use crate::prelude::*;
use std::sync::Arc;
use tracing::instrument;

pub struct {name}<R> {{
    repo: Arc<R>,
}}

impl<R> {name}<R> {{
    pub fn new(repo: Arc<R>) -> Self {{
        Self {{ repo }}
    }}
}}

impl<R: Send + Sync> {name}<R> {{
    #[instrument(skip(self), name = "{name}::execute")]
    pub async fn execute(&self) -> Result<(), ApplicationError> {{
        // TODO: implement usecase logic using self.repo
        Ok(())
    }}
}}
"#
    )
}
