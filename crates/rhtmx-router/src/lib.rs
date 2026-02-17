//! # RHTMX Router
//!
//! Zero-dependency file-system-based routing with support for:
//! - Static routes (`/about`), dynamic params (`/users/:id`), optional (`/posts/:id?`)
//! - Catch-all routes (`/docs/*slug`), nested layouts, error pages
//! - Route aliases, named routes, redirects, parameter constraints
//!
//! ```
//! use rhtmx_router::{Router, Route};
//!
//! let router = Router::new()
//!     .with_route(Route::from_path("pages/about.rs", "pages"))
//!     .with_route(Route::from_path("pages/users/[id].rs", "pages"));
//!
//! let m = router.match_route("/users/123").unwrap();
//! assert_eq!(m.params.get("id"), Some(&"123".to_string()));
//! ```

use std::collections::HashMap;

mod constraint;
mod intercept;
mod layout;
pub mod path;
pub mod route;

pub use constraint::ParameterConstraint;
pub use intercept::InterceptLevel;
pub use layout::LayoutOption;
pub use path::{is_valid_path, normalize_path, PathHierarchy};
pub use route::pattern::{classify_segment, parse_param_with_constraint, PatternSegmentType};

// ============================================================================
// Core Types
// ============================================================================

/// A route with its URL pattern, parameters, and metadata
#[derive(Debug, Clone)]
pub struct Route {
    pub pattern: String,
    pub template_path: String,
    pub params: Vec<String>,
    pub priority: usize,
    pub is_layout: bool,
    pub has_catch_all: bool,
    pub optional_params: Vec<String>,
    pub is_error_page: bool,
    pub is_nolayout_marker: bool,
    pub is_loading: bool,
    pub is_template: bool,
    pub is_not_found: bool,
    pub is_parallel_route: bool,
    pub parallel_slot: Option<String>,
    pub is_intercepting: bool,
    pub intercept_level: Option<InterceptLevel>,
    pub intercept_target: Option<String>,
    pub layout_option: LayoutOption,
    pub layout_name: Option<String>,
    pub metadata: HashMap<String, String>,
    pub param_constraints: HashMap<String, ParameterConstraint>,
    pub aliases: Vec<String>,
    pub name: Option<String>,
    pub is_redirect: bool,
    pub redirect_to: Option<String>,
    pub redirect_status: Option<u16>,
}

/// Result of matching a route against a URL path
#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub route: Route,
    pub params: HashMap<String, String>,
}

impl RouteMatch {
    pub fn is_redirect(&self) -> bool {
        self.route.is_redirect
    }

    /// Gets the redirect target URL with parameters substituted
    pub fn redirect_target(&self) -> Option<String> {
        self.route.redirect_target(&self.params)
    }

    pub fn redirect_status(&self) -> Option<u16> {
        self.route.redirect_status
    }
}

// ============================================================================
// Route Implementation
// ============================================================================

impl Route {
    /// Creates a route from a file system path (e.g. `pages/users/[id]/page.rs`)
    pub fn from_path(file_path: &str, pages_dir: &str) -> Self {
        let relative = file_path
            .strip_prefix(pages_dir)
            .unwrap_or(file_path)
            .trim_start_matches('/');

        let without_ext = relative.strip_suffix(".rs").unwrap_or(relative);
        let filename = without_ext.split('/').last().unwrap_or("");

        let is_layout = filename == "_layout" || filename.starts_with("_layout.");
        let is_error_page = filename == "_error";
        let is_nolayout_marker = filename == "_nolayout";
        let is_loading = filename == "loading";
        let is_template = filename == "_template";
        let is_not_found = filename == "not-found";

        let (is_parallel_route, parallel_slot) = route::detect_parallel_route(without_ext);
        let (is_intercepting, intercept_level, intercept_target) =
            route::detect_intercepting_route(without_ext);

        let layout_name = if is_layout {
            route::extract_layout_name(filename)
        } else {
            None
        };

        let (pattern, params, optional_params, dynamic_count, has_catch_all, param_constraints) =
            route::parse_pattern(without_ext);

        let depth = pattern.matches('/').count();
        let priority =
            route::calculate_priority(has_catch_all, dynamic_count, depth, &optional_params);

        Route {
            pattern, template_path: file_path.to_string(), params, priority,
            is_layout, has_catch_all, optional_params, is_error_page, is_nolayout_marker,
            is_loading, is_template, is_not_found, is_parallel_route, parallel_slot,
            is_intercepting, intercept_level, intercept_target,
            layout_option: LayoutOption::default(), layout_name,
            metadata: HashMap::new(), param_constraints,
            aliases: Vec::new(), name: None,
            is_redirect: false, redirect_to: None, redirect_status: None,
        }
    }

    /// Matches this route against a path (case-sensitive)
    pub fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        self.matches_with_options(path, false)
    }

    /// Matches this route against a path with case-sensitivity option
    pub fn matches_with_options(
        &self,
        path: &str,
        case_insensitive: bool,
    ) -> Option<HashMap<String, String>> {
        // For redirect routes with no parameters, do exact matching
        if self.is_redirect && self.params.is_empty() {
            let matches = if case_insensitive {
                self.pattern.eq_ignore_ascii_case(path)
            } else {
                self.pattern == path
            };
            return if matches { Some(HashMap::new()) } else { None };
        }

        let pattern_segments: Vec<&str> =
            self.pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        fn match_segments(
            pattern_segments: &[&str],
            path_segments: &[&str],
            pi: usize,
            si: usize,
            params: HashMap<String, String>,
            ci: bool,
        ) -> Option<HashMap<String, String>> {
            if pi >= pattern_segments.len() {
                return if si == path_segments.len() { Some(params) } else { None };
            }

            let seg = pattern_segments[pi];

            match seg.chars().next() {
                Some('*') => {
                    let is_optional = seg.ends_with('?');
                    let name = if is_optional { &seg[1..seg.len() - 1] } else { &seg[1..] };
                    let remaining: Vec<&str> = path_segments[si..].to_vec();
                    if remaining.is_empty() && !is_optional { return None; }
                    let mut p = params;
                    p.insert(name.to_string(), remaining.join("/"));
                    Some(p)
                }
                Some(':') if seg.ends_with('?') => {
                    let name = &seg[1..seg.len() - 1];
                    let mut p = params;
                    if si < path_segments.len() {
                        let should_consume = if pi + 1 < pattern_segments.len() {
                            let next = pattern_segments[pi + 1];
                            match next.chars().next() {
                                Some(':') | Some('*') => true,
                                _ => if ci { !next.eq_ignore_ascii_case(path_segments[si]) }
                                     else { next != path_segments[si] }
                            }
                        } else { true };
                        if should_consume {
                            p.insert(name.to_string(), path_segments[si].to_string());
                            return match_segments(pattern_segments, path_segments, pi + 1, si + 1, p, ci);
                        }
                    }
                    match_segments(pattern_segments, path_segments, pi + 1, si, p, ci)
                }
                Some(':') => {
                    if si >= path_segments.len() { return None; }
                    let name = &seg[1..];
                    let mut p = params;
                    p.insert(name.to_string(), path_segments[si].to_string());
                    match_segments(pattern_segments, path_segments, pi + 1, si + 1, p, ci)
                }
                _ => {
                    if si >= path_segments.len() { return None; }
                    let ok = if ci { seg.eq_ignore_ascii_case(path_segments[si]) }
                             else { seg == path_segments[si] };
                    if !ok { return None; }
                    match_segments(pattern_segments, path_segments, pi + 1, si + 1, params, ci)
                }
            }
        }

        let params = match_segments(&pattern_segments, &path_segments, 0, 0, HashMap::new(), case_insensitive)?;

        // Validate constraints
        let all_valid = params.iter().all(|(name, value)| {
            self.param_constraints.get(name).map(|c| c.validate(value)).unwrap_or(true)
        });

        if all_valid { Some(params) } else { None }
    }

    /// Returns the parent pattern for layout lookup
    pub fn layout_pattern(&self) -> Option<String> {
        self.pattern.rfind('/').and_then(|pos| {
            if pos == 0 { None } else { Some(self.pattern[..pos].to_string()) }
        })
    }

    // -- Builder methods --

    pub fn with_layout_option(mut self, option: LayoutOption) -> Self {
        self.layout_option = option; self
    }

    pub fn with_no_layout(self) -> Self { self.with_layout_option(LayoutOption::None) }
    pub fn with_root_layout(self) -> Self { self.with_layout_option(LayoutOption::Root) }

    pub fn with_named_layout(self, name: impl Into<String>) -> Self {
        self.with_layout_option(LayoutOption::Named(name.into()))
    }

    pub fn with_layout_pattern(self, pattern: impl Into<String>) -> Self {
        self.with_layout_option(LayoutOption::Pattern(pattern.into()))
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into()); self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata); self
    }

    pub fn get_meta(&self, key: &str) -> Option<&String> { self.metadata.get(key) }
    pub fn has_meta(&self, key: &str) -> bool { self.metadata.contains_key(key) }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into()); self
    }

    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.aliases.extend(aliases.into_iter().map(|s| s.into())); self
    }

    /// Checks if path matches this route or any alias
    pub fn matches_any(&self, path: &str) -> Option<HashMap<String, String>> {
        if let Some(params) = self.matches(path) { return Some(params); }
        self.aliases.iter().find_map(|alias| {
            if self.matches_static_alias(path, alias) { Some(HashMap::new()) } else { None }
        })
    }

    pub fn matches_static_alias(&self, path: &str, alias: &str) -> bool {
        let a = path.trim_end_matches('/');
        let b = alias.trim_end_matches('/');
        if a.is_empty() && b.is_empty() { return true; }
        a == b
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into()); self
    }

    /// Generates a URL by substituting parameters into the pattern
    pub fn generate_url(&self, params: &HashMap<String, String>) -> Option<String> {
        let segments: Vec<&str> = self.pattern.split('/').filter(|s| !s.is_empty()).collect();
        let result: Option<Vec<String>> = segments.iter().map(|seg| {
            match seg.chars().next() {
                Some(':') => {
                    let name = seg.trim_start_matches(':').trim_end_matches('?');
                    if seg.ends_with('?') {
                        Some(params.get(name).cloned().unwrap_or_default())
                    } else {
                        params.get(name).cloned()
                    }
                }
                Some('*') => params.get(&seg[1..]).cloned(),
                _ => Some(seg.to_string()),
            }
        }).collect();

        result.map(|segs| {
            let filtered: Vec<String> = segs.into_iter().filter(|s| !s.is_empty()).collect();
            if filtered.is_empty() { "/".to_string() } else { format!("/{}", filtered.join("/")) }
        })
    }

    /// Creates a redirect route
    pub fn redirect(from: impl Into<String>, to: impl Into<String>, status: u16) -> Self {
        let from = from.into();
        let target = to.into();
        let has_params = from.contains('[') || from.contains(':');

        let normalized_from = if from.contains(':') && !from.contains('[') {
            from.split('/').map(|s| {
                if s.starts_with(':') { format!("[{}]", &s[1..]) } else { s.to_string() }
            }).collect::<Vec<_>>().join("/")
        } else {
            from.clone()
        };

        let (pattern, params, optional_params, dynamic_count, has_catch_all, param_constraints) =
            if has_params {
                route::parse_pattern(&normalized_from)
            } else {
                let n = if from.starts_with('/') { from.clone() } else { format!("/{}", from) };
                (n, Vec::new(), Vec::new(), 0, false, HashMap::new())
            };

        let depth = pattern.matches('/').count();
        let priority = route::calculate_priority(has_catch_all, dynamic_count, depth, &optional_params);

        Route {
            pattern, template_path: format!("redirect:{}", target), params, priority,
            is_layout: false, has_catch_all, optional_params, is_error_page: false,
            is_nolayout_marker: false, is_loading: false, is_template: false,
            is_not_found: false, is_parallel_route: false, parallel_slot: None,
            is_intercepting: false, intercept_level: None, intercept_target: None,
            layout_option: LayoutOption::None, layout_name: None,
            metadata: HashMap::new(), param_constraints,
            aliases: Vec::new(), name: None,
            is_redirect: true, redirect_to: Some(target), redirect_status: Some(status),
        }
    }

    /// Generates the redirect target URL with parameter substitution
    pub fn redirect_target(&self, params: &HashMap<String, String>) -> Option<String> {
        if !self.is_redirect { return None; }
        let target = self.redirect_to.as_ref()?;
        if params.is_empty() { return Some(target.clone()); }
        Some(params.iter().fold(target.clone(), |acc, (k, v)| {
            acc.replace(&format!(":{}", k), v)
        }))
    }
}

// ============================================================================
// Router
// ============================================================================

/// Route collection with matching, layout resolution, and URL generation
#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
    layouts: HashMap<String, Route>,
    named_layouts: HashMap<String, Route>,
    named_routes: HashMap<String, Route>,
    error_pages: HashMap<String, Route>,
    loading_pages: HashMap<String, Route>,
    templates: HashMap<String, Route>,
    not_found_pages: HashMap<String, Route>,
    parallel_routes: HashMap<String, HashMap<String, Route>>,
    intercepting_routes: HashMap<String, Route>,
    nolayout_patterns: std::collections::HashSet<String>,
    case_insensitive: bool,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            layouts: HashMap::new(),
            named_layouts: HashMap::new(),
            named_routes: HashMap::new(),
            error_pages: HashMap::new(),
            loading_pages: HashMap::new(),
            templates: HashMap::new(),
            not_found_pages: HashMap::new(),
            parallel_routes: HashMap::new(),
            intercepting_routes: HashMap::new(),
            nolayout_patterns: std::collections::HashSet::new(),
            case_insensitive: false,
        }
    }

    /// Creates a router with case-insensitive matching
    ///
    /// ```
    /// use rhtmx_router::Router;
    /// let router = Router::with_case_insensitive(true);
    /// ```
    pub fn with_case_insensitive(case_insensitive: bool) -> Self {
        let mut r = Self::new();
        r.case_insensitive = case_insensitive;
        r
    }

    // -- Functional builder methods --

    /// ```
    /// use rhtmx_router::Router;
    /// let router = Router::new().with_case_sensitivity(false);
    /// ```
    pub fn with_case_sensitivity(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive; self
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let router = Router::new()
    ///     .with_route(Route::from_path("pages/about.rs", "pages"));
    /// ```
    pub fn with_route(mut self, route: Route) -> Self {
        self.add_route_internal(&route); self
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let routes = vec![
    ///     Route::from_path("pages/about.rs", "pages"),
    ///     Route::from_path("pages/users/[id].rs", "pages"),
    /// ];
    /// let router = Router::new().with_routes(routes);
    /// ```
    pub fn with_routes<I: IntoIterator<Item = Route>>(mut self, routes: I) -> Self {
        for route in routes { self.add_route_internal(&route); }
        self
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let router = Router::new()
    ///     .with_route(Route::from_path("pages/about.rs", "pages"))
    ///     .without_route("/about");
    /// ```
    pub fn without_route(mut self, pattern: &str) -> Self {
        self.remove_route_internal(pattern); self
    }

    /// ```
    /// use rhtmx_router::Router;
    /// let router = Router::new().with_sorted_routes();
    /// ```
    pub fn with_sorted_routes(mut self) -> Self {
        self.routes.sort_by_key(|r| r.priority); self
    }

    // -- Mutable methods (used by template_loader and tests) --

    #[deprecated(since = "0.2.0", note = "Use with_case_sensitivity() for functional programming style")]
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/about.rs", "pages"));
    /// ```
    #[deprecated(since = "0.2.0", note = "Use with_route() for functional programming style")]
    pub fn add_route(&mut self, route: Route) {
        self.add_route_internal(&route);
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/about.rs", "pages"));
    /// router.remove_route("/about");
    /// ```
    #[deprecated(since = "0.2.0", note = "Use without_route() for functional programming style")]
    pub fn remove_route(&mut self, pattern: &str) {
        self.remove_route_internal(pattern);
    }

    /// ```
    /// use rhtmx_router::Router;
    /// let mut router = Router::new();
    /// router.sort_routes();
    /// ```
    #[deprecated(since = "0.2.0", note = "Use with_sorted_routes() for functional programming style")]
    pub fn sort_routes(&mut self) {
        self.routes.sort_by_key(|r| r.priority);
    }

    // -- Internal helpers --

    fn add_route_internal(&mut self, route: &Route) {
        if route.is_nolayout_marker {
            self.nolayout_patterns.insert(route.pattern.clone());
            return;
        }
        if let Some(ref name) = route.name {
            self.named_routes.insert(name.clone(), route.clone());
        }
        if route.is_layout {
            self.layouts.insert(route.pattern.clone(), route.clone());
            if let Some(ref name) = route.layout_name {
                self.named_layouts.insert(name.clone(), route.clone());
            }
        } else if route.is_error_page {
            self.error_pages.insert(route.pattern.clone(), route.clone());
        } else if route.is_loading {
            self.loading_pages.insert(route.pattern.clone(), route.clone());
        } else if route.is_template {
            self.templates.insert(route.pattern.clone(), route.clone());
        } else if route.is_not_found {
            self.not_found_pages.insert(route.pattern.clone(), route.clone());
        } else if route.is_parallel_route {
            if let Some(ref slot) = route.parallel_slot {
                self.parallel_routes
                    .entry(route.pattern.clone())
                    .or_default()
                    .insert(slot.clone(), route.clone());
            }
        } else if route.is_intercepting {
            self.intercepting_routes.insert(route.pattern.clone(), route.clone());
        } else {
            self.routes.push(route.clone());
            self.routes.sort_by_key(|r| r.priority);
        }
    }

    fn remove_route_internal(&mut self, pattern: &str) {
        if let Some(pos) = self.routes.iter().position(|r| r.pattern == pattern) {
            let route = &self.routes[pos];
            if let Some(name) = &route.name { self.named_routes.remove(name); }
            self.routes.remove(pos);
        }
        if let Some(layout) = self.layouts.remove(pattern) {
            if let Some(name) = &layout.layout_name { self.named_layouts.remove(name); }
        }
        self.error_pages.remove(pattern);
        self.loading_pages.remove(pattern);
        self.templates.remove(pattern);
        self.not_found_pages.remove(pattern);
        self.parallel_routes.remove(pattern);
        self.intercepting_routes.remove(pattern);
    }

    // -- Query methods --

    /// Matches a URL path against registered routes
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/users/[id].rs", "pages"));
    /// let m = router.match_route("/users/123").unwrap();
    /// assert_eq!(m.params.get("id"), Some(&"123".to_string()));
    /// ```
    pub fn match_route(&self, path: &str) -> Option<RouteMatch> {
        self.routes.iter().find_map(|route| {
            if let Some(params) = route.matches_with_options(path, self.case_insensitive) {
                return Some(RouteMatch { route: route.clone(), params });
            }
            route.aliases.iter().find_map(|alias| {
                if route.matches_static_alias(path, alias) {
                    Some(RouteMatch { route: route.clone(), params: HashMap::new() })
                } else { None }
            })
        })
    }

    /// Walks up directory hierarchy to find nearest scoped resource
    fn get_scoped_resource<'a>(&'a self, pattern: &str, map: &'a HashMap<String, Route>) -> Option<&'a Route> {
        let normalized = normalize_path(pattern);
        PathHierarchy::new(&normalized).find_map(|path| map.get(path))
    }

    /// Finds the nearest layout for a route pattern
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    /// let layout = router.get_layout("/dashboard/settings").unwrap();
    /// assert_eq!(layout.pattern, "/dashboard");
    /// ```
    pub fn get_layout(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.layouts)
    }

    /// Finds layout for a RouteMatch, respecting its layout_option
    pub fn get_layout_for_match(&self, route_match: &RouteMatch) -> Option<&Route> {
        self.get_layout_with_option(&route_match.route.pattern, &route_match.route.layout_option)
    }

    pub fn get_layout_with_option(&self, pattern: &str, option: &LayoutOption) -> Option<&Route> {
        match option {
            LayoutOption::None => None,
            LayoutOption::Root => self.layouts.get("/"),
            LayoutOption::Named(name) => self.named_layouts.get(name),
            LayoutOption::Pattern(pat) => {
                let normalized = normalize_path(pat);
                self.layouts.get(normalized.as_ref())
            }
            LayoutOption::Inherit => {
                if self.is_under_nolayout_marker(pattern) { return None; }
                self.get_scoped_resource(pattern, &self.layouts)
            }
        }
    }

    pub fn is_under_nolayout_marker(&self, pattern: &str) -> bool {
        let normalized = normalize_path(pattern);
        PathHierarchy::new(&normalized).any(|path| self.nolayout_patterns.contains(path))
    }

    pub fn get_layout_by_name(&self, name: &str) -> Option<&Route> {
        self.named_layouts.get(name)
    }

    pub fn routes(&self) -> &[Route] { &self.routes }
    pub fn layouts(&self) -> &HashMap<String, Route> { &self.layouts }

    /// Finds the nearest error page for a route pattern
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_error.rs", "pages"));
    /// router.add_route(Route::from_path("pages/api/_error.rs", "pages"));
    /// let err = router.get_error_page("/api/users").unwrap();
    /// assert_eq!(err.pattern, "/api");
    /// ```
    pub fn get_error_page(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.error_pages)
    }

    pub fn error_pages(&self) -> &HashMap<String, Route> { &self.error_pages }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/dashboard/loading.rs", "pages"));
    /// let loading = router.get_loading_page("/dashboard/users").unwrap();
    /// assert_eq!(loading.pattern, "/dashboard");
    /// ```
    pub fn get_loading_page(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.loading_pages)
    }

    pub fn loading_pages(&self) -> &HashMap<String, Route> { &self.loading_pages }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_template.rs", "pages"));
    /// let template = router.get_template("/about").unwrap();
    /// assert_eq!(template.pattern, "/");
    /// ```
    pub fn get_template(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.templates)
    }

    pub fn templates(&self) -> &HashMap<String, Route> { &self.templates }

    pub fn get_not_found_page(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.not_found_pages)
    }

    pub fn not_found_pages(&self) -> &HashMap<String, Route> { &self.not_found_pages }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/dashboard/@analytics/page.rs", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/@team/page.rs", "pages"));
    /// let slots = router.get_parallel_routes("/dashboard").unwrap();
    /// assert_eq!(slots.len(), 2);
    /// ```
    pub fn get_parallel_routes(&self, pattern: &str) -> Option<&HashMap<String, Route>> {
        self.parallel_routes.get(pattern)
    }

    pub fn parallel_routes(&self) -> &HashMap<String, HashMap<String, Route>> { &self.parallel_routes }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/dashboard/@analytics/page.rs", "pages"));
    /// let route = router.get_parallel_route("/dashboard", "analytics").unwrap();
    /// assert_eq!(route.parallel_slot, Some("analytics".to_string()));
    /// ```
    pub fn get_parallel_route(&self, pattern: &str, slot: &str) -> Option<&Route> {
        self.parallel_routes.get(pattern).and_then(|slots| slots.get(slot))
    }

    pub fn get_intercepting_route(&self, pattern: &str) -> Option<&Route> {
        self.intercepting_routes.get(pattern)
    }

    pub fn intercepting_routes(&self) -> &HashMap<String, Route> { &self.intercepting_routes }

    // -- Named route URL generation --

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// use std::collections::HashMap;
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/users/[id].rs", "pages").with_name("user.profile"));
    /// let mut params = HashMap::new();
    /// params.insert("id".to_string(), "123".to_string());
    /// let url = router.url_for("user.profile", &params).unwrap();
    /// assert_eq!(url, "/users/123");
    /// ```
    pub fn url_for(&self, name: &str, params: &HashMap<String, String>) -> Option<String> {
        self.named_routes.get(name).and_then(|route| route.generate_url(params))
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/posts/[year]/[slug].rs", "pages").with_name("post.show"));
    /// let url = router.url_for_params("post.show", &[("year", "2024"), ("slug", "hello-world")]).unwrap();
    /// assert_eq!(url, "/posts/2024/hello-world");
    /// ```
    pub fn url_for_params(&self, name: &str, params: &[(&str, &str)]) -> Option<String> {
        let map: HashMap<String, String> = params.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        self.url_for(name, &map)
    }

    /// ```
    /// use rhtmx_router::{Router, Route};
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/about.rs", "pages").with_name("about"));
    /// let route = router.get_route_by_name("about").unwrap();
    /// assert_eq!(route.pattern, "/about");
    /// ```
    pub fn get_route_by_name(&self, name: &str) -> Option<&Route> {
        self.named_routes.get(name)
    }
}

impl Default for Router {
    fn default() -> Self { Self::new() }
}
