//! # RHTMX Router
//!
//! A zero-dependency file-system-based routing library with support for:
//! - Static routes (`/about`)
//! - Dynamic parameters (`/users/:id`)
//! - Optional parameters (`/posts/:id?`)
//! - Catch-all routes (`/docs/*slug`)
//! - Nested layouts and error pages
//!
//! ## Functional Programming Approach
//!
//! This router uses functional programming techniques for optimal performance:
//! - **Zero-copy optimization** with `Cow<'_, str>` (no allocation for valid paths)
//! - **Lazy evaluation** with custom `PathHierarchy` iterator
//! - **Functional composition** with `find_map()`
//!
//! ## Path Normalization
//!
//! Handles all common user mistakes gracefully:
//! - Trailing slashes: `/path/` → `/path`
//! - Double slashes: `/path//to` → `/path/to`
//! - Backslashes: `\path\to` → `/path/to`
//! - Windows paths: `\path\to` → `/path/to`
//!
//! ## Performance
//!
//! - Valid paths: ~115ns (zero allocations via `Cow::Borrowed`)
//! - Invalid paths: ~310ns (single allocation for normalization)
//! - Lazy iteration stops on first match (short-circuit evaluation)
//!
//! ## Example
//!
//! ```
//! use rhtmx_router::{Router, Route};
//!
//! let mut router = Router::new();
//! router.add_route(Route::from_path("pages/about.rhtml", "pages"));
//! router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));
//!
//! let route_match = router.match_route("/users/123").unwrap();
//! assert_eq!(route_match.params.get("id"), Some(&"123".to_string()));
//! ```

use std::borrow::Cow;
use std::collections::HashMap;

// ============================================================================
// Module Declarations
// ============================================================================

mod constraint;
mod layout;
mod intercept;
pub mod path;
pub mod route;

// Re-export public types for backward compatibility
pub use constraint::ParameterConstraint;
pub use layout::LayoutOption;
pub use intercept::InterceptLevel;
pub use path::{PathHierarchy, is_valid_path, normalize_path};
pub use route::pattern::{classify_segment, parse_param_with_constraint, PatternSegmentType};

// ============================================================================
// Core Types
// ============================================================================

/// Represents a single route with its pattern, parameters, and metadata
#[derive(Debug, Clone)]
pub struct Route {
    /// URL pattern like "/users/:id"
    pub pattern: String,
    /// File path to the template
    pub template_path: String,
    /// List of parameter names
    pub params: Vec<String>,
    /// Priority for matching (lower = higher priority)
    pub priority: usize,
    /// Whether this is a layout route
    pub is_layout: bool,
    /// Whether this route has a catch-all parameter
    pub has_catch_all: bool,
    /// List of optional parameter names
    pub optional_params: Vec<String>,
    /// Whether this is an error page
    pub is_error_page: bool,
    /// Whether this is a no-layout marker
    pub is_nolayout_marker: bool,
    /// Whether this is a loading UI file (Phase 4.3)
    pub is_loading: bool,
    /// Whether this is a template file (Phase 4.4)
    pub is_template: bool,
    /// Whether this is a not-found page (Phase 4.5)
    pub is_not_found: bool,
    /// Whether this is a parallel route (Phase 5.1)
    pub is_parallel_route: bool,
    /// Slot name for parallel routes, e.g., "analytics" from @analytics (Phase 5.1)
    pub parallel_slot: Option<String>,
    /// Whether this is an intercepting route (Phase 5.2)
    pub is_intercepting: bool,
    /// Interception level for intercepting routes (Phase 5.2)
    pub intercept_level: Option<InterceptLevel>,
    /// Original pattern before interception (Phase 5.2)
    pub intercept_target: Option<String>,
    /// Layout resolution strategy
    pub layout_option: LayoutOption,
    /// Name of this layout (if it's a named layout)
    pub layout_name: Option<String>,
    /// Arbitrary metadata for the route (titles, permissions, cache settings, etc.)
    pub metadata: HashMap<String, String>,
    /// Parameter constraints for validation (param_name → constraint)
    pub param_constraints: HashMap<String, ParameterConstraint>,
    /// Alternative URL patterns that map to this route (for legacy URLs, i18n, etc.)
    pub aliases: Vec<String>,
    /// Optional name for this route (for URL generation and type-safe references)
    pub name: Option<String>,
    /// Whether this is a redirect route
    pub is_redirect: bool,
    /// Target URL for redirect routes
    pub redirect_to: Option<String>,
    /// HTTP status code for redirects (301, 302, 307, 308)
    pub redirect_status: Option<u16>,
}

/// Result of matching a route against a path
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// The matched route
    pub route: Route,
    /// Extracted parameters from the path
    pub params: HashMap<String, String>,
}

impl RouteMatch {
    /// Checks if this match is a redirect route
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::redirect("/old-page", "/new-page", 301));
    ///
    /// let route_match = router.match_route("/old-page").unwrap();
    /// assert!(route_match.is_redirect());
    /// ```
    pub fn is_redirect(&self) -> bool {
        self.route.is_redirect
    }

    /// Gets the redirect target URL with parameters substituted
    ///
    /// Returns None if this is not a redirect route.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::redirect("/blog/:slug", "/articles/:slug", 301));
    ///
    /// let route_match = router.match_route("/blog/hello-world").unwrap();
    /// let target = route_match.redirect_target().unwrap();
    /// assert_eq!(target, "/articles/hello-world");
    /// ```
    pub fn redirect_target(&self) -> Option<String> {
        self.route.redirect_target(&self.params)
    }

    /// Gets the HTTP status code for this redirect
    ///
    /// Returns None if this is not a redirect route.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::redirect("/temp", "/new-location", 302));
    ///
    /// let route_match = router.match_route("/temp").unwrap();
    /// assert_eq!(route_match.redirect_status(), Some(302));
    /// ```
    pub fn redirect_status(&self) -> Option<u16> {
        self.route.redirect_status
    }
}

// Pattern parsing types and functions now in route::pattern module

// ============================================================================
// Route Implementation
// ============================================================================

impl Route {
    /// Creates a route from a file system path
    ///
    /// Converts file paths like `pages/users/[id].rhtml` into route patterns like `/users/:id`
    ///
    /// Detects layout options from file naming conventions:
    /// - `_nolayout` marker file → LayoutOption::None
    /// - `_layout.root.rhtml` → LayoutOption::Root (named "root")
    /// - `_layout.admin.rhtml` → Named layout "admin"
    ///
    /// # Arguments
    ///
    /// * `file_path` - Full path to the template file
    /// * `pages_dir` - Base directory to strip from the path
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/users/[id].rhtml", "pages");
    /// assert_eq!(route.pattern, "/users/:id");
    /// assert_eq!(route.params, vec!["id"]);
    /// ```
    pub fn from_path(file_path: &str, pages_dir: &str) -> Self {
        let relative = file_path
            .strip_prefix(pages_dir)
            .unwrap_or(file_path)
            .trim_start_matches('/');

        let without_ext = relative.strip_suffix(".rhtml").unwrap_or(relative);

        // Extract filename to check for special files
        let filename = without_ext.split('/').last().unwrap_or("");

        // Check if it's a layout file (either _layout or _layout.name)
        let is_layout = filename == "_layout" || filename.starts_with("_layout.");
        let is_error_page = filename == "_error";
        let is_nolayout_marker = filename == "_nolayout";
        let is_loading = filename == "loading"; // Phase 4.3
        let is_template = filename == "_template"; // Phase 4.4
        let is_not_found = filename == "not-found"; // Phase 4.5

        // Phase 5.1: Detect parallel routes (@slot_name)
        let (is_parallel_route, parallel_slot) = Self::detect_parallel_route(without_ext);

        // Phase 5.2: Detect intercepting routes ((.), (..), (...), (....))
        let (is_intercepting, intercept_level, intercept_target) =
            Self::detect_intercepting_route(without_ext);

        // Detect named layouts: _layout.name.rhtml
        let layout_name = if is_layout {
            Self::extract_layout_name(filename)
        } else {
            None
        };

        let (pattern, params, optional_params, dynamic_count, has_catch_all, param_constraints) =
            Self::parse_pattern(without_ext);

        let depth = pattern.matches('/').count();
        let priority =
            Self::calculate_priority(has_catch_all, dynamic_count, depth, &optional_params);

        Route {
            pattern,
            template_path: file_path.to_string(),
            params,
            priority,
            is_layout,
            has_catch_all,
            optional_params,
            is_error_page,
            is_nolayout_marker,
            is_loading,
            is_template,
            is_not_found,
            is_parallel_route,
            parallel_slot,
            is_intercepting,
            intercept_level,
            intercept_target,
            layout_option: LayoutOption::default(),
            layout_name,
            metadata: HashMap::new(),
            param_constraints,
            aliases: Vec::new(),
            name: None,
            is_redirect: false,
            redirect_to: None,
            redirect_status: None,
        }
    }

    /// Extracts layout name from filename using functional pattern matching
    ///
    /// # Examples
    /// - `_layout` → None (default layout)
    /// - `_layout.admin` → Some("admin")
    /// - `_layout.root` → Some("root")
    fn extract_layout_name(filename: &str) -> Option<String> {
        // Match: _layout.name
        filename
            .strip_prefix("_layout.")
            .map(|name| name.to_string())
    }

    /// Detects parallel route slot from path (Phase 5.1)
    ///
    /// # Examples
    /// - `dashboard/@analytics/page` → (true, Some("analytics"))
    /// - `dashboard/users` → (false, None)
    /// - `@team/settings/@nested` → (true, Some("team"))  (first @ slot)
    fn detect_parallel_route(path: &str) -> (bool, Option<String>) {
        // Find first segment starting with @
        path.split('/')
            .find(|seg| seg.starts_with('@') && seg.len() > 1)
            .map(|seg| {
                let slot_name = seg[1..].to_string();
                (true, Some(slot_name))
            })
            .unwrap_or((false, None))
    }

    /// Detects intercepting route level from path (Phase 5.2)
    ///
    /// # Examples
    /// - `feed/(.)photo` → (true, Some(SameLevel), Some("photo"))
    /// - `feed/(..)photo/[id]` → (true, Some(OneLevelUp), Some("photo/[id]"))
    /// - `(...)photo/[id]` → (true, Some(FromRoot), Some("photo/[id]"))
    /// - `normal/path` → (false, None, None)
    fn detect_intercepting_route(path: &str) -> (bool, Option<InterceptLevel>, Option<String>) {
        let segments: Vec<&str> = path.split('/').collect();

        for (idx, seg) in segments.iter().enumerate() {
            let level = match *seg {
                "(.)" => Some(InterceptLevel::SameLevel),
                "(..)" => Some(InterceptLevel::OneLevelUp),
                "(...)" => Some(InterceptLevel::FromRoot),
                "(....)" => Some(InterceptLevel::TwoLevelsUp),
                _ => None,
            };

            if let Some(intercept_level) = level {
                // Capture the remaining path after the intercept marker
                let target = if idx + 1 < segments.len() {
                    Some(segments[idx + 1..].join("/"))
                } else {
                    None
                };
                return (true, Some(intercept_level), target);
            }
        }

        (false, None, None)
    }

    /// Parses a file path pattern into route components
    ///
    /// Returns: (pattern, params, optional_params, dynamic_count, has_catch_all, constraints)
    fn parse_pattern(
        path: &str,
    ) -> (
        String,
        Vec<String>,
        Vec<String>,
        usize,
        bool,
        HashMap<String, ParameterConstraint>,
    ) {
        let mut pattern = String::new();
        let mut params = Vec::new();
        let mut optional_params = Vec::new();
        let mut dynamic_count = 0;
        let mut has_catch_all = false;
        let mut param_constraints = HashMap::new();

        for segment in path.split('/') {
            // Skip empty segments and special directory names
            if segment.is_empty()
                || segment == "_layout"
                || segment.starts_with("_layout.") // Skip named layouts like _layout.admin
                || segment == "_error"
                || segment == "_nolayout" // Skip nolayout markers
                || segment == "loading" // Phase 4.3
                || segment == "_template" // Phase 4.4
                || segment == "not-found" // Phase 4.5
                || segment == "index"
            {
                continue;
            }

            // Phase 5.2: Skip intercepting route markers ((.), (..), (...), (....))
            // These modify matching behavior but aren't part of the pattern
            // Check this BEFORE route groups because they also use parentheses
            if matches!(segment, "(.)" | "(..)" | "(...)" | "(....)") {
                continue;
            }

            // Skip route groups: (folder) - Phase 4.2
            // These organize code but don't affect URL structure
            // Must check AFTER intercepting routes to avoid false positives
            if segment.starts_with('(') && segment.ends_with(')') {
                continue;
            }

            // Phase 5.1: Skip parallel route slots (@slot_name)
            // These are rendered in parallel, not part of URL
            if segment.starts_with('@') {
                continue;
            }

            // Classify the segment and handle accordingly
            match classify_segment(segment) {
                PatternSegmentType::CatchAll(param_name, constraint) => {
                    pattern.push_str("/*");
                    pattern.push_str(&param_name);
                    params.push(param_name.clone());

                    // Store constraint if present
                    if let Some(c) = constraint {
                        param_constraints.insert(param_name, c);
                    }

                    has_catch_all = true;
                    dynamic_count += 100;
                }
                PatternSegmentType::OptionalCatchAll(param_name, constraint) => {
                    // Optional catch-all: [[...slug]] - matches zero or more segments
                    pattern.push_str("/*");
                    pattern.push_str(&param_name);
                    pattern.push('?');
                    params.push(param_name.clone());
                    optional_params.push(param_name.clone());

                    // Store constraint if present
                    if let Some(c) = constraint {
                        param_constraints.insert(param_name, c);
                    }

                    has_catch_all = true;
                    // Lower priority than required catch-all but still high
                    dynamic_count += 99;
                }
                PatternSegmentType::Optional(param_name, constraint) => {
                    pattern.push_str("/:");
                    pattern.push_str(&param_name);
                    pattern.push('?');
                    params.push(param_name.clone());
                    optional_params.push(param_name.clone());

                    // Store constraint if present
                    if let Some(c) = constraint {
                        param_constraints.insert(param_name, c);
                    }

                    dynamic_count += 1;
                }
                PatternSegmentType::Required(param_name, constraint) => {
                    pattern.push_str("/:");
                    pattern.push_str(&param_name);
                    params.push(param_name.clone());

                    // Store constraint if present
                    if let Some(c) = constraint {
                        param_constraints.insert(param_name, c);
                    }

                    dynamic_count += 1;
                }
                PatternSegmentType::Static(seg) => {
                    pattern.push('/');
                    pattern.push_str(&seg);
                }
            }
        }

        if pattern.is_empty() {
            pattern = "/".to_string();
        }

        (
            pattern,
            params,
            optional_params,
            dynamic_count,
            has_catch_all,
            param_constraints,
        )
    }

    /// Calculates route priority for matching order
    ///
    /// Lower number = higher priority (matched first)
    /// Priority order:
    /// 1. Static routes (0)
    /// 2. Dynamic routes (1-999)
    /// 3. Required catch-all (1000+)
    /// 4. Optional catch-all (2000+) - Phase 4.1
    fn calculate_priority(
        has_catch_all: bool,
        dynamic_count: usize,
        depth: usize,
        optional_params: &[String],
    ) -> usize {
        if has_catch_all {
            // Check if catch-all is optional (present in optional_params)
            if optional_params.iter().any(|p| p.len() > 0) {
                // Optional catch-all: lower priority (higher number)
                2000 + depth
            } else {
                // Required catch-all
                1000 + depth
            }
        } else if dynamic_count > 0 {
            let optional_bonus = if optional_params.is_empty() { 1 } else { 0 };
            dynamic_count + depth + optional_bonus
        } else {
            0
        }
    }

    /// Matches this route against a path (case-sensitive)
    pub fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        self.matches_with_options(path, false)
    }

    /// Matches this route against a path with options
    ///
    /// # Arguments
    ///
    /// * `path` - URL path to match
    /// * `case_insensitive` - Whether to perform case-insensitive matching
    pub fn matches_with_options(
        &self,
        path: &str,
        case_insensitive: bool,
    ) -> Option<HashMap<String, String>> {
        // For redirect routes with no parameters, do exact string matching
        // This allows matching trailing slashes for canonical URL redirects
        if self.is_redirect && self.params.is_empty() {
            let matches = if case_insensitive {
                self.pattern.eq_ignore_ascii_case(path)
            } else {
                self.pattern == path
            };
            return if matches {
                Some(HashMap::new())
            } else {
                None
            };
        }

        let pattern_segments: Vec<&str> =
            self.pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut params = HashMap::new();
        let mut pattern_idx = 0;
        let mut path_idx = 0;

        while pattern_idx < pattern_segments.len() {
            let pattern_seg = pattern_segments[pattern_idx];

            match pattern_seg.chars().next() {
                // Catch-all segment: *slug or *slug? (optional)
                Some('*') => {
                    let is_optional = pattern_seg.ends_with('?');
                    let param_name = if is_optional {
                        &pattern_seg[1..pattern_seg.len() - 1]
                    } else {
                        &pattern_seg[1..]
                    };

                    // Check if we have remaining path segments
                    let remaining: Vec<&str> = path_segments[path_idx..].to_vec();

                    // For required catch-all, need at least one segment
                    if remaining.is_empty() && !is_optional {
                        return None;
                    }

                    // For optional catch-all, allow zero segments
                    params.insert(param_name.to_string(), remaining.join("/"));
                    return Some(params);
                }
                // Optional parameter: :id?
                Some(':') if pattern_seg.ends_with('?') => {
                    let param_name = &pattern_seg[1..pattern_seg.len() - 1];

                    if path_idx < path_segments.len() {
                        let should_consume = if pattern_idx + 1 < pattern_segments.len() {
                            let next_pattern = pattern_segments[pattern_idx + 1];
                            match next_pattern.chars().next() {
                                Some(':') | Some('*') => true,
                                _ => {
                                    if case_insensitive {
                                        !next_pattern.eq_ignore_ascii_case(path_segments[path_idx])
                                    } else {
                                        next_pattern != path_segments[path_idx]
                                    }
                                }
                            }
                        } else {
                            true
                        };

                        if should_consume && path_idx < path_segments.len() {
                            params.insert(
                                param_name.to_string(),
                                path_segments[path_idx].to_string(),
                            );
                            path_idx += 1;
                        }
                    }
                    pattern_idx += 1;
                }
                // Required parameter: :id
                Some(':') => {
                    if path_idx >= path_segments.len() {
                        return None;
                    }
                    let param_name = &pattern_seg[1..];
                    params.insert(param_name.to_string(), path_segments[path_idx].to_string());
                    path_idx += 1;
                    pattern_idx += 1;
                }
                // Static segment
                _ => {
                    if path_idx >= path_segments.len() {
                        return None;
                    }

                    let matches = if case_insensitive {
                        pattern_seg.eq_ignore_ascii_case(path_segments[path_idx])
                    } else {
                        pattern_seg == path_segments[path_idx]
                    };

                    if !matches {
                        return None;
                    }

                    path_idx += 1;
                    pattern_idx += 1;
                }
            }
        }

        if path_idx == path_segments.len() {
            // Validate all parameters against constraints (functional validation)
            let all_valid = params
                .iter()
                .all(|(param_name, param_value)| {
                    self.param_constraints
                        .get(param_name)
                        .map(|constraint| constraint.validate(param_value))
                        .unwrap_or(true) // No constraint = always valid
                });

            if all_valid {
                Some(params)
            } else {
                None // Constraint validation failed
            }
        } else {
            None
        }
    }

    /// Returns the parent pattern for layout lookup
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/users/profile.rhtml", "pages");
    /// assert_eq!(route.layout_pattern(), Some("/users".to_string()));
    /// ```
    pub fn layout_pattern(&self) -> Option<String> {
        if let Some(last_slash) = self.pattern.rfind('/') {
            if last_slash == 0 {
                None
            } else {
                Some(self.pattern[..last_slash].to_string())
            }
        } else {
            None
        }
    }

    // ========================================================================
    // Functional Builder Methods
    // ========================================================================
    //
    // These methods follow functional programming principles:
    // - Consume self and return new instance (move semantics)
    // - Composable via method chaining
    // - Immutable transformations
    // - Type-safe configuration

    /// Sets the layout option for this route
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Route, LayoutOption};
    ///
    /// let route = Route::from_path("pages/print.rhtml", "pages")
    ///     .with_layout_option(LayoutOption::None);
    /// ```
    pub fn with_layout_option(mut self, option: LayoutOption) -> Self {
        self.layout_option = option;
        self
    }

    /// Configures route to use no layout (standalone rendering)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/login.rhtml", "pages")
    ///     .with_no_layout();
    /// ```
    pub fn with_no_layout(self) -> Self {
        self.with_layout_option(LayoutOption::None)
    }

    /// Configures route to use root layout only (skip intermediate)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/dashboard/print.rhtml", "pages")
    ///     .with_root_layout();
    /// ```
    pub fn with_root_layout(self) -> Self {
        self.with_layout_option(LayoutOption::Root)
    }

    /// Configures route to use a named layout
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/dashboard/settings.rhtml", "pages")
    ///     .with_named_layout("admin");
    /// ```
    pub fn with_named_layout(self, name: impl Into<String>) -> Self {
        self.with_layout_option(LayoutOption::Named(name.into()))
    }

    /// Configures route to use layout at specific pattern
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/dashboard/admin/users.rhtml", "pages")
    ///     .with_layout_pattern("/dashboard");
    /// ```
    pub fn with_layout_pattern(self, pattern: impl Into<String>) -> Self {
        self.with_layout_option(LayoutOption::Pattern(pattern.into()))
    }

    // ========================================================================
    // Metadata Builder Methods (Phase 2.2)
    // ========================================================================
    //
    // Functional metadata manipulation following builder pattern:
    // - Chainable methods
    // - Type-safe keys via Into<String>
    // - Pure functional transformations

    /// Sets a metadata key-value pair
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/users/[id].rhtml", "pages")
    ///     .with_meta("title", "User Profile")
    ///     .with_meta("permission", "users.read");
    /// ```
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets multiple metadata entries at once (functional batch update)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    /// use std::collections::HashMap;
    ///
    /// let mut meta = HashMap::new();
    /// meta.insert("title".to_string(), "Admin Dashboard".to_string());
    /// meta.insert("permission".to_string(), "admin.read".to_string());
    ///
    /// let route = Route::from_path("pages/admin/dashboard.rhtml", "pages")
    ///     .with_metadata(meta);
    /// ```
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Gets a metadata value by key (functional accessor)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/users/[id].rhtml", "pages")
    ///     .with_meta("title", "User Profile");
    ///
    /// assert_eq!(route.get_meta("title"), Some(&"User Profile".to_string()));
    /// assert_eq!(route.get_meta("missing"), None);
    /// ```
    pub fn get_meta(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Checks if metadata key exists (functional predicate)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/admin/users.rhtml", "pages")
    ///     .with_meta("permission", "admin.read");
    ///
    /// assert!(route.has_meta("permission"));
    /// assert!(!route.has_meta("title"));
    /// ```
    pub fn has_meta(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    // ========================================================================
    // Route Alias Builder Methods (Phase 3.1)
    // ========================================================================
    //
    // Functional methods for managing route aliases:
    // - Immutable transformations via builder pattern
    // - Composable via method chaining
    // - Support for legacy URLs, i18n, URL variations

    /// Adds a single alias to this route
    ///
    /// Aliases allow multiple URL patterns to map to the same route handler.
    /// Useful for legacy URL support, internationalization, and URL variations.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/about.rhtml", "pages")
    ///     .with_alias("/about-us")
    ///     .with_alias("/company");
    ///
    /// assert_eq!(route.aliases, vec!["/about-us", "/company"]);
    /// ```
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Adds multiple aliases at once (functional batch operation)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/about.rhtml", "pages")
    ///     .with_aliases(["/about-us", "/company", "/über"]);
    ///
    /// assert_eq!(route.aliases.len(), 3);
    /// ```
    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.aliases.extend(aliases.into_iter().map(|s| s.into()));
        self
    }

    /// Checks if a path matches this route or any of its aliases (functional predicate)
    ///
    /// Uses functional iteration to check primary pattern and all aliases.
    /// Returns true if any pattern matches (short-circuit evaluation).
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/about.rhtml", "pages")
    ///     .with_aliases(["/about-us", "/company"]);
    ///
    /// assert!(route.matches_any("/about").is_some());
    /// assert!(route.matches_any("/about-us").is_some());
    /// assert!(route.matches_any("/company").is_some());
    /// assert!(route.matches_any("/other").is_none());
    /// ```
    pub fn matches_any(&self, path: &str) -> Option<HashMap<String, String>> {
        // First try the primary pattern
        if let Some(params) = self.matches(path) {
            return Some(params);
        }

        // Then try all aliases (functional iteration with short-circuit)
        self.aliases
            .iter()
            .find_map(|alias_pattern| {
                // For aliases, we need to parse them as if they were routes
                // For now, static aliases only (no parameters in aliases)
                if self.matches_static_alias(path, alias_pattern) {
                    Some(HashMap::new())
                } else {
                    None
                }
            })
    }

    /// Helper to match static alias patterns (functional helper)
    fn matches_static_alias(&self, path: &str, alias: &str) -> bool {
        // Normalize both paths for comparison
        let normalized_path = path.trim_end_matches('/');
        let normalized_alias = alias.trim_end_matches('/');

        if normalized_path.is_empty() && normalized_alias.is_empty() {
            return true;
        }

        normalized_path == normalized_alias
    }

    // ========================================================================
    // Named Route Builder Methods (Phase 3.2)
    // ========================================================================
    //
    // Functional methods for route naming and URL generation:
    // - Type-safe route references
    // - URL generation from parameters
    // - Route refactoring support (change pattern, keep name)

    /// Sets a name for this route (for URL generation and type-safe references)
    ///
    /// Named routes enable URL generation and provide stable references
    /// even when route patterns change.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/users/[id].rhtml", "pages")
    ///     .with_name("user.profile");
    ///
    /// assert_eq!(route.name, Some("user.profile".to_string()));
    /// ```
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Generates a URL for this route by substituting parameters
    ///
    /// Uses functional programming to transform pattern into URL:
    /// - Maps over pattern segments
    /// - Substitutes parameters where found
    /// - Validates all required parameters are provided
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    /// use std::collections::HashMap;
    ///
    /// let route = Route::from_path("pages/users/[id].rhtml", "pages");
    ///
    /// let mut params = HashMap::new();
    /// params.insert("id".to_string(), "123".to_string());
    ///
    /// let url = route.generate_url(&params).unwrap();
    /// assert_eq!(url, "/users/123");
    /// ```
    pub fn generate_url(&self, params: &HashMap<String, String>) -> Option<String> {
        // Split pattern into segments
        let segments: Vec<&str> = self.pattern.split('/').filter(|s| !s.is_empty()).collect();

        // Transform each segment using functional map
        let result_segments: Option<Vec<String>> = segments
            .iter()
            .map(|segment| {
                match segment.chars().next() {
                    // Dynamic parameter: :id or :id?
                    Some(':') => {
                        let param_name = segment
                            .trim_start_matches(':')
                            .trim_end_matches('?');

                        // Optional parameter
                        if segment.ends_with('?') {
                            // Optional - use param if provided, otherwise skip
                            Some(
                                params
                                    .get(param_name)
                                    .map(|v| v.clone())
                                    .unwrap_or_default()
                            )
                        } else {
                            // Required - must be present
                            params.get(param_name).map(|v| v.clone())
                        }
                    }
                    // Catch-all parameter: *slug
                    Some('*') => {
                        let param_name = &segment[1..];
                        params.get(param_name).map(|v| v.clone())
                    }
                    // Static segment
                    _ => Some(segment.to_string()),
                }
            })
            .collect(); // Collect into Option<Vec<String>>

        // If any required parameter was missing, result_segments will be None
        result_segments.map(|segs| {
            let filtered: Vec<String> = segs
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect();

            if filtered.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", filtered.join("/"))
            }
        })
    }

    // ========================================================================
    // Redirect Route Methods (Phase 3.3)
    // ========================================================================
    //
    // Functional methods for creating and managing redirect routes:
    // - Static redirects for legacy URLs
    // - Pattern-based redirects with parameter preservation
    // - HTTP status code specification (301, 302, 307, 308)

    /// Creates a redirect route (functional static constructor)
    ///
    /// Redirects are useful for:
    /// - Legacy URL support (/old-url → /new-url)
    /// - Canonical URL enforcement (/page/ → /page)
    /// - Shortlinks (/blog → /articles)
    ///
    /// # Arguments
    ///
    /// * `from_pattern` - Source URL pattern to match
    /// * `to_url` - Target URL to redirect to
    /// * `status` - HTTP status code (301 = permanent, 302 = temporary, 307/308 = preserve method)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// // Permanent redirect
    /// let route = Route::redirect("/old-page", "/new-page", 301);
    /// assert!(route.is_redirect);
    /// assert_eq!(route.redirect_to, Some("/new-page".to_string()));
    ///
    /// // Temporary redirect
    /// let route = Route::redirect("/maintenance", "/status", 302);
    /// assert_eq!(route.redirect_status, Some(302));
    /// ```
    pub fn redirect(from_pattern: impl Into<String>, to_url: impl Into<String>, status: u16) -> Self {
        let from = from_pattern.into();
        let target = to_url.into();

        // Check if pattern has parameters (using :param or [param] syntax)
        let has_params = from.contains('[') || from.contains(':');

        // For redirects, we support both :param and [param] syntax
        // Convert :param to [param] for parsing
        let normalized_from = if from.contains(':') && !from.contains('[') {
            // Convert :param to [param] syntax for parsing
            let mut result = String::new();
            let segments: Vec<&str> = from.split('/').collect();
            for (i, segment) in segments.iter().enumerate() {
                if i > 0 {
                    result.push('/');
                }
                if segment.starts_with(':') {
                    // Convert :param to [param]
                    result.push('[');
                    result.push_str(&segment[1..]);
                    result.push(']');
                } else {
                    result.push_str(segment);
                }
            }
            result
        } else {
            from.clone()
        };

        let (pattern, params, optional_params, dynamic_count, has_catch_all, param_constraints) = if has_params {
            Self::parse_pattern(&normalized_from)
        } else {
            // Static redirect - use pattern as-is, ensuring it starts with /
            let normalized = if from.starts_with('/') {
                from.clone()
            } else {
                format!("/{}", from)
            };
            (normalized, Vec::new(), Vec::new(), 0, false, HashMap::new())
        };

        let depth = pattern.matches('/').count();
        let priority =
            Self::calculate_priority(has_catch_all, dynamic_count, depth, &optional_params);

        Route {
            pattern,
            template_path: format!("redirect:{}", target),
            params,
            priority,
            is_layout: false,
            has_catch_all,
            optional_params,
            is_error_page: false,
            is_nolayout_marker: false,
            is_loading: false,
            is_template: false,
            is_not_found: false,
            is_parallel_route: false,
            parallel_slot: None,
            is_intercepting: false,
            intercept_level: None,
            intercept_target: None,
            layout_option: LayoutOption::None,
            layout_name: None,
            metadata: HashMap::new(),
            param_constraints,
            aliases: Vec::new(),
            name: None,
            is_redirect: true,
            redirect_to: Some(target),
            redirect_status: Some(status),
        }
    }

    /// Generates the redirect target URL with parameter substitution
    ///
    /// For dynamic redirects, substitutes matched parameters into target URL.
    /// Uses functional map/replace pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    /// use std::collections::HashMap;
    ///
    /// let route = Route::redirect("/blog/:slug", "/articles/:slug", 301);
    ///
    /// let mut params = HashMap::new();
    /// params.insert("slug".to_string(), "hello-world".to_string());
    ///
    /// let target = route.redirect_target(&params).unwrap();
    /// assert_eq!(target, "/articles/hello-world");
    /// ```
    pub fn redirect_target(&self, params: &HashMap<String, String>) -> Option<String> {
        if !self.is_redirect {
            return None;
        }

        let target = self.redirect_to.as_ref()?;

        // If no parameters, return target as-is
        if params.is_empty() {
            return Some(target.clone());
        }

        // Functional parameter substitution
        let mut result = target.clone();
        for (param_name, param_value) in params {
            let placeholder = format!(":{}", param_name);
            result = result.replace(&placeholder, param_value);
        }

        Some(result)
    }
}

// ============================================================================
// Path Utilities - Functional Approach
// ============================================================================

/// Checks if a path is already in valid canonical form
// Path utilities now in path module

// ============================================================================
// Router Implementation
// ============================================================================

/// Main router that manages route collections and performs matching
///
/// The router maintains separate collections using functional principles:
/// - Regular routes for page rendering (Vec for priority ordering)
/// - Layout routes for nested layouts (HashMap for O(1) lookup)
/// - Named layouts for explicit layout selection (HashMap by name)
/// - Named routes for URL generation (HashMap for O(1) lookup)
/// - Error page routes for error handling (HashMap for O(1) lookup)
/// - Loading UI routes for loading states (HashMap for O(1) lookup) - Phase 4.3
/// - Template routes for re-mounting layouts (HashMap for O(1) lookup) - Phase 4.4
/// - Not-found routes for 404 pages (HashMap for O(1) lookup) - Phase 4.5
/// - Parallel routes for simultaneous rendering (nested HashMap: pattern -> slot -> Route) - Phase 5.1
/// - Intercepting routes for modal/overlay patterns (HashMap for O(1) lookup) - Phase 5.2
/// - No-layout markers for directories that should render without layouts
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
    /// Creates a new router with default settings (case-sensitive)
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
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Router;
    ///
    /// let router = Router::with_case_insensitive(true);
    /// ```
    pub fn with_case_insensitive(case_insensitive: bool) -> Self {
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
            case_insensitive,
        }
    }

    /// Configures case sensitivity for route matching
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }

    /// Adds a route to the router
    ///
    /// Routes are automatically sorted by priority after addition.
    /// Layout and error page routes are stored in separate collections.
    /// Named layouts are stored both by pattern and by name for O(1) lookup.
    /// Named routes are stored for URL generation (Phase 3.2).
    ///
    /// # Functional Design
    /// - Uses pattern matching for classification
    /// - Automatic organization into appropriate collections
    /// - Named layouts stored in dual indexes for flexible lookup
    /// - Named routes indexed by name for URL generation
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/about.rhtml", "pages"));
    /// ```
    pub fn add_route(&mut self, route: Route) {
        // Handle nolayout markers first
        if route.is_nolayout_marker {
            self.nolayout_patterns.insert(route.pattern.clone());
            return;
        }

        // Store named routes for URL generation (Phase 3.2)
        if let Some(ref name) = route.name {
            self.named_routes.insert(name.clone(), route.clone());
        }

        // Classify route into appropriate collection
        if route.is_layout {
            // Store in layouts by pattern
            self.layouts.insert(route.pattern.clone(), route.clone());

            // Also store in named_layouts if it has a name
            if let Some(ref name) = route.layout_name {
                self.named_layouts.insert(name.clone(), route);
            }
        } else if route.is_error_page {
            self.error_pages.insert(route.pattern.clone(), route);
        } else if route.is_loading {
            // Phase 4.3: Loading UI pages
            self.loading_pages.insert(route.pattern.clone(), route);
        } else if route.is_template {
            // Phase 4.4: Template pages
            self.templates.insert(route.pattern.clone(), route);
        } else if route.is_not_found {
            // Phase 4.5: Not-found pages
            self.not_found_pages.insert(route.pattern.clone(), route);
        } else if route.is_parallel_route {
            // Phase 5.1: Parallel routes
            // Store by pattern -> slot -> route
            if let Some(ref slot) = route.parallel_slot {
                self.parallel_routes
                    .entry(route.pattern.clone())
                    .or_insert_with(HashMap::new)
                    .insert(slot.clone(), route);
            }
        } else if route.is_intercepting {
            // Phase 5.2: Intercepting routes
            self.intercepting_routes.insert(route.pattern.clone(), route);
        } else {
            // Regular route
            self.routes.push(route);
            self.routes.sort_by_key(|r| r.priority);
        }
    }

    /// Removes a route by its pattern
    ///
    /// Removes the route from all collections (routes, layouts, named_layouts, named_routes,
    /// error_pages, loading_pages, templates, not_found_pages, parallel_routes, intercepting_routes)
    pub fn remove_route(&mut self, pattern: &str) {
        // Remove from routes and also from named_routes if it has a name
        if let Some(pos) = self.routes.iter().position(|r| r.pattern == pattern) {
            let route = &self.routes[pos];
            if let Some(name) = &route.name {
                self.named_routes.remove(name);
            }
            self.routes.remove(pos);
        }

        // Remove from layouts and also from named_layouts if it has a name
        if let Some(layout) = self.layouts.remove(pattern) {
            if let Some(name) = &layout.layout_name {
                self.named_layouts.remove(name);
            }
        }

        self.error_pages.remove(pattern);
        self.loading_pages.remove(pattern);
        self.templates.remove(pattern);
        self.not_found_pages.remove(pattern);
        self.parallel_routes.remove(pattern);
        self.intercepting_routes.remove(pattern);
    }

    /// Manually sorts routes by priority
    ///
    /// Note: Routes are automatically sorted when added via `add_route()`,
    /// so this method is rarely needed unless routes are modified externally.
    pub fn sort_routes(&mut self) {
        self.routes.sort_by_key(|r| r.priority);
    }

    /// Helper function to recursively search for layouts or error pages
    ///
    /// Uses functional programming approach:
    /// 1. Zero-copy normalization with `Cow` (no allocation for valid paths)
    /// 2. Lazy iterator for parent traversal (stops on first match)
    /// 3. Functional composition with `find_map()`
    ///
    /// Handles all user mistakes:
    /// - Trailing slashes, double slashes, backslashes, Windows paths
    ///
    /// # Performance
    /// - Valid path: ~115ns (zero allocations)
    /// - Invalid path: ~310ns (single allocation for normalization)
    fn get_scoped_resource<'a>(
        &'a self,
        pattern: &str,
        map: &'a HashMap<String, Route>,
    ) -> Option<&'a Route> {
        // Normalize path using zero-copy Cow when possible
        let normalized = normalize_path(pattern);

        // Generate parent paths lazily and find first match
        PathHierarchy::new(&normalized).find_map(|path| map.get(path))
    }

    /// Matches a path against all routes and returns the first match
    ///
    /// Routes are checked in priority order (static > optional > dynamic > catch-all).
    /// Also checks route aliases for matching (Phase 3.1).
    ///
    /// Uses functional iteration with short-circuit evaluation - stops at first match.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));
    ///
    /// let route_match = router.match_route("/users/123").unwrap();
    /// assert_eq!(route_match.params.get("id"), Some(&"123".to_string()));
    /// ```
    pub fn match_route(&self, path: &str) -> Option<RouteMatch> {
        // Functional iteration with short-circuit on first match
        self.routes.iter().find_map(|route| {
            // Try primary pattern first
            if let Some(params) = route.matches_with_options(path, self.case_insensitive) {
                return Some(RouteMatch {
                    route: route.clone(),
                    params,
                });
            }

            // Then try aliases (functional iteration)
            route.aliases.iter().find_map(|alias| {
                if route.matches_static_alias(path, alias) {
                    Some(RouteMatch {
                        route: route.clone(),
                        params: HashMap::new(),
                    })
                } else {
                    None
                }
            })
        })
    }

    /// Finds the appropriate layout for a given route pattern
    ///
    /// Uses a functional programming approach for optimal performance:
    /// 1. Zero-copy normalization (no allocation for valid paths)
    /// 2. Lazy parent traversal (stops on first match)
    /// 3. Handles malformed input gracefully
    ///
    /// Walks up the directory hierarchy to find the nearest layout.
    /// For `/dashboard/admin/settings`, checks in order:
    /// 1. `/dashboard/admin/settings`
    /// 2. `/dashboard/admin`
    /// 3. `/dashboard`
    /// 4. `/`
    ///
    /// **Handles user mistakes:**
    /// - Trailing slashes: `/path/` → `/path`
    /// - Double slashes: `/path//to` → `/path/to`
    /// - Backslashes: `\path\to` → `/path/to`
    /// - Windows paths: `\path\to` → `/path/to`
    ///
    /// # Performance
    /// - Valid path: ~115ns (zero allocations)
    /// - Invalid path: ~310ns (single allocation)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
    ///
    /// // Works with any path format
    /// let layout = router.get_layout("/dashboard/settings").unwrap();
    /// assert_eq!(layout.pattern, "/dashboard");
    ///
    /// // Handles malformed paths
    /// let layout = router.get_layout("/dashboard//settings/").unwrap();
    /// assert_eq!(layout.pattern, "/dashboard");
    /// ```
    pub fn get_layout(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.layouts)
    }

    /// Finds layout for a route match, respecting the route's layout option
    ///
    /// Uses functional pattern matching to resolve layouts based on preferences:
    /// - `Inherit` → Walk up hierarchy (default behavior)
    /// - `None` → No layout
    /// - `Root` → Use root layout only
    /// - `Named(name)` → Find layout with matching name
    /// - `Pattern(pat)` → Use layout at specific pattern
    ///
    /// # Functional Design
    /// - Pattern matching for control flow
    /// - Composition of functional helpers
    /// - Short-circuit evaluation
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
    ///
    /// // Use root layout, skip dashboard
    /// let route = Route::from_path("pages/dashboard/print.rhtml", "pages")
    ///     .with_root_layout();
    /// router.add_route(route.clone());
    ///
    /// let route_match = router.match_route("/dashboard/print").unwrap();
    /// let layout = router.get_layout_for_match(&route_match).unwrap();
    /// assert_eq!(layout.pattern, "/");
    /// ```
    pub fn get_layout_for_match(&self, route_match: &RouteMatch) -> Option<&Route> {
        self.get_layout_with_option(&route_match.route.pattern, &route_match.route.layout_option)
    }

    /// Finds layout with specific option (functional core logic)
    ///
    /// Pure function that maps LayoutOption → Option<&Route>
    ///
    /// Uses pattern matching and HashMap lookups for O(1) performance.
    /// Checks nolayout markers when using Inherit option.
    pub fn get_layout_with_option(
        &self,
        pattern: &str,
        option: &LayoutOption,
    ) -> Option<&Route> {
        // Functional pattern matching for layout resolution
        match option {
            // No layout - early return (short-circuit)
            LayoutOption::None => None,

            // Root layout only - direct lookup at "/"
            LayoutOption::Root => self.layouts.get("/"),

            // Named layout - O(1) lookup in named_layouts HashMap
            LayoutOption::Named(name) => self.named_layouts.get(name),

            // Specific pattern - direct lookup with normalization
            LayoutOption::Pattern(pat) => {
                let normalized = normalize_path(pat);
                self.layouts.get(normalized.as_ref())
            }

            // Inherit - check nolayout markers first, then walk up hierarchy
            LayoutOption::Inherit => {
                // Check if this path is under a nolayout marker
                if self.is_under_nolayout_marker(pattern) {
                    return None;
                }
                self.get_scoped_resource(pattern, &self.layouts)
            }
        }
    }

    /// Checks if a path is under a nolayout marker (functional helper)
    ///
    /// Uses functional iteration over hierarchy to find nolayout markers
    fn is_under_nolayout_marker(&self, pattern: &str) -> bool {
        let normalized = normalize_path(pattern);

        // Walk up hierarchy and check if any parent has a nolayout marker
        PathHierarchy::new(&normalized)
            .any(|path| self.nolayout_patterns.contains(path))
    }

    /// Finds layout by name (O(1) HashMap lookup)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));
    ///
    /// let layout = router.get_layout_by_name("admin").unwrap();
    /// assert_eq!(layout.layout_name, Some("admin".to_string()));
    /// ```
    pub fn get_layout_by_name(&self, name: &str) -> Option<&Route> {
        self.named_layouts.get(name)
    }

    /// Returns all registered routes (excluding layouts and error pages)
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Returns all registered layout routes
    pub fn layouts(&self) -> &HashMap<String, Route> {
        &self.layouts
    }

    /// Finds the appropriate error page for a given route pattern
    ///
    /// Works the same as `get_layout()` but for error pages.
    /// Uses functional programming for optimal performance and
    /// handles malformed paths gracefully.
    ///
    /// Walks up the directory hierarchy to find the nearest error page.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_error.rhtml", "pages"));
    /// router.add_route(Route::from_path("pages/api/_error.rhtml", "pages"));
    ///
    /// // Works with clean paths
    /// let error_page = router.get_error_page("/api/users").unwrap();
    /// assert_eq!(error_page.pattern, "/api");
    ///
    /// // Handles malformed paths
    /// let error_page = router.get_error_page("/api//users/").unwrap();
    /// assert_eq!(error_page.pattern, "/api");
    /// ```
    pub fn get_error_page(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.error_pages)
    }

    /// Returns all registered error page routes
    pub fn error_pages(&self) -> &HashMap<String, Route> {
        &self.error_pages
    }

    // ========================================================================
    // Loading UI, Template, and Not-Found Page Accessors (Phase 4.3-4.5)
    // ========================================================================

    /// Gets a loading UI page for a given pattern (with hierarchy)
    ///
    /// Similar to get_error_page, searches up the path hierarchy.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/dashboard/loading.rhtml", "pages"));
    ///
    /// let loading = router.get_loading_page("/dashboard/users").unwrap();
    /// assert_eq!(loading.pattern, "/dashboard");
    /// ```
    pub fn get_loading_page(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.loading_pages)
    }

    /// Returns all registered loading UI routes
    pub fn loading_pages(&self) -> &HashMap<String, Route> {
        &self.loading_pages
    }

    /// Gets a template for a given pattern (with hierarchy)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_template.rhtml", "pages"));
    ///
    /// let template = router.get_template("/about").unwrap();
    /// assert_eq!(template.pattern, "/");
    /// ```
    pub fn get_template(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.templates)
    }

    /// Returns all registered template routes
    pub fn templates(&self) -> &HashMap<String, Route> {
        &self.templates
    }

    /// Gets a not-found page for a given pattern (with hierarchy)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/api/not-found.rhtml", "pages"));
    ///
    /// let not_found = router.get_not_found_page("/api/unknown").unwrap();
    /// assert_eq!(not_found.pattern, "/api");
    /// ```
    pub fn get_not_found_page(&self, pattern: &str) -> Option<&Route> {
        self.get_scoped_resource(pattern, &self.not_found_pages)
    }

    /// Returns all registered not-found routes
    pub fn not_found_pages(&self) -> &HashMap<String, Route> {
        &self.not_found_pages
    }

    // ========================================================================
    // Parallel Routes and Intercepting Routes Accessors (Phase 5.1-5.2)
    // ========================================================================

    /// Gets all parallel routes for a given pattern (Phase 5.1)
    ///
    /// Returns a HashMap of slot names to routes.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/dashboard/@analytics/page.rhtml", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/@team/page.rhtml", "pages"));
    ///
    /// let slots = router.get_parallel_routes("/dashboard").unwrap();
    /// assert_eq!(slots.len(), 2);
    /// assert!(slots.contains_key("analytics"));
    /// assert!(slots.contains_key("team"));
    /// ```
    pub fn get_parallel_routes(&self, pattern: &str) -> Option<&HashMap<String, Route>> {
        self.parallel_routes.get(pattern)
    }

    /// Returns all registered parallel routes (pattern -> slot -> route)
    pub fn parallel_routes(&self) -> &HashMap<String, HashMap<String, Route>> {
        &self.parallel_routes
    }

    /// Gets a specific parallel route by pattern and slot name (Phase 5.1)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/dashboard/@analytics/page.rhtml", "pages"));
    ///
    /// let route = router.get_parallel_route("/dashboard", "analytics").unwrap();
    /// assert_eq!(route.parallel_slot, Some("analytics".to_string()));
    /// ```
    pub fn get_parallel_route(&self, pattern: &str, slot: &str) -> Option<&Route> {
        self.parallel_routes
            .get(pattern)
            .and_then(|slots| slots.get(slot))
    }

    /// Gets an intercepting route for a given pattern (Phase 5.2)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/feed/(.)photo/[id].rhtml", "pages"));
    ///
    /// let route = router.get_intercepting_route("/feed/photo/:id");
    /// // Intercepts /photo/[id] when navigating from /feed
    /// ```
    pub fn get_intercepting_route(&self, pattern: &str) -> Option<&Route> {
        self.intercepting_routes.get(pattern)
    }

    /// Returns all registered intercepting routes
    pub fn intercepting_routes(&self) -> &HashMap<String, Route> {
        &self.intercepting_routes
    }

    // ========================================================================
    // Named Route URL Generation (Phase 3.2)
    // ========================================================================

    /// Generates a URL from a named route and parameters
    ///
    /// Uses functional programming for URL generation:
    /// - O(1) route lookup by name (HashMap)
    /// - Functional parameter substitution
    /// - Type-safe URL generation
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the route
    /// * `params` - Parameter values to substitute into the URL
    ///
    /// # Returns
    ///
    /// `Some(url)` if the route exists and all required parameters are provided,
    /// `None` if the route doesn't exist or required parameters are missing
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    ///
    /// router.add_route(
    ///     Route::from_path("pages/users/[id].rhtml", "pages")
    ///         .with_name("user.profile")
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("id".to_string(), "123".to_string());
    ///
    /// let url = router.url_for("user.profile", &params).unwrap();
    /// assert_eq!(url, "/users/123");
    /// ```
    pub fn url_for(&self, name: &str, params: &HashMap<String, String>) -> Option<String> {
        // O(1) lookup of named route
        self.named_routes
            .get(name)
            .and_then(|route| route.generate_url(params))
    }

    /// Convenience method for generating URLs with an array of parameter tuples
    ///
    /// Functional alternative to manually constructing HashMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    ///
    /// router.add_route(
    ///     Route::from_path("pages/posts/[year]/[slug].rhtml", "pages")
    ///         .with_name("post.show")
    /// );
    ///
    /// let url = router.url_for_params("post.show", &[
    ///     ("year", "2024"),
    ///     ("slug", "hello-world")
    /// ]).unwrap();
    ///
    /// assert_eq!(url, "/posts/2024/hello-world");
    /// ```
    pub fn url_for_params(&self, name: &str, params: &[(&str, &str)]) -> Option<String> {
        // Functional transformation: array of tuples → HashMap
        let param_map: HashMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        self.url_for(name, &param_map)
    }

    /// Gets a route by its name (O(1) HashMap lookup)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    ///
    /// router.add_route(
    ///     Route::from_path("pages/about.rhtml", "pages")
    ///         .with_name("about")
    /// );
    ///
    /// let route = router.get_route_by_name("about").unwrap();
    /// assert_eq!(route.pattern, "/about");
    /// ```
    pub fn get_route_by_name(&self, name: &str) -> Option<&Route> {
        self.named_routes.get(name)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_from_path_static() {
        let route = Route::from_path("pages/about.rhtml", "pages");
        assert_eq!(route.pattern, "/about");
        assert_eq!(route.params.len(), 0);
        assert_eq!(route.priority, 0);
    }

    #[test]
    fn test_route_from_path_dynamic() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages");
        assert_eq!(route.pattern, "/users/:id");
        assert_eq!(route.params, vec!["id"]);
        assert!(route.priority > 0);
    }

    #[test]
    fn test_route_from_path_index() {
        let route = Route::from_path("pages/index.rhtml", "pages");
        assert_eq!(route.pattern, "/");
    }

    #[test]
    fn test_route_from_path_nested_index() {
        let route = Route::from_path("pages/users/index.rhtml", "pages");
        assert_eq!(route.pattern, "/users");
    }

    #[test]
    fn test_route_matches_static() {
        let route = Route::from_path("pages/about.rhtml", "pages");
        assert!(route.matches("/about").is_some());
        assert!(route.matches("/about/").is_some());
        assert!(route.matches("/other").is_none());
    }

    #[test]
    fn test_route_matches_dynamic() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages");
        let params = route.matches("/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_route_priority() {
        let static_route = Route::from_path("pages/users/new.rhtml", "pages");
        let dynamic_route = Route::from_path("pages/users/[id].rhtml", "pages");

        assert!(static_route.priority < dynamic_route.priority);
    }

    #[test]
    fn test_router_matching() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/users/new.rhtml", "pages"));
        router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));

        let m = router.match_route("/users/new").unwrap();
        assert_eq!(m.route.pattern, "/users/new");
        assert_eq!(m.params.len(), 0);

        let m = router.match_route("/users/123").unwrap();
        assert_eq!(m.route.pattern, "/users/:id");
        assert_eq!(m.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_layout_route() {
        let route = Route::from_path("pages/users/_layout.rhtml", "pages");
        assert_eq!(route.pattern, "/users");
        assert!(route.is_layout);
    }

    #[test]
    fn test_catch_all_route() {
        let route = Route::from_path("pages/docs/[...slug].rhtml", "pages");
        assert_eq!(route.pattern, "/docs/*slug");
        assert_eq!(route.params, vec!["slug"]);
        assert!(route.has_catch_all);
        assert!(route.priority > 100);
    }

    #[test]
    fn test_catch_all_matches() {
        let route = Route::from_path("pages/docs/[...slug].rhtml", "pages");

        let params = route.matches("/docs/guide/getting-started").unwrap();
        assert_eq!(
            params.get("slug"),
            Some(&"guide/getting-started".to_string())
        );

        let params = route.matches("/docs/intro").unwrap();
        assert_eq!(params.get("slug"), Some(&"intro".to_string()));

        // Required catch-all does NOT match zero segments (use [[...slug]] for that)
        assert!(route.matches("/docs").is_none());
    }

    #[test]
    fn test_optional_param_route() {
        let route = Route::from_path("pages/posts/[id?].rhtml", "pages");
        assert_eq!(route.pattern, "/posts/:id?");
        assert_eq!(route.params, vec!["id"]);
        assert_eq!(route.optional_params, vec!["id"]);
        assert!(!route.has_catch_all);
    }

    #[test]
    fn test_optional_param_matches() {
        let route = Route::from_path("pages/posts/[id?].rhtml", "pages");

        let params = route.matches("/posts/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));

        let params = route.matches("/posts").unwrap();
        assert_eq!(params.get("id"), None);
    }

    #[test]
    fn test_error_page_route() {
        let route = Route::from_path("pages/_error.rhtml", "pages");
        assert_eq!(route.pattern, "/");
        assert!(route.is_error_page);
        assert!(!route.is_layout);
    }

    #[test]
    fn test_section_error_page() {
        let route = Route::from_path("pages/api/_error.rhtml", "pages");
        assert_eq!(route.pattern, "/api");
        assert!(route.is_error_page);
    }

    #[test]
    fn test_route_priority_ordering() {
        let static_route = Route::from_path("pages/users/new.rhtml", "pages");
        let optional_route = Route::from_path("pages/users/[id?].rhtml", "pages");
        let dynamic_route = Route::from_path("pages/users/[id].rhtml", "pages");
        let catchall_route = Route::from_path("pages/users/[...rest].rhtml", "pages");

        assert!(static_route.priority < optional_route.priority);
        assert!(optional_route.priority < dynamic_route.priority);
        assert!(dynamic_route.priority < catchall_route.priority);
    }

    #[test]
    fn test_router_with_all_route_types() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/docs/[...slug].rhtml", "pages"));
        router.add_route(Route::from_path("pages/docs/api.rhtml", "pages"));
        router.add_route(Route::from_path("pages/posts/[id?].rhtml", "pages"));
        router.add_route(Route::from_path("pages/posts/new.rhtml", "pages"));

        let m = router.match_route("/docs/api").unwrap();
        assert_eq!(m.route.pattern, "/docs/api");

        let m = router.match_route("/docs/guide/intro").unwrap();
        assert_eq!(m.route.pattern, "/docs/*slug");
        assert_eq!(m.params.get("slug"), Some(&"guide/intro".to_string()));

        let m = router.match_route("/posts/new").unwrap();
        assert_eq!(m.route.pattern, "/posts/new");

        let m = router.match_route("/posts/123").unwrap();
        assert_eq!(m.route.pattern, "/posts/:id?");
        assert_eq!(m.params.get("id"), Some(&"123".to_string()));

        let m = router.match_route("/posts").unwrap();
        assert_eq!(m.route.pattern, "/posts/:id?");
        assert_eq!(m.params.get("id"), None);
    }

    #[test]
    fn test_error_page_lookup() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/_error.rhtml", "pages"));
        router.add_route(Route::from_path("pages/api/_error.rhtml", "pages"));

        let error_page = router.get_error_page("/").unwrap();
        assert_eq!(error_page.pattern, "/");

        let error_page = router.get_error_page("/api").unwrap();
        assert_eq!(error_page.pattern, "/api");

        let error_page = router.get_error_page("/users").unwrap();
        assert_eq!(error_page.pattern, "/");
    }

    #[test]
    fn test_nested_layout_three_levels() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/_layout.rhtml",
            "pages",
        ));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/settings.rhtml",
            "pages",
        ));

        let layout = router.get_layout("/dashboard/admin/settings").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");

        let layout = router.get_layout("/dashboard/admin").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");

        let layout = router.get_layout("/dashboard").unwrap();
        assert_eq!(layout.pattern, "/dashboard");

        let layout = router.get_layout("/other").unwrap();
        assert_eq!(layout.pattern, "/");
    }

    #[test]
    fn test_nested_error_page_three_levels() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_error.rhtml", "pages"));
        router.add_route(Route::from_path("pages/api/_error.rhtml", "pages"));
        router.add_route(Route::from_path("pages/api/v1/_error.rhtml", "pages"));
        router.add_route(Route::from_path("pages/api/v1/users.rhtml", "pages"));

        let error = router.get_error_page("/api/v1/users").unwrap();
        assert_eq!(error.pattern, "/api/v1");

        let error = router.get_error_page("/api/v2").unwrap();
        assert_eq!(error.pattern, "/api");

        let error = router.get_error_page("/other").unwrap();
        assert_eq!(error.pattern, "/");
    }

    #[test]
    fn test_case_insensitive_matching() {
        let mut router = Router::with_case_insensitive(true);
        router.add_route(Route::from_path("pages/about.rhtml", "pages"));
        router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));

        assert!(router.match_route("/ABOUT").is_some());
        assert!(router.match_route("/About").is_some());
        assert!(router.match_route("/aBouT").is_some());

        let m = router.match_route("/USERS/123").unwrap();
        assert_eq!(m.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_layout_skips_missing_intermediate() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/users/settings.rhtml",
            "pages",
        ));

        let layout = router
            .get_layout("/dashboard/admin/users/settings")
            .unwrap();
        assert_eq!(layout.pattern, "/dashboard");
    }

    // ========================================================================
    // Path Normalization Tests - All 7 User Mistake Cases
    // ========================================================================

    #[test]
    fn test_layout_with_trailing_slash() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/_layout.rhtml",
            "pages",
        ));

        // Case 1: Single trailing slash
        let layout = router.get_layout("/dashboard/admin/settings/").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");

        // Case 2: Double trailing slash
        let layout = router.get_layout("/dashboard/admin/settings//").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");
    }

    #[test]
    fn test_layout_with_double_slashes() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/_layout.rhtml",
            "pages",
        ));

        // Case 3: Mid-path double slash
        let layout = router.get_layout("/dashboard//admin/settings").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");

        // Case 4: Multiple double slashes
        let layout = router.get_layout("/dashboard//admin//settings/").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");
    }

    #[test]
    fn test_layout_with_backslashes() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/_layout.rhtml",
            "pages",
        ));

        // Case 5: Backslash trailing
        let layout = router.get_layout("/dashboard/admin/settings\\").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");

        // Case 6: Backslash separators
        let layout = router.get_layout("/dashboard\\admin\\settings").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");

        // Case 7: Windows-style path
        let layout = router.get_layout("\\dashboard\\admin\\settings").unwrap();
        assert_eq!(layout.pattern, "/dashboard/admin");
    }

    #[test]
    fn test_layout_edge_cases() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));

        // Empty string → root
        let layout = router.get_layout("").unwrap();
        assert_eq!(layout.pattern, "/");

        // Just slashes → root
        let layout = router.get_layout("///").unwrap();
        assert_eq!(layout.pattern, "/");

        // Mixed separators
        let layout = router.get_layout("/about\\/test//page\\").unwrap();
        assert_eq!(layout.pattern, "/");
    }

    #[test]
    fn test_error_page_with_malformed_paths() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_error.rhtml", "pages"));
        router.add_route(Route::from_path("pages/api/_error.rhtml", "pages"));

        // Trailing slash
        let error = router.get_error_page("/api/users/").unwrap();
        assert_eq!(error.pattern, "/api");

        // Double slashes
        let error = router.get_error_page("/api//users").unwrap();
        assert_eq!(error.pattern, "/api");

        // Backslashes
        let error = router.get_error_page("/api\\users").unwrap();
        assert_eq!(error.pattern, "/api");
    }

    // ========================================================================
    // Path Normalization Unit Tests
    // ========================================================================

    #[test]
    fn test_normalize_path_valid_unchanged() {
        use super::normalize_path;

        // Valid paths should return Cow::Borrowed (zero-copy)
        let result = normalize_path("/dashboard/admin");
        assert_eq!(result, "/dashboard/admin");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_normalize_path_fixes_issues() {
        use super::normalize_path;

        // Trailing slash
        assert_eq!(normalize_path("/dashboard/"), "/dashboard");

        // Double slashes
        assert_eq!(normalize_path("/dashboard//admin"), "/dashboard/admin");

        // Backslashes
        assert_eq!(normalize_path("/dashboard\\admin"), "/dashboard/admin");

        // Windows path
        assert_eq!(normalize_path("\\dashboard\\admin"), "/dashboard/admin");

        // Mixed issues
        assert_eq!(
            normalize_path("/dashboard//admin\\settings/"),
            "/dashboard/admin/settings"
        );
    }

    #[test]
    fn test_path_hierarchy_iterator() {
        use super::PathHierarchy;

        let paths: Vec<&str> = PathHierarchy::new("/a/b/c/d").collect();
        assert_eq!(paths, vec!["/a/b/c/d", "/a/b/c", "/a/b", "/a", "/"]);

        let paths: Vec<&str> = PathHierarchy::new("/a").collect();
        assert_eq!(paths, vec!["/a", "/"]);

        let paths: Vec<&str> = PathHierarchy::new("/").collect();
        assert_eq!(paths, vec!["/"]);
    }

    // ========================================================================
    // Layout Control Tests (Phase 1: Skip Parent Layouts)
    // ========================================================================

    #[test]
    fn test_layout_option_none() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));

        let route = Route::from_path("pages/dashboard/print.rhtml", "pages").with_no_layout();
        router.add_route(route.clone());

        let route_match = router.match_route("/dashboard/print").unwrap();
        let layout = router.get_layout_for_match(&route_match);

        assert!(layout.is_none(), "Should have no layout");
    }

    #[test]
    fn test_layout_option_root() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/_layout.rhtml",
            "pages",
        ));

        let route =
            Route::from_path("pages/dashboard/admin/print.rhtml", "pages").with_root_layout();
        router.add_route(route.clone());

        let route_match = router.match_route("/dashboard/admin/print").unwrap();
        let layout = router.get_layout_for_match(&route_match).unwrap();

        assert_eq!(layout.pattern, "/", "Should use root layout only");
    }

    #[test]
    fn test_layout_option_pattern() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/admin/_layout.rhtml",
            "pages",
        ));

        let route = Route::from_path("pages/dashboard/admin/users/edit.rhtml", "pages")
            .with_layout_pattern("/dashboard");
        router.add_route(route.clone());

        let route_match = router.match_route("/dashboard/admin/users/edit").unwrap();
        let layout = router.get_layout_for_match(&route_match).unwrap();

        assert_eq!(
            layout.pattern, "/dashboard",
            "Should use dashboard layout, skipping admin"
        );
    }

    #[test]
    fn test_named_layout_detection() {
        let route = Route::from_path("pages/_layout.admin.rhtml", "pages");
        assert_eq!(route.layout_name, Some("admin".to_string()));
        assert_eq!(route.pattern, "/");

        let route = Route::from_path("pages/dashboard/_layout.marketing.rhtml", "pages");
        assert_eq!(route.layout_name, Some("marketing".to_string()));
        assert_eq!(route.pattern, "/dashboard");

        let route = Route::from_path("pages/_layout.rhtml", "pages");
        assert_eq!(route.layout_name, None);
    }

    #[test]
    fn test_layout_option_named() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));

        let route = Route::from_path("pages/dashboard/settings.rhtml", "pages")
            .with_named_layout("admin");
        router.add_route(route.clone());

        let route_match = router.match_route("/dashboard/settings").unwrap();
        let layout = router.get_layout_for_match(&route_match).unwrap();

        assert_eq!(layout.layout_name, Some("admin".to_string()));
    }

    #[test]
    fn test_get_layout_by_name() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));
        router.add_route(Route::from_path("pages/_layout.marketing.rhtml", "pages"));

        let admin_layout = router.get_layout_by_name("admin").unwrap();
        assert_eq!(admin_layout.layout_name, Some("admin".to_string()));

        let marketing_layout = router.get_layout_by_name("marketing").unwrap();
        assert_eq!(marketing_layout.layout_name, Some("marketing".to_string()));

        assert!(router.get_layout_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_functional_builder_chaining() {
        let route = Route::from_path("pages/dashboard/print.rhtml", "pages")
            .with_root_layout();

        assert_eq!(route.layout_option, LayoutOption::Root);
        assert_eq!(route.pattern, "/dashboard/print");

        let route = Route::from_path("pages/login.rhtml", "pages").with_no_layout();
        assert_eq!(route.layout_option, LayoutOption::None);

        let route = Route::from_path("pages/admin/users.rhtml", "pages")
            .with_named_layout("admin");
        assert_eq!(
            route.layout_option,
            LayoutOption::Named("admin".to_string())
        );
    }

    #[test]
    fn test_layout_inherit_default() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));

        // Default behavior - should inherit
        let route = Route::from_path("pages/dashboard/settings.rhtml", "pages");
        router.add_route(route.clone());

        let route_match = router.match_route("/dashboard/settings").unwrap();
        let layout = router.get_layout_for_match(&route_match).unwrap();

        assert_eq!(layout.pattern, "/dashboard", "Should inherit from parent");
    }

    #[test]
    fn test_complex_layout_scenario() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));

        // Route 1: Use admin layout
        let route1 =
            Route::from_path("pages/dashboard/users.rhtml", "pages").with_named_layout("admin");
        router.add_route(route1);

        // Route 2: No layout
        let route2 = Route::from_path("pages/dashboard/print.rhtml", "pages").with_no_layout();
        router.add_route(route2);

        // Route 3: Root layout only
        let route3 =
            Route::from_path("pages/dashboard/export.rhtml", "pages").with_root_layout();
        router.add_route(route3);

        // Route 4: Default (inherit)
        let route4 = Route::from_path("pages/dashboard/settings.rhtml", "pages");
        router.add_route(route4);

        // Test each route
        let m1 = router.match_route("/dashboard/users").unwrap();
        let layout1 = router.get_layout_for_match(&m1).unwrap();
        assert_eq!(layout1.layout_name, Some("admin".to_string()));

        let m2 = router.match_route("/dashboard/print").unwrap();
        assert!(router.get_layout_for_match(&m2).is_none());

        let m3 = router.match_route("/dashboard/export").unwrap();
        let layout3 = router.get_layout_for_match(&m3).unwrap();
        assert_eq!(layout3.pattern, "/");

        let m4 = router.match_route("/dashboard/settings").unwrap();
        let layout4 = router.get_layout_for_match(&m4).unwrap();
        assert_eq!(layout4.pattern, "/dashboard");
    }

    #[test]
    fn test_layout_option_with_option() {
        let router = Router::new();

        // Test all layout options directly
        assert!(router
            .get_layout_with_option("/any/path", &LayoutOption::None)
            .is_none());

        // Root requires root layout to exist
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        assert!(router
            .get_layout_with_option("/any/path", &LayoutOption::Root)
            .is_some());

        // Named layout
        router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));
        let layout = router
            .get_layout_with_option("/any", &LayoutOption::Named("admin".to_string()))
            .unwrap();
        assert_eq!(layout.layout_name, Some("admin".to_string()));

        // Pattern
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        let layout = router
            .get_layout_with_option("/anywhere", &LayoutOption::Pattern("/dashboard".to_string()))
            .unwrap();
        assert_eq!(layout.pattern, "/dashboard");
    }

    // ========================================================================
    // No-Layout Marker Tests (Phase 1.2)
    // ========================================================================

    #[test]
    fn test_nolayout_marker_detection() {
        let route = Route::from_path("pages/dashboard/_nolayout.rhtml", "pages");
        assert!(route.is_nolayout_marker);
        assert_eq!(route.pattern, "/dashboard");

        let route = Route::from_path("pages/_nolayout.rhtml", "pages");
        assert!(route.is_nolayout_marker);
        assert_eq!(route.pattern, "/");
    }

    #[test]
    fn test_nolayout_marker_effect() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/print/_nolayout.rhtml", "pages"));
        router.add_route(Route::from_path(
            "pages/dashboard/print/invoice.rhtml",
            "pages",
        ));

        // Route under nolayout marker should have no layout
        let route_match = router.match_route("/dashboard/print/invoice").unwrap();
        let layout = router.get_layout_for_match(&route_match);
        assert!(layout.is_none(), "Routes under _nolayout should have no layout");
    }

    #[test]
    fn test_nolayout_marker_hierarchy() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_nolayout.rhtml", "pages"));

        // Direct child - no layout
        router.add_route(Route::from_path("pages/dashboard/print.rhtml", "pages"));
        let m = router.match_route("/dashboard/print").unwrap();
        assert!(router.get_layout_for_match(&m).is_none());

        // Nested child - also no layout
        router.add_route(Route::from_path(
            "pages/dashboard/reports/monthly.rhtml",
            "pages",
        ));
        let m = router.match_route("/dashboard/reports/monthly").unwrap();
        assert!(router.get_layout_for_match(&m).is_none());

        // Outside the nolayout directory - has layout
        router.add_route(Route::from_path("pages/settings.rhtml", "pages"));
        let m = router.match_route("/settings").unwrap();
        let layout = router.get_layout_for_match(&m).unwrap();
        assert_eq!(layout.pattern, "/");
    }

    #[test]
    fn test_nolayout_marker_vs_explicit_option() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_nolayout.rhtml", "pages"));

        // Explicit layout option should override nolayout marker
        let route = Route::from_path("pages/dashboard/settings.rhtml", "pages")
            .with_named_layout("admin");
        router.add_route(route);

        let m = router.match_route("/dashboard/settings").unwrap();
        let layout = router.get_layout_for_match(&m).unwrap();
        assert_eq!(
            layout.layout_name,
            Some("admin".to_string()),
            "Explicit layout option should override nolayout marker"
        );
    }

    #[test]
    fn test_is_under_nolayout_marker() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/dashboard/_nolayout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/api/v1/_nolayout.rhtml", "pages"));

        assert!(router.is_under_nolayout_marker("/dashboard"));
        assert!(router.is_under_nolayout_marker("/dashboard/print"));
        assert!(router.is_under_nolayout_marker("/dashboard/print/invoice"));
        assert!(router.is_under_nolayout_marker("/api/v1/users"));

        assert!(!router.is_under_nolayout_marker("/"));
        assert!(!router.is_under_nolayout_marker("/settings"));
        assert!(!router.is_under_nolayout_marker("/api"));
        assert!(!router.is_under_nolayout_marker("/api/v2"));
    }

    // ========================================================================
    // Metadata Tests (Phase 2.2)
    // ========================================================================

    #[test]
    fn test_route_with_meta() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages")
            .with_meta("title", "User Profile")
            .with_meta("permission", "users.read")
            .with_meta("cache_ttl", "300");

        assert_eq!(route.get_meta("title"), Some(&"User Profile".to_string()));
        assert_eq!(
            route.get_meta("permission"),
            Some(&"users.read".to_string())
        );
        assert_eq!(route.get_meta("cache_ttl"), Some(&"300".to_string()));
        assert_eq!(route.get_meta("missing"), None);
    }

    #[test]
    fn test_route_with_metadata_batch() {
        let mut meta = HashMap::new();
        meta.insert("title".to_string(), "Admin Dashboard".to_string());
        meta.insert("permission".to_string(), "admin.read".to_string());
        meta.insert("description".to_string(), "Main admin page".to_string());

        let route =
            Route::from_path("pages/admin/dashboard.rhtml", "pages").with_metadata(meta.clone());

        assert_eq!(route.get_meta("title"), Some(&"Admin Dashboard".to_string()));
        assert_eq!(route.get_meta("permission"), Some(&"admin.read".to_string()));
        assert_eq!(
            route.get_meta("description"),
            Some(&"Main admin page".to_string())
        );
    }

    #[test]
    fn test_route_has_meta() {
        let route = Route::from_path("pages/admin/users.rhtml", "pages")
            .with_meta("permission", "admin.read")
            .with_meta("title", "User Management");

        assert!(route.has_meta("permission"));
        assert!(route.has_meta("title"));
        assert!(!route.has_meta("missing"));
        assert!(!route.has_meta("cache_ttl"));
    }

    #[test]
    fn test_metadata_chaining() {
        let route = Route::from_path("pages/products/[id].rhtml", "pages")
            .with_meta("title", "Product Details")
            .with_no_layout()
            .with_meta("permission", "products.read")
            .with_meta("cache_ttl", "600");

        // Check metadata
        assert_eq!(route.get_meta("title"), Some(&"Product Details".to_string()));
        assert_eq!(
            route.get_meta("permission"),
            Some(&"products.read".to_string())
        );
        assert_eq!(route.get_meta("cache_ttl"), Some(&"600".to_string()));

        // Check layout option still works
        assert_eq!(route.layout_option, LayoutOption::None);
    }

    #[test]
    fn test_metadata_in_route_match() {
        let mut router = Router::new();

        let route = Route::from_path("pages/users/[id].rhtml", "pages")
            .with_meta("title", "User Profile")
            .with_meta("permission", "users.read");

        router.add_route(route);

        let route_match = router.match_route("/users/123").unwrap();

        // Metadata should be accessible from matched route
        assert_eq!(
            route_match.route.get_meta("title"),
            Some(&"User Profile".to_string())
        );
        assert_eq!(
            route_match.route.get_meta("permission"),
            Some(&"users.read".to_string())
        );
    }

    #[test]
    fn test_metadata_override() {
        let route = Route::from_path("pages/settings.rhtml", "pages")
            .with_meta("title", "Settings")
            .with_meta("title", "User Settings"); // Override

        assert_eq!(
            route.get_meta("title"),
            Some(&"User Settings".to_string()),
            "Later metadata should override earlier"
        );
    }

    #[test]
    fn test_empty_metadata() {
        let route = Route::from_path("pages/about.rhtml", "pages");

        assert_eq!(route.metadata.len(), 0);
        assert!(!route.has_meta("anything"));
        assert_eq!(route.get_meta("anything"), None);
    }

    // ========================================================================
    // Parameter Constraints Tests (Phase 2.3)
    // ========================================================================

    #[test]
    fn test_constraint_parsing_from_filename() {
        // Integer constraint
        let route = Route::from_path("pages/users/[id:int].rhtml", "pages");
        assert_eq!(route.pattern, "/users/:id");
        assert_eq!(
            route.param_constraints.get("id"),
            Some(&ParameterConstraint::Int)
        );

        // Alpha constraint
        let route = Route::from_path("pages/tags/[name:alpha].rhtml", "pages");
        assert_eq!(
            route.param_constraints.get("name"),
            Some(&ParameterConstraint::Alpha)
        );

        // Slug constraint
        let route = Route::from_path("pages/posts/[slug:slug].rhtml", "pages");
        assert_eq!(
            route.param_constraints.get("slug"),
            Some(&ParameterConstraint::Slug)
        );
    }

    #[test]
    fn test_constraint_int_validation() {
        let route = Route::from_path("pages/users/[id:int].rhtml", "pages");

        // Valid integers
        assert!(route.matches("/users/123").is_some());
        assert!(route.matches("/users/0").is_some());
        assert!(route.matches("/users/-456").is_some());

        // Invalid (non-integers)
        assert!(route.matches("/users/abc").is_none());
        assert!(route.matches("/users/12.34").is_none());
        assert!(route.matches("/users/hello").is_none());
    }

    #[test]
    fn test_constraint_uint_validation() {
        let route = Route::from_path("pages/products/[id:uint].rhtml", "pages");

        // Valid unsigned integers
        assert!(route.matches("/products/123").is_some());
        assert!(route.matches("/products/0").is_some());

        // Invalid (negative or non-integer)
        assert!(route.matches("/products/-456").is_none());
        assert!(route.matches("/products/abc").is_none());
    }

    #[test]
    fn test_constraint_alpha_validation() {
        let route = Route::from_path("pages/tags/[name:alpha].rhtml", "pages");

        // Valid alphabetic
        assert!(route.matches("/tags/hello").is_some());
        assert!(route.matches("/tags/ABC").is_some());

        // Invalid (contains numbers or special chars)
        assert!(route.matches("/tags/hello123").is_none());
        assert!(route.matches("/tags/hello-world").is_none());
    }

    #[test]
    fn test_constraint_alphanum_validation() {
        let route = Route::from_path("pages/codes/[code:alphanum].rhtml", "pages");

        // Valid alphanumeric
        assert!(route.matches("/codes/abc123").is_some());
        assert!(route.matches("/codes/Test99").is_some());

        // Invalid (special characters)
        assert!(route.matches("/codes/hello-world").is_none());
        assert!(route.matches("/codes/test_123").is_none());
    }

    #[test]
    fn test_constraint_slug_validation() {
        let route = Route::from_path("pages/posts/[slug:slug].rhtml", "pages");

        // Valid slugs
        assert!(route.matches("/posts/hello-world").is_some());
        assert!(route.matches("/posts/my_post").is_some());
        assert!(route.matches("/posts/test123").is_some());

        // Invalid (special characters)
        assert!(route.matches("/posts/hello world").is_none());
        assert!(route.matches("/posts/test@123").is_none());
    }

    #[test]
    fn test_constraint_uuid_validation() {
        let route = Route::from_path("pages/items/[id:uuid].rhtml", "pages");

        // Valid UUID
        assert!(route
            .matches("/items/550e8400-e29b-41d4-a716-446655440000")
            .is_some());

        // Invalid UUID
        assert!(route.matches("/items/not-a-uuid").is_none());
        assert!(route.matches("/items/123-456-789").is_none());
    }

    #[test]
    fn test_constraint_optional_parameter() {
        let route = Route::from_path("pages/posts/[id:int?].rhtml", "pages");

        // Valid with integer
        assert!(route.matches("/posts/123").is_some());

        // Valid without parameter
        assert!(route.matches("/posts").is_some());

        // Invalid with non-integer
        assert!(route.matches("/posts/abc").is_none());
    }

    #[test]
    fn test_constraint_catch_all() {
        let route = Route::from_path("pages/docs/[...slug:slug].rhtml", "pages");

        // Valid slugs
        assert!(route.matches("/docs/hello-world").is_some());
        assert!(route.matches("/docs/guide/intro").is_some());

        // Note: This test would fail because catch-all captures everything,
        // including slashes, making it hard to validate individual segments
        // In practice, catch-all constraints are less useful than per-segment ones
    }

    #[test]
    fn test_multiple_constraints() {
        let route = Route::from_path("pages/posts/[year:int]/[slug:slug].rhtml", "pages");

        // Valid: integer year + valid slug
        assert!(route.matches("/posts/2024/hello-world").is_some());

        // Invalid: non-integer year
        assert!(route.matches("/posts/twenty/hello-world").is_none());

        // Invalid: invalid slug
        assert!(route.matches("/posts/2024/hello world").is_none());
    }

    #[test]
    fn test_constraint_with_router_matching() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/users/[id:int].rhtml", "pages"));
        router.add_route(Route::from_path("pages/users/[name:alpha].rhtml", "pages"));

        // Should match integer route
        let m = router.match_route("/users/123");
        assert!(m.is_some());
        assert_eq!(m.unwrap().route.template_path, "pages/users/[id:int].rhtml");

        // Should match alpha route
        let m = router.match_route("/users/john");
        assert!(m.is_some());
        assert_eq!(
            m.unwrap().route.template_path,
            "pages/users/[name:alpha].rhtml"
        );
    }

    #[test]
    fn test_no_constraint_default_behavior() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages");

        // No constraint = accepts anything
        assert!(route.matches("/users/123").is_some());
        assert!(route.matches("/users/abc").is_some());
        assert!(route.matches("/users/anything-goes").is_some());
    }

    #[test]
    fn test_parameter_constraint_from_str() {
        assert_eq!(ParameterConstraint::from_str("int"), ParameterConstraint::Int);
        assert_eq!(
            ParameterConstraint::from_str("integer"),
            ParameterConstraint::Int
        );
        assert_eq!(
            ParameterConstraint::from_str("uint"),
            ParameterConstraint::UInt
        );
        assert_eq!(
            ParameterConstraint::from_str("alpha"),
            ParameterConstraint::Alpha
        );
        assert_eq!(
            ParameterConstraint::from_str("slug"),
            ParameterConstraint::Slug
        );
        assert_eq!(
            ParameterConstraint::from_str("uuid"),
            ParameterConstraint::Uuid
        );
        assert_eq!(
            ParameterConstraint::from_str("unknown"),
            ParameterConstraint::Any
        );
    }

    // ========================================================================
    // Route Alias Tests (Phase 3.1)
    // ========================================================================

    #[test]
    fn test_route_with_single_alias() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_alias("/about-us");

        assert_eq!(route.pattern, "/about");
        assert_eq!(route.aliases, vec!["/about-us"]);
    }

    #[test]
    fn test_route_with_multiple_aliases_chained() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_alias("/about-us")
            .with_alias("/company")
            .with_alias("/über");

        assert_eq!(route.aliases.len(), 3);
        assert_eq!(route.aliases, vec!["/about-us", "/company", "/über"]);
    }

    #[test]
    fn test_route_with_aliases_batch() {
        let route = Route::from_path("pages/contact.rhtml", "pages")
            .with_aliases(["/contact-us", "/get-in-touch", "/reach-us"]);

        assert_eq!(route.aliases.len(), 3);
        assert!(route.aliases.contains(&"/contact-us".to_string()));
        assert!(route.aliases.contains(&"/get-in-touch".to_string()));
        assert!(route.aliases.contains(&"/reach-us".to_string()));
    }

    #[test]
    fn test_route_matches_any_primary() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_aliases(["/about-us", "/company"]);

        // Primary pattern should match
        assert!(route.matches_any("/about").is_some());
    }

    #[test]
    fn test_route_matches_any_alias() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_aliases(["/about-us", "/company"]);

        // Aliases should match
        assert!(route.matches_any("/about-us").is_some());
        assert!(route.matches_any("/company").is_some());
    }

    #[test]
    fn test_route_matches_any_no_match() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_aliases(["/about-us", "/company"]);

        // Non-matching path
        assert!(route.matches_any("/other").is_none());
        assert!(route.matches_any("/contact").is_none());
    }

    #[test]
    fn test_router_match_route_with_alias() {
        let mut router = Router::new();

        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_aliases(["/about-us", "/company"]);
        router.add_route(route);

        // Primary pattern
        let m = router.match_route("/about");
        assert!(m.is_some());
        assert_eq!(m.unwrap().route.pattern, "/about");

        // First alias
        let m = router.match_route("/about-us");
        assert!(m.is_some());
        assert_eq!(m.unwrap().route.pattern, "/about");

        // Second alias
        let m = router.match_route("/company");
        assert!(m.is_some());
        assert_eq!(m.unwrap().route.pattern, "/about");

        // No match
        let m = router.match_route("/other");
        assert!(m.is_none());
    }

    #[test]
    fn test_multiple_routes_with_aliases() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_aliases(["/about-us", "/company"])
        );

        router.add_route(
            Route::from_path("pages/contact.rhtml", "pages")
                .with_aliases(["/contact-us", "/reach-us"])
        );

        // About route
        assert!(router.match_route("/about").is_some());
        assert!(router.match_route("/about-us").is_some());
        assert!(router.match_route("/company").is_some());

        // Contact route
        assert!(router.match_route("/contact").is_some());
        assert!(router.match_route("/contact-us").is_some());
        assert!(router.match_route("/reach-us").is_some());

        // Verify correct route is matched
        let m = router.match_route("/about-us").unwrap();
        assert_eq!(m.route.template_path, "pages/about.rhtml");

        let m = router.match_route("/contact-us").unwrap();
        assert_eq!(m.route.template_path, "pages/contact.rhtml");
    }

    #[test]
    fn test_alias_with_trailing_slash() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_alias("/about-us");

        // Should match with and without trailing slash
        assert!(route.matches_any("/about-us").is_some());
        assert!(route.matches_any("/about-us/").is_some());
    }

    #[test]
    fn test_alias_internationalization() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_aliases(["/über", "/acerca", "/à-propos"]);

        assert!(route.matches_any("/über").is_some());
        assert!(route.matches_any("/acerca").is_some());
        assert!(route.matches_any("/à-propos").is_some());
    }

    #[test]
    fn test_alias_legacy_url_support() {
        let route = Route::from_path("pages/products/index.rhtml", "pages")
            .with_aliases(["/old-products", "/legacy/products", "/shop"]);

        let mut router = Router::new();
        router.add_route(route);

        // New URL
        assert!(router.match_route("/products").is_some());

        // Legacy URLs
        assert!(router.match_route("/old-products").is_some());
        assert!(router.match_route("/legacy/products").is_some());
        assert!(router.match_route("/shop").is_some());
    }

    #[test]
    fn test_alias_with_metadata() {
        let route = Route::from_path("pages/about.rhtml", "pages")
            .with_meta("title", "About Us")
            .with_aliases(["/about-us", "/company"])
            .with_meta("description", "Learn about our company");

        assert_eq!(route.aliases.len(), 2);
        assert_eq!(route.get_meta("title"), Some(&"About Us".to_string()));
        assert_eq!(
            route.get_meta("description"),
            Some(&"Learn about our company".to_string())
        );
    }

    #[test]
    fn test_alias_chaining_with_other_builders() {
        let route = Route::from_path("pages/dashboard/print.rhtml", "pages")
            .with_root_layout()
            .with_alias("/print-dashboard")
            .with_meta("title", "Print View")
            .with_alias("/dashboard-print");

        assert_eq!(route.layout_option, LayoutOption::Root);
        assert_eq!(route.aliases.len(), 2);
        assert_eq!(route.get_meta("title"), Some(&"Print View".to_string()));
    }

    #[test]
    fn test_empty_aliases_default() {
        let route = Route::from_path("pages/about.rhtml", "pages");
        assert_eq!(route.aliases.len(), 0);
        assert!(route.aliases.is_empty());
    }

    #[test]
    fn test_alias_priority_order() {
        let mut router = Router::new();

        // Add static route with aliases
        router.add_route(
            Route::from_path("pages/products/new.rhtml", "pages")
                .with_alias("/create-product")
        );

        // Add dynamic route
        router.add_route(Route::from_path("pages/products/[id].rhtml", "pages"));

        // Static route should match first
        let m = router.match_route("/products/new").unwrap();
        assert_eq!(m.route.template_path, "pages/products/new.rhtml");

        // Alias should also match
        let m = router.match_route("/create-product").unwrap();
        assert_eq!(m.route.template_path, "pages/products/new.rhtml");

        // Dynamic route should match other paths
        let m = router.match_route("/products/123").unwrap();
        assert_eq!(m.route.template_path, "pages/products/[id].rhtml");
    }

    #[test]
    fn test_route_static_alias_matching() {
        let route = Route::from_path("pages/about.rhtml", "pages");

        // Test static alias matching helper
        assert!(route.matches_static_alias("/about", "/about"));
        assert!(route.matches_static_alias("/about/", "/about"));
        assert!(route.matches_static_alias("/about", "/about/"));
        assert!(!route.matches_static_alias("/about", "/other"));
    }

    #[test]
    fn test_alias_case_sensitivity() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_alias("/About-Us")
        );

        // Case-sensitive by default
        assert!(router.match_route("/About-Us").is_some());
        assert!(router.match_route("/about-us").is_none());
    }

    #[test]
    fn test_functional_alias_composition() {
        // Test functional builder pattern composition
        let route = Route::from_path("pages/home.rhtml", "pages")
            .with_aliases(vec!["/index", "/start"])  // Vec
            .with_aliases(["/main", "/home"])        // Array
            .with_alias("/landing");                 // Single

        assert_eq!(route.aliases.len(), 5);
        assert!(route.aliases.contains(&"/index".to_string()));
        assert!(route.aliases.contains(&"/start".to_string()));
        assert!(route.aliases.contains(&"/main".to_string()));
        assert!(route.aliases.contains(&"/home".to_string()));
        assert!(route.aliases.contains(&"/landing".to_string()));
    }

    // ========================================================================
    // Named Route Tests (Phase 3.2)
    // ========================================================================

    #[test]
    fn test_route_with_name() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages")
            .with_name("user.profile");

        assert_eq!(route.name, Some("user.profile".to_string()));
        assert_eq!(route.pattern, "/users/:id");
    }

    #[test]
    fn test_route_without_name() {
        let route = Route::from_path("pages/about.rhtml", "pages");
        assert_eq!(route.name, None);
    }

    #[test]
    fn test_generate_url_static_route() {
        let route = Route::from_path("pages/about.rhtml", "pages");
        let params = HashMap::new();

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/about");
    }

    #[test]
    fn test_generate_url_with_single_parameter() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages");

        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/users/123");
    }

    #[test]
    fn test_generate_url_with_multiple_parameters() {
        let route = Route::from_path("pages/posts/[year]/[slug].rhtml", "pages");

        let mut params = HashMap::new();
        params.insert("year".to_string(), "2024".to_string());
        params.insert("slug".to_string(), "hello-world".to_string());

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/posts/2024/hello-world");
    }

    #[test]
    fn test_generate_url_missing_required_parameter() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages");

        let params = HashMap::new(); // Missing "id"

        let url = route.generate_url(&params);
        assert!(url.is_none(), "Should return None when required parameter is missing");
    }

    #[test]
    fn test_generate_url_optional_parameter_provided() {
        let route = Route::from_path("pages/posts/[id?].rhtml", "pages");

        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/posts/123");
    }

    #[test]
    fn test_generate_url_optional_parameter_missing() {
        let route = Route::from_path("pages/posts/[id?].rhtml", "pages");

        let params = HashMap::new(); // No "id" provided

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/posts");
    }

    #[test]
    fn test_generate_url_catch_all() {
        let route = Route::from_path("pages/docs/[...slug].rhtml", "pages");

        let mut params = HashMap::new();
        params.insert("slug".to_string(), "guide/getting-started".to_string());

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/docs/guide/getting-started");
    }

    #[test]
    fn test_router_url_for() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/users/[id].rhtml", "pages")
                .with_name("user.profile")
        );

        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let url = router.url_for("user.profile", &params).unwrap();
        assert_eq!(url, "/users/123");
    }

    #[test]
    fn test_router_url_for_nonexistent_name() {
        let router = Router::new();

        let params = HashMap::new();
        let url = router.url_for("nonexistent", &params);

        assert!(url.is_none(), "Should return None for non-existent route name");
    }

    #[test]
    fn test_router_url_for_params() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/posts/[year]/[slug].rhtml", "pages")
                .with_name("post.show")
        );

        let url = router.url_for_params("post.show", &[
            ("year", "2024"),
            ("slug", "hello-world")
        ]).unwrap();

        assert_eq!(url, "/posts/2024/hello-world");
    }

    #[test]
    fn test_router_get_route_by_name() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_name("about")
        );

        let route = router.get_route_by_name("about").unwrap();
        assert_eq!(route.pattern, "/about");
        assert_eq!(route.template_path, "pages/about.rhtml");
    }

    #[test]
    fn test_router_get_route_by_name_nonexistent() {
        let router = Router::new();
        assert!(router.get_route_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_named_route_with_metadata() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages")
            .with_name("user.profile")
            .with_meta("title", "User Profile")
            .with_meta("permission", "users.read");

        assert_eq!(route.name, Some("user.profile".to_string()));
        assert_eq!(route.get_meta("title"), Some(&"User Profile".to_string()));
    }

    #[test]
    fn test_named_route_with_alias() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_name("about")
                .with_alias("/about-us")
        );

        // Can generate URL from name
        let url = router.url_for("about", &HashMap::new()).unwrap();
        assert_eq!(url, "/about");

        // Can also match via alias
        assert!(router.match_route("/about-us").is_some());
    }

    #[test]
    fn test_multiple_named_routes() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/index.rhtml", "pages")
                .with_name("home")
        );

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_name("about")
        );

        router.add_route(
            Route::from_path("pages/users/[id].rhtml", "pages")
                .with_name("user.profile")
        );

        // Test each route
        assert_eq!(router.url_for("home", &HashMap::new()).unwrap(), "/");
        assert_eq!(router.url_for("about", &HashMap::new()).unwrap(), "/about");

        let mut params = HashMap::new();
        params.insert("id".to_string(), "456".to_string());
        assert_eq!(router.url_for("user.profile", &params).unwrap(), "/users/456");
    }

    #[test]
    fn test_url_generation_root_route() {
        let route = Route::from_path("pages/index.rhtml", "pages");
        let url = route.generate_url(&HashMap::new()).unwrap();
        assert_eq!(url, "/");
    }

    #[test]
    fn test_url_for_params_empty() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_name("about")
        );

        let url = router.url_for_params("about", &[]).unwrap();
        assert_eq!(url, "/about");
    }

    #[test]
    fn test_named_route_functional_chaining() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages")
            .with_name("user.profile")
            .with_meta("title", "User Profile")
            .with_alias("/profile")
            .with_root_layout();

        assert_eq!(route.name, Some("user.profile".to_string()));
        assert_eq!(route.aliases.len(), 1);
        assert_eq!(route.layout_option, LayoutOption::Root);
        assert_eq!(route.get_meta("title"), Some(&"User Profile".to_string()));
    }

    #[test]
    fn test_url_generation_preserves_order() {
        let route = Route::from_path("pages/events/[year]/[month]/[day].rhtml", "pages");

        let mut params = HashMap::new();
        params.insert("year".to_string(), "2024".to_string());
        params.insert("month".to_string(), "12".to_string());
        params.insert("day".to_string(), "25".to_string());

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/events/2024/12/25");
    }

    #[test]
    fn test_remove_route_removes_from_named_routes() {
        let mut router = Router::new();

        router.add_route(
            Route::from_path("pages/about.rhtml", "pages")
                .with_name("about")
        );

        assert!(router.get_route_by_name("about").is_some());

        router.remove_route("/about");

        assert!(router.get_route_by_name("about").is_none());
    }

    #[test]
    fn test_url_generation_with_extra_params() {
        let route = Route::from_path("pages/users/[id].rhtml", "pages");

        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        params.insert("extra".to_string(), "ignored".to_string());

        let url = route.generate_url(&params).unwrap();
        assert_eq!(url, "/users/123"); // Extra params are ignored
    }

    #[test]
    fn test_named_route_type_safe_reference() {
        let mut router = Router::new();

        // Add route with name
        router.add_route(
            Route::from_path("pages/api/v1/users/[id].rhtml", "pages")
                .with_name("api.v1.users.show")
        );

        // Change pattern (simulating refactoring)
        router.remove_route("/api/v1/users/:id");
        router.add_route(
            Route::from_path("pages/api/v2/users/[id].rhtml", "pages")
                .with_name("api.v1.users.show") // Keep same name
        );

        // Name still works after pattern change
        let mut params = HashMap::new();
        params.insert("id".to_string(), "789".to_string());

        let url = router.url_for("api.v1.users.show", &params).unwrap();
        assert_eq!(url, "/api/v2/users/789");
    }

    // ========================================================================
    // Redirect Route Tests (Phase 3.3)
    // ========================================================================

    #[test]
    fn test_redirect_route_creation() {
        let route = Route::redirect("/old-page", "/new-page", 301);

        assert!(route.is_redirect);
        assert_eq!(route.pattern, "/old-page");
        assert_eq!(route.redirect_to, Some("/new-page".to_string()));
        assert_eq!(route.redirect_status, Some(301));
    }

    #[test]
    fn test_redirect_route_permanent() {
        let route = Route::redirect("/permanent", "/new-location", 301);
        assert_eq!(route.redirect_status, Some(301));
    }

    #[test]
    fn test_redirect_route_temporary() {
        let route = Route::redirect("/temporary", "/new-location", 302);
        assert_eq!(route.redirect_status, Some(302));
    }

    #[test]
    fn test_redirect_route_preserve_method() {
        let route307 = Route::redirect("/preserve-get", "/new-get", 307);
        assert_eq!(route307.redirect_status, Some(307));

        let route308 = Route::redirect("/preserve-post", "/new-post", 308);
        assert_eq!(route308.redirect_status, Some(308));
    }

    #[test]
    fn test_redirect_static_route() {
        let route = Route::redirect("/old-about", "/about", 301);

        let params = HashMap::new();
        let target = route.redirect_target(&params).unwrap();
        assert_eq!(target, "/about");
    }

    #[test]
    fn test_redirect_with_parameters() {
        let route = Route::redirect("/blog/:slug", "/articles/:slug", 301);

        assert_eq!(route.pattern, "/blog/:slug");
        assert_eq!(route.params, vec!["slug"]);

        let mut params = HashMap::new();
        params.insert("slug".to_string(), "hello-world".to_string());

        let target = route.redirect_target(&params).unwrap();
        assert_eq!(target, "/articles/hello-world");
    }

    #[test]
    fn test_redirect_with_multiple_parameters() {
        let route = Route::redirect("/old/:year/:month/:slug", "/archive/:year/:month/:slug", 301);

        let mut params = HashMap::new();
        params.insert("year".to_string(), "2024".to_string());
        params.insert("month".to_string(), "12".to_string());
        params.insert("slug".to_string(), "hello-world".to_string());

        let target = route.redirect_target(&params).unwrap();
        assert_eq!(target, "/archive/2024/12/hello-world");
    }

    #[test]
    fn test_redirect_router_matching() {
        let mut router = Router::new();

        router.add_route(Route::redirect("/old-page", "/new-page", 301));

        let route_match = router.match_route("/old-page").unwrap();

        assert!(route_match.is_redirect());
        assert_eq!(route_match.redirect_target().unwrap(), "/new-page");
        assert_eq!(route_match.redirect_status().unwrap(), 301);
    }

    #[test]
    fn test_redirect_with_dynamic_params_in_router() {
        let mut router = Router::new();

        router.add_route(Route::redirect("/blog/:slug", "/articles/:slug", 301));

        let route_match = router.match_route("/blog/hello-world").unwrap();

        assert!(route_match.is_redirect());
        assert_eq!(route_match.redirect_target().unwrap(), "/articles/hello-world");
        assert_eq!(route_match.redirect_status().unwrap(), 301);
    }

    #[test]
    fn test_redirect_priority_with_static_routes() {
        let mut router = Router::new();

        // Add static route first
        router.add_route(Route::from_path("pages/about.rhtml", "pages"));

        // Add redirect
        router.add_route(Route::redirect("/old-about", "/about", 301));

        // Static route should be accessible
        let m = router.match_route("/about").unwrap();
        assert!(!m.is_redirect());

        // Redirect should work
        let m = router.match_route("/old-about").unwrap();
        assert!(m.is_redirect());
        assert_eq!(m.redirect_target().unwrap(), "/about");
    }

    #[test]
    fn test_redirect_legacy_url_support() {
        let mut router = Router::new();

        // New route
        router.add_route(Route::from_path("pages/products/index.rhtml", "pages"));

        // Legacy redirects
        router.add_route(Route::redirect("/old-products", "/products", 301));
        router.add_route(Route::redirect("/shop", "/products", 301));

        // New URL works
        assert!(!router.match_route("/products").unwrap().is_redirect());

        // Legacy URLs redirect
        assert_eq!(
            router.match_route("/old-products").unwrap().redirect_target().unwrap(),
            "/products"
        );
        assert_eq!(
            router.match_route("/shop").unwrap().redirect_target().unwrap(),
            "/products"
        );
    }

    #[test]
    fn test_redirect_shortlink() {
        let mut router = Router::new();

        router.add_route(Route::redirect("/docs", "/documentation/getting-started", 302));

        let m = router.match_route("/docs").unwrap();
        assert!(m.is_redirect());
        assert_eq!(m.redirect_target().unwrap(), "/documentation/getting-started");
        assert_eq!(m.redirect_status().unwrap(), 302);
    }

    #[test]
    fn test_redirect_canonical_url() {
        let mut router = Router::new();

        // Redirect trailing slash to canonical (add redirect first)
        router.add_route(Route::redirect("/about/", "/about", 301));

        // Canonical route
        router.add_route(Route::from_path("pages/about.rhtml", "pages"));

        let m = router.match_route("/about/").unwrap();
        assert!(m.is_redirect());
        assert_eq!(m.redirect_target().unwrap(), "/about");
    }

    #[test]
    fn test_redirect_target_non_redirect_route() {
        let route = Route::from_path("pages/about.rhtml", "pages");
        let params = HashMap::new();

        assert!(!route.is_redirect);
        assert_eq!(route.redirect_target(&params), None);
    }

    #[test]
    fn test_route_match_redirect_methods_non_redirect() {
        let mut router = Router::new();
        router.add_route(Route::from_path("pages/about.rhtml", "pages"));

        let m = router.match_route("/about").unwrap();

        assert!(!m.is_redirect());
        assert_eq!(m.redirect_target(), None);
        assert_eq!(m.redirect_status(), None);
    }

    #[test]
    fn test_redirect_pattern_matching() {
        let route = Route::redirect("/users/:id/profile", "/profiles/:id", 301);

        // Check pattern parsing
        assert_eq!(route.pattern, "/users/:id/profile");
        assert_eq!(route.params, vec!["id"]);

        // Check redirect target
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let target = route.redirect_target(&params).unwrap();
        assert_eq!(target, "/profiles/123");
    }

    #[test]
    fn test_redirect_with_different_param_names() {
        let route = Route::redirect("/blog/:slug", "/articles/:post", 301);

        let mut params = HashMap::new();
        params.insert("slug".to_string(), "hello-world".to_string());

        // Target has :post but we're passing :slug
        // This should work - it just won't substitute :post
        let target = route.redirect_target(&params).unwrap();
        // :slug doesn't exist in target, so it stays as-is
        // :post doesn't have a value, so it stays as placeholder
        assert_eq!(target, "/articles/:post");
    }

    #[test]
    fn test_redirect_empty_params() {
        let route = Route::redirect("/old", "/new", 301);

        let params = HashMap::new();
        let target = route.redirect_target(&params).unwrap();

        assert_eq!(target, "/new");
    }

    #[test]
    fn test_multiple_redirects_in_router() {
        let mut router = Router::new();

        router.add_route(Route::redirect("/old-1", "/new-1", 301));
        router.add_route(Route::redirect("/old-2", "/new-2", 302));
        router.add_route(Route::redirect("/old-3", "/new-3", 307));

        let m1 = router.match_route("/old-1").unwrap();
        assert_eq!(m1.redirect_target().unwrap(), "/new-1");
        assert_eq!(m1.redirect_status().unwrap(), 301);

        let m2 = router.match_route("/old-2").unwrap();
        assert_eq!(m2.redirect_target().unwrap(), "/new-2");
        assert_eq!(m2.redirect_status().unwrap(), 302);

        let m3 = router.match_route("/old-3").unwrap();
        assert_eq!(m3.redirect_target().unwrap(), "/new-3");
        assert_eq!(m3.redirect_status().unwrap(), 307);
    }

    #[test]
    fn test_redirect_template_path_marker() {
        let route = Route::redirect("/old", "/new", 301);

        // template_path should have "redirect:" prefix for identification
        assert!(route.template_path.starts_with("redirect:"));
        assert_eq!(route.template_path, "redirect:/new");
    }

    #[test]
    fn test_redirect_priority_ordering() {
        let mut router = Router::new();

        // Static redirect has priority 0 (like static routes)
        router.add_route(Route::redirect("/exact-match", "/target-1", 301));

        // Dynamic redirect has higher priority number (lower actual priority)
        router.add_route(Route::redirect("/:slug", "/target-2", 301));

        // Static redirect should match first
        let m = router.match_route("/exact-match").unwrap();
        assert_eq!(m.redirect_target().unwrap(), "/target-1");

        // Dynamic redirect should match others
        let m = router.match_route("/other").unwrap();
        assert_eq!(m.redirect_target().unwrap(), "/target-2");
    }

    #[test]
    fn test_redirect_internationalization() {
        let mut router = Router::new();

        // Main route
        router.add_route(Route::from_path("pages/about.rhtml", "pages"));

        // i18n redirects
        router.add_route(Route::redirect("/über", "/about", 302));
        router.add_route(Route::redirect("/acerca", "/about", 302));

        assert_eq!(router.match_route("/über").unwrap().redirect_target().unwrap(), "/about");
        assert_eq!(router.match_route("/acerca").unwrap().redirect_target().unwrap(), "/about");
    }

    #[test]
    fn test_redirect_chain_not_followed() {
        let mut router = Router::new();

        // Redirect chain: /a → /b → /c
        router.add_route(Route::redirect("/a", "/b", 301));
        router.add_route(Route::redirect("/b", "/c", 301));
        router.add_route(Route::from_path("pages/c.rhtml", "pages"));

        // Router doesn't follow chains - just returns first redirect
        let m = router.match_route("/a").unwrap();
        assert_eq!(m.redirect_target().unwrap(), "/b");

        // User would need to match again to follow chain
        let m = router.match_route("/b").unwrap();
        assert_eq!(m.redirect_target().unwrap(), "/c");
    }

    // ========================================================================
    // Optional Catch-All Tests (Phase 4.1)
    // ========================================================================

    #[test]
    fn test_optional_catch_all_route_creation() {
        let route = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");

        assert_eq!(route.pattern, "/docs/*slug?");
        assert_eq!(route.params, vec!["slug"]);
        assert_eq!(route.optional_params, vec!["slug"]);
        assert!(route.has_catch_all);
    }

    #[test]
    fn test_optional_catch_all_matches_zero_segments() {
        let route = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");

        // Should match /docs (zero segments)
        let params = route.matches("/docs").unwrap();
        assert_eq!(params.get("slug"), Some(&String::new()));
    }

    #[test]
    fn test_optional_catch_all_matches_single_segment() {
        let route = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");

        // Should match /docs/intro
        let params = route.matches("/docs/intro").unwrap();
        assert_eq!(params.get("slug"), Some(&"intro".to_string()));
    }

    #[test]
    fn test_optional_catch_all_matches_multiple_segments() {
        let route = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");

        // Should match /docs/getting-started/installation
        let params = route.matches("/docs/getting-started/installation").unwrap();
        assert_eq!(
            params.get("slug"),
            Some(&"getting-started/installation".to_string())
        );
    }

    #[test]
    fn test_optional_catch_all_with_constraint() {
        let route = Route::from_path("pages/docs/[[...slug:alpha]].rhtml", "pages");

        assert_eq!(route.pattern, "/docs/*slug?");
        assert!(route.param_constraints.contains_key("slug"));
        assert_eq!(
            route.param_constraints.get("slug"),
            Some(&ParameterConstraint::Alpha)
        );
    }

    #[test]
    fn test_optional_catch_all_vs_required_catch_all() {
        let optional = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");
        let required = Route::from_path("pages/blog/[...slug].rhtml", "pages");

        // Optional matches zero segments
        assert!(optional.matches("/docs").is_some());

        // Required does NOT match zero segments
        assert!(required.matches("/blog").is_none());
        assert!(required.matches("/blog/post").is_some());
    }

    #[test]
    fn test_optional_catch_all_priority() {
        let optional = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");
        let required = Route::from_path("pages/blog/[...slug].rhtml", "pages");

        // Debug: print priorities
        eprintln!("Optional priority: {}, params: {:?}, optional: {:?}",
            optional.priority, optional.params, optional.optional_params);
        eprintln!("Required priority: {}, params: {:?}, optional: {:?}",
            required.priority, required.params, required.optional_params);

        // Optional catch-all should have lower priority (higher number) than required
        assert!(optional.priority > required.priority,
            "Optional priority ({}) should be > Required priority ({})",
            optional.priority, required.priority);
    }

    #[test]
    fn test_optional_catch_all_in_router() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/docs/[[...slug]].rhtml", "pages"));

        // Test zero segments
        let m = router.match_route("/docs").unwrap();
        assert_eq!(m.params.get("slug"), Some(&String::new()));
        assert_eq!(m.route.template_path, "pages/docs/[[...slug]].rhtml");

        // Test single segment
        let m = router.match_route("/docs/intro").unwrap();
        assert_eq!(m.params.get("slug"), Some(&"intro".to_string()));

        // Test multiple segments
        let m = router.match_route("/docs/api/reference").unwrap();
        assert_eq!(m.params.get("slug"), Some(&"api/reference".to_string()));
    }

    #[test]
    fn test_optional_catch_all_with_static_prefix() {
        let route = Route::from_path("pages/api/v1/[[...path]].rhtml", "pages");

        assert_eq!(route.pattern, "/api/v1/*path?");

        // Should match base path
        let params = route.matches("/api/v1").unwrap();
        assert_eq!(params.get("path"), Some(&String::new()));

        // Should match with segments
        let params = route.matches("/api/v1/users/123").unwrap();
        assert_eq!(params.get("path"), Some(&"users/123".to_string()));
    }

    #[test]
    fn test_optional_catch_all_route_priority_ordering() {
        let mut router = Router::new();

        // Static route should have highest priority
        router.add_route(Route::from_path("pages/docs/getting-started.rhtml", "pages"));

        // Dynamic route
        router.add_route(Route::from_path("pages/docs/[section].rhtml", "pages"));

        // Optional catch-all has lower priority
        router.add_route(Route::from_path("pages/docs/[[...slug]].rhtml", "pages"));

        // Static should match first
        let m = router.match_route("/docs/getting-started").unwrap();
        assert_eq!(m.route.template_path, "pages/docs/getting-started.rhtml");

        // Dynamic should match next
        let m = router.match_route("/docs/api").unwrap();
        assert_eq!(m.route.template_path, "pages/docs/[section].rhtml");

        // Optional catch-all should match base and deep paths
        let m = router.match_route("/docs").unwrap();
        assert_eq!(m.route.template_path, "pages/docs/[[...slug]].rhtml");

        let m = router.match_route("/docs/guides/advanced/tips").unwrap();
        assert_eq!(m.route.template_path, "pages/docs/[[...slug]].rhtml");
    }

    #[test]
    fn test_optional_catch_all_empty_string_vs_none() {
        let route = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");

        // When matching zero segments, should return empty string not None
        let params = route.matches("/docs").unwrap();
        assert!(params.contains_key("slug"));
        assert_eq!(params.get("slug"), Some(&String::new()));
    }

    #[test]
    fn test_optional_catch_all_segment_classification() {
        // Test the classify_segment function
        let seg = classify_segment("[[...slug]]");
        match seg {
            PatternSegmentType::OptionalCatchAll(name, constraint) => {
                assert_eq!(name, "slug");
                assert_eq!(constraint, None);
            }
            _ => panic!("Expected OptionalCatchAll"),
        }

        let seg = classify_segment("[[...path:alpha]]");
        match seg {
            PatternSegmentType::OptionalCatchAll(name, constraint) => {
                assert_eq!(name, "path");
                assert_eq!(constraint, Some(ParameterConstraint::Alpha));
            }
            _ => panic!("Expected OptionalCatchAll with constraint"),
        }
    }

    #[test]
    fn test_optional_catch_all_vs_optional_param() {
        // [[...slug]] is different from [slug?]
        let catch_all = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");
        let optional_param = Route::from_path("pages/users/[id?].rhtml", "pages");

        // Catch-all captures all remaining segments as one param
        let m = catch_all.matches("/docs/a/b/c").unwrap();
        assert_eq!(m.get("slug"), Some(&"a/b/c".to_string()));

        // Optional param only captures one segment
        let m = optional_param.matches("/users/123").unwrap();
        assert_eq!(m.get("id"), Some(&"123".to_string()));

        // Optional param can skip
        let m = optional_param.matches("/users").unwrap();
        assert_eq!(m.get("id"), None);
    }

    #[test]
    fn test_optional_catch_all_trailing_slash() {
        let route = Route::from_path("pages/docs/[[...slug]].rhtml", "pages");

        // Should match with or without trailing slash
        let m1 = route.matches("/docs").unwrap();
        let m2 = route.matches("/docs/").unwrap();

        assert_eq!(m1.get("slug"), Some(&String::new()));
        assert_eq!(m2.get("slug"), Some(&String::new()));
    }

    #[test]
    fn test_optional_catch_all_real_world_use_case() {
        // Typical Next.js docs pattern
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/docs/[[...slug]].rhtml", "pages"));

        // Index page
        let m = router.match_route("/docs").unwrap();
        assert_eq!(m.params.get("slug"), Some(&String::new()));

        // Category page
        let m = router.match_route("/docs/getting-started").unwrap();
        assert_eq!(m.params.get("slug"), Some(&"getting-started".to_string()));

        // Nested page
        let m = router.match_route("/docs/api/components/button").unwrap();
        assert_eq!(
            m.params.get("slug"),
            Some(&"api/components/button".to_string())
        );
    }

    #[test]
    fn test_optional_catch_all_with_constraints_validation() {
        let route = Route::from_path("pages/files/[[...path:slug]].rhtml", "pages");

        // Constraint should be stored
        assert_eq!(
            route.param_constraints.get("path"),
            Some(&ParameterConstraint::Slug)
        );

        // Valid slug path
        let params = route.matches("/files/my-document").unwrap();
        assert_eq!(params.get("path"), Some(&"my-document".to_string()));

        // Validation happens at application level, router just stores constraints
        assert!(route
            .param_constraints
            .get("path")
            .unwrap()
            .validate("valid-slug"));
        assert!(!route
            .param_constraints
            .get("path")
            .unwrap()
            .validate("invalid slug!"));
    }

    // ========================================================================
    // Route Groups Tests (Phase 4.2)
    // ========================================================================

    #[test]
    fn test_route_group_basic() {
        let route = Route::from_path("pages/(marketing)/about.rhtml", "pages");

        // Route group (marketing) should not appear in the pattern
        assert_eq!(route.pattern, "/about");
        assert_eq!(route.template_path, "pages/(marketing)/about.rhtml");
    }

    #[test]
    fn test_route_group_multiple() {
        let route = Route::from_path("pages/(marketing)/blog/posts.rhtml", "pages");

        // Only (marketing) is skipped, blog is kept
        assert_eq!(route.pattern, "/blog/posts");
        assert_eq!(route.template_path, "pages/(marketing)/blog/posts.rhtml");
    }

    #[test]
    fn test_route_group_nested() {
        let route = Route::from_path("pages/(shop)/(products)/list.rhtml", "pages");

        // Both (shop) and (products) should be skipped
        assert_eq!(route.pattern, "/list");
        assert_eq!(route.template_path, "pages/(shop)/(products)/list.rhtml");
    }

    #[test]
    fn test_route_group_with_dynamic_params() {
        let route = Route::from_path("pages/(shop)/products/[id].rhtml", "pages");

        // (shop) skipped, dynamic param kept
        assert_eq!(route.pattern, "/products/:id");
        assert_eq!(route.params, vec!["id"]);
    }

    #[test]
    fn test_route_group_with_catch_all() {
        let route = Route::from_path("pages/(docs)/[[...slug]].rhtml", "pages");

        // (docs) skipped, optional catch-all at root
        assert_eq!(route.pattern, "/*slug?");
        assert!(route.has_catch_all);
    }

    #[test]
    fn test_route_group_organizational_structure() {
        let mut router = Router::new();

        // Marketing routes
        router.add_route(Route::from_path("pages/(marketing)/about.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(marketing)/blog/index.rhtml", "pages"));

        // Shop routes
        router.add_route(Route::from_path("pages/(shop)/products/index.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(shop)/cart.rhtml", "pages"));

        // All should match without the group names in URL
        assert!(router.match_route("/about").is_some());
        assert!(router.match_route("/blog").is_some());
        assert!(router.match_route("/products").is_some());
        assert!(router.match_route("/cart").is_some());
    }

    #[test]
    fn test_route_group_with_layout() {
        let marketing_layout = Route::from_path("pages/(marketing)/_layout.rhtml", "pages");
        let shop_layout = Route::from_path("pages/(shop)/_layout.rhtml", "pages");

        // Layouts should still be detected
        assert!(marketing_layout.is_layout);
        assert!(shop_layout.is_layout);

        // But route group is removed from pattern
        assert_eq!(marketing_layout.pattern, "/");
        assert_eq!(shop_layout.pattern, "/");
    }

    #[test]
    fn test_route_group_same_path_different_groups() {
        let mut router = Router::new();

        // Two different files with same URL pattern (different groups)
        router.add_route(Route::from_path("pages/(v1)/api/users.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(v2)/api/users.rhtml", "pages"));

        // Both map to /api/users - last one wins (or could be an error)
        let matches: Vec<_> = router.routes().iter()
            .filter(|r| r.pattern == "/api/users")
            .collect();

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_route_group_empty_name() {
        // Edge case: group with no name
        let route = Route::from_path("pages/()/about.rhtml", "pages");

        // Should still skip it
        assert_eq!(route.pattern, "/about");
    }

    #[test]
    fn test_route_group_not_at_start() {
        let route = Route::from_path("pages/admin/(dashboard)/stats.rhtml", "pages");

        // Group in the middle
        assert_eq!(route.pattern, "/admin/stats");
    }

    #[test]
    fn test_route_group_with_special_chars() {
        let route = Route::from_path("pages/(admin-panel)/users.rhtml", "pages");

        // Group names can have hyphens
        assert_eq!(route.pattern, "/users");
        assert_eq!(route.template_path, "pages/(admin-panel)/users.rhtml");
    }

    #[test]
    fn test_route_group_priority_unchanged() {
        let grouped = Route::from_path("pages/(shop)/products.rhtml", "pages");
        let non_grouped = Route::from_path("pages/products.rhtml", "pages");

        // Both should have same priority (both static)
        assert_eq!(grouped.priority, non_grouped.priority);
        assert_eq!(grouped.pattern, non_grouped.pattern);
    }

    #[test]
    fn test_route_group_with_named_layout() {
        let route = Route::from_path("pages/(admin)/_layout.dashboard.rhtml", "pages");

        assert!(route.is_layout);
        assert_eq!(route.layout_name, Some("dashboard".to_string()));
        assert_eq!(route.pattern, "/");
    }

    #[test]
    fn test_route_group_real_world_organization() {
        let mut router = Router::new();

        // Auth group
        router.add_route(Route::from_path("pages/(auth)/login.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(auth)/signup.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(auth)/reset-password.rhtml", "pages"));

        // Dashboard group
        router.add_route(Route::from_path("pages/(dashboard)/home.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(dashboard)/settings.rhtml", "pages"));

        // Public group
        router.add_route(Route::from_path("pages/(public)/about.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(public)/contact.rhtml", "pages"));

        // All accessible via clean URLs
        assert!(router.match_route("/login").is_some());
        assert!(router.match_route("/signup").is_some());
        assert!(router.match_route("/reset-password").is_some());
        assert!(router.match_route("/home").is_some());
        assert!(router.match_route("/settings").is_some());
        assert!(router.match_route("/about").is_some());
        assert!(router.match_route("/contact").is_some());

        // Verify correct templates are matched
        let m = router.match_route("/login").unwrap();
        assert_eq!(m.route.template_path, "pages/(auth)/login.rhtml");

        let m = router.match_route("/home").unwrap();
        assert_eq!(m.route.template_path, "pages/(dashboard)/home.rhtml");
    }

    #[test]
    fn test_route_group_with_route_aliases() {
        let route = Route::from_path("pages/(marketing)/about.rhtml", "pages")
            .with_aliases(["/about-us", "/company"]);

        assert_eq!(route.pattern, "/about");
        assert_eq!(route.aliases.len(), 2);
    }

    #[test]
    fn test_route_group_with_metadata() {
        let route = Route::from_path("pages/(admin)/users.rhtml", "pages")
            .with_meta("permission", "admin.read")
            .with_meta("title", "User Management");

        assert_eq!(route.pattern, "/users");
        assert_eq!(route.get_meta("permission"), Some(&"admin.read".to_string()));
    }

    #[test]
    fn test_route_group_does_not_affect_params() {
        let route = Route::from_path("pages/(api)/users/[id]/posts/[postId].rhtml", "pages");

        assert_eq!(route.pattern, "/users/:id/posts/:postId");
        assert_eq!(route.params, vec!["id", "postId"]);
    }

    #[test]
    fn test_route_group_multiple_levels() {
        let route = Route::from_path("pages/(app)/(dashboard)/(main)/home.rhtml", "pages");

        // All three groups should be skipped
        assert_eq!(route.pattern, "/home");
        assert_eq!(route.template_path, "pages/(app)/(dashboard)/(main)/home.rhtml");
    }

    // ===== Phase 4.3: Loading UI Tests =====

    #[test]
    fn test_loading_ui_detection() {
        let loading = Route::from_path("pages/dashboard/loading.rhtml", "pages");

        assert!(loading.is_loading);
        assert_eq!(loading.pattern, "/dashboard");
        assert_eq!(loading.template_path, "pages/dashboard/loading.rhtml");
    }

    #[test]
    fn test_loading_ui_hierarchical_resolution() {
        let mut router = Router::new();

        // Root loading
        router.add_route(Route::from_path("pages/loading.rhtml", "pages"));

        // Dashboard loading
        router.add_route(Route::from_path("pages/dashboard/loading.rhtml", "pages"));

        // Stats loading
        router.add_route(Route::from_path("pages/dashboard/stats/loading.rhtml", "pages"));

        // Check hierarchical lookup
        assert!(router.get_loading_page("/dashboard/stats").is_some());
        assert!(router.get_loading_page("/dashboard").is_some());
        assert!(router.get_loading_page("/").is_some());
    }

    #[test]
    fn test_loading_pages_collection() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/loading.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/loading.rhtml", "pages"));

        assert_eq!(router.loading_pages().len(), 2);
        assert!(router.loading_pages().contains_key("/"));
        assert!(router.loading_pages().contains_key("/dashboard"));
    }

    // ===== Phase 4.4: Template Files Tests =====

    #[test]
    fn test_template_detection() {
        let template = Route::from_path("pages/dashboard/_template.rhtml", "pages");

        assert!(template.is_template);
        assert_eq!(template.pattern, "/dashboard");
        assert_eq!(template.template_path, "pages/dashboard/_template.rhtml");
    }

    #[test]
    fn test_template_hierarchical_resolution() {
        let mut router = Router::new();

        // Root template
        router.add_route(Route::from_path("pages/_template.rhtml", "pages"));

        // Dashboard template
        router.add_route(Route::from_path("pages/dashboard/_template.rhtml", "pages"));

        // Stats template
        router.add_route(Route::from_path("pages/dashboard/stats/_template.rhtml", "pages"));

        // Check hierarchical lookup
        assert!(router.get_template("/dashboard/stats").is_some());
        assert!(router.get_template("/dashboard").is_some());
        assert!(router.get_template("/").is_some());
    }

    #[test]
    fn test_templates_collection() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/_template.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_template.rhtml", "pages"));

        assert_eq!(router.templates().len(), 2);
        assert!(router.templates().contains_key("/"));
        assert!(router.templates().contains_key("/dashboard"));
    }

    // ===== Phase 4.5: Not-Found Pages Tests =====

    #[test]
    fn test_not_found_detection() {
        let not_found = Route::from_path("pages/dashboard/not-found.rhtml", "pages");

        assert!(not_found.is_not_found);
        assert_eq!(not_found.pattern, "/dashboard");
        assert_eq!(not_found.template_path, "pages/dashboard/not-found.rhtml");
    }

    #[test]
    fn test_not_found_hierarchical_resolution() {
        let mut router = Router::new();

        // Root not-found
        router.add_route(Route::from_path("pages/not-found.rhtml", "pages"));

        // Dashboard not-found
        router.add_route(Route::from_path("pages/dashboard/not-found.rhtml", "pages"));

        // Stats not-found
        router.add_route(Route::from_path("pages/dashboard/stats/not-found.rhtml", "pages"));

        // Check hierarchical lookup
        assert!(router.get_not_found_page("/dashboard/stats").is_some());
        assert!(router.get_not_found_page("/dashboard").is_some());
        assert!(router.get_not_found_page("/").is_some());
    }

    #[test]
    fn test_not_found_pages_collection() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/not-found.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/not-found.rhtml", "pages"));

        assert_eq!(router.not_found_pages().len(), 2);
        assert!(router.not_found_pages().contains_key("/"));
        assert!(router.not_found_pages().contains_key("/dashboard"));
    }

    // ===== Integration Tests: All Special Files Together =====

    #[test]
    fn test_all_special_files_together() {
        let mut router = Router::new();

        // Add all special file types for dashboard section
        router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/loading.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_template.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/not-found.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/_error.rhtml", "pages"));

        // Regular page
        router.add_route(Route::from_path("pages/dashboard/index.rhtml", "pages"));

        // Verify all are accessible
        assert!(router.get_layout("/dashboard").is_some());
        assert!(router.get_loading_page("/dashboard").is_some());
        assert!(router.get_template("/dashboard").is_some());
        assert!(router.get_not_found_page("/dashboard").is_some());
        assert!(router.get_error_page("/dashboard").is_some());
        assert!(router.match_route("/dashboard").is_some());
    }

    #[test]
    fn test_special_files_do_not_create_routes() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/loading.rhtml", "pages"));
        router.add_route(Route::from_path("pages/_template.rhtml", "pages"));
        router.add_route(Route::from_path("pages/not-found.rhtml", "pages"));

        // These should NOT be in regular routes
        assert_eq!(router.routes().len(), 0);

        // But they should be in special collections
        assert_eq!(router.loading_pages().len(), 1);
        assert_eq!(router.templates().len(), 1);
        assert_eq!(router.not_found_pages().len(), 1);
    }

    #[test]
    fn test_special_files_with_route_groups() {
        let mut router = Router::new();

        // Special files inside route groups
        router.add_route(Route::from_path("pages/(app)/loading.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(app)/_template.rhtml", "pages"));
        router.add_route(Route::from_path("pages/(app)/not-found.rhtml", "pages"));

        // Pattern should have route group removed
        assert!(router.get_loading_page("/").is_some());
        assert!(router.get_template("/").is_some());
        assert!(router.get_not_found_page("/").is_some());

        // But template_path should preserve the group
        let loading = router.loading_pages().get("/").unwrap();
        assert_eq!(loading.template_path, "pages/(app)/loading.rhtml");
    }

    // ===== Phase 5.1: Parallel Routes Tests =====

    #[test]
    fn test_parallel_route_detection() {
        let route = Route::from_path("pages/dashboard/@analytics/page.rhtml", "pages");

        assert!(route.is_parallel_route);
        assert_eq!(route.parallel_slot, Some("analytics".to_string()));
        // @analytics is skipped from pattern
        assert_eq!(route.pattern, "/dashboard/page");
        assert_eq!(route.template_path, "pages/dashboard/@analytics/page.rhtml");
    }

    #[test]
    fn test_parallel_route_multiple_slots() {
        let mut router = Router::new();

        // Multiple parallel slots for same pattern
        router.add_route(Route::from_path("pages/dashboard/@analytics/page.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@team/page.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@settings/page.rhtml", "pages"));

        let slots = router.get_parallel_routes("/dashboard/page").unwrap();
        assert_eq!(slots.len(), 3);
        assert!(slots.contains_key("analytics"));
        assert!(slots.contains_key("team"));
        assert!(slots.contains_key("settings"));
    }

    #[test]
    fn test_parallel_route_specific_slot() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/dashboard/@analytics/page.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@team/page.rhtml", "pages"));

        let analytics = router.get_parallel_route("/dashboard/page", "analytics").unwrap();
        assert_eq!(analytics.parallel_slot, Some("analytics".to_string()));
        assert_eq!(analytics.template_path, "pages/dashboard/@analytics/page.rhtml");

        let team = router.get_parallel_route("/dashboard/page", "team").unwrap();
        assert_eq!(team.parallel_slot, Some("team".to_string()));
        assert_eq!(team.template_path, "pages/dashboard/@team/page.rhtml");
    }

    #[test]
    fn test_parallel_route_with_dynamic_params() {
        let route = Route::from_path("pages/products/@reviews/[id].rhtml", "pages");

        assert!(route.is_parallel_route);
        assert_eq!(route.parallel_slot, Some("reviews".to_string()));
        assert_eq!(route.pattern, "/products/:id");
        assert_eq!(route.params, vec!["id"]);
    }

    #[test]
    fn test_parallel_route_with_route_groups() {
        let route = Route::from_path("pages/(shop)/products/@sidebar/list.rhtml", "pages");

        // Both (shop) and @sidebar are skipped
        assert!(route.is_parallel_route);
        assert_eq!(route.parallel_slot, Some("sidebar".to_string()));
        assert_eq!(route.pattern, "/products/list");
        assert_eq!(route.template_path, "pages/(shop)/products/@sidebar/list.rhtml");
    }

    #[test]
    fn test_parallel_route_nested() {
        let route = Route::from_path("pages/app/@modal/profile/@details/page.rhtml", "pages");

        // Only first @ is detected
        assert!(route.is_parallel_route);
        assert_eq!(route.parallel_slot, Some("modal".to_string()));
        // Both @modal and @details are skipped from pattern
        assert_eq!(route.pattern, "/app/profile/page");
    }

    #[test]
    fn test_parallel_route_collection_accessor() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/dash/@a/page.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dash/@b/page.rhtml", "pages"));
        router.add_route(Route::from_path("pages/other/@c/page.rhtml", "pages"));

        assert_eq!(router.parallel_routes().len(), 2);
        assert!(router.parallel_routes().contains_key("/dash/page"));
        assert!(router.parallel_routes().contains_key("/other/page"));
    }

    #[test]
    fn test_parallel_route_with_index() {
        let route = Route::from_path("pages/dashboard/@analytics/index.rhtml", "pages");

        assert!(route.is_parallel_route);
        assert_eq!(route.parallel_slot, Some("analytics".to_string()));
        // index is skipped, @analytics is skipped
        assert_eq!(route.pattern, "/dashboard");
    }

    #[test]
    fn test_parallel_route_real_world_dashboard() {
        let mut router = Router::new();

        // Dashboard with multiple parallel sections
        router.add_route(Route::from_path("pages/dashboard/index.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@analytics/index.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@team/index.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@notifications/index.rhtml", "pages"));

        // Main page is a regular route
        assert!(router.match_route("/dashboard").is_some());

        // Parallel slots are available
        let slots = router.get_parallel_routes("/dashboard").unwrap();
        assert_eq!(slots.len(), 3);
        assert!(slots.contains_key("analytics"));
        assert!(slots.contains_key("team"));
        assert!(slots.contains_key("notifications"));
    }

    // ===== Phase 5.2: Intercepting Routes Tests =====

    #[test]
    fn test_intercepting_route_same_level() {
        let route = Route::from_path("pages/feed/(.)/photo/[id].rhtml", "pages");

        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
        assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
        // (.) is skipped from pattern
        assert_eq!(route.pattern, "/feed/photo/:id");
    }

    #[test]
    fn test_intercepting_route_one_level_up() {
        let route = Route::from_path("pages/feed/(..)/photo/[id].rhtml", "pages");

        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::OneLevelUp));
        assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
        assert_eq!(route.pattern, "/feed/photo/:id");
    }

    #[test]
    fn test_intercepting_route_from_root() {
        let route = Route::from_path("pages/feed/(...)/photo/[id].rhtml", "pages");

        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::FromRoot));
        assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
        assert_eq!(route.pattern, "/feed/photo/:id");
    }

    #[test]
    fn test_intercepting_route_two_levels_up() {
        let route = Route::from_path("pages/feed/(....)/photo/[id].rhtml", "pages");

        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::TwoLevelsUp));
        assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
        assert_eq!(route.pattern, "/feed/photo/:id");
    }

    #[test]
    fn test_intercepting_route_in_router() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rhtml", "pages"));

        let route = router.get_intercepting_route("/feed/photo/:id").unwrap();
        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
    }

    #[test]
    fn test_intercepting_route_with_route_groups() {
        let route = Route::from_path("pages/(app)/feed/(.)/photo/[id].rhtml", "pages");

        // Both (app) and (.) are skipped
        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
        assert_eq!(route.pattern, "/feed/photo/:id");
    }

    #[test]
    fn test_intercepting_route_modal_pattern() {
        let mut router = Router::new();

        // Regular photo page
        router.add_route(Route::from_path("pages/photo/[id].rhtml", "pages"));

        // Intercepting route when coming from feed
        router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rhtml", "pages"));

        // Regular route exists
        assert!(router.match_route("/photo/123").is_some());

        // Intercepting route also exists
        let intercept = router.get_intercepting_route("/feed/photo/:id").unwrap();
        assert_eq!(intercept.intercept_target, Some("photo/[id]".to_string()));
    }

    #[test]
    fn test_intercepting_route_collection_accessor() {
        let mut router = Router::new();

        router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rhtml", "pages"));
        router.add_route(Route::from_path("pages/gallery/(..)/image/[id].rhtml", "pages"));

        assert_eq!(router.intercepting_routes().len(), 2);
        assert!(router.intercepting_routes().contains_key("/feed/photo/:id"));
        assert!(router.intercepting_routes().contains_key("/gallery/image/:id"));
    }

    #[test]
    fn test_intercepting_route_with_catch_all() {
        let route = Route::from_path("pages/app/(...)/docs/[...slug].rhtml", "pages");

        assert!(route.is_intercepting);
        assert_eq!(route.intercept_level, Some(InterceptLevel::FromRoot));
        assert_eq!(route.intercept_target, Some("docs/[...slug]".to_string()));
        assert_eq!(route.pattern, "/app/docs/*slug");
        assert!(route.has_catch_all);
    }

    #[test]
    fn test_intercepting_route_real_world_modal() {
        let mut router = Router::new();

        // Feed page
        router.add_route(Route::from_path("pages/feed/index.rhtml", "pages"));

        // Photo detail page (standalone)
        router.add_route(Route::from_path("pages/photo/[id].rhtml", "pages"));

        // Intercepting route - show photo as modal when navigating from feed
        router.add_route(Route::from_path("pages/feed/(...)/photo/[id].rhtml", "pages"));

        // All routes are accessible
        assert!(router.match_route("/feed").is_some());
        assert!(router.match_route("/photo/123").is_some());

        // Intercepting route is registered
        let intercept = router.get_intercepting_route("/feed/photo/:id").unwrap();
        assert_eq!(intercept.intercept_level, Some(InterceptLevel::FromRoot));
        assert_eq!(intercept.intercept_target, Some("photo/[id]".to_string()));
    }

    // ===== Integration Tests: Parallel + Intercepting Routes =====

    #[test]
    fn test_parallel_and_intercepting_together() {
        let mut router = Router::new();

        // Dashboard with parallel slots
        router.add_route(Route::from_path("pages/dashboard/@analytics/index.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dashboard/@team/index.rhtml", "pages"));

        // Intercepting route from dashboard
        router.add_route(Route::from_path("pages/dashboard/(.)/settings/index.rhtml", "pages"));

        // Both features work together
        let slots = router.get_parallel_routes("/dashboard").unwrap();
        assert_eq!(slots.len(), 2);

        let intercept = router.get_intercepting_route("/dashboard/settings").unwrap();
        assert!(intercept.is_intercepting);
    }

    #[test]
    fn test_parallel_route_in_intercepting_route() {
        let route = Route::from_path("pages/feed/(.)/modal/@content/page.rhtml", "pages");

        // Has both intercepting and parallel route markers
        assert!(route.is_intercepting);
        assert!(route.is_parallel_route);
        assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
        assert_eq!(route.parallel_slot, Some("content".to_string()));
        // Both (.) and @content are skipped
        assert_eq!(route.pattern, "/feed/modal/page");
    }

    #[test]
    fn test_phase_5_with_all_previous_features() {
        let mut router = Router::new();

        // Regular route with dynamic params
        router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));

        // Layout
        router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));

        // Loading UI (Phase 4.3)
        router.add_route(Route::from_path("pages/loading.rhtml", "pages"));

        // Route group (Phase 4.2)
        router.add_route(Route::from_path("pages/(app)/dashboard/index.rhtml", "pages"));

        // Parallel routes (Phase 5.1)
        router.add_route(Route::from_path("pages/dash/@analytics/index.rhtml", "pages"));
        router.add_route(Route::from_path("pages/dash/@team/index.rhtml", "pages"));

        // Intercepting route (Phase 5.2)
        router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rhtml", "pages"));

        // All features work together
        assert!(router.match_route("/users/123").is_some());
        assert!(router.get_layout("/").is_some());
        assert!(router.get_loading_page("/").is_some());
        assert!(router.match_route("/dashboard").is_some());
        assert_eq!(router.get_parallel_routes("/dash").unwrap().len(), 2);
        assert!(router.get_intercepting_route("/feed/photo/:id").is_some());
    }
}
