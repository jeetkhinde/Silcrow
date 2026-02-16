use anyhow::{Context, Result};
use rhtmx_router::{Route, Router};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a loaded template
#[derive(Debug, Clone)]
pub struct Template {
    pub path: PathBuf,
    pub content: String,
    pub scoped_css: Option<()>,
}

/// Template loader that reads and caches template files from a pages directory
#[derive(Clone)]
pub struct TemplateLoader {
    pages_dir: PathBuf,
    components_dir: PathBuf,
    templates: HashMap<String, Template>,
    components: HashMap<String, Template>,
    router: Router,
    case_insensitive: bool,
}

impl TemplateLoader {
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

    /// Load all templates from pages directory (builder pattern)
    #[allow(deprecated)]
    pub fn with_templates_from_pages(self) -> Result<Self> {
        let pages_dir = self.pages_dir.clone();
        let components_dir = self.components_dir.clone();

        let template_data = load_templates_from_dir(&pages_dir, &pages_dir)?;
        let component_data = load_components_from_dir(&components_dir)?;

        let mut new_self = self;
        for (key, template, route) in template_data {
            new_self.templates.insert(key, template);
            new_self.router.add_route(route);
        }
        for (name, template) in component_data {
            new_self.components.insert(name, template);
        }
        new_self.router.sort_routes();
        Ok(new_self)
    }

    /// Mutable load_all for use by server
    pub fn load_all(&mut self) -> Result<()> {
        let pages_dir = self.pages_dir.clone();
        let components_dir = self.components_dir.clone();
        let ci = self.case_insensitive;
        let new = std::mem::replace(self, Self::with_config(&pages_dir, &components_dir, ci))
            .with_templates_from_pages()?;
        *self = new;
        Ok(())
    }

    /// Reload a specific template file
    #[allow(deprecated)]
    pub fn reload_template(&mut self, path: &Path) -> Result<()> {
        let is_component = path.to_str().unwrap_or("")
            .contains(&format!("/{}/", &self.components_dir.to_string_lossy()));

        if is_component {
            let relative = normalize_path(path);
            let (name, template) = load_component(&relative)?;
            self.components.remove(&name);
            self.components.insert(name, template);
        } else {
            let relative = normalize_path(path);
            let (key, template, route) = load_template(&relative, &self.pages_dir)?;
            self.templates.remove(&route.pattern);
            self.router.remove_route(&route.pattern);
            self.templates.insert(key, template);
            self.router.add_route(route);
            self.router.sort_routes();
        }
        Ok(())
    }

    pub fn get(&self, route: &str) -> Option<&Template> {
        self.templates.get(route)
    }

    pub fn get_layout(&self) -> Option<&Template> {
        self.templates.get("/_layout")
    }

    pub fn get_layout_for_route(&self, pattern: &str) -> Option<&Template> {
        if let Some(layout_route) = self.router.get_layout(pattern) {
            let layout_key = if layout_route.pattern == "/" {
                "/_layout".to_string()
            } else {
                format!("{}/_layout", layout_route.pattern)
            };
            self.templates.get(&layout_key)
        } else {
            self.get_layout()
        }
    }

    pub fn get_error_page_for_route(&self, pattern: &str) -> Option<&Template> {
        if let Some(error_route) = self.router.get_error_page(pattern) {
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

    pub fn get_error_page(&self) -> Option<&Template> {
        self.templates.get("/_error")
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn get_component(&self, name: &str) -> Option<&Template> {
        self.components.get(name)
    }

    pub fn list_routes(&self) -> Vec<String> {
        let mut routes: Vec<_> = self.templates.keys().cloned().collect();
        routes.sort();
        routes
    }

    pub fn count(&self) -> usize {
        self.templates.len()
    }
}

// --- Pure helper functions ---

fn load_templates_from_dir(dir: &Path, pages_dir: &Path) -> Result<Vec<(String, Template, Route)>> {
    let mut results = Vec::new();
    if !dir.exists() { return Ok(results); }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            results.extend(load_templates_from_dir(&path, pages_dir)?);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rsx") {
            results.push(load_template(&path, pages_dir)?);
        }
    }
    Ok(results)
}

fn load_components_from_dir(dir: &Path) -> Result<HashMap<String, Template>> {
    let mut components = HashMap::new();
    if !dir.exists() { return Ok(components); }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rhtmx") {
            let (name, template) = load_component(&path)?;
            components.insert(name, template);
        }
    }
    Ok(components)
}

fn load_template(path: &Path, pages_dir: &Path) -> Result<(String, Template, Route)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read template: {:?}", path))?;
    let route_obj = Route::from_path(
        path.to_str().unwrap_or(""),
        pages_dir.to_str().unwrap_or("pages"),
    );
    let template = Template { path: path.to_path_buf(), content, scoped_css: None };
    let storage_key = if route_obj.is_layout || route_obj.is_error_page {
        path_to_route(path, pages_dir)
    } else {
        route_obj.pattern.clone()
    };
    Ok((storage_key, template, route_obj))
}

fn load_component(path: &Path) -> Result<(String, Template)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read component: {:?}", path))?;
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let template = Template { path: path.to_path_buf(), content, scoped_css: None };
    Ok((name, template))
}

fn path_to_route(path: &Path, pages_dir: &Path) -> String {
    let relative = path.strip_prefix(pages_dir).unwrap_or(path);
    let route = relative.with_extension("").to_string_lossy().replace('\\', "/");

    if route == "_error" { return "/_error".to_string(); }
    if route.ends_with("/_error") {
        return if route.starts_with('/') { route } else { format!("/{}", route) };
    }
    if route == "_layout" { return "/_layout".to_string(); }
    if route.ends_with("/_layout") {
        return if route.starts_with('/') { route } else { format!("/{}", route) };
    }
    if route == "page" || route.is_empty() { return "/".to_string(); }
    if route.ends_with("/page") {
        let without = route[..route.len() - 5].to_string();
        return if without.is_empty() { "/".to_string() }
            else if without.starts_with('/') { without }
            else { format!("/{}", without) };
    }
    if route.starts_with('/') { route } else { format!("/{}", route) }
}

fn normalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        let current_dir = std::env::current_dir().unwrap_or_default();
        path.strip_prefix(&current_dir).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_route() {
        let pages_dir = PathBuf::from("pages");
        assert_eq!(path_to_route(Path::new("pages/page.rsx"), &pages_dir), "/");
        assert_eq!(path_to_route(Path::new("pages/about/page.rsx"), &pages_dir), "/about");
        assert_eq!(path_to_route(Path::new("pages/users/profile/page.rsx"), &pages_dir), "/users/profile");
    }

    #[test]
    fn test_builder_pattern() {
        let mut loader = TemplateLoader::new("pages");
        loader.templates.insert("/test".to_string(), Template {
            path: PathBuf::from("test.rhtmx"),
            content: "test content".to_string(),
            scoped_css: None,
        });
        assert!(loader.get("/test").is_some());
        assert_eq!(loader.get("/test").unwrap().content, "test content");
    }
}
