use crate::util::{append_to_mod, find_project_root, to_snake_case, validate_pascal_case, write_if_new};

pub fn run(name: &str) -> Result<(), String> {
    validate_pascal_case(name)?;
    let root = find_project_root()?;
    let snake = to_snake_case(name);

    println!("Generating template: {name}");

    write_if_new(
        &root.join(format!("src/presentation/templates/{snake}.rs")),
        &template_template(name, &snake),
    )?;

    append_to_mod(
        &root.join("src/presentation/templates/mod.rs"),
        &format!("pub(crate) mod {snake};"),
    ).map_err(|e| e.to_string())?;
    append_to_mod(
        &root.join("src/presentation/prelude.rs"),
        &format!("pub use crate::presentation::templates::{snake}::{snake};"),
    ).map_err(|e| e.to_string())?;

    println!("\nTemplate '{}' created.", name);
    println!("  file: src/presentation/templates/{snake}.rs");
    Ok(())
}

fn template_template(name: &str, snake: &str) -> String {
    format!(
        r#"use crate::prelude::*;
use maud::{{html, Markup}};

/// Render the {name} template.
pub fn {snake}() -> Markup {{
    html! {{
        div class="{snake}" {{
            h2 {{ "{name}" }}
            // TODO: add template content
        }}
    }}
}}
"#
    )
}
