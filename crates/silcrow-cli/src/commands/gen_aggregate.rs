use crate::util::{append_to_mod, find_project_root, insert_at_marker, to_snake_case, validate_pascal_case, write_if_new};

pub fn run(name: &str) -> Result<(), String> {
    validate_pascal_case(name)?;
    let root = find_project_root()?;
    let snake = to_snake_case(name);

    println!("Generating aggregate: {name}");

    // 1. Domain entity
    write_if_new(
        &root.join(format!("src/domain/entities/{snake}.rs")),
        &entity_template(name),
    )?;
    append_to_mod(
        &root.join("src/domain/entities/mod.rs"),
        &format!("pub mod {snake};"),
    ).map_err(|e| e.to_string())?;
    append_to_mod(
        &root.join("src/domain/prelude.rs"),
        &format!("pub use crate::domain::entities::{snake}::{name};"),
    ).map_err(|e| e.to_string())?;

    // 2. Domain repository trait
    let repo_file = format!("{snake}_repository");
    write_if_new(
        &root.join(format!("src/domain/repositories/{repo_file}.rs")),
        &repo_trait_template(name, &snake),
    )?;
    append_to_mod(
        &root.join("src/domain/repositories/mod.rs"),
        &format!("pub mod {repo_file};"),
    ).map_err(|e| e.to_string())?;
    append_to_mod(
        &root.join("src/domain/prelude.rs"),
        &format!("pub use crate::domain::repositories::{repo_file}::{name}Repository;"),
    ).map_err(|e| e.to_string())?;

    // 3. Infrastructure repository implementation
    let impl_file = format!("{snake}_repository_impl");
    write_if_new(
        &root.join(format!("src/infrastructure/repositories/{impl_file}.rs")),
        &repo_impl_template(name, &snake),
    )?;
    append_to_mod(
        &root.join("src/infrastructure/repositories/mod.rs"),
        &format!("pub(crate) mod {impl_file};"),
    ).map_err(|e| e.to_string())?;
    append_to_mod(
        &root.join("src/infrastructure/prelude.rs"),
        &format!("pub use crate::infrastructure::repositories::{impl_file}::Pg{name}Repository;"),
    ).map_err(|e| e.to_string())?;

    // 4. Update main.rs AppState
    let main_rs = root.join("src/main.rs");
    let pg_type = format!("Pg{name}Repository");

    insert_at_marker(
        &main_rs,
        "REPO_FIELDS",
        &format!("    pub {snake}_repo: Arc<infrastructure::repositories::{impl_file}::{pg_type}>,"),
    )?;

    insert_at_marker(
        &main_rs,
        "REPO_INIT",
        &format!("    let {snake}_repo = Arc::new(infrastructure::repositories::{impl_file}::{pg_type}::new(db.clone()));"),
    )?;

    insert_at_marker(
        &main_rs,
        "STATE_FIELDS",
        &format!("        {snake}_repo,"),
    )?;

    println!("\nAggregate '{}' created.", name);
    println!("  entity:      src/domain/entities/{snake}.rs");
    println!("  repo trait:  src/domain/repositories/{repo_file}.rs");
    println!("  repo impl:   src/infrastructure/repositories/{impl_file}.rs");
    println!("  AppState:    updated with {snake}_repo field");
    Ok(())
}

fn entity_template(name: &str) -> String {
    format!(
        r#"use crate::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {name} {{
    pub id: i64,
    // TODO: add fields
}}

impl {name} {{
    pub fn new(id: i64) -> Self {{
        Self {{ id }}
    }}
}}
"#
    )
}

fn repo_trait_template(name: &str, _snake: &str) -> String {
    format!(
        r#"use crate::prelude::*;

pub trait {name}Repository: Send + Sync {{
    fn find_by_id(&self, id: i64) -> impl std::future::Future<Output = Result<Option<{name}>, DomainError>> + Send;
    fn save(&self, entity: &{name}) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;
    fn delete(&self, id: i64) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;
}}
"#
    )
}

fn repo_impl_template(name: &str, snake: &str) -> String {
    format!(
        r#"use crate::prelude::*;
use sqlx::PgPool;

pub struct Pg{name}Repository {{
    pool: PgPool,
}}

impl Pg{name}Repository {{
    pub fn new(pool: PgPool) -> Self {{
        Self {{ pool }}
    }}
}}

impl {name}Repository for Pg{name}Repository {{
    async fn find_by_id(&self, id: i64) -> Result<Option<{name}>, DomainError> {{
        // TODO: implement query
        // sqlx::query_as!(
        //     {name},
        //     "SELECT * FROM {snake}s WHERE id = $1",
        //     id
        // )
        // .fetch_optional(&self.pool)
        // .await
        // .map_err(|e| DomainError::Internal(e.to_string()))
        todo!("Implement find_by_id")
    }}

    async fn save(&self, _entity: &{name}) -> Result<(), DomainError> {{
        todo!("Implement save")
    }}

    async fn delete(&self, _id: i64) -> Result<(), DomainError> {{
        todo!("Implement delete")
    }}
}}
"#
    )
}
