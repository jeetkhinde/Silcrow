//! Integration tests for rhtmx-router
//!
//! This file contains all router tests extracted from lib.rs.
//! These tests can be safely deleted without affecting the router functionality.
//!
//! Tests are organized by feature area and cover:
//! - Basic routing (static, dynamic, catch-all)
//! - Layouts and error pages
//! - Parameter constraints
//! - Metadata
//! - Aliases
//! - Named routes
//! - Redirects
//! - Route groups
//! - Optional catch-all
//! - Parallel routes
//! - Intercepting routes
//! - Special files (loading, template, not-found)

use rhtmx_router::*;
use std::borrow::Cow;
use std::collections::HashMap;

#[test]
fn test_route_from_path_static() {
    let route = Route::from_path("pages/about.rs", "pages");
    assert_eq!(route.pattern, "/about");
    assert_eq!(route.params.len(), 0);
    assert_eq!(route.priority, 0);
}

#[test]
fn test_route_from_path_dynamic() {
    let route = Route::from_path("pages/users/[id].rs", "pages");
    assert_eq!(route.pattern, "/users/:id");
    assert_eq!(route.params, vec!["id"]);
    assert!(route.priority > 0);
}

#[test]
fn test_route_from_path_index() {
    let route = Route::from_path("pages/page.rs", "pages");
    assert_eq!(route.pattern, "/");
}

#[test]
fn test_route_from_path_nested_index() {
    let route = Route::from_path("pages/users/page.rs", "pages");
    assert_eq!(route.pattern, "/users");
}

#[test]
fn test_route_matches_static() {
    let route = Route::from_path("pages/about.rs", "pages");
    assert!(route.matches("/about").is_some());
    assert!(route.matches("/about/").is_some());
    assert!(route.matches("/other").is_none());
}

#[test]
fn test_route_matches_dynamic() {
    let route = Route::from_path("pages/users/[id].rs", "pages");
    let params = route.matches("/users/123").unwrap();
    assert_eq!(params.get("id"), Some(&"123".to_string()));
}

#[test]
fn test_route_priority() {
    let static_route = Route::from_path("pages/users/new.rs", "pages");
    let dynamic_route = Route::from_path("pages/users/[id].rs", "pages");

    assert!(static_route.priority < dynamic_route.priority);
}

#[test]
fn test_router_matching() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/users/new.rs", "pages"));
    router.add_route(Route::from_path("pages/users/[id].rs", "pages"));

    let m = router.match_route("/users/new").unwrap();
    assert_eq!(m.route.pattern, "/users/new");
    assert_eq!(m.params.len(), 0);

    let m = router.match_route("/users/123").unwrap();
    assert_eq!(m.route.pattern, "/users/:id");
    assert_eq!(m.params.get("id"), Some(&"123".to_string()));
}

#[test]
fn test_layout_route() {
    let route = Route::from_path("pages/users/_layout.rs", "pages");
    assert_eq!(route.pattern, "/users");
    assert!(route.is_layout);
}

#[test]
fn test_catch_all_route() {
    let route = Route::from_path("pages/docs/[...slug].rs", "pages");
    assert_eq!(route.pattern, "/docs/*slug");
    assert_eq!(route.params, vec!["slug"]);
    assert!(route.has_catch_all);
    assert!(route.priority > 100);
}

#[test]
fn test_catch_all_matches() {
    let route = Route::from_path("pages/docs/[...slug].rs", "pages");

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
    let route = Route::from_path("pages/posts/[id?].rs", "pages");
    assert_eq!(route.pattern, "/posts/:id?");
    assert_eq!(route.params, vec!["id"]);
    assert_eq!(route.optional_params, vec!["id"]);
    assert!(!route.has_catch_all);
}

#[test]
fn test_optional_param_matches() {
    let route = Route::from_path("pages/posts/[id?].rs", "pages");

    let params = route.matches("/posts/123").unwrap();
    assert_eq!(params.get("id"), Some(&"123".to_string()));

    let params = route.matches("/posts").unwrap();
    assert_eq!(params.get("id"), None);
}

#[test]
fn test_error_page_route() {
    let route = Route::from_path("pages/_error.rs", "pages");
    assert_eq!(route.pattern, "/");
    assert!(route.is_error_page);
    assert!(!route.is_layout);
}

#[test]
fn test_section_error_page() {
    let route = Route::from_path("pages/api/_error.rs", "pages");
    assert_eq!(route.pattern, "/api");
    assert!(route.is_error_page);
}

#[test]
fn test_route_priority_ordering() {
    let static_route = Route::from_path("pages/users/new.rs", "pages");
    let optional_route = Route::from_path("pages/users/[id?].rs", "pages");
    let dynamic_route = Route::from_path("pages/users/[id].rs", "pages");
    let catchall_route = Route::from_path("pages/users/[...rest].rs", "pages");

    assert!(static_route.priority < optional_route.priority);
    assert!(optional_route.priority < dynamic_route.priority);
    assert!(dynamic_route.priority < catchall_route.priority);
}

#[test]
fn test_router_with_all_route_types() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/docs/[...slug].rs", "pages"));
    router.add_route(Route::from_path("pages/docs/api.rs", "pages"));
    router.add_route(Route::from_path("pages/posts/[id?].rs", "pages"));
    router.add_route(Route::from_path("pages/posts/new.rs", "pages"));

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

    router.add_route(Route::from_path("pages/_error.rs", "pages"));
    router.add_route(Route::from_path("pages/api/_error.rs", "pages"));

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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/_layout.rs",
        "pages",
    ));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/settings.rs",
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
    router.add_route(Route::from_path("pages/_error.rs", "pages"));
    router.add_route(Route::from_path("pages/api/_error.rs", "pages"));
    router.add_route(Route::from_path("pages/api/v1/_error.rs", "pages"));
    router.add_route(Route::from_path("pages/api/v1/users.rs", "pages"));

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
    router.add_route(Route::from_path("pages/about.rs", "pages"));
    router.add_route(Route::from_path("pages/users/[id].rs", "pages"));

    assert!(router.match_route("/ABOUT").is_some());
    assert!(router.match_route("/About").is_some());
    assert!(router.match_route("/aBouT").is_some());

    let m = router.match_route("/USERS/123").unwrap();
    assert_eq!(m.params.get("id"), Some(&"123".to_string()));
}

#[test]
fn test_layout_skips_missing_intermediate() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/users/settings.rs",
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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/_layout.rs",
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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/_layout.rs",
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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/_layout.rs",
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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));

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
    router.add_route(Route::from_path("pages/_error.rs", "pages"));
    router.add_route(Route::from_path("pages/api/_error.rs", "pages"));

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
    use rhtmx_router::normalize_path;

    // Valid paths should return Cow::Borrowed (zero-copy)
    let result = normalize_path("/dashboard/admin");
    assert_eq!(result, "/dashboard/admin");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn test_normalize_path_fixes_issues() {
    use rhtmx_router::normalize_path;

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
    use rhtmx_router::PathHierarchy;

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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));

    let route = Route::from_path("pages/dashboard/print.rs", "pages").with_no_layout();
    router.add_route(route.clone());

    let route_match = router.match_route("/dashboard/print").unwrap();
    let layout = router.get_layout_for_match(&route_match);

    assert!(layout.is_none(), "Should have no layout");
}

#[test]
fn test_layout_option_root() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/_layout.rs",
        "pages",
    ));

    let route =
        Route::from_path("pages/dashboard/admin/print.rs", "pages").with_root_layout();
    router.add_route(route.clone());

    let route_match = router.match_route("/dashboard/admin/print").unwrap();
    let layout = router.get_layout_for_match(&route_match).unwrap();

    assert_eq!(layout.pattern, "/", "Should use root layout only");
}

#[test]
fn test_layout_option_pattern() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/admin/_layout.rs",
        "pages",
    ));

    let route = Route::from_path("pages/dashboard/admin/users/edit.rs", "pages")
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
    let route = Route::from_path("pages/_layout.admin.rs", "pages");
    assert_eq!(route.layout_name, Some("admin".to_string()));
    assert_eq!(route.pattern, "/");

    let route = Route::from_path("pages/dashboard/_layout.marketing.rs", "pages");
    assert_eq!(route.layout_name, Some("marketing".to_string()));
    assert_eq!(route.pattern, "/dashboard");

    let route = Route::from_path("pages/_layout.rs", "pages");
    assert_eq!(route.layout_name, None);
}

#[test]
fn test_layout_option_named() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/_layout.admin.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));

    let route = Route::from_path("pages/dashboard/settings.rs", "pages")
        .with_named_layout("admin");
    router.add_route(route.clone());

    let route_match = router.match_route("/dashboard/settings").unwrap();
    let layout = router.get_layout_for_match(&route_match).unwrap();

    assert_eq!(layout.layout_name, Some("admin".to_string()));
}

#[test]
fn test_get_layout_by_name() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.admin.rs", "pages"));
    router.add_route(Route::from_path("pages/_layout.marketing.rs", "pages"));

    let admin_layout = router.get_layout_by_name("admin").unwrap();
    assert_eq!(admin_layout.layout_name, Some("admin".to_string()));

    let marketing_layout = router.get_layout_by_name("marketing").unwrap();
    assert_eq!(marketing_layout.layout_name, Some("marketing".to_string()));

    assert!(router.get_layout_by_name("nonexistent").is_none());
}

#[test]
fn test_functional_builder_chaining() {
    let route = Route::from_path("pages/dashboard/print.rs", "pages")
        .with_root_layout();

    assert_eq!(route.layout_option, LayoutOption::Root);
    assert_eq!(route.pattern, "/dashboard/print");

    let route = Route::from_path("pages/login.rs", "pages").with_no_layout();
    assert_eq!(route.layout_option, LayoutOption::None);

    let route = Route::from_path("pages/admin/users.rs", "pages")
        .with_named_layout("admin");
    assert_eq!(
        route.layout_option,
        LayoutOption::Named("admin".to_string())
    );
}

#[test]
fn test_layout_inherit_default() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));

    // Default behavior - should inherit
    let route = Route::from_path("pages/dashboard/settings.rs", "pages");
    router.add_route(route.clone());

    let route_match = router.match_route("/dashboard/settings").unwrap();
    let layout = router.get_layout_for_match(&route_match).unwrap();

    assert_eq!(layout.pattern, "/dashboard", "Should inherit from parent");
}

#[test]
fn test_complex_layout_scenario() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/_layout.admin.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));

    // Route 1: Use admin layout
    let route1 =
        Route::from_path("pages/dashboard/users.rs", "pages").with_named_layout("admin");
    router.add_route(route1);

    // Route 2: No layout
    let route2 = Route::from_path("pages/dashboard/print.rs", "pages").with_no_layout();
    router.add_route(route2);

    // Route 3: Root layout only
    let route3 =
        Route::from_path("pages/dashboard/export.rs", "pages").with_root_layout();
    router.add_route(route3);

    // Route 4: Default (inherit)
    let route4 = Route::from_path("pages/dashboard/settings.rs", "pages");
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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    assert!(router
        .get_layout_with_option("/any/path", &LayoutOption::Root)
        .is_some());

    // Named layout
    router.add_route(Route::from_path("pages/_layout.admin.rs", "pages"));
    let layout = router
        .get_layout_with_option("/any", &LayoutOption::Named("admin".to_string()))
        .unwrap();
    assert_eq!(layout.layout_name, Some("admin".to_string()));

    // Pattern
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
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
    let route = Route::from_path("pages/dashboard/_nolayout.rs", "pages");
    assert!(route.is_nolayout_marker);
    assert_eq!(route.pattern, "/dashboard");

    let route = Route::from_path("pages/_nolayout.rs", "pages");
    assert!(route.is_nolayout_marker);
    assert_eq!(route.pattern, "/");
}

#[test]
fn test_nolayout_marker_effect() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/print/_nolayout.rs", "pages"));
    router.add_route(Route::from_path(
        "pages/dashboard/print/invoice.rs",
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
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_nolayout.rs", "pages"));

    // Direct child - no layout
    router.add_route(Route::from_path("pages/dashboard/print.rs", "pages"));
    let m = router.match_route("/dashboard/print").unwrap();
    assert!(router.get_layout_for_match(&m).is_none());

    // Nested child - also no layout
    router.add_route(Route::from_path(
        "pages/dashboard/reports/monthly.rs",
        "pages",
    ));
    let m = router.match_route("/dashboard/reports/monthly").unwrap();
    assert!(router.get_layout_for_match(&m).is_none());

    // Outside the nolayout directory - has layout
    router.add_route(Route::from_path("pages/settings.rs", "pages"));
    let m = router.match_route("/settings").unwrap();
    let layout = router.get_layout_for_match(&m).unwrap();
    assert_eq!(layout.pattern, "/");
}

#[test]
fn test_nolayout_marker_vs_explicit_option() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/_layout.admin.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_nolayout.rs", "pages"));

    // Explicit layout option should override nolayout marker
    let route = Route::from_path("pages/dashboard/settings.rs", "pages")
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
    router.add_route(Route::from_path("pages/dashboard/_nolayout.rs", "pages"));
    router.add_route(Route::from_path("pages/api/v1/_nolayout.rs", "pages"));

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
    let route = Route::from_path("pages/users/[id].rs", "pages")
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
        Route::from_path("pages/admin/dashboard.rs", "pages").with_metadata(meta.clone());

    assert_eq!(route.get_meta("title"), Some(&"Admin Dashboard".to_string()));
    assert_eq!(route.get_meta("permission"), Some(&"admin.read".to_string()));
    assert_eq!(
        route.get_meta("description"),
        Some(&"Main admin page".to_string())
    );
}

#[test]
fn test_route_has_meta() {
    let route = Route::from_path("pages/admin/users.rs", "pages")
        .with_meta("permission", "admin.read")
        .with_meta("title", "User Management");

    assert!(route.has_meta("permission"));
    assert!(route.has_meta("title"));
    assert!(!route.has_meta("missing"));
    assert!(!route.has_meta("cache_ttl"));
}

#[test]
fn test_metadata_chaining() {
    let route = Route::from_path("pages/products/[id].rs", "pages")
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

    let route = Route::from_path("pages/users/[id].rs", "pages")
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
    let route = Route::from_path("pages/settings.rs", "pages")
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
    let route = Route::from_path("pages/about.rs", "pages");

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
    let route = Route::from_path("pages/users/[id:int].rs", "pages");
    assert_eq!(route.pattern, "/users/:id");
    assert_eq!(
        route.param_constraints.get("id"),
        Some(&ParameterConstraint::Int)
    );

    // Alpha constraint
    let route = Route::from_path("pages/tags/[name:alpha].rs", "pages");
    assert_eq!(
        route.param_constraints.get("name"),
        Some(&ParameterConstraint::Alpha)
    );

    // Slug constraint
    let route = Route::from_path("pages/posts/[slug:slug].rs", "pages");
    assert_eq!(
        route.param_constraints.get("slug"),
        Some(&ParameterConstraint::Slug)
    );
}

#[test]
fn test_constraint_int_validation() {
    let route = Route::from_path("pages/users/[id:int].rs", "pages");

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
    let route = Route::from_path("pages/products/[id:uint].rs", "pages");

    // Valid unsigned integers
    assert!(route.matches("/products/123").is_some());
    assert!(route.matches("/products/0").is_some());

    // Invalid (negative or non-integer)
    assert!(route.matches("/products/-456").is_none());
    assert!(route.matches("/products/abc").is_none());
}

#[test]
fn test_constraint_alpha_validation() {
    let route = Route::from_path("pages/tags/[name:alpha].rs", "pages");

    // Valid alphabetic
    assert!(route.matches("/tags/hello").is_some());
    assert!(route.matches("/tags/ABC").is_some());

    // Invalid (contains numbers or special chars)
    assert!(route.matches("/tags/hello123").is_none());
    assert!(route.matches("/tags/hello-world").is_none());
}

#[test]
fn test_constraint_alphanum_validation() {
    let route = Route::from_path("pages/codes/[code:alphanum].rs", "pages");

    // Valid alphanumeric
    assert!(route.matches("/codes/abc123").is_some());
    assert!(route.matches("/codes/Test99").is_some());

    // Invalid (special characters)
    assert!(route.matches("/codes/hello-world").is_none());
    assert!(route.matches("/codes/test_123").is_none());
}

#[test]
fn test_constraint_slug_validation() {
    let route = Route::from_path("pages/posts/[slug:slug].rs", "pages");

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
    let route = Route::from_path("pages/items/[id:uuid].rs", "pages");

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
    let route = Route::from_path("pages/posts/[id:int?].rs", "pages");

    // Valid with integer
    assert!(route.matches("/posts/123").is_some());

    // Valid without parameter
    assert!(route.matches("/posts").is_some());

    // Invalid with non-integer
    assert!(route.matches("/posts/abc").is_none());
}

#[test]
fn test_constraint_catch_all() {
    // App Router: pages/docs/[...slug]/page.rs
    let route = Route::from_path("pages/docs/[...slug]/page.rs", "pages");

    // Valid slugs
    assert!(route.matches("/docs/hello-world").is_some());
    assert!(route.matches("/docs/guide/intro").is_some());

    // Note: This test would fail because catch-all captures everything,
    // including slashes, making it hard to validate individual segments
    // In practice, catch-all constraints are less useful than per-segment ones
}

#[test]
fn test_multiple_constraints() {
    let route = Route::from_path("pages/posts/[year:int]/[slug:slug].rs", "pages");

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

    router.add_route(Route::from_path("pages/users/[id:int].rs", "pages"));
    router.add_route(Route::from_path("pages/users/[name:alpha].rs", "pages"));

    // Should match integer route
    let m = router.match_route("/users/123");
    assert!(m.is_some());
    assert_eq!(m.unwrap().route.template_path, "pages/users/[id:int].rs");

    // Should match alpha route
    let m = router.match_route("/users/john");
    assert!(m.is_some());
    assert_eq!(
        m.unwrap().route.template_path,
        "pages/users/[name:alpha].rs"
    );
}

#[test]
fn test_no_constraint_default_behavior() {
    let route = Route::from_path("pages/users/[id].rs", "pages");

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
    let route = Route::from_path("pages/about.rs", "pages")
        .with_alias("/about-us");

    assert_eq!(route.pattern, "/about");
    assert_eq!(route.aliases, vec!["/about-us"]);
}

#[test]
fn test_route_with_multiple_aliases_chained() {
    let route = Route::from_path("pages/about.rs", "pages")
        .with_alias("/about-us")
        .with_alias("/company")
        .with_alias("/über");

    assert_eq!(route.aliases.len(), 3);
    assert_eq!(route.aliases, vec!["/about-us", "/company", "/über"]);
}

#[test]
fn test_route_with_aliases_batch() {
    let route = Route::from_path("pages/contact.rs", "pages")
        .with_aliases(["/contact-us", "/get-in-touch", "/reach-us"]);

    assert_eq!(route.aliases.len(), 3);
    assert!(route.aliases.contains(&"/contact-us".to_string()));
    assert!(route.aliases.contains(&"/get-in-touch".to_string()));
    assert!(route.aliases.contains(&"/reach-us".to_string()));
}

#[test]
fn test_route_matches_any_primary() {
    let route = Route::from_path("pages/about.rs", "pages")
        .with_aliases(["/about-us", "/company"]);

    // Primary pattern should match
    assert!(route.matches_any("/about").is_some());
}

#[test]
fn test_route_matches_any_alias() {
    let route = Route::from_path("pages/about.rs", "pages")
        .with_aliases(["/about-us", "/company"]);

    // Aliases should match
    assert!(route.matches_any("/about-us").is_some());
    assert!(route.matches_any("/company").is_some());
}

#[test]
fn test_route_matches_any_no_match() {
    let route = Route::from_path("pages/about.rs", "pages")
        .with_aliases(["/about-us", "/company"]);

    // Non-matching path
    assert!(route.matches_any("/other").is_none());
    assert!(route.matches_any("/contact").is_none());
}

#[test]
fn test_router_match_route_with_alias() {
    let mut router = Router::new();

    let route = Route::from_path("pages/about.rs", "pages")
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
        Route::from_path("pages/about.rs", "pages")
            .with_aliases(["/about-us", "/company"])
    );

    router.add_route(
        Route::from_path("pages/contact.rs", "pages")
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
    assert_eq!(m.route.template_path, "pages/about.rs");

    let m = router.match_route("/contact-us").unwrap();
    assert_eq!(m.route.template_path, "pages/contact.rs");
}

#[test]
fn test_alias_with_trailing_slash() {
    let route = Route::from_path("pages/about.rs", "pages")
        .with_alias("/about-us");

    // Should match with and without trailing slash
    assert!(route.matches_any("/about-us").is_some());
    assert!(route.matches_any("/about-us/").is_some());
}

#[test]
fn test_alias_internationalization() {
    let route = Route::from_path("pages/about.rs", "pages")
        .with_aliases(["/über", "/acerca", "/à-propos"]);

    assert!(route.matches_any("/über").is_some());
    assert!(route.matches_any("/acerca").is_some());
    assert!(route.matches_any("/à-propos").is_some());
}

#[test]
fn test_alias_legacy_url_support() {
    let route = Route::from_path("pages/products/page.rs", "pages")
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
    let route = Route::from_path("pages/about.rs", "pages")
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
    let route = Route::from_path("pages/dashboard/print.rs", "pages")
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
    let route = Route::from_path("pages/about.rs", "pages");
    assert_eq!(route.aliases.len(), 0);
    assert!(route.aliases.is_empty());
}

#[test]
fn test_alias_priority_order() {
    let mut router = Router::new();

    // Add static route with aliases
    router.add_route(
        Route::from_path("pages/products/new.rs", "pages")
            .with_alias("/create-product")
    );

    // Add dynamic route
    router.add_route(Route::from_path("pages/products/[id].rs", "pages"));

    // Static route should match first
    let m = router.match_route("/products/new").unwrap();
    assert_eq!(m.route.template_path, "pages/products/new.rs");

    // Alias should also match
    let m = router.match_route("/create-product").unwrap();
    assert_eq!(m.route.template_path, "pages/products/new.rs");

    // Dynamic route should match other paths
    let m = router.match_route("/products/123").unwrap();
    assert_eq!(m.route.template_path, "pages/products/[id].rs");
}

#[test]
fn test_route_static_alias_matching() {
    let route = Route::from_path("pages/about.rs", "pages");

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
        Route::from_path("pages/about.rs", "pages")
            .with_alias("/About-Us")
    );

    // Case-sensitive by default
    assert!(router.match_route("/About-Us").is_some());
    assert!(router.match_route("/about-us").is_none());
}

#[test]
fn test_functional_alias_composition() {
    // Test functional builder pattern composition
    let route = Route::from_path("pages/home.rs", "pages")
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
    let route = Route::from_path("pages/users/[id].rs", "pages")
        .with_name("user.profile");

    assert_eq!(route.name, Some("user.profile".to_string()));
    assert_eq!(route.pattern, "/users/:id");
}

#[test]
fn test_route_without_name() {
    let route = Route::from_path("pages/about.rs", "pages");
    assert_eq!(route.name, None);
}

#[test]
fn test_generate_url_static_route() {
    let route = Route::from_path("pages/about.rs", "pages");
    let params = HashMap::new();

    let url = route.generate_url(&params).unwrap();
    assert_eq!(url, "/about");
}

#[test]
fn test_generate_url_with_single_parameter() {
    let route = Route::from_path("pages/users/[id].rs", "pages");

    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let url = route.generate_url(&params).unwrap();
    assert_eq!(url, "/users/123");
}

#[test]
fn test_generate_url_with_multiple_parameters() {
    let route = Route::from_path("pages/posts/[year]/[slug].rs", "pages");

    let mut params = HashMap::new();
    params.insert("year".to_string(), "2024".to_string());
    params.insert("slug".to_string(), "hello-world".to_string());

    let url = route.generate_url(&params).unwrap();
    assert_eq!(url, "/posts/2024/hello-world");
}

#[test]
fn test_generate_url_missing_required_parameter() {
    let route = Route::from_path("pages/users/[id].rs", "pages");

    let params = HashMap::new(); // Missing "id"

    let url = route.generate_url(&params);
    assert!(url.is_none(), "Should return None when required parameter is missing");
}

#[test]
fn test_generate_url_optional_parameter_provided() {
    let route = Route::from_path("pages/posts/[id?].rs", "pages");

    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let url = route.generate_url(&params).unwrap();
    assert_eq!(url, "/posts/123");
}

#[test]
fn test_generate_url_optional_parameter_missing() {
    let route = Route::from_path("pages/posts/[id?].rs", "pages");

    let params = HashMap::new(); // No "id" provided

    let url = route.generate_url(&params).unwrap();
    assert_eq!(url, "/posts");
}

#[test]
fn test_generate_url_catch_all() {
    let route = Route::from_path("pages/docs/[...slug].rs", "pages");

    let mut params = HashMap::new();
    params.insert("slug".to_string(), "guide/getting-started".to_string());

    let url = route.generate_url(&params).unwrap();
    assert_eq!(url, "/docs/guide/getting-started");
}

#[test]
fn test_router_url_for() {
    let mut router = Router::new();

    router.add_route(
        Route::from_path("pages/users/[id].rs", "pages")
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
        Route::from_path("pages/posts/[year]/[slug].rs", "pages")
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
        Route::from_path("pages/about.rs", "pages")
            .with_name("about")
    );

    let route = router.get_route_by_name("about").unwrap();
    assert_eq!(route.pattern, "/about");
    assert_eq!(route.template_path, "pages/about.rs");
}

#[test]
fn test_router_get_route_by_name_nonexistent() {
    let router = Router::new();
    assert!(router.get_route_by_name("nonexistent").is_none());
}

#[test]
fn test_named_route_with_metadata() {
    let route = Route::from_path("pages/users/[id].rs", "pages")
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
        Route::from_path("pages/about.rs", "pages")
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
        Route::from_path("pages/page.rs", "pages")
            .with_name("home")
    );

    router.add_route(
        Route::from_path("pages/about.rs", "pages")
            .with_name("about")
    );

    router.add_route(
        Route::from_path("pages/users/[id].rs", "pages")
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
    let route = Route::from_path("pages/page.rs", "pages");
    let url = route.generate_url(&HashMap::new()).unwrap();
    assert_eq!(url, "/");
}

#[test]
fn test_url_for_params_empty() {
    let mut router = Router::new();

    router.add_route(
        Route::from_path("pages/about.rs", "pages")
            .with_name("about")
    );

    let url = router.url_for_params("about", &[]).unwrap();
    assert_eq!(url, "/about");
}

#[test]
fn test_named_route_functional_chaining() {
    let route = Route::from_path("pages/users/[id].rs", "pages")
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
    let route = Route::from_path("pages/events/[year]/[month]/[day].rs", "pages");

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
        Route::from_path("pages/about.rs", "pages")
            .with_name("about")
    );

    assert!(router.get_route_by_name("about").is_some());

    router.remove_route("/about");

    assert!(router.get_route_by_name("about").is_none());
}

#[test]
fn test_url_generation_with_extra_params() {
    let route = Route::from_path("pages/users/[id].rs", "pages");

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
        Route::from_path("pages/api/v1/users/[id].rs", "pages")
            .with_name("api.v1.users.show")
    );

    // Change pattern (simulating refactoring)
    router.remove_route("/api/v1/users/:id");
    router.add_route(
        Route::from_path("pages/api/v2/users/[id].rs", "pages")
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
    router.add_route(Route::from_path("pages/about.rs", "pages"));

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
    router.add_route(Route::from_path("pages/products/page.rs", "pages"));

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
    router.add_route(Route::from_path("pages/about.rs", "pages"));

    let m = router.match_route("/about/").unwrap();
    assert!(m.is_redirect());
    assert_eq!(m.redirect_target().unwrap(), "/about");
}

#[test]
fn test_redirect_target_non_redirect_route() {
    let route = Route::from_path("pages/about.rs", "pages");
    let params = HashMap::new();

    assert!(!route.is_redirect);
    assert_eq!(route.redirect_target(&params), None);
}

#[test]
fn test_route_match_redirect_methods_non_redirect() {
    let mut router = Router::new();
    router.add_route(Route::from_path("pages/about.rs", "pages"));

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
    router.add_route(Route::from_path("pages/about.rs", "pages"));

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
    router.add_route(Route::from_path("pages/c.rs", "pages"));

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
    let route = Route::from_path("pages/docs/[[...slug]].rs", "pages");

    assert_eq!(route.pattern, "/docs/*slug?");
    assert_eq!(route.params, vec!["slug"]);
    assert_eq!(route.optional_params, vec!["slug"]);
    assert!(route.has_catch_all);
}

#[test]
fn test_optional_catch_all_matches_zero_segments() {
    let route = Route::from_path("pages/docs/[[...slug]].rs", "pages");

    // Should match /docs (zero segments)
    let params = route.matches("/docs").unwrap();
    assert_eq!(params.get("slug"), Some(&String::new()));
}

#[test]
fn test_optional_catch_all_matches_single_segment() {
    let route = Route::from_path("pages/docs/[[...slug]].rs", "pages");

    // Should match /docs/intro
    let params = route.matches("/docs/intro").unwrap();
    assert_eq!(params.get("slug"), Some(&"intro".to_string()));
}

#[test]
fn test_optional_catch_all_matches_multiple_segments() {
    let route = Route::from_path("pages/docs/[[...slug]].rs", "pages");

    // Should match /docs/getting-started/installation
    let params = route.matches("/docs/getting-started/installation").unwrap();
    assert_eq!(
        params.get("slug"),
        Some(&"getting-started/installation".to_string())
    );
}

#[test]
fn test_optional_catch_all_with_constraint() {
    let route = Route::from_path("pages/docs/[[...slug:alpha]].rs", "pages");

    assert_eq!(route.pattern, "/docs/*slug?");
    assert!(route.param_constraints.contains_key("slug"));
    assert_eq!(
        route.param_constraints.get("slug"),
        Some(&ParameterConstraint::Alpha)
    );
}

#[test]
fn test_optional_catch_all_vs_required_catch_all() {
    let optional = Route::from_path("pages/docs/[[...slug]].rs", "pages");
    let required = Route::from_path("pages/blog/[...slug].rs", "pages");

    // Optional matches zero segments
    assert!(optional.matches("/docs").is_some());

    // Required does NOT match zero segments
    assert!(required.matches("/blog").is_none());
    assert!(required.matches("/blog/post").is_some());
}

#[test]
fn test_optional_catch_all_priority() {
    let optional = Route::from_path("pages/docs/[[...slug]].rs", "pages");
    let required = Route::from_path("pages/blog/[...slug].rs", "pages");

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

    router.add_route(Route::from_path("pages/docs/[[...slug]].rs", "pages"));

    // Test zero segments
    let m = router.match_route("/docs").unwrap();
    assert_eq!(m.params.get("slug"), Some(&String::new()));
    assert_eq!(m.route.template_path, "pages/docs/[[...slug]].rs");

    // Test single segment
    let m = router.match_route("/docs/intro").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"intro".to_string()));

    // Test multiple segments
    let m = router.match_route("/docs/api/reference").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"api/reference".to_string()));
}

#[test]
fn test_optional_catch_all_with_static_prefix() {
    let route = Route::from_path("pages/api/v1/[[...path]].rs", "pages");

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
    router.add_route(Route::from_path("pages/docs/getting-started.rs", "pages"));

    // Dynamic route
    router.add_route(Route::from_path("pages/docs/[section].rs", "pages"));

    // Optional catch-all has lower priority
    router.add_route(Route::from_path("pages/docs/[[...slug]].rs", "pages"));

    // Static should match first
    let m = router.match_route("/docs/getting-started").unwrap();
    assert_eq!(m.route.template_path, "pages/docs/getting-started.rs");

    // Dynamic should match next
    let m = router.match_route("/docs/api").unwrap();
    assert_eq!(m.route.template_path, "pages/docs/[section].rs");

    // Optional catch-all should match base and deep paths
    let m = router.match_route("/docs").unwrap();
    assert_eq!(m.route.template_path, "pages/docs/[[...slug]].rs");

    let m = router.match_route("/docs/guides/advanced/tips").unwrap();
    assert_eq!(m.route.template_path, "pages/docs/[[...slug]].rs");
}

#[test]
fn test_optional_catch_all_empty_string_vs_none() {
    let route = Route::from_path("pages/docs/[[...slug]].rs", "pages");

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
    let catch_all = Route::from_path("pages/docs/[[...slug]].rs", "pages");
    let optional_param = Route::from_path("pages/users/[id?].rs", "pages");

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
    let route = Route::from_path("pages/docs/[[...slug]].rs", "pages");

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

    router.add_route(Route::from_path("pages/docs/[[...slug]].rs", "pages"));

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
    let route = Route::from_path("pages/files/[[...path:slug]].rs", "pages");

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
    let route = Route::from_path("pages/(marketing)/about.rs", "pages");

    // Route group (marketing) should not appear in the pattern
    assert_eq!(route.pattern, "/about");
    assert_eq!(route.template_path, "pages/(marketing)/about.rs");
}

#[test]
fn test_route_group_multiple() {
    let route = Route::from_path("pages/(marketing)/blog/posts.rs", "pages");

    // Only (marketing) is skipped, blog is kept
    assert_eq!(route.pattern, "/blog/posts");
    assert_eq!(route.template_path, "pages/(marketing)/blog/posts.rs");
}

#[test]
fn test_route_group_nested() {
    let route = Route::from_path("pages/(shop)/(products)/list.rs", "pages");

    // Both (shop) and (products) should be skipped
    assert_eq!(route.pattern, "/list");
    assert_eq!(route.template_path, "pages/(shop)/(products)/list.rs");
}

#[test]
fn test_route_group_with_dynamic_params() {
    let route = Route::from_path("pages/(shop)/products/[id].rs", "pages");

    // (shop) skipped, dynamic param kept
    assert_eq!(route.pattern, "/products/:id");
    assert_eq!(route.params, vec!["id"]);
}

#[test]
fn test_route_group_with_catch_all() {
    let route = Route::from_path("pages/(docs)/[[...slug]].rs", "pages");

    // (docs) skipped, optional catch-all at root
    assert_eq!(route.pattern, "/*slug?");
    assert!(route.has_catch_all);
}

#[test]
fn test_route_group_organizational_structure() {
    let mut router = Router::new();

    // Marketing routes
    router.add_route(Route::from_path("pages/(marketing)/about.rs", "pages"));
    router.add_route(Route::from_path("pages/(marketing)/blog/page.rs", "pages"));

    // Shop routes
    router.add_route(Route::from_path("pages/(shop)/products/page.rs", "pages"));
    router.add_route(Route::from_path("pages/(shop)/cart.rs", "pages"));

    // All should match without the group names in URL
    assert!(router.match_route("/about").is_some());
    assert!(router.match_route("/blog").is_some());
    assert!(router.match_route("/products").is_some());
    assert!(router.match_route("/cart").is_some());
}

#[test]
fn test_route_group_with_layout() {
    let marketing_layout = Route::from_path("pages/(marketing)/_layout.rs", "pages");
    let shop_layout = Route::from_path("pages/(shop)/_layout.rs", "pages");

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
    router.add_route(Route::from_path("pages/(v1)/api/users.rs", "pages"));
    router.add_route(Route::from_path("pages/(v2)/api/users.rs", "pages"));

    // Both map to /api/users - last one wins (or could be an error)
    let matches: Vec<_> = router.routes().iter()
        .filter(|r| r.pattern == "/api/users")
        .collect();

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_route_group_empty_name() {
    // Edge case: group with no name
    let route = Route::from_path("pages/()/about.rs", "pages");

    // Should still skip it
    assert_eq!(route.pattern, "/about");
}

#[test]
fn test_route_group_not_at_start() {
    let route = Route::from_path("pages/admin/(dashboard)/stats.rs", "pages");

    // Group in the middle
    assert_eq!(route.pattern, "/admin/stats");
}

#[test]
fn test_route_group_with_special_chars() {
    let route = Route::from_path("pages/(admin-panel)/users.rs", "pages");

    // Group names can have hyphens
    assert_eq!(route.pattern, "/users");
    assert_eq!(route.template_path, "pages/(admin-panel)/users.rs");
}

#[test]
fn test_route_group_priority_unchanged() {
    let grouped = Route::from_path("pages/(shop)/products.rs", "pages");
    let non_grouped = Route::from_path("pages/products.rs", "pages");

    // Both should have same priority (both static)
    assert_eq!(grouped.priority, non_grouped.priority);
    assert_eq!(grouped.pattern, non_grouped.pattern);
}

#[test]
fn test_route_group_with_named_layout() {
    let route = Route::from_path("pages/(admin)/_layout.dashboard.rs", "pages");

    assert!(route.is_layout);
    assert_eq!(route.layout_name, Some("dashboard".to_string()));
    assert_eq!(route.pattern, "/");
}

#[test]
fn test_route_group_real_world_organization() {
    let mut router = Router::new();

    // Auth group
    router.add_route(Route::from_path("pages/(auth)/login.rs", "pages"));
    router.add_route(Route::from_path("pages/(auth)/signup.rs", "pages"));
    router.add_route(Route::from_path("pages/(auth)/reset-password.rs", "pages"));

    // Dashboard group
    router.add_route(Route::from_path("pages/(dashboard)/home.rs", "pages"));
    router.add_route(Route::from_path("pages/(dashboard)/settings.rs", "pages"));

    // Public group
    router.add_route(Route::from_path("pages/(public)/about.rs", "pages"));
    router.add_route(Route::from_path("pages/(public)/contact.rs", "pages"));

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
    assert_eq!(m.route.template_path, "pages/(auth)/login.rs");

    let m = router.match_route("/home").unwrap();
    assert_eq!(m.route.template_path, "pages/(dashboard)/home.rs");
}

#[test]
fn test_route_group_with_route_aliases() {
    let route = Route::from_path("pages/(marketing)/about.rs", "pages")
        .with_aliases(["/about-us", "/company"]);

    assert_eq!(route.pattern, "/about");
    assert_eq!(route.aliases.len(), 2);
}

#[test]
fn test_route_group_with_metadata() {
    let route = Route::from_path("pages/(admin)/users.rs", "pages")
        .with_meta("permission", "admin.read")
        .with_meta("title", "User Management");

    assert_eq!(route.pattern, "/users");
    assert_eq!(route.get_meta("permission"), Some(&"admin.read".to_string()));
}

#[test]
fn test_route_group_does_not_affect_params() {
    let route = Route::from_path("pages/(api)/users/[id]/posts/[postId].rs", "pages");

    assert_eq!(route.pattern, "/users/:id/posts/:postId");
    assert_eq!(route.params, vec!["id", "postId"]);
}

#[test]
fn test_route_group_multiple_levels() {
    let route = Route::from_path("pages/(app)/(dashboard)/(main)/home.rs", "pages");

    // All three groups should be skipped
    assert_eq!(route.pattern, "/home");
    assert_eq!(route.template_path, "pages/(app)/(dashboard)/(main)/home.rs");
}

// ===== Phase 4.3: Loading UI Tests =====

#[test]
fn test_loading_ui_detection() {
    let loading = Route::from_path("pages/dashboard/loading.rs", "pages");

    assert!(loading.is_loading);
    assert_eq!(loading.pattern, "/dashboard");
    assert_eq!(loading.template_path, "pages/dashboard/loading.rs");
}

#[test]
fn test_loading_ui_hierarchical_resolution() {
    let mut router = Router::new();

    // Root loading
    router.add_route(Route::from_path("pages/loading.rs", "pages"));

    // Dashboard loading
    router.add_route(Route::from_path("pages/dashboard/loading.rs", "pages"));

    // Stats loading
    router.add_route(Route::from_path("pages/dashboard/stats/loading.rs", "pages"));

    // Check hierarchical lookup
    assert!(router.get_loading_page("/dashboard/stats").is_some());
    assert!(router.get_loading_page("/dashboard").is_some());
    assert!(router.get_loading_page("/").is_some());
}

#[test]
fn test_loading_pages_collection() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/loading.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/loading.rs", "pages"));

    assert_eq!(router.loading_pages().len(), 2);
    assert!(router.loading_pages().contains_key("/"));
    assert!(router.loading_pages().contains_key("/dashboard"));
}

// ===== Phase 4.4: Template Files Tests =====

#[test]
fn test_template_detection() {
    let template = Route::from_path("pages/dashboard/_template.rs", "pages");

    assert!(template.is_template);
    assert_eq!(template.pattern, "/dashboard");
    assert_eq!(template.template_path, "pages/dashboard/_template.rs");
}

#[test]
fn test_template_hierarchical_resolution() {
    let mut router = Router::new();

    // Root template
    router.add_route(Route::from_path("pages/_template.rs", "pages"));

    // Dashboard template
    router.add_route(Route::from_path("pages/dashboard/_template.rs", "pages"));

    // Stats template
    router.add_route(Route::from_path("pages/dashboard/stats/_template.rs", "pages"));

    // Check hierarchical lookup
    assert!(router.get_template("/dashboard/stats").is_some());
    assert!(router.get_template("/dashboard").is_some());
    assert!(router.get_template("/").is_some());
}

#[test]
fn test_templates_collection() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/_template.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_template.rs", "pages"));

    assert_eq!(router.templates().len(), 2);
    assert!(router.templates().contains_key("/"));
    assert!(router.templates().contains_key("/dashboard"));
}

// ===== Phase 4.5: Not-Found Pages Tests =====

#[test]
fn test_not_found_detection() {
    let not_found = Route::from_path("pages/dashboard/not-found.rs", "pages");

    assert!(not_found.is_not_found);
    assert_eq!(not_found.pattern, "/dashboard");
    assert_eq!(not_found.template_path, "pages/dashboard/not-found.rs");
}

#[test]
fn test_not_found_hierarchical_resolution() {
    let mut router = Router::new();

    // Root not-found
    router.add_route(Route::from_path("pages/not-found.rs", "pages"));

    // Dashboard not-found
    router.add_route(Route::from_path("pages/dashboard/not-found.rs", "pages"));

    // Stats not-found
    router.add_route(Route::from_path("pages/dashboard/stats/not-found.rs", "pages"));

    // Check hierarchical lookup
    assert!(router.get_not_found_page("/dashboard/stats").is_some());
    assert!(router.get_not_found_page("/dashboard").is_some());
    assert!(router.get_not_found_page("/").is_some());
}

#[test]
fn test_not_found_pages_collection() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/not-found.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/not-found.rs", "pages"));

    assert_eq!(router.not_found_pages().len(), 2);
    assert!(router.not_found_pages().contains_key("/"));
    assert!(router.not_found_pages().contains_key("/dashboard"));
}

// ===== Integration Tests: All Special Files Together =====

#[test]
fn test_all_special_files_together() {
    let mut router = Router::new();

    // Add all special file types for dashboard section
    router.add_route(Route::from_path("pages/dashboard/_layout.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/loading.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_template.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/not-found.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_error.rs", "pages"));

    // Regular page
    router.add_route(Route::from_path("pages/dashboard/page.rs", "pages"));

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

    router.add_route(Route::from_path("pages/loading.rs", "pages"));
    router.add_route(Route::from_path("pages/_template.rs", "pages"));
    router.add_route(Route::from_path("pages/not-found.rs", "pages"));

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
    router.add_route(Route::from_path("pages/(app)/loading.rs", "pages"));
    router.add_route(Route::from_path("pages/(app)/_template.rs", "pages"));
    router.add_route(Route::from_path("pages/(app)/not-found.rs", "pages"));

    // Pattern should have route group removed
    assert!(router.get_loading_page("/").is_some());
    assert!(router.get_template("/").is_some());
    assert!(router.get_not_found_page("/").is_some());

    // But template_path should preserve the group
    let loading = router.loading_pages().get("/").unwrap();
    assert_eq!(loading.template_path, "pages/(app)/loading.rs");
}

// ===== Phase 5.1: Parallel Routes Tests =====

#[test]
fn test_parallel_route_detection() {
    let route = Route::from_path("pages/dashboard/@analytics/page.rs", "pages");

    assert!(route.is_parallel_route);
    assert_eq!(route.parallel_slot, Some("analytics".to_string()));
    // @analytics is skipped from pattern
    assert_eq!(route.pattern, "/dashboard");
    assert_eq!(route.template_path, "pages/dashboard/@analytics/page.rs");
}

#[test]
fn test_parallel_route_multiple_slots() {
    let mut router = Router::new();

    // Multiple parallel slots for same pattern
    router.add_route(Route::from_path("pages/dashboard/@analytics/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@team/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@settings/page.rs", "pages"));

    let slots = router.get_parallel_routes("/dashboard").unwrap();
    assert_eq!(slots.len(), 3);
    assert!(slots.contains_key("analytics"));
    assert!(slots.contains_key("team"));
    assert!(slots.contains_key("settings"));
}

#[test]
fn test_parallel_route_specific_slot() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/dashboard/@analytics/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@team/page.rs", "pages"));

    let analytics = router.get_parallel_route("/dashboard", "analytics").unwrap();
    assert_eq!(analytics.parallel_slot, Some("analytics".to_string()));
    assert_eq!(analytics.template_path, "pages/dashboard/@analytics/page.rs");

    let team = router.get_parallel_route("/dashboard", "team").unwrap();
    assert_eq!(team.parallel_slot, Some("team".to_string()));
    assert_eq!(team.template_path, "pages/dashboard/@team/page.rs");
}

#[test]
fn test_parallel_route_with_dynamic_params() {
    let route = Route::from_path("pages/products/@reviews/[id].rs", "pages");

    assert!(route.is_parallel_route);
    assert_eq!(route.parallel_slot, Some("reviews".to_string()));
    assert_eq!(route.pattern, "/products/:id");
    assert_eq!(route.params, vec!["id"]);
}

#[test]
fn test_parallel_route_with_route_groups() {
    let route = Route::from_path("pages/(shop)/products/@sidebar/list.rs", "pages");

    // Both (shop) and @sidebar are skipped
    assert!(route.is_parallel_route);
    assert_eq!(route.parallel_slot, Some("sidebar".to_string()));
    assert_eq!(route.pattern, "/products/list");
    assert_eq!(route.template_path, "pages/(shop)/products/@sidebar/list.rs");
}

#[test]
fn test_parallel_route_nested() {
    let route = Route::from_path("pages/app/@modal/profile/@details/page.rs", "pages");

    // Only first @ is detected
    assert!(route.is_parallel_route);
    assert_eq!(route.parallel_slot, Some("modal".to_string()));
    // Both @modal and @details are skipped from pattern
    assert_eq!(route.pattern, "/app/profile");
}

#[test]
fn test_parallel_route_collection_accessor() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/dash/@a/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dash/@b/page.rs", "pages"));
    router.add_route(Route::from_path("pages/other/@c/page.rs", "pages"));

    assert_eq!(router.parallel_routes().len(), 2);
    assert!(router.parallel_routes().contains_key("/dash"));
    assert!(router.parallel_routes().contains_key("/other"));
}

#[test]
fn test_parallel_route_with_index() {
    let route = Route::from_path("pages/dashboard/@analytics/page.rs", "pages");

    assert!(route.is_parallel_route);
    assert_eq!(route.parallel_slot, Some("analytics".to_string()));
    // index is skipped, @analytics is skipped
    assert_eq!(route.pattern, "/dashboard");
}

#[test]
fn test_parallel_route_real_world_dashboard() {
    let mut router = Router::new();

    // Dashboard with multiple parallel sections
    router.add_route(Route::from_path("pages/dashboard/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@analytics/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@team/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@notifications/page.rs", "pages"));

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
    let route = Route::from_path("pages/feed/(.)/photo/[id].rs", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
    assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
    // (.) is skipped from pattern
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_intercepting_route_one_level_up() {
    let route = Route::from_path("pages/feed/(..)/photo/[id].rs", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::OneLevelUp));
    assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_intercepting_route_from_root() {
    let route = Route::from_path("pages/feed/(...)/photo/[id].rs", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::FromRoot));
    assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_intercepting_route_two_levels_up() {
    let route = Route::from_path("pages/feed/(....)/photo/[id].rs", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::TwoLevelsUp));
    assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_intercepting_route_in_router() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rs", "pages"));

    let route = router.get_intercepting_route("/feed/photo/:id").unwrap();
    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
}

#[test]
fn test_intercepting_route_with_route_groups() {
    let route = Route::from_path("pages/(app)/feed/(.)/photo/[id].rs", "pages");

    // Both (app) and (.) are skipped
    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_intercepting_route_modal_pattern() {
    let mut router = Router::new();

    // Regular photo page
    router.add_route(Route::from_path("pages/photo/[id].rs", "pages"));

    // Intercepting route when coming from feed
    router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rs", "pages"));

    // Regular route exists
    assert!(router.match_route("/photo/123").is_some());

    // Intercepting route also exists
    let intercept = router.get_intercepting_route("/feed/photo/:id").unwrap();
    assert_eq!(intercept.intercept_target, Some("photo/[id]".to_string()));
}

#[test]
fn test_intercepting_route_collection_accessor() {
    let mut router = Router::new();

    router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rs", "pages"));
    router.add_route(Route::from_path("pages/gallery/(..)/image/[id].rs", "pages"));

    assert_eq!(router.intercepting_routes().len(), 2);
    assert!(router.intercepting_routes().contains_key("/feed/photo/:id"));
    assert!(router.intercepting_routes().contains_key("/gallery/image/:id"));
}

#[test]
fn test_intercepting_route_with_catch_all() {
    let route = Route::from_path("pages/app/(...)/docs/[...slug].rs", "pages");

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
    router.add_route(Route::from_path("pages/feed/page.rs", "pages"));

    // Photo detail page (standalone)
    router.add_route(Route::from_path("pages/photo/[id].rs", "pages"));

    // Intercepting route - show photo as modal when navigating from feed
    router.add_route(Route::from_path("pages/feed/(...)/photo/[id].rs", "pages"));

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
    router.add_route(Route::from_path("pages/dashboard/@analytics/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@team/page.rs", "pages"));

    // Intercepting route from dashboard
    router.add_route(Route::from_path("pages/dashboard/(.)/settings/page.rs", "pages"));

    // Both features work together
    let slots = router.get_parallel_routes("/dashboard").unwrap();
    assert_eq!(slots.len(), 2);

    let intercept = router.get_intercepting_route("/dashboard/settings").unwrap();
    assert!(intercept.is_intercepting);
}

#[test]
fn test_parallel_route_in_intercepting_route() {
    let route = Route::from_path("pages/feed/(.)/modal/@content/page.rs", "pages");

    // Has both intercepting and parallel route markers
    assert!(route.is_intercepting);
    assert!(route.is_parallel_route);
    assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
    assert_eq!(route.parallel_slot, Some("content".to_string()));
    // Both (.) and @content are skipped
    assert_eq!(route.pattern, "/feed/modal");
}

#[test]
fn test_phase_5_with_all_previous_features() {
    let mut router = Router::new();

    // Regular route with dynamic params
    router.add_route(Route::from_path("pages/users/[id].rs", "pages"));

    // Layout
    router.add_route(Route::from_path("pages/_layout.rs", "pages"));

    // Loading UI (Phase 4.3)
    router.add_route(Route::from_path("pages/loading.rs", "pages"));

    // Route group (Phase 4.2)
    router.add_route(Route::from_path("pages/(app)/dashboard/page.rs", "pages"));

    // Parallel routes (Phase 5.1)
    router.add_route(Route::from_path("pages/dash/@analytics/page.rs", "pages"));
    router.add_route(Route::from_path("pages/dash/@team/page.rs", "pages"));

    // Intercepting route (Phase 5.2)
    router.add_route(Route::from_path("pages/feed/(.)/photo/[id].rs", "pages"));

    // All features work together
    assert!(router.match_route("/users/123").is_some());
    assert!(router.get_layout("/").is_some());
    assert!(router.get_loading_page("/").is_some());
    assert!(router.match_route("/dashboard").is_some());
    assert_eq!(router.get_parallel_routes("/dash").unwrap().len(), 2);
    assert!(router.get_intercepting_route("/feed/photo/:id").is_some());
}
