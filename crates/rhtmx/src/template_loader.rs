// File: src/template_loader.rs
// Purpose: Loads rhtmx templates from the pages/ directory
// Refactored to follow functional programming principles

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
    pub partials: Vec<String>,  // Names of partials defined in this template
}

/// Template loader that reads and caches rhtmx files
#[derive(Clone)]
pub struct TemplateLoader {
    pages_dir: PathBuf,
    components_dir: PathBuf,
    templates: HashMap<String, Template>,
    components: HashMap<String, Template>,
    router: Router,
    case_insensitive: bool,
}

// ============================================================================
// Constructor Methods
// ============================================================================

impl TemplateLoader {
    /// Create a new template loader with default directories and case-sensitive routing
    pub fn new(pages_dir: impl Into<PathBuf>) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            components_dir: "components".into(),
            templates: HashMap::new(),
            components: HashMap::new(),
            router: Router::new(),
            case_insensitive: false,
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
            case_insensitive,
        }
    }

    /// Create a new template loader with case-insensitive routing
    /// Use with_config() for more control, or chain .with_case_insensitive() on new()
    pub fn with_case_insensitive_routing(
        pages_dir: impl Into<PathBuf>,
        case_insensitive: bool,
    ) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            components_dir: "components".into(),
            templates: HashMap::new(),
            components: HashMap::new(),
            router: Router::with_case_insensitive(case_insensitive),
            case_insensitive,
        }
    }
}

// ============================================================================
// Functional Builder Methods (Immutable API)
// ============================================================================

impl TemplateLoader {
    /// Load all templates from the pages directory (functional style)
    /// Returns a new TemplateLoader instance with loaded templates
    pub fn with_templates_from_pages(self) -> Result<Self> {
        let pages_dir = self.pages_dir.clone();
        let components_dir = self.components_dir.clone();

        // Load templates from pages directory
        let template_data = load_templates_from_dir_pure(&pages_dir, &pages_dir)?;

        // Load components
        let component_data = load_components_from_dir_pure(&components_dir)?;

        // Build new instance with loaded data
        let mut new_self = self;
        for (key, template, route) in template_data {
            new_self = new_self.with_template(key, template).with_route(route);
        }

        for (name, template) in component_data {
            new_self = new_self.with_component(name, template);
        }

        // Sort routes by priority
        new_self.router.sort_routes();

        Ok(new_self)
    }

    /// Add a single template with its route pattern (builder pattern)
    pub fn with_template(mut self, key: String, template: Template) -> Self {
        self.templates.insert(key, template);
        self
    }

    /// Add multiple templates at once (builder pattern)
    pub fn with_templates(mut self, templates: HashMap<String, Template>) -> Self {
        self.templates.extend(templates);
        self
    }

    /// Add a single component (builder pattern)
    pub fn with_component(mut self, name: String, template: Template) -> Self {
        self.components.insert(name, template);
        self
    }

    /// Add multiple components at once (builder pattern)
    pub fn with_components(mut self, components: HashMap<String, Template>) -> Self {
        self.components.extend(components);
        self
    }

    /// Add a route to the router (builder pattern)
    pub fn with_route(mut self, route: Route) -> Self {
        self.router.add_route(route);
        self
    }

    /// Set case-insensitive mode (builder pattern)
    pub fn with_case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.router.set_case_insensitive(case_insensitive);
        self.case_insensitive = case_insensitive;
        self
    }

    /// Reload a specific template, returning a new instance
    pub fn with_reloaded_template(self, path: &Path) -> Result<Self> {
        let is_component = path
            .to_str()
            .unwrap_or("")
            .contains(&format!("/{}/", &self.components_dir.to_string_lossy()))
            || path
                .to_str()
                .unwrap_or("")
                .contains(&format!("\\{}\\", &self.components_dir.to_string_lossy()));

        if is_component {
            self.with_reloaded_component(path)
        } else {
            let relative_path = normalize_path(path);

            // Load the new template
            let pages_dir = self.pages_dir.clone();
            let (key, template, route) = load_template_pure(&relative_path, &pages_dir)?;

            // Remove old template and route
            let mut new_self = self;
            new_self.templates.remove(&route.pattern);
            new_self.router.remove_route(&route.pattern);

            // Add new template and route
            new_self = new_self.with_template(key, template).with_route(route);

            // Re-sort routes
            new_self.router.sort_routes();

            Ok(new_self)
        }
    }

    /// Reload a specific component, returning a new instance
    pub fn with_reloaded_component(self, path: &Path) -> Result<Self> {
        let relative_path = normalize_path(path);

        let (name, template) = load_component_pure(&relative_path)?;

        // Remove old component and add new one
        let mut new_self = self;
        new_self.components.remove(&name);
        new_self = new_self.with_component(name, template);

        Ok(new_self)
    }

    /// Reload all templates and components, returning a new instance
    pub fn with_reloaded_all(self) -> Result<Self> {
        // Create a fresh instance with the same config
        let new_loader = Self {
            pages_dir: self.pages_dir.clone(),
            components_dir: self.components_dir.clone(),
            templates: HashMap::new(),
            components: HashMap::new(),
            router: Router::with_case_insensitive(self.case_insensitive),
            case_insensitive: self.case_insensitive,
        };

        // Load all templates and components
        new_loader.with_templates_from_pages()
    }
}

// ============================================================================
// Pure Query Methods (Read-only)
// ============================================================================

impl TemplateLoader {
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

    /// Convert file path to route (e.g., pages/index.rhtmx -> "/")
    /// This is a pure function that doesn't modify state
    fn path_to_route(&self, path: &Path) -> String {
        path_to_route_pure(path, &self.pages_dir)
    }
}

// ============================================================================
// Deprecated Mutable Methods (for backward compatibility)
// ============================================================================

impl TemplateLoader {
    /// Load all templates from the pages directory
    #[deprecated(
        note = "Use with_templates_from_pages() for functional programming style. This method will be removed in a future version."
    )]
    pub fn load_all(&mut self) -> Result<()> {
        let pages_dir = self.pages_dir.clone();
        let case_insensitive = self.case_insensitive;
        let new_self = std::mem::replace(
            self,
            Self::with_case_insensitive_routing(&pages_dir, case_insensitive),
        )
        .with_templates_from_pages()?;
        *self = new_self;
        Ok(())
    }

    /// Set case-insensitive mode
    #[deprecated(
        note = "Use with_case_insensitive() for functional programming style. This method will be removed in a future version."
    )]
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.router.set_case_insensitive(case_insensitive);
        self.case_insensitive = case_insensitive;
    }

    /// Reload a specific template file
    #[deprecated(
        note = "Use with_reloaded_template() for functional programming style. This method will be removed in a future version."
    )]
    pub fn reload_template(&mut self, path: &Path) -> Result<()> {
        let pages_dir = self.pages_dir.clone();
        let case_insensitive = self.case_insensitive;
        let new_self = std::mem::replace(
            self,
            Self::with_case_insensitive_routing(&pages_dir, case_insensitive),
        )
        .with_reloaded_template(path)?;
        *self = new_self;
        Ok(())
    }

    /// Reload a specific component file
    #[deprecated(
        note = "Use with_reloaded_component() for functional programming style. This method will be removed in a future version."
    )]
    pub fn reload_component(&mut self, path: &Path) -> Result<()> {
        let pages_dir = self.pages_dir.clone();
        let case_insensitive = self.case_insensitive;
        let new_self = std::mem::replace(
            self,
            Self::with_case_insensitive_routing(&pages_dir, case_insensitive),
        )
        .with_reloaded_component(path)?;
        *self = new_self;
        Ok(())
    }

    /// Reload all templates and components
    #[deprecated(
        note = "Use with_reloaded_all() for functional programming style. This method will be removed in a future version."
    )]
    pub fn reload_all(&mut self) -> Result<()> {
        let pages_dir = self.pages_dir.clone();
        let case_insensitive = self.case_insensitive;
        let new_self = std::mem::replace(
            self,
            Self::with_case_insensitive_routing(&pages_dir, case_insensitive),
        )
        .with_reloaded_all()?;
        *self = new_self;
        Ok(())
    }
}

// ============================================================================
// Pure Helper Functions (No I/O side effects on state)
// ============================================================================

/// Pure function to load all templates from a directory recursively
/// Returns a vector of (key, template, route) tuples
fn load_templates_from_dir_pure(
    dir: &Path,
    pages_dir: &Path,
) -> Result<Vec<(String, Template, Route)>> {
    let mut results = Vec::new();

    if !dir.exists() {
        return Ok(results);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively load subdirectories
            let sub_results = load_templates_from_dir_pure(&path, pages_dir)?;
            results.extend(sub_results);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rhtmx") {
            // Load .rhtmx files
            let template_data = load_template_pure(&path, pages_dir)?;
            results.push(template_data);
        }
    }

    Ok(results)
}

/// Pure function to load all components from the components directory
/// Returns a HashMap of component name -> Template
fn load_components_from_dir_pure(components_dir: &Path) -> Result<HashMap<String, Template>> {
    let mut components = HashMap::new();

    if !components_dir.exists() {
        return Ok(components);
    }

    for entry in fs::read_dir(components_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rhtmx") {
            let (name, template) = load_component_pure(&path)?;
            components.insert(name, template);
        }
    }

    Ok(components)
}

/// Pure function to load a single template file
/// Returns (storage_key, template, route) tuple
fn load_template_pure(path: &Path, pages_dir: &Path) -> Result<(String, Template, Route)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read template: {:?}", path))?;

    // Create a Route for the router
    let route_obj = Route::from_path(
        path.to_str().unwrap_or(""),
        pages_dir.to_str().unwrap_or("pages"),
    );

    // Process CSS and extract partials info
    // TODO: Implement CSS parsing with CssParser
    let partials = vec![]; // Placeholder for extracting partials
    let content_without_css = content;
    let scoped_css = None;

    let template = Template {
        path: path.to_path_buf(),
        content: content_without_css,
        scoped_css,
        partials,
    };

    // Determine storage key
    let storage_key = if route_obj.is_layout || route_obj.is_error_page {
        path_to_route_pure(path, pages_dir)
    } else {
        route_obj.pattern.clone()
    };

    Ok((storage_key, template, route_obj))
}

/// Pure function to load a single component file
/// Returns (component_name, template) tuple
fn load_component_pure(path: &Path) -> Result<(String, Template)> {
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
    let content_without_css = content;
    let scoped_css = None;

    let template = Template {
        path: path.to_path_buf(),
        content: content_without_css,
        scoped_css,
        partials,
    };

    Ok((name, template))
}

/// Pure function to convert file path to route (e.g., pages/index.rhtmx -> "/")
fn path_to_route_pure(path: &Path, pages_dir: &Path) -> String {
    let relative = path.strip_prefix(pages_dir).unwrap_or(path);

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

/// Helper function to normalize path (convert absolute to relative if needed)
fn normalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        let current_dir = std::env::current_dir().unwrap_or_default();
        path.strip_prefix(&current_dir)
            .unwrap_or(path)
            .to_path_buf()
    } else {
        path.to_path_buf()
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

    #[test]
    fn test_path_to_route_pure() {
        let pages_dir = PathBuf::from("pages");

        // Test cases
        assert_eq!(
            path_to_route_pure(Path::new("pages/index.rhtmx"), &pages_dir),
            "/"
        );
        assert_eq!(
            path_to_route_pure(Path::new("pages/about.rhtmx"), &pages_dir),
            "/about"
        );
        assert_eq!(
            path_to_route_pure(Path::new("pages/users/profile.rhtmx"), &pages_dir),
            "/users/profile"
        );
    }

    #[test]
    fn test_builder_pattern() {
        let loader = TemplateLoader::new("pages")
            .with_case_insensitive(true)
            .with_template(
                "/test".to_string(),
                Template {
                    path: PathBuf::from("test.rhtmx"),
                    content: "test content".to_string(),
                    scoped_css: None,
                    partials: vec![],
                },
            );

        assert!(loader.get("/test").is_some());
        assert_eq!(loader.get("/test").unwrap().content, "test content");
    }

    #[test]
    fn test_immutable_api() {
        let loader1 = TemplateLoader::new("pages");
        let loader2 = loader1.clone().with_template(
            "/new".to_string(),
            Template {
                path: PathBuf::from("new.rhtmx"),
                content: "new content".to_string(),
                scoped_css: None,
                partials: vec![],
            },
        );

        // Original loader should not have the new template
        assert!(loader1.get("/new").is_none());
        // New loader should have the template
        assert!(loader2.get("/new").is_some());
    }
}
