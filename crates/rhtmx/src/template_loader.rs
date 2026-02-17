use anyhow::Result;
use axum::response::Response;
use rhtmx_router::{Route, Router};
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::{fmt, fs};

use crate::request_context::RequestContext;

/// Handler function type for page routes
///
/// Each page .rs file exports a handler that receives a RequestContext
/// and returns an Axum Response.
pub type HandlerFn =
    Arc<dyn Fn(RequestContext) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// A discovered page route with its handler
pub struct PageRoute {
    pub path: PathBuf,
    pub handler: Option<HandlerFn>,
}

impl Clone for PageRoute {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            handler: self.handler.clone(),
        }
    }
}

impl fmt::Debug for PageRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageRoute")
            .field("path", &self.path)
            .field("has_handler", &self.handler.is_some())
            .finish()
    }
}

/// Route discoverer and handler registry for file-based routing
///
/// Scans the pages directory for .rs files to discover routes from
/// the file structure. Handlers are registered separately since .rs
/// files are compiled code, not runtime templates.
#[derive(Clone)]
pub struct TemplateLoader {
    pages_dir: PathBuf,
    routes: HashMap<String, PageRoute>,
    router: Router,
    case_insensitive: bool,
}

impl TemplateLoader {
    pub fn new(pages_dir: impl Into<PathBuf>) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            routes: HashMap::new(),
            router: Router::new(),
            case_insensitive: false,
        }
    }

    pub fn with_config(
        pages_dir: impl Into<PathBuf>,
        _components_dir: impl Into<PathBuf>,
        case_insensitive: bool,
    ) -> Self {
        Self {
            pages_dir: pages_dir.into(),
            routes: HashMap::new(),
            router: Router::with_case_insensitive(case_insensitive),
            case_insensitive,
        }
    }

    /// Discover all routes from .rs files in pages directory
    #[allow(deprecated)]
    pub fn discover_routes(&mut self) -> Result<()> {
        let pages_dir = self.pages_dir.clone();
        let ci = self.case_insensitive;
        *self = Self::with_config(&pages_dir, "", ci);

        let discovered = discover_routes_from_dir(&pages_dir, &pages_dir)?;
        for (key, page_route, route) in discovered {
            self.routes.insert(key, page_route);
            self.router.add_route(route);
        }
        self.router.sort_routes();
        Ok(())
    }

    /// Register a handler for a discovered route pattern
    pub fn register_handler(&mut self, pattern: &str, handler: HandlerFn) {
        if let Some(page_route) = self.routes.get_mut(pattern) {
            page_route.handler = Some(handler);
        }
    }

    /// Register a route with its handler directly (bypasses file discovery)
    #[allow(deprecated)]
    pub fn register_route(&mut self, pattern: impl Into<String>, handler: HandlerFn) {
        let pattern = pattern.into();
        let route = Route::from_path(&format!("pages{}.rs", pattern), "pages");
        let page_route = PageRoute {
            path: PathBuf::from(format!("pages{}.rs", pattern)),
            handler: Some(handler),
        };
        self.routes.insert(pattern, page_route);
        self.router.add_route(route);
        self.router.sort_routes();
    }

    /// Get a page route by pattern
    pub fn get(&self, route: &str) -> Option<&PageRoute> {
        self.routes.get(route)
    }

    /// Get the handler for a route pattern
    pub fn get_handler(&self, route: &str) -> Option<&HandlerFn> {
        self.routes.get(route).and_then(|pr| pr.handler.as_ref())
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn list_routes(&self) -> Vec<String> {
        let mut routes: Vec<_> = self.routes.keys().cloned().collect();
        routes.sort();
        routes
    }

    pub fn count(&self) -> usize {
        self.routes.len()
    }
}

// --- Pure helper functions ---

fn discover_routes_from_dir(
    dir: &Path,
    pages_dir: &Path,
) -> Result<Vec<(String, PageRoute, Route)>> {
    let mut results = Vec::new();
    if !dir.exists() {
        return Ok(results);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            results.extend(discover_routes_from_dir(&path, pages_dir)?);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            results.push(discover_route(&path, pages_dir)?);
        }
    }
    Ok(results)
}

fn discover_route(path: &Path, pages_dir: &Path) -> Result<(String, PageRoute, Route)> {
    let route_obj = Route::from_path(
        path.to_str().unwrap_or(""),
        pages_dir.to_str().unwrap_or("pages"),
    );

    let page_route = PageRoute {
        path: path.to_path_buf(),
        handler: None, // Handler is registered separately after compilation
    };

    let storage_key = if route_obj.is_layout || route_obj.is_error_page {
        path_to_route(path, pages_dir)
    } else {
        route_obj.pattern.clone()
    };

    Ok((storage_key, page_route, route_obj))
}

fn path_to_route(path: &Path, pages_dir: &Path) -> String {
    let relative = path.strip_prefix(pages_dir).unwrap_or(path);
    let route = relative
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/");

    if route == "_error" {
        return "/_error".to_string();
    }
    if route.ends_with("/_error") {
        return if route.starts_with('/') {
            route
        } else {
            format!("/{}", route)
        };
    }
    if route == "_layout" {
        return "/_layout".to_string();
    }
    if route.ends_with("/_layout") {
        return if route.starts_with('/') {
            route
        } else {
            format!("/{}", route)
        };
    }
    if route == "page" || route.is_empty() {
        return "/".to_string();
    }
    if route.ends_with("/page") {
        let without = route[..route.len() - 5].to_string();
        return if without.is_empty() {
            "/".to_string()
        } else if without.starts_with('/') {
            without
        } else {
            format!("/{}", without)
        };
    }
    if route.starts_with('/') {
        route
    } else {
        format!("/{}", route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_route() {
        let pages_dir = PathBuf::from("pages");
        assert_eq!(path_to_route(Path::new("pages/page.rs"), &pages_dir), "/");
        assert_eq!(
            path_to_route(Path::new("pages/about/page.rs"), &pages_dir),
            "/about"
        );
        assert_eq!(
            path_to_route(Path::new("pages/users/profile/page.rs"), &pages_dir),
            "/users/profile"
        );
    }

    #[test]
    fn test_register_route() {
        let mut loader = TemplateLoader::new("pages");
        let handler: HandlerFn = Arc::new(|_ctx| {
            Box::pin(async { axum::response::IntoResponse::into_response("test") })
        });
        loader.register_route("/test", handler);
        assert!(loader.get("/test").is_some());
        assert!(loader.get_handler("/test").is_some());
    }
}
