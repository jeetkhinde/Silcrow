// File: rhtml-macro/src/layout_resolver.rs
// Purpose: Find _layout.rhtml files for pages

use std::path::{Path, PathBuf};

/// Find the layout file for a given page file
///
/// Algorithm:
/// 1. Start at page's directory
/// 2. Look for _layout.rhtml in that directory
/// 3. If not found, walk up to parent directory
/// 4. Repeat until found or reach Pages/ directory
/// 5. Use Pages/_layout.rhtml as fallback
/// 6. Error if fallback doesn't exist
///
/// # Examples
///
/// ```ignore
/// // For pages/users/index.rhtml -> looks in:
/// // 1. pages/users/_layout.rhtml
/// // 2. pages/_layout.rhtml (fallback)
/// ```
///
/// Note: Part of future layout system infrastructure
#[allow(dead_code)]
pub fn find_layout_for_page(page_path: &Path) -> Result<PathBuf, String> {
    let mut current = page_path
        .parent()
        .ok_or_else(|| "Page has no parent directory".to_string())?;

    // Walk up the directory tree
    loop {
        let layout_path = current.join("_layout.rhtml");

        if layout_path.exists() {
            return Ok(layout_path);
        }

        // Check if we've reached the Pages directory
        if current.ends_with("pages") || current.ends_with("Pages") {
            break;
        }

        // Move to parent directory
        current = match current.parent() {
            Some(parent) => parent,
            None => break, // Reached filesystem root
        };
    }

    // Fallback to root layout
    let root_layout = current.join("_layout.rhtml");
    if root_layout.exists() {
        Ok(root_layout)
    } else {
        Err(format!(
            "No root layout found at {}. Root layout is required.",
            root_layout.display()
        ))
    }
}

/// Get the directory path (as string) for registry lookup
///
/// Converts an absolute file path to a directory string for the registry
///
/// Note: Part of future layout system infrastructure
#[allow(dead_code)]
pub fn get_directory_key(layout_path: &Path) -> String {
    layout_path
        .parent()
        .unwrap_or(layout_path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_directory_key() {
        let path = Path::new("/home/user/project/pages/_layout.rhtml");
        let key = get_directory_key(path);
        assert!(key.contains("pages"));
    }
}
