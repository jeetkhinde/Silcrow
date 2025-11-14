// File: src/template_loader.rs
// Purpose: Loads rhtmx templates from the pages/ directory

use anyhow::{Context, Result};
// use rhtmx_parser::{CssParser, ScopedCss};
use rhtmx_router::{Route, Router};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a loaded rhtmx template
#[derive(Debug, Clone)]
pub struct Template {
    pub path: PathBuf,
    pub content: String,
    pub scoped_css: Option<()>, // TODO: Implement CSS parsing
    pub partials: Vec<String>, // Names of partials defined in this template
}

/// Template loader that reads and caches rhtmx files
#[derive(Clone)]
pub struct TemplateLoader {
    pages_dir: PathBuf,
    components_dir: PathBuf,
    templates: HashMap<String, Template>,
    components: HashMap<String, Template>,
    router: Router,
}

impl TemplateLoader {
    /// Create a new template loader with default directories and case-sensitive routing
    pub fn new(pages_dir: impl Into<PathBuf>) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            components_dir: "components".into(),
            templates: HashMap::new(),
            components: HashMap::new(),
            router: Router::new(),
        }
    }

    /// Create a new template loader with custom directories and routing options
    pub fn with_config(
        pages_dir: impl Into<PathBuf>,
        components_dir: impl Into<PathBuf>,
        case_insensitive: bool,
    ) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            components_dir: components_dir.into(),
            templates: HashMap::new(),
            components: HashMap::new(),
            router: Router::with_case_insensitive(case_insensitive),
        }
    }

    /// Create a new template loader with case-insensitive routing (legacy)
    pub fn with_case_insensitive(pages_dir: impl Into<PathBuf>, case_insensitive: bool) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            components_dir: "components".into(),
            templates: HashMap::new(),
            components: HashMap::new(),
            router: Router::with_case_insensitive(case_insensitive),
        }
    }

    /// Set case-insensitive mode
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.router.set_case_insensitive(case_insensitive);
    }

    /// Load all templates from the pages directory
    pub fn load_all(&mut self) -> Result<()> {
        self.load_directory(&self.pages_dir.clone())?;
        self.load_components()?;

        // Sort routes by priority after loading all templates
        self.router.sort_routes();

        Ok(())
    }

    /// Load all components from the components directory
    fn load_components(&mut self) -> Result<()> {
        let components_dir = self.components_dir.clone();
        if !components_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&components_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rhtmx") {
                // Load component file
                self.load_component(&path)?;
            }
        }

        Ok(())
    }

    /// Load a single component file
    fn load_component(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read component: {:?}", path))?;

        // Component name is the file name without extension
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Process CSS and extract partials info
        // TODO: Implement CSS parsing with CssParser
        let partials = vec![]; // Placeholder for extracting partials
        let content_without_css = content.clone();
        let scoped_css = None;

        let template = Template {
            path: path.to_path_buf(),
            content: content_without_css.clone(),
            scoped_css,
            partials: partials.clone(),
        };

        self.components.insert(name.clone(), template.clone());

        // If any components in this file are marked as @partial, also register them as routes
        for partial_name in &partials {
            let partial_route = format!("/partials/{}", partial_name.to_lowercase());

            // Create a partial template that can be accessed as a route
            let partial_template = Template {
                path: path.to_path_buf(),
                content: content_without_css.clone(),
                scoped_css,
                partials: vec![partial_name.clone()],
            };

            self.templates
                .insert(partial_route.clone(), partial_template);

            println!(
                "📄 Registered partial route: {} -> {} (from component file)",
                partial_route, partial_name
            );
        }

        println!(
            "🧩 Loaded component: {} -> {:?}{}",
            name,
            path.file_name().unwrap(),
            if !partials.is_empty() {
                format!(" (with partials: {})", partials.join(", "))
            } else {
                String::new()
            }
        );

        Ok(())
    }

    /// Recursively load templates from a directory
    fn load_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively load subdirectories
                self.load_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rhtmx") {
                // Load .rhtmx files
                self.load_template(&path)?;
            }
        }

        Ok(())
    }

    /// Load a single template file
    fn load_template(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read template: {:?}", path))?;

        // Create a Route for the router
        let route_obj = Route::from_path(
            path.to_str().unwrap_or(""),
            self.pages_dir.to_str().unwrap_or("pages"),
        );

        // Process CSS and extract partials info
        // TODO: Implement CSS parsing with CssParser
        let partials = vec![]; // Placeholder for extracting partials
        let content_without_css = content.clone();
        let scoped_css = None;

        let template = Template {
            path: path.to_path_buf(),
            content: content_without_css,
            scoped_css,
            partials,
        };

        // For layouts, only store with the old-style key (e.g., "/_layout", "/users/_layout")
        // For error pages, only store with the old-style key (e.g., "/_error", "/users/_error")
        // For pages, store with both pattern key and old-style key
        if route_obj.is_layout {
            // Layouts: only use old-style key to avoid collision with pages
            let old_route = self.path_to_route(path);
            self.templates.insert(old_route, template);
        } else if route_obj.is_error_page {
            // Error pages: only use old-style key to avoid collision with pages
            let old_route = self.path_to_route(path);
            self.templates.insert(old_route, template);
        } else {
            // Pages: store with pattern key
            self.templates
                .insert(route_obj.pattern.clone(), template.clone());

            // Also store using old key format for backward compatibility
            let old_route = self.path_to_route(path);
            if old_route != route_obj.pattern {
                self.templates.insert(old_route, template);
            }
        }

        // Determine what key this will be stored with for clarity
        let storage_key = if route_obj.is_layout || route_obj.is_error_page {
            self.path_to_route(path)
        } else {
            route_obj.pattern.clone()
        };

        println!(
            "📄 Loaded template: {} (stored as: {}) -> {:?} (priority: {})",
            route_obj.pattern,
            storage_key,
            path.file_name().unwrap(),
            route_obj.priority
        );

        // Add to router
        self.router.add_route(route_obj);

        Ok(())
    }

    /// Convert file path to route (e.g., pages/index.rhtmx -> "/")
    fn path_to_route(&self, path: &Path) -> String {
        let relative = path.strip_prefix(&self.pages_dir).unwrap_or(path);

        let route = relative
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");

        // Handle "_error" files specially - keep the _error suffix
        if route == "_error" {
            "/_error".to_string()
        } else if route.ends_with("/_error") {
            // Ensure leading slash
            if route.starts_with('/') {
                route
            } else {
                format!("/{}", route)
            }
        }
        // Handle "_layout" files specially - keep the _layout suffix
        else if route == "_layout" {
            "/_layout".to_string()
        } else if route.ends_with("/_layout") {
            // Ensure leading slash
            if route.starts_with('/') {
                route
            } else {
                format!("/{}", route)
            }
        }
        // Convert "index" to "/" and "users/index" to "/users"
        else if route == "index" || route.is_empty() {
            "/".to_string()
        } else if route.ends_with("/index") {
            let without_index = route[..route.len() - 6].to_string(); // Remove "/index"
            if without_index.is_empty() {
                "/".to_string()
            } else if without_index.starts_with('/') {
                without_index
            } else {
                format!("/{}", without_index)
            }
        } else if route.starts_with('/') {
            route
        } else {
            format!("/{}", route)
        }
    }

    /// Get a template by route
    pub fn get(&self, route: &str) -> Option<&Template> {
        self.templates.get(route)
    }

    /// Get the layout template
    pub fn get_layout(&self) -> Option<&Template> {
        self.templates.get("/_layout")
    }

    /// Get the layout for a specific route pattern
    pub fn get_layout_for_route(&self, pattern: &str) -> Option<&Template> {
        if let Some(layout_route) = self.router.get_layout(pattern) {
            // Convert pattern back to template key
            let layout_key = if layout_route.pattern == "/" {
                "/_layout".to_string()
            } else {
                format!("{}/_layout", layout_route.pattern)
            };
            self.templates.get(&layout_key)
        } else {
            // Fall back to root layout
            self.get_layout()
        }
    }

    /// Get the error page for a specific route pattern
    /// Looks for section-specific error page first, then root error page
    pub fn get_error_page_for_route(&self, pattern: &str) -> Option<&Template> {
        if let Some(error_route) = self.router.get_error_page(pattern) {
            // Convert pattern back to template key
            let error_key = if error_route.pattern == "/" {
                "/_error".to_string()
            } else {
                format!("{}/_error", error_route.pattern)
            };
            self.templates.get(&error_key)
        } else {
            None
        }
    }

    /// Get the root error page
    pub fn get_error_page(&self) -> Option<&Template> {
        self.templates.get("/_error")
    }

    /// Get the router
    pub fn router(&self) -> &Router {
        &self.router
    }

    /// Get a component by name
    pub fn get_component(&self, name: &str) -> Option<&Template> {
        self.components.get(name)
    }

    /// List all loaded templates
    pub fn list_routes(&self) -> Vec<String> {
        let mut routes: Vec<_> = self.templates.keys().cloned().collect();
        routes.sort();
        routes
    }

    /// Get total number of loaded templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }

    /// Reload a specific template file
    pub fn reload_template(&mut self, path: &Path) -> Result<()> {
        if path.to_str().unwrap_or("").contains("/components/")
            || path.to_str().unwrap_or("").contains("\\components\\")
        {
            self.reload_component(path)?;
        } else {
            // Convert absolute path to relative if needed
            let relative_path = if path.is_absolute() {
                // Try to strip current directory
                let current_dir = std::env::current_dir().unwrap_or_default();
                path.strip_prefix(&current_dir).unwrap_or(path)
            } else {
                path
            };

            // Remove old template
            let route_obj = Route::from_path(
                relative_path.to_str().unwrap_or(""),
                self.pages_dir.to_str().unwrap_or("pages"),
            );
            self.templates.remove(&route_obj.pattern);

            // Remove from router
            self.router.remove_route(&route_obj.pattern);

            // Reload template using relative path
            self.load_template(relative_path)?;

            // Re-sort routes
            self.router.sort_routes();
        }
        Ok(())
    }

    /// Reload a specific component file
    pub fn reload_component(&mut self, path: &Path) -> Result<()> {
        // Convert absolute path to relative if needed
        let relative_path = if path.is_absolute() {
            let current_dir = std::env::current_dir().unwrap_or_default();
            path.strip_prefix(&current_dir).unwrap_or(path)
        } else {
            path
        };

        let name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Remove old component
        self.components.remove(&name);

        // Reload component using relative path
        self.load_component(relative_path)?;

        Ok(())
    }

    /// Reload all templates and components
    pub fn reload_all(&mut self) -> Result<()> {
        // Clear all templates and components
        self.templates.clear();
        self.components.clear();
        self.router = Router::new();

        // Reload everything
        self.load_all()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_route() {
        let loader = TemplateLoader::new("pages");

        // Test cases
        assert_eq!(loader.path_to_route(Path::new("pages/index.rhtmx")), "/");
        assert_eq!(
            loader.path_to_route(Path::new("pages/about.rhtmx")),
            "/about"
        );
        assert_eq!(
            loader.path_to_route(Path::new("pages/users/profile.rhtmx")),
            "/users/profile"
        );
    }
}
