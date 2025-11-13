use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::process::Command;
use crate::BuildMode;
use crate::theme::ThemeManager;

pub fn execute(mode: BuildMode, release: bool) -> Result<()> {
    let mode_str = match mode {
        BuildMode::Ssr => "SSR (Server-Side Rendering)",
        BuildMode::Ssg => "SSG (Static Site Generation)",
        BuildMode::Isr => "ISR (Incremental Static Regeneration)",
    };

    println!("{}", "Building project...".green().bold());
    println!();
    println!("Mode: {}", mode_str.cyan());
    println!("Release: {}", if release { "Yes" } else { "No" });
    println!();

    match mode {
        BuildMode::Ssr => build_ssr(release),
        BuildMode::Ssg => {
            println!("{}", "⚠ SSG mode not yet implemented".yellow());
            println!("Coming soon: Will compile + run in generator mode + write HTML files");
            Ok(())
        }
        BuildMode::Isr => {
            println!("{}", "⚠ ISR mode not yet implemented".yellow());
            println!("Coming soon: Will compile with caching layer");
            Ok(())
        }
    }
}

fn build_ssr(release: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    // Step 1: Ensure theme is merged
    println!("{}", "  ⚙  Preparing build environment...".cyan());
    let manager = ThemeManager::new(&current_dir);
    manager.load_and_merge(false)?;

    let merged_path = manager.merged_path();

    // Step 2: Copy Cargo.toml to merged directory
    println!("{}", "  ⚙  Setting up build configuration...".cyan());
    let cargo_toml_src = current_dir.join("Cargo.toml");
    let cargo_toml_dst = merged_path.join("Cargo.toml");

    if !cargo_toml_src.exists() {
        anyhow::bail!("Cargo.toml not found in project root. Is this a valid RHTMX project?");
    }

    fs::copy(&cargo_toml_src, &cargo_toml_dst)
        .context("Failed to copy Cargo.toml to merged directory")?;

    // Step 3: Get project name from Cargo.toml
    let cargo_toml_content = fs::read_to_string(&cargo_toml_dst)
        .context("Failed to read Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)
        .context("Failed to parse Cargo.toml")?;
    let project_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Failed to get package name from Cargo.toml"))?
        .to_string();

    // Step 4: Run cargo build in merged directory
    println!("{}", "  🔨 Compiling project...".cyan());
    println!();

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .current_dir(merged_path);

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()
        .context("Failed to execute cargo build. Is cargo installed?")?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    println!();

    // Step 5: Copy binary to project target directory
    let build_mode = if release { "release" } else { "debug" };
    let binary_src = merged_path.join(format!("target/{}/{}", build_mode, project_name));
    let binary_dst = current_dir.join(format!("target/{}/{}", build_mode, project_name));

    if !binary_src.exists() {
        anyhow::bail!(
            "Binary not found at expected location: {}\nDid the build succeed?",
            binary_src.display()
        );
    }

    // Create target directory if it doesn't exist
    if let Some(parent) = binary_dst.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create target directory")?;
    }

    println!("{}", "  📦 Copying binary...".cyan());
    fs::copy(&binary_src, &binary_dst)
        .context("Failed to copy binary to target directory")?;

    // Make binary executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_dst)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_dst, perms)?;
    }

    println!();
    println!("{}", "✓ Build complete!".green().bold());
    println!();
    println!("Binary location: {}", binary_dst.display().to_string().cyan());
    println!();
    println!("Run your application:");
    println!("  {}", format!("./{}", binary_dst.display()).yellow());

    Ok(())
}
