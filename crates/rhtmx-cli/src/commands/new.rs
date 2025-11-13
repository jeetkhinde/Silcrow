use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::project::scaffold;

pub fn execute(name: &str, theme: Option<&str>, init_git: bool) -> Result<()> {
    println!("{}", "Creating new RHTMX project...".green().bold());
    println!();

    // Validate project name
    if !is_valid_project_name(name) {
        anyhow::bail!("Invalid project name. Use alphanumeric characters, hyphens, and underscores only.");
    }

    let project_path = Path::new(name);

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create project structure
    scaffold::create_project(project_path, theme)
        .context("Failed to create project structure")?;

    println!("  {} Project structure", "✓".green());

    // Initialize git repository
    if init_git {
        if let Err(e) = init_git_repo(project_path) {
            println!("  {} Git initialization ({})", "⚠".yellow(), e);
        } else {
            println!("  {} Git repository", "✓".green());
        }
    }

    println!();
    println!("{}", "Project created successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  cargo build");
    println!("  rhtmx dev");
    println!();

    if let Some(theme_name) = theme {
        println!("Using theme: {}", theme_name.cyan());
        println!();
    }

    Ok(())
}

fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

fn init_git_repo(path: &Path) -> Result<()> {
    use std::process::Command;

    // Initialize git repo
    Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .context("Failed to initialize git repository")?;

    // Initial commit
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .context("Failed to add files to git")?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .context("Failed to create initial commit")?;

    Ok(())
}
