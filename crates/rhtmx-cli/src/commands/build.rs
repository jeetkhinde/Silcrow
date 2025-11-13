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
        BuildMode::Ssg => build_ssg(release),
        BuildMode::Isr => build_isr(release),
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

fn build_ssg(release: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    // Step 1: Read configuration
    let config_path = current_dir.join("rhtmx.toml");
    let config = crate::theme::manifest::ProjectConfig::from_file(&config_path)
        .context("Failed to read rhtmx.toml")?;

    let ssg_config = config.ssg.as_ref();
    let output_dir = ssg_config
        .map(|s| s.output_dir.clone())
        .unwrap_or_else(|| "dist".to_string());

    // Step 2: Ensure theme is merged
    println!("{}", "  ⚙  Preparing build environment...".cyan());
    let manager = ThemeManager::new(&current_dir);
    manager.load_and_merge(false)?;
    let merged_path = manager.merged_path();

    // Step 3: Discover routes from pages directory
    println!("{}", "  🔍 Discovering routes...".cyan());
    let pages_dir = merged_path.join("pages");
    let routes = discover_routes(&pages_dir)?;
    println!("     Found {} static routes", routes.len());

    // Step 4: Expand dynamic routes from configuration
    let mut all_routes = routes.clone();
    if let Some(ssg_cfg) = ssg_config {
        if !ssg_cfg.dynamic_routes.is_empty() {
            println!("{}", "  📋 Expanding dynamic routes...".cyan());
            for dynamic_source in &ssg_cfg.dynamic_routes {
                let expanded = expand_dynamic_route(&current_dir, dynamic_source)?;
                println!("     {} -> {} routes", dynamic_source.pattern, expanded.len());
                all_routes.extend(expanded);
            }
        }
    }

    println!();
    println!("  Total routes to render: {}", all_routes.len());
    println!();

    // Step 5: Create output directory
    let dist_path = current_dir.join(&output_dir);
    if dist_path.exists() {
        fs::remove_dir_all(&dist_path)
            .context("Failed to remove existing dist directory")?;
    }
    fs::create_dir_all(&dist_path)
        .context("Failed to create dist directory")?;

    // Step 6: Generate HTML for each route
    println!("{}", "  📝 Generating static HTML...".cyan());
    generate_html_files(merged_path, &all_routes, &dist_path)?;

    // Step 7: Copy static assets
    println!("{}", "  📦 Copying static assets...".cyan());
    copy_static_assets(merged_path, &dist_path)?;

    println!();
    println!("{}", "✓ SSG build complete!".green().bold());
    println!();
    println!("Output directory: {}", dist_path.display().to_string().cyan());
    println!("Total files: {}", all_routes.len() + 1); // +1 for static dir
    println!();
    println!("Deploy your site:");
    println!("  Serve the {} directory with any static file server", output_dir);

    Ok(())
}

/// Discover all routes by scanning the pages directory
fn discover_routes(pages_dir: &std::path::Path) -> Result<Vec<RouteInfo>> {
    let mut routes = Vec::new();
    discover_routes_recursive(pages_dir, pages_dir, &mut routes)?;
    Ok(routes)
}

fn discover_routes_recursive(
    base_dir: &std::path::Path,
    current_dir: &std::path::Path,
    routes: &mut Vec<RouteInfo>,
) -> Result<()> {
    use std::fs;

    if !current_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            discover_routes_recursive(base_dir, &path, routes)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rhtmx") {
            // Skip layouts and error pages
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                if filename.starts_with('_') {
                    continue;
                }
            }

            // Convert file path to route pattern
            let relative = path.strip_prefix(base_dir)?;
            let route_path = file_path_to_route(relative);

            // Skip dynamic routes (they'll be handled by config)
            if route_path.contains('[') {
                continue;
            }

            routes.push(RouteInfo {
                pattern: route_path.clone(),
                file_path: path.clone(),
                params: std::collections::HashMap::new(),
            });
        }
    }

    Ok(())
}

/// Convert file path to route pattern
fn file_path_to_route(path: &std::path::Path) -> String {
    let mut route = String::from("/");

    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(segment) = os_str.to_str() {
                // Remove .rhtmx extension
                let segment = segment.strip_suffix(".rhtmx").unwrap_or(segment);

                // Skip index files
                if segment == "index" {
                    continue;
                }

                if !route.ends_with('/') {
                    route.push('/');
                }
                route.push_str(segment);
            }
        }
    }

    // Ensure we have at least "/"
    if route.is_empty() {
        route = "/".to_string();
    }

    route
}

/// Expand dynamic routes based on configuration
fn expand_dynamic_route(
    project_root: &std::path::Path,
    source: &crate::theme::manifest::DynamicRouteSource,
) -> Result<Vec<RouteInfo>> {
    use glob::glob;
    use std::collections::HashMap;

    let mut routes = Vec::new();

    // Resolve glob pattern relative to project root
    let glob_pattern = project_root.join(&source.source);
    let glob_pattern_str = glob_pattern.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid glob pattern"))?;

    for entry in glob(glob_pattern_str)
        .context("Failed to read glob pattern")?
    {
        let path = entry.context("Failed to read glob entry")?;

        // Extract slug from filename
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Replace [slug] or [id] in pattern with actual value
        let route_pattern = source.pattern
            .replace("[slug]", &slug)
            .replace("[id]", &slug);

        let mut params = HashMap::new();
        if source.pattern.contains("[slug]") {
            params.insert("slug".to_string(), slug.clone());
        } else if source.pattern.contains("[id]") {
            params.insert("id".to_string(), slug.clone());
        }

        routes.push(RouteInfo {
            pattern: route_pattern,
            file_path: path,
            params,
        });
    }

    Ok(routes)
}

/// Generate HTML files for all routes
fn generate_html_files(
    merged_path: &std::path::Path,
    routes: &[RouteInfo],
    output_dir: &std::path::Path,
) -> Result<()> {
    use std::fs;

    for (idx, route) in routes.iter().enumerate() {
        print!("     [{}/{}] {} ", idx + 1, routes.len(), route.pattern);

        // Read template file
        let template_content = fs::read_to_string(&route.file_path)
            .with_context(|| format!("Failed to read template: {:?}", route.file_path))?;

        // Generate HTML (simplified - just wrap in basic HTML for now)
        let html = generate_html_for_route(&route.pattern, &template_content, &route.params)?;

        // Determine output file path
        let output_file = route_to_file_path(output_dir, &route.pattern);

        // Create parent directories if needed
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // Write HTML file
        fs::write(&output_file, html)
            .with_context(|| format!("Failed to write HTML file: {:?}", output_file))?;

        println!("{}", "✓".green());
    }

    Ok(())
}

/// Generate HTML for a route (simplified version)
fn generate_html_for_route(
    route: &str,
    _template_content: &str,
    params: &std::collections::HashMap<String, String>,
) -> Result<String> {
    // For now, generate a simple HTML page
    // In a real implementation, this would:
    // 1. Compile the template with Rust macros
    // 2. Execute the rendering code
    // 3. Return the generated HTML

    let params_display = if params.is_empty() {
        String::new()
    } else {
        format!("<p>Route params: {:?}</p>", params)
    };

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RHTMX - {route}</title>
    <link rel="stylesheet" href="/static/css/styles.css">
</head>
<body>
    <nav>
        <a href="/">Home</a>
    </nav>
    <main>
        <h1>Route: {route}</h1>
        {params_display}
        <p>This is a static generated page.</p>
        <p class="note">Note: Full template rendering will be implemented in the next iteration.</p>
    </main>
    <footer>
        <p>Built with RHTMX SSG</p>
    </footer>
</body>
</html>"#
    ))
}

/// Convert route pattern to file path
fn route_to_file_path(base: &std::path::Path, route: &str) -> std::path::PathBuf {
    if route == "/" {
        base.join("index.html")
    } else {
        let clean_route = route.trim_start_matches('/');
        base.join(format!("{}.html", clean_route))
    }
}

/// Copy static assets to output directory
fn copy_static_assets(merged_path: &std::path::Path, output_dir: &std::path::Path) -> Result<()> {
    let static_src = merged_path.join("static");
    if !static_src.exists() {
        println!("     No static directory found, skipping");
        return Ok(());
    }

    let static_dst = output_dir.join("static");
    copy_directory_recursive(&static_src, &static_dst)?;

    println!("     ✓ Copied static assets");
    Ok(())
}

fn copy_directory_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    use std::fs;

    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory: {:?}", dst))?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_directory_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)
                .with_context(|| format!("Failed to copy file: {:?}", path))?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct RouteInfo {
    pattern: String,
    file_path: std::path::PathBuf,
    params: std::collections::HashMap<String, String>,
}

fn build_isr(release: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    // Step 1: Ensure theme is merged
    println!("{}", "  ⚙  Preparing build environment...".cyan());
    let manager = ThemeManager::new(&current_dir);
    manager.load_and_merge(false)?;

    let merged_path = manager.merged_path();

    // Step 2: Copy and modify Cargo.toml to add ISR dependencies
    println!("{}", "  ⚙  Setting up ISR configuration...".cyan());
    let cargo_toml_src = current_dir.join("Cargo.toml");
    let cargo_toml_dst = merged_path.join("Cargo.toml");

    if !cargo_toml_src.exists() {
        anyhow::bail!("Cargo.toml not found in project root. Is this a valid RHTMX project?");
    }

    // Read and modify Cargo.toml to add ISR dependency
    let mut cargo_toml_content = fs::read_to_string(&cargo_toml_src)
        .context("Failed to read Cargo.toml")?;

    // Parse TOML
    let mut cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)
        .context("Failed to parse Cargo.toml")?;

    // Add rhtmx-isr dependency if not already present
    if let Some(deps) = cargo_toml.get_mut("dependencies").and_then(|v| v.as_table_mut()) {
        if !deps.contains_key("rhtmx-isr") {
            // Determine the path to rhtmx-isr crate
            // Assume it's in the workspace or use a version
            let isr_dep = toml::Value::Table({
                let mut table = toml::map::Map::new();
                table.insert("path".to_string(), toml::Value::String("../rhtmx-isr".to_string()));
                table.insert("features".to_string(), toml::Value::Array(vec![
                    toml::Value::String("all".to_string())
                ]));
                table
            });
            deps.insert("rhtmx-isr".to_string(), isr_dep);
        }
    }

    // Write modified Cargo.toml
    let modified_toml = toml::to_string_pretty(&cargo_toml)
        .context("Failed to serialize Cargo.toml")?;
    fs::write(&cargo_toml_dst, modified_toml)
        .context("Failed to write modified Cargo.toml")?;

    // Step 3: Read ISR configuration from rhtmx.toml
    println!("{}", "  ⚙  Reading ISR configuration...".cyan());
    let config_path = current_dir.join("rhtmx.toml");
    let config = if config_path.exists() {
        crate::theme::manifest::ProjectConfig::from_file(&config_path)
            .context("Failed to read rhtmx.toml")?
    } else {
        crate::theme::manifest::ProjectConfig::default()
    };

    // Display ISR configuration
    if let Some(ref isr_config) = config.isr {
        println!("     Revalidation: {}s", isr_config.default_revalidate);
        println!("     Primary storage: {}", isr_config.storage.primary);
        if let Some(ref fallback) = isr_config.storage.fallback {
            println!("     Fallback storage: {}", fallback);
        }
    } else {
        println!("     Using default ISR configuration");
        println!("     (Add [isr] section to rhtmx.toml to customize)");
    }

    // Step 4: Get project name from Cargo.toml
    let project_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Failed to get package name from Cargo.toml"))?
        .to_string();

    // Step 5: Run cargo build in merged directory with ISR features
    println!("{}", "  🔨 Compiling project with ISR support...".cyan());
    println!();

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--features")
        .arg("isr")
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

    // Step 6: Copy binary to project target directory
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
    println!("{}", "✓ ISR build complete!".green().bold());
    println!();
    println!("Binary location: {}", binary_dst.display().to_string().cyan());
    println!();
    println!("Run your application:");
    println!("  {}", format!("./{}", binary_dst.display()).yellow());
    println!();
    println!("ISR features:");
    println!("  • Cached page serving");
    println!("  • Background revalidation");
    println!("  • Multiple storage backends (memory, filesystem, dragonfly)");
    if config.isr.is_some() {
        println!();
        println!("Configuration from rhtmx.toml will be used at runtime.");
    }

    Ok(())
}
