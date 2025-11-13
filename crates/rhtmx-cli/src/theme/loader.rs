use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::theme::manifest::{ProjectConfig, ThemeSource};

/// Theme manager handles downloading, caching, and merging themes
pub struct ThemeManager {
    project_root: PathBuf,
    themes_cache_dir: PathBuf,
    merged_dir: PathBuf,
}

impl ThemeManager {
    /// Create a new theme manager for a project
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            themes_cache_dir: project_root.join(".themes"),
            merged_dir: project_root.join(".rhtmx/merged"),
        }
    }

    /// Main entry point: Load theme and merge with user files
    pub fn load_and_merge(&self, force_reload: bool) -> Result<()> {
        // Read project config
        let config_path = self.project_root.join("rhtmx.toml");
        let config = ProjectConfig::from_file(&config_path)
            .context("Failed to read rhtmx.toml")?;

        if !config.has_theme() {
            // No theme, just copy user files to merged directory
            println!("  {} No theme configured", "ℹ".cyan());
            return self.merge_user_files_only();
        }

        let theme_config = config.theme.as_ref().unwrap();
        let theme_name = config.theme_name()
            .ok_or_else(|| anyhow::anyhow!("Could not determine theme name"))?;

        println!("  {} Loading theme: {}", "→".cyan(), theme_name.bold());

        // Get or download theme to cache
        let theme_cache_path = self.themes_cache_dir.join(&theme_name);

        if force_reload || !theme_cache_path.exists() {
            println!("  {} Downloading theme...", "↓".cyan());
            self.download_theme(&theme_config.source, &theme_cache_path)?;
            println!("  {} Theme downloaded", "✓".green());
        } else {
            println!("  {} Using cached theme", "✓".green());
        }

        // Merge theme + user files
        println!("  {} Merging theme and project files...", "⚙".cyan());
        self.merge_theme_and_user(&theme_cache_path)?;
        println!("  {} Merge complete", "✓".green());

        Ok(())
    }

    /// Download theme from source to cache directory
    fn download_theme(&self, source: &ThemeSource, cache_path: &Path) -> Result<()> {
        // Remove existing cache if present
        if cache_path.exists() {
            std::fs::remove_dir_all(cache_path)
                .context("Failed to remove existing theme cache")?;
        }

        match source {
            ThemeSource::Git { url, tag, branch } => {
                self.download_git_theme(url, tag.as_deref(), branch.as_deref(), cache_path)?;
            }
            ThemeSource::Local { path } => {
                self.copy_local_theme(path, cache_path)?;
            }
            ThemeSource::Registry { name, version } => {
                anyhow::bail!("Registry themes not yet supported: {}@{}", name, version);
            }
        }

        // Verify theme structure
        self.verify_theme(cache_path)?;

        Ok(())
    }

    /// Download theme from git repository
    fn download_git_theme(
        &self,
        url: &str,
        tag: Option<&str>,
        branch: Option<&str>,
        dest: &Path,
    ) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("clone")
            .arg("--depth").arg("1");

        // Use tag or branch if specified
        if let Some(tag_name) = tag {
            cmd.arg("--branch").arg(tag_name);
        } else if let Some(branch_name) = branch {
            cmd.arg("--branch").arg(branch_name);
        }

        cmd.arg(url).arg(dest);

        let output = cmd.output()
            .context("Failed to execute git clone. Is git installed?")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Remove .git directory to save space
        let git_dir = dest.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(git_dir).ok();
        }

        Ok(())
    }

    /// Copy theme from local path
    fn copy_local_theme(&self, src: &Path, dest: &Path) -> Result<()> {
        if !src.exists() {
            anyhow::bail!("Local theme path does not exist: {}", src.display());
        }

        std::fs::create_dir_all(dest)?;
        copy_directory(src, dest)
            .with_context(|| format!("Failed to copy theme from {}", src.display()))?;

        Ok(())
    }

    /// Verify theme has required structure
    fn verify_theme(&self, theme_path: &Path) -> Result<()> {
        let theme_toml = theme_path.join("theme.toml");

        if !theme_toml.exists() {
            anyhow::bail!(
                "Invalid theme: missing theme.toml in {}",
                theme_path.display()
            );
        }

        // Optional: could verify pages/ directory exists, etc.

        Ok(())
    }

    /// Merge theme files with user files
    /// Priority: User files > Theme files
    fn merge_theme_and_user(&self, theme_path: &Path) -> Result<()> {
        // Clean merged directory
        if self.merged_dir.exists() {
            std::fs::remove_dir_all(&self.merged_dir)?;
        }
        std::fs::create_dir_all(&self.merged_dir)?;

        // Step 1: Copy ALL theme files to merged (base layer)
        for dir in &["pages", "components", "static"] {
            let theme_dir = theme_path.join(dir);
            if theme_dir.exists() {
                let merged_subdir = self.merged_dir.join(dir);
                std::fs::create_dir_all(&merged_subdir)?;
                copy_directory(&theme_dir, &merged_subdir)?;
            }
        }

        // Step 2: Copy user files to merged (override layer)
        for dir in &["pages", "components", "static"] {
            let user_dir = self.project_root.join(dir);
            if user_dir.exists() {
                let merged_subdir = self.merged_dir.join(dir);
                std::fs::create_dir_all(&merged_subdir)?;
                copy_directory(&user_dir, &merged_subdir)?;
            }
        }

        // Step 3: Copy user's src/ if exists
        let user_src = self.project_root.join("src");
        if user_src.exists() {
            let merged_src = self.merged_dir.join("src");
            std::fs::create_dir_all(&merged_src)?;
            copy_directory(&user_src, &merged_src)?;
        }

        Ok(())
    }

    /// Just copy user files (no theme)
    fn merge_user_files_only(&self) -> Result<()> {
        // Clean merged directory
        if self.merged_dir.exists() {
            std::fs::remove_dir_all(&self.merged_dir)?;
        }
        std::fs::create_dir_all(&self.merged_dir)?;

        // Copy user directories
        for dir in &["pages", "components", "static", "src"] {
            let user_dir = self.project_root.join(dir);
            if user_dir.exists() {
                let merged_subdir = self.merged_dir.join(dir);
                std::fs::create_dir_all(&merged_subdir)?;
                copy_directory(&user_dir, &merged_subdir)?;
            }
        }

        Ok(())
    }

    /// Clear theme cache
    pub fn clear_cache(&self) -> Result<()> {
        if self.themes_cache_dir.exists() {
            std::fs::remove_dir_all(&self.themes_cache_dir)
                .context("Failed to clear theme cache")?;
            println!("  {} Theme cache cleared", "✓".green());
        } else {
            println!("  {} No theme cache to clear", "ℹ".cyan());
        }
        Ok(())
    }

    /// Get merged directory path
    pub fn merged_path(&self) -> &Path {
        &self.merged_dir
    }
}

/// Recursively copy directory contents
fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Skip .git directories
            !e.path().components().any(|c| c.as_os_str() == ".git")
        })
    {
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
