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
//! - **Immutable builder API** for route configuration
//! - **Tail recursion** for route matching
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
//! router.add_route(Route::from_path("pages/about.rsx", "pages"));
//! router.add_route(Route::from_path("pages/users/[id].rsx", "pages"));
//!
//! let route_match = router.match_route("/users/123").unwrap();
//! assert_eq!(route_match.params.get("id"), Some(&"123".to_string()));
//! ```

use std::collections::HashMap;

// ============================================================================
// Module Declarations
// ============================================================================

mod constraint;
mod intercept;
mod layout;
pub mod path;
pub mod route;

// Re-export public types for backward compatibility
pub use constraint::ParameterConstraint;
pub use intercept::InterceptLevel;
pub use layout::LayoutOption;
pub use path::{is_valid_path, normalize_path, PathHierarchy};
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
    /// Converts file paths like `pages/users/[id]/page.rsx` into route patterns like `/users/:id`
    ///
    /// Detects layout options from file naming conventions:
    /// - `_nolayout` marker file → LayoutOption::None
    /// - `_layout.root.rsx` → LayoutOption::Root (named "root")
    /// - `_layout.admin.rsx` → Named layout "admin"
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
    /// let route = Route::from_path("pages/users/[id].rsx", "pages");
    /// assert_eq!(route.pattern, "/users/:id");
    /// assert_eq!(route.params, vec!["id"]);
    /// ```
    pub fn from_path(file_path: &str, pages_dir: &str) -> Self {
        let relative = file_path
            .strip_prefix(pages_dir)
            .unwrap_or(file_path)
            .trim_start_matches('/');

        // Strip .rsx extension (App Router convention)
        let without_ext = relative.strip_suffix(".rsx").unwrap_or(relative);

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
        let (is_parallel_route, parallel_slot) = route::detect_parallel_route(without_ext);

        // Phase 5.2: Detect intercepting routes ((.), (..), (...), (....))
        let (is_intercepting, intercept_level, intercept_target) =
            route::detect_intercepting_route(without_ext);

        // Detect named layouts: _layout.name.rsx
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

    // Helper functions now in route module (detection.rs and parser.rs)

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
    ///
    /// # Implementation Note
    ///
    /// Uses tail-recursive helper function for functional programming style.
    /// The matching algorithm walks through pattern and path segments simultaneously,
    /// handling catch-all, optional, required parameters, and static segments.
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
            return if matches { Some(HashMap::new()) } else { None };
        }

        let pattern_segments: Vec<&str> =
            self.pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Tail-recursive helper function for functional matching
        fn match_segments(
            pattern_segments: &[&str],
            path_segments: &[&str],
            pattern_idx: usize,
            path_idx: usize,
            params: HashMap<String, String>,
            case_insensitive: bool,
        ) -> Option<HashMap<String, String>> {
            // Base case: consumed all pattern segments
            if pattern_idx >= pattern_segments.len() {
                // Success if we also consumed all path segments
                return if path_idx == path_segments.len() {
                    Some(params)
                } else {
                    None
                };
            }

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
                    let mut new_params = params;
                    new_params.insert(param_name.to_string(), remaining.join("/"));
                    Some(new_params)
                }
                // Optional parameter: :id?
                Some(':') if pattern_seg.ends_with('?') => {
                    let param_name = &pattern_seg[1..pattern_seg.len() - 1];
                    let mut new_params = params;

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
                            new_params.insert(
                                param_name.to_string(),
                                path_segments[path_idx].to_string(),
                            );
                            return match_segments(
                                pattern_segments,
                                path_segments,
                                pattern_idx + 1,
                                path_idx + 1,
                                new_params,
                                case_insensitive,
                            );
                        }
                    }
                    match_segments(
                        pattern_segments,
                        path_segments,
                        pattern_idx + 1,
                        path_idx,
                        new_params,
                        case_insensitive,
                    )
                }
                // Required parameter: :id
                Some(':') => {
                    if path_idx >= path_segments.len() {
                        return None;
                    }
                    let param_name = &pattern_seg[1..];
                    let mut new_params = params;
                    new_params.insert(param_name.to_string(), path_segments[path_idx].to_string());
                    match_segments(
                        pattern_segments,
                        path_segments,
                        pattern_idx + 1,
                        path_idx + 1,
                        new_params,
                        case_insensitive,
                    )
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

                    match_segments(
                        pattern_segments,
                        path_segments,
                        pattern_idx + 1,
                        path_idx + 1,
                        params,
                        case_insensitive,
                    )
                }
            }
        }

        // Start tail-recursive matching
        let params = match_segments(
            &pattern_segments,
            &path_segments,
            0,
            0,
            HashMap::new(),
            case_insensitive,
        )?;

        // Validate all parameters against constraints (functional validation)
        let all_valid = params.iter().all(|(param_name, param_value)| {
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
    }

    /// Returns the parent pattern for layout lookup
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Route;
    ///
    /// let route = Route::from_path("pages/users/profile.rsx", "pages");
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
    /// let route = Route::from_path("pages/print.rsx", "pages")
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
    /// let route = Route::from_path("pages/login.rsx", "pages")
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
    /// let route = Route::from_path("pages/dashboard/print.rsx", "pages")
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
    /// let route = Route::from_path("pages/dashboard/settings.rsx", "pages")
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
    /// let route = Route::from_path("pages/dashboard/admin/users.rsx", "pages")
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
    /// let route = Route::from_path("pages/users/[id].rsx", "pages")
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
    /// let route = Route::from_path("pages/admin/dashboard.rsx", "pages")
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
    /// let route = Route::from_path("pages/users/[id].rsx", "pages")
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
    /// let route = Route::from_path("pages/admin/users.rsx", "pages")
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
    /// let route = Route::from_path("pages/about.rsx", "pages")
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
    /// let route = Route::from_path("pages/about.rsx", "pages")
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
    /// let route = Route::from_path("pages/about.rsx", "pages")
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
        self.aliases.iter().find_map(|alias_pattern| {
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
    pub fn matches_static_alias(&self, path: &str, alias: &str) -> bool {
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
    /// let route = Route::from_path("pages/users/[id].rsx", "pages")
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
    /// let route = Route::from_path("pages/users/[id].rsx", "pages");
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
                        let param_name = segment.trim_start_matches(':').trim_end_matches('?');

                        // Optional parameter
                        if segment.ends_with('?') {
                            // Optional - use param if provided, otherwise skip
                            Some(
                                params
                                    .get(param_name)
                                    .map(|v| v.clone())
                                    .unwrap_or_default(),
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
            let filtered: Vec<String> = segs.into_iter().filter(|s| !s.is_empty()).collect();

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
    pub fn redirect(
        from_pattern: impl Into<String>,
        to_url: impl Into<String>,
        status: u16,
    ) -> Self {
        let from = from_pattern.into();
        let target = to_url.into();

        // Check if pattern has parameters (using :param or [param] syntax)
        let has_params = from.contains('[') || from.contains(':');

        // For redirects, we support both :param and [param] syntax
        // Convert :param to [param] for parsing using functional approach
        let normalized_from = if from.contains(':') && !from.contains('[') {
            // Functional conversion: map segments and join
            from.split('/')
                .map(|segment| {
                    if segment.starts_with(':') {
                        // Convert :param to [param]
                        format!("[{}]", &segment[1..])
                    } else {
                        segment.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("/")
        } else {
            from.clone()
        };

        let (pattern, params, optional_params, dynamic_count, has_catch_all, param_constraints) =
            if has_params {
                route::parse_pattern(&normalized_from)
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
            route::calculate_priority(has_catch_all, dynamic_count, depth, &optional_params);

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

        // Functional parameter substitution using fold
        Some(params.iter().fold(target.clone(), |acc, (param_name, param_value)| {
            let placeholder = format!(":{}", param_name);
            acc.replace(&placeholder, param_value)
        }))
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

    // ========================================================================
    // Functional Builder Methods for Router Configuration
    // ========================================================================

    /// Configures case sensitivity (functional builder)
    ///
    /// Consumes self and returns new Router with updated case sensitivity.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Router;
    ///
    /// let router = Router::new()
    ///     .with_case_sensitivity(false);
    /// ```
    pub fn with_case_sensitivity(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    /// Adds a route to the router (functional builder)
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
    /// - Returns new Router instance (move semantics)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let router = Router::new()
    ///     .with_route(Route::from_path("pages/about.rsx", "pages"))
    ///     .with_route(Route::from_path("pages/users/[id].rsx", "pages"));
    /// ```
    pub fn with_route(mut self, route: Route) -> Self {
        self.add_route_internal(&route);
        self
    }

    /// Adds multiple routes at once (functional batch operation)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let routes = vec![
    ///     Route::from_path("pages/about.rsx", "pages"),
    ///     Route::from_path("pages/users/[id].rsx", "pages"),
    /// ];
    ///
    /// let router = Router::new().with_routes(routes);
    /// ```
    pub fn with_routes<I>(mut self, routes: I) -> Self
    where
        I: IntoIterator<Item = Route>,
    {
        for route in routes {
            self.add_route_internal(&route);
        }
        self
    }

    /// Removes a route by pattern (functional builder)
    ///
    /// Returns new Router instance without the specified route.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let router = Router::new()
    ///     .with_route(Route::from_path("pages/about.rsx", "pages"))
    ///     .without_route("/about");
    /// ```
    pub fn without_route(mut self, pattern: &str) -> Self {
        self.remove_route_internal(pattern);
        self
    }

    /// Sorts routes by priority (functional builder)
    ///
    /// Note: Routes are automatically sorted when added via `with_route()`,
    /// so this method is rarely needed unless routes are modified externally.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Router;
    ///
    /// let router = Router::new().with_sorted_routes();
    /// ```
    pub fn with_sorted_routes(mut self) -> Self {
        self.routes.sort_by_key(|r| r.priority);
        self
    }

    // ========================================================================
    // Deprecated Mutable Methods (for backward compatibility)
    // ========================================================================

    /// Configures case sensitivity for route matching
    ///
    /// # Deprecated
    ///
    /// Use `with_case_sensitivity()` for functional programming style.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Router;
    ///
    /// // Old style (deprecated)
    /// let mut router = Router::new();
    /// router.set_case_insensitive(true);
    ///
    /// // New style (functional)
    /// let router = Router::new()
    ///     .with_case_sensitivity(true);
    /// ```
    #[deprecated(
        since = "0.2.0",
        note = "Use with_case_sensitivity() for functional programming style"
    )]
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
    /// # Deprecated
    ///
    /// Use `with_route()` for functional programming style.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// // Old style (deprecated)
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/about.rsx", "pages"));
    ///
    /// // New style (functional)
    /// let router = Router::new()
    ///     .with_route(Route::from_path("pages/about.rsx", "pages"));
    /// ```
    #[deprecated(since = "0.2.0", note = "Use with_route() for functional programming style")]
    pub fn add_route(&mut self, route: Route) {
        self.add_route_internal(&route);
    }

    /// Internal helper for adding routes (used by both functional and mutable APIs)
    fn add_route_internal(&mut self, route: &Route) {
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
                self.named_layouts.insert(name.clone(), route.clone());
            }
        } else if route.is_error_page {
            self.error_pages.insert(route.pattern.clone(), route.clone());
        } else if route.is_loading {
            // Phase 4.3: Loading UI pages
            self.loading_pages.insert(route.pattern.clone(), route.clone());
        } else if route.is_template {
            // Phase 4.4: Template pages
            self.templates.insert(route.pattern.clone(), route.clone());
        } else if route.is_not_found {
            // Phase 4.5: Not-found pages
            self.not_found_pages.insert(route.pattern.clone(), route.clone());
        } else if route.is_parallel_route {
            // Phase 5.1: Parallel routes
            // Store by pattern -> slot -> route
            if let Some(ref slot) = route.parallel_slot {
                self.parallel_routes
                    .entry(route.pattern.clone())
                    .or_insert_with(HashMap::new)
                    .insert(slot.clone(), route.clone());
            }
        } else if route.is_intercepting {
            // Phase 5.2: Intercepting routes
            self.intercepting_routes
                .insert(route.pattern.clone(), route.clone());
        } else {
            // Regular route
            self.routes.push(route.clone());
            self.routes.sort_by_key(|r| r.priority);
        }
    }

    /// Removes a route by its pattern
    ///
    /// Removes the route from all collections (routes, layouts, named_layouts, named_routes,
    /// error_pages, loading_pages, templates, not_found_pages, parallel_routes, intercepting_routes)
    ///
    /// # Deprecated
    ///
    /// Use `without_route()` for functional programming style.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// // Old style (deprecated)
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/about.rsx", "pages"));
    /// router.remove_route("/about");
    ///
    /// // New style (functional)
    /// let router = Router::new()
    ///     .with_route(Route::from_path("pages/about.rsx", "pages"))
    ///     .without_route("/about");
    /// ```
    #[deprecated(since = "0.2.0", note = "Use without_route() for functional programming style")]
    pub fn remove_route(&mut self, pattern: &str) {
        self.remove_route_internal(pattern);
    }

    /// Internal helper for removing routes (used by both functional and mutable APIs)
    fn remove_route_internal(&mut self, pattern: &str) {
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
    ///
    /// # Deprecated
    ///
    /// Use `with_sorted_routes()` for functional programming style.
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::Router;
    ///
    /// // Old style (deprecated)
    /// let mut router = Router::new();
    /// router.sort_routes();
    ///
    /// // New style (functional)
    /// let router = Router::new().with_sorted_routes();
    /// ```
    #[deprecated(
        since = "0.2.0",
        note = "Use with_sorted_routes() for functional programming style"
    )]
    pub fn sort_routes(&mut self) {
        self.routes.sort_by_key(|r| r.priority);
    }

    // ========================================================================
    // Route Matching and Lookup Methods
    // ========================================================================

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
    /// router.add_route(Route::from_path("pages/users/[id].rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/_layout.rsx", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/_layout.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/_layout.rsx", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/_layout.rsx", "pages"));
    ///
    /// // Use root layout, skip dashboard
    /// let route = Route::from_path("pages/dashboard/print.rsx", "pages")
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
    pub fn get_layout_with_option(&self, pattern: &str, option: &LayoutOption) -> Option<&Route> {
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
    pub fn is_under_nolayout_marker(&self, pattern: &str) -> bool {
        let normalized = normalize_path(pattern);

        // Walk up hierarchy and check if any parent has a nolayout marker
        PathHierarchy::new(&normalized).any(|path| self.nolayout_patterns.contains(path))
    }

    /// Finds layout by name (O(1) HashMap lookup)
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::{Router, Route};
    ///
    /// let mut router = Router::new();
    /// router.add_route(Route::from_path("pages/_layout.admin.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/_error.rsx", "pages"));
    /// router.add_route(Route::from_path("pages/api/_error.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/dashboard/loading.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/_template.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/api/not-found.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/dashboard/@analytics/page.rsx", "pages"));
    /// router.add_route(Route::from_path("pages/dashboard/@team/page.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/dashboard/@analytics/page.rsx", "pages"));
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
    /// router.add_route(Route::from_path("pages/feed/(.)photo/[id].rsx", "pages"));
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
    ///     Route::from_path("pages/users/[id].rsx", "pages")
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
    ///     Route::from_path("pages/posts/[year]/[slug].rsx", "pages")
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
    ///     Route::from_path("pages/about.rsx", "pages")
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
