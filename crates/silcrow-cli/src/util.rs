use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Convert PascalCase to snake_case.
/// "OrderItem" → "order_item", "CreateOrder" → "create_order"
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Validate that a name is PascalCase (starts with uppercase, alphanumeric only).
pub fn validate_pascal_case(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if !name.chars().next().unwrap().is_uppercase() {
        return Err(format!("Name '{}' must be PascalCase (start with uppercase)", name));
    }
    if !name.chars().all(|c| c.is_alphanumeric()) {
        return Err(format!("Name '{}' must contain only alphanumeric characters", name));
    }
    Ok(())
}

/// Find the project root by walking up from cwd looking for Cargo.toml with silcrow dep.
pub fn find_project_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("Cannot get cwd: {e}"))?;

    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let contents = fs::read_to_string(&cargo_toml)
                .map_err(|e| format!("Cannot read {}: {e}", cargo_toml.display()))?;
            // Check if this Cargo.toml has silcrow as a dependency
            if contents.contains("silcrow") && contents.contains("[dependencies]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            return Err("Not inside a Silcrow project (no Cargo.toml with silcrow dependency found)".into());
        }
    }
}

/// Append a module declaration to a mod.rs file if it doesn't already exist.
pub fn append_to_mod(mod_file: &Path, line: &str) -> io::Result<()> {
    let contents = if mod_file.exists() {
        fs::read_to_string(mod_file)?
    } else {
        String::new()
    };

    // Don't add if already present
    if contents.lines().any(|l| l.trim() == line.trim()) {
        return Ok(());
    }

    let mut new_contents = contents;
    if !new_contents.is_empty() && !new_contents.ends_with('\n') {
        new_contents.push('\n');
    }
    new_contents.push_str(line);
    new_contents.push('\n');

    fs::write(mod_file, new_contents)
}

/// Insert content above a marker comment line in a file.
/// Marker format: `// SC:<MARKER_NAME>`
pub fn insert_at_marker(file: &Path, marker: &str, content: &str) -> Result<(), String> {
    let contents = fs::read_to_string(file)
        .map_err(|e| format!("Cannot read {}: {e}", file.display()))?;

    let marker_pattern = format!("// SC:{marker}");
    let mut found = false;
    let mut result = String::new();

    for line in contents.lines() {
        if line.trim() == marker_pattern {
            found = true;
            result.push_str(content);
            if !content.ends_with('\n') {
                result.push('\n');
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    if !found {
        return Err(format!("Marker '{}' not found in {}", marker_pattern, file.display()));
    }

    fs::write(file, result)
        .map_err(|e| format!("Cannot write {}: {e}", file.display()))
}

/// Write a file only if it doesn't already exist. Returns Ok(true) if created.
pub fn write_if_new(path: &Path, content: &str) -> Result<bool, String> {
    if path.exists() {
        println!("  skip: {} (already exists)", path.display());
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create dir {}: {e}", parent.display()))?;
    }
    fs::write(path, content)
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    println!("  create: {}", path.display());
    Ok(true)
}
