use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

/// Load theme files from a directory
pub fn load_theme(theme_path: &Path) -> Result<()> {
    if !theme_path.exists() {
        anyhow::bail!("Theme directory does not exist: {}", theme_path.display());
    }

    // TODO: Implement theme loading
    // 1. Verify theme.toml exists
    // 2. Parse theme manifest
    // 3. Copy theme files to .rhtmx/merged/
    // 4. Use rhtmx-router to discover routes

    Ok(())
}

/// Merge theme files with user project files
pub fn merge_theme(
    theme_path: &Path,
    project_path: &Path,
    merged_path: &Path,
) -> Result<()> {
    // Create merged directory
    std::fs::create_dir_all(merged_path)
        .context("Failed to create merged directory")?;

    // Copy theme files first
    copy_directory(theme_path, merged_path)
        .context("Failed to copy theme files")?;

    // Copy user files (overwrites theme files if same path)
    copy_directory(project_path, merged_path)
        .context("Failed to copy user files")?;

    Ok(())
}

/// Recursively copy directory contents
fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &target)?;
        }
    }

    Ok(())
}
