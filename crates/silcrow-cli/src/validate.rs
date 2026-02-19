use crate::util::find_project_root;
use std::path::Path;

struct Violation {
    file: String,
    line: usize,
    layer: String,
    import: String,
    reason: String,
}

pub fn run() -> Result<(), String> {
    let root = find_project_root()?;
    let src = root.join("src");

    if !src.exists() {
        return Err("No src/ directory found".into());
    }

    let mut violations = Vec::new();

    for entry in walkdir::WalkDir::new(&src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map_or(true, |ext| ext != "rs") {
            continue;
        }

        let layer = match classify_layer(&src, path) {
            Some(l) => l,
            None => continue, // main.rs or root files — skip
        };

        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for (line_num, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("use ") && !trimmed.starts_with("pub use ") {
                continue;
            }

            check_violation(&layer, trimmed, &rel_path, line_num + 1, &mut violations);
        }
    }

    if violations.is_empty() {
        println!("Architecture check PASSED. No violations found.");
        Ok(())
    } else {
        for v in &violations {
            eprintln!(
                "\nVIOLATION [{}] {}:{}",
                v.layer, v.file, v.line
            );
            eprintln!("  {}", v.import);
            eprintln!("  → {}", v.reason);
        }
        eprintln!(
            "\nFound {} violation(s). Architecture check FAILED.",
            violations.len()
        );
        std::process::exit(1);
    }
}

fn classify_layer(src: &Path, file: &Path) -> Option<String> {
    let relative = file.strip_prefix(src).ok()?;
    let first_component = relative.components().next()?;
    let dir_name = first_component.as_os_str().to_str()?;

    match dir_name {
        "domain" => Some("domain".into()),
        "application" => Some("application".into()),
        "infrastructure" => Some("infrastructure".into()),
        "presentation" => Some("presentation".into()),
        _ => None,
    }
}

fn check_violation(
    layer: &str,
    import_line: &str,
    file: &str,
    line: usize,
    violations: &mut Vec<Violation>,
) {
    match layer {
        "domain" => {
            // Domain cannot import from any other layer
            if contains_crate_import(import_line, &[
                "crate::infrastructure",
                "crate::presentation",
                "crate::application",
            ]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Domain cannot import from other layers".into(),
                });
            }
            // Domain cannot use framework crates
            if contains_external_import(import_line, &["sqlx", "axum", "maud", "silcrow"]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Domain cannot use framework crates (sqlx, axum, maud, silcrow)".into(),
                });
            }
        }
        "application" => {
            // Application cannot import from infrastructure or presentation
            if contains_crate_import(import_line, &[
                "crate::infrastructure",
                "crate::presentation",
            ]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Application cannot import from infrastructure or presentation".into(),
                });
            }
            // Application cannot use framework crates
            if contains_external_import(import_line, &["axum", "maud", "silcrow"]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Application cannot use framework crates (axum, maud, silcrow)".into(),
                });
            }
        }
        "infrastructure" => {
            // Infrastructure cannot import from presentation or application
            if contains_crate_import(import_line, &[
                "crate::presentation",
                "crate::application",
            ]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Infrastructure cannot import from presentation or application".into(),
                });
            }
        }
        "presentation" => {
            // Presentation cannot import from infrastructure
            if contains_crate_import(import_line, &["crate::infrastructure"]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Presentation cannot import from infrastructure".into(),
                });
            }
            // Presentation cannot use sqlx directly
            if contains_external_import(import_line, &["sqlx"]) {
                violations.push(Violation {
                    file: file.into(),
                    line,
                    layer: layer.into(),
                    import: import_line.into(),
                    reason: "Presentation cannot use sqlx directly".into(),
                });
            }
        }
        _ => {}
    }
}

/// Check if a `use` statement imports from one of the given crate:: paths.
fn contains_crate_import(line: &str, paths: &[&str]) -> bool {
    for path in paths {
        // Match: use crate::infrastructure, use crate::infrastructure::*
        // Also match inside braces: use crate::{infrastructure::*, ...}
        if line.contains(path) {
            return true;
        }
    }
    false
}

/// Check if a `use` statement imports from an external crate.
fn contains_external_import(line: &str, crates: &[&str]) -> bool {
    for krate in crates {
        // Match: `use sqlx::`, `use sqlx;`, `use sqlx::{`
        let pattern_colon = format!("use {krate}::");
        let pattern_semi = format!("use {krate};");
        if line.contains(&pattern_colon) || line.contains(&pattern_semi) {
            return true;
        }
    }
    false
}
