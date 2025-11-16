/// Integration tests comparing rhtmx-router with Next.js App Router
///
/// This test suite validates feature parity with Next.js App Router patterns.
/// Each test documents the equivalent Next.js pattern for reference.

use rhtmx_router::{Router, Route, InterceptLevel};
use std::collections::HashMap;

// ============================================================================
// File-System Routing Conventions (Next.js App Router Parity)
// ============================================================================

#[test]
fn test_nextjs_basic_routing() {
    // Next.js: app/page.tsx → /
    // Next.js: app/about/page.tsx → /about
    // Next.js: app/blog/posts/page.tsx → /blog/posts

    let mut router = Router::new();

    router.add_route(Route::from_path("pages/index.rhtml", "pages"));
    router.add_route(Route::from_path("pages/about.rhtml", "pages"));
    router.add_route(Route::from_path("pages/blog/posts.rhtml", "pages"));

    assert!(router.match_route("/").is_some());
    assert!(router.match_route("/about").is_some());
    assert!(router.match_route("/blog/posts").is_some());
}

#[test]
fn test_nextjs_dynamic_segments() {
    // Next.js: app/blog/[slug]/page.tsx → /blog/:slug
    // Next.js: app/shop/[category]/[item]/page.tsx → /shop/:category/:item

    let mut router = Router::new();

    let blog_route = Route::from_path("pages/blog/[slug].rhtml", "pages");
    assert_eq!(blog_route.pattern, "/blog/:slug");
    assert_eq!(blog_route.params, vec!["slug"]);

    let shop_route = Route::from_path("pages/shop/[category]/[item].rhtml", "pages");
    assert_eq!(shop_route.pattern, "/shop/:category/:item");
    assert_eq!(shop_route.params, vec!["category", "item"]);

    router.add_route(blog_route);
    router.add_route(shop_route);

    let m = router.match_route("/blog/hello-world").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"hello-world".to_string()));

    let m = router.match_route("/shop/electronics/laptop").unwrap();
    assert_eq!(m.params.get("category"), Some(&"electronics".to_string()));
    assert_eq!(m.params.get("item"), Some(&"laptop".to_string()));
}

#[test]
fn test_nextjs_catch_all_segments() {
    // Next.js: app/shop/[...slug]/page.tsx → /shop/* (1+ segments required)
    // rhtmx: pages/shop/[...slug].rhtml → /shop/*slug

    let route = Route::from_path("pages/shop/[...slug].rhtml", "pages");

    assert_eq!(route.pattern, "/shop/*slug");
    assert!(route.has_catch_all);
    assert_eq!(route.params, vec!["slug"]);

    let mut router = Router::new();
    router.add_route(route);

    // Should match 1+ segments
    let m = router.match_route("/shop/electronics").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"electronics".to_string()));

    let m = router.match_route("/shop/electronics/laptops/gaming").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"electronics/laptops/gaming".to_string()));

    // Should NOT match zero segments
    assert!(router.match_route("/shop").is_none());
}

#[test]
fn test_nextjs_optional_catch_all_segments() {
    // Next.js: app/shop/[[...slug]]/page.tsx → /shop/* (0+ segments)
    // rhtmx: pages/shop/[[...slug]].rhtml → /shop/*slug?

    let route = Route::from_path("pages/shop/[[...slug]].rhtml", "pages");

    assert_eq!(route.pattern, "/shop/*slug?");
    assert!(route.has_catch_all);
    assert_eq!(route.params, vec!["slug"]);
    assert_eq!(route.optional_params, vec!["slug"]);

    let mut router = Router::new();
    router.add_route(route);

    // Should match 0 segments
    let m = router.match_route("/shop").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"".to_string()));

    // Should match 1+ segments
    let m = router.match_route("/shop/electronics/laptops").unwrap();
    assert_eq!(m.params.get("slug"), Some(&"electronics/laptops".to_string()));
}

#[test]
fn test_nextjs_route_groups() {
    // Next.js: app/(marketing)/about/page.tsx → /about
    // Next.js: app/(shop)/products/page.tsx → /products
    // Route groups organize code without affecting URL structure

    let mut router = Router::new();

    let marketing_route = Route::from_path("pages/(marketing)/about.rhtml", "pages");
    assert_eq!(marketing_route.pattern, "/about");
    assert_eq!(marketing_route.template_path, "pages/(marketing)/about.rhtml");

    let shop_route = Route::from_path("pages/(shop)/products.rhtml", "pages");
    assert_eq!(shop_route.pattern, "/products");
    assert_eq!(shop_route.template_path, "pages/(shop)/products.rhtml");

    router.add_route(marketing_route);
    router.add_route(shop_route);

    assert!(router.match_route("/about").is_some());
    assert!(router.match_route("/products").is_some());
    assert!(router.match_route("/(marketing)/about").is_none());
}

// ============================================================================
// Layouts (Next.js App Router Parity)
// ============================================================================

#[test]
fn test_nextjs_layouts() {
    // Next.js: app/layout.tsx → Root layout
    // Next.js: app/dashboard/layout.tsx → Dashboard layout
    // Layouts are preserved during navigation

    let mut router = Router::new();

    router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/settings/_layout.rhtml", "pages"));

    // Root layout
    assert!(router.get_layout("/").is_some());

    // Dashboard layout (inherits from root)
    assert!(router.get_layout("/dashboard").is_some());
    assert!(router.get_layout("/dashboard/analytics").is_some());

    // Settings layout (inherits from dashboard)
    assert!(router.get_layout("/dashboard/settings").is_some());
}

#[test]
fn test_nextjs_loading_ui() {
    // Next.js: app/dashboard/loading.tsx → Loading UI for /dashboard
    // Automatic loading states during data fetching

    let mut router = Router::new();

    router.add_route(Route::from_path("pages/loading.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/loading.rhtml", "pages"));

    let loading = Route::from_path("pages/dashboard/loading.rhtml", "pages");
    assert!(loading.is_loading);
    assert_eq!(loading.pattern, "/dashboard");

    // Hierarchical resolution
    assert!(router.get_loading_page("/dashboard/analytics").is_some());
    assert!(router.get_loading_page("/").is_some());
}

#[test]
fn test_nextjs_error_handling() {
    // Next.js: app/dashboard/error.tsx → Error UI for /dashboard
    // Automatic error boundaries

    let mut router = Router::new();

    router.add_route(Route::from_path("pages/_error.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_error.rhtml", "pages"));

    let error = Route::from_path("pages/dashboard/_error.rhtml", "pages");
    assert!(error.is_error_page);
    assert_eq!(error.pattern, "/dashboard");

    // Hierarchical resolution
    assert!(router.get_error_page("/dashboard/settings").is_some());
    assert!(router.get_error_page("/").is_some());
}

#[test]
fn test_nextjs_not_found() {
    // Next.js: app/dashboard/not-found.tsx → 404 UI for /dashboard
    // Section-specific 404 pages

    let mut router = Router::new();

    router.add_route(Route::from_path("pages/not-found.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/not-found.rhtml", "pages"));

    let not_found = Route::from_path("pages/dashboard/not-found.rhtml", "pages");
    assert!(not_found.is_not_found);
    assert_eq!(not_found.pattern, "/dashboard");

    // Hierarchical resolution
    assert!(router.get_not_found_page("/dashboard/missing").is_some());
    assert!(router.get_not_found_page("/").is_some());
}

#[test]
fn test_nextjs_templates() {
    // Next.js: app/template.tsx → Re-mounts on navigation (unlike layouts)
    // Useful for resetting state, animations, etc.

    let mut router = Router::new();

    let template = Route::from_path("pages/dashboard/_template.rhtml", "pages");
    assert!(template.is_template);
    assert_eq!(template.pattern, "/dashboard");

    router.add_route(template);

    assert!(router.get_template("/dashboard").is_some());
    assert!(router.get_template("/dashboard/analytics").is_some());
}

// ============================================================================
// Parallel Routes (Next.js App Router Parity)
// ============================================================================

#[test]
fn test_nextjs_parallel_routes() {
    // Next.js: app/dashboard/@analytics/page.tsx → Slot "analytics"
    // Next.js: app/dashboard/@team/page.tsx → Slot "team"
    // Render multiple pages simultaneously in same layout

    let mut router = Router::new();

    let analytics = Route::from_path("pages/dashboard/@analytics/page.rhtml", "pages");
    assert!(analytics.is_parallel_route);
    assert_eq!(analytics.parallel_slot, Some("analytics".to_string()));
    assert_eq!(analytics.pattern, "/dashboard/page");

    let team = Route::from_path("pages/dashboard/@team/page.rhtml", "pages");
    assert!(team.is_parallel_route);
    assert_eq!(team.parallel_slot, Some("team".to_string()));

    router.add_route(analytics);
    router.add_route(team);

    let slots = router.get_parallel_routes("/dashboard/page").unwrap();
    assert_eq!(slots.len(), 2);
    assert!(slots.contains_key("analytics"));
    assert!(slots.contains_key("team"));
}

#[test]
fn test_nextjs_parallel_routes_with_dynamic_params() {
    // Next.js: app/products/@reviews/[id]/page.tsx
    // Parallel slot with dynamic segment

    let route = Route::from_path("pages/products/@reviews/[id].rhtml", "pages");

    assert!(route.is_parallel_route);
    assert_eq!(route.parallel_slot, Some("reviews".to_string()));
    assert_eq!(route.pattern, "/products/:id");
    assert_eq!(route.params, vec!["id"]);
}

// ============================================================================
// Intercepting Routes (Next.js App Router Parity)
// ============================================================================

#[test]
fn test_nextjs_intercepting_same_level() {
    // Next.js: app/feed/(.)/photo/[id]/page.tsx
    // Intercept /photo/[id] at same level as feed

    let route = Route::from_path("pages/feed/(.)/photo/[id].rhtml", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
    assert_eq!(route.intercept_target, Some("photo/[id]".to_string()));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_nextjs_intercepting_one_up() {
    // Next.js: app/feed/(..)/photo/[id]/page.tsx
    // Intercept /photo/[id] one level up

    let route = Route::from_path("pages/feed/(..)/photo/[id].rhtml", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::OneLevelUp));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_nextjs_intercepting_from_root() {
    // Next.js: app/feed/(...)/photo/[id]/page.tsx
    // Intercept /photo/[id] from root

    let route = Route::from_path("pages/feed/(...)/photo/[id].rhtml", "pages");

    assert!(route.is_intercepting);
    assert_eq!(route.intercept_level, Some(InterceptLevel::FromRoot));
    assert_eq!(route.pattern, "/feed/photo/:id");
}

#[test]
fn test_nextjs_modal_pattern() {
    // Next.js Modal Pattern:
    // - app/photo/[id]/page.tsx → Standalone page
    // - app/feed/(...)/photo/[id]/page.tsx → Modal when from feed

    let mut router = Router::new();

    // Standalone photo page
    router.add_route(Route::from_path("pages/photo/[id].rhtml", "pages"));

    // Intercepting route (modal)
    router.add_route(Route::from_path("pages/feed/(...)/photo/[id].rhtml", "pages"));

    // Both routes exist
    assert!(router.match_route("/photo/123").is_some());

    let intercept = router.get_intercepting_route("/feed/photo/:id").unwrap();
    assert_eq!(intercept.intercept_level, Some(InterceptLevel::FromRoot));
    assert_eq!(intercept.intercept_target, Some("photo/[id]".to_string()));
}

// ============================================================================
// Route Priority (Next.js Behavior)
// ============================================================================

#[test]
fn test_nextjs_route_priority() {
    // Next.js route priority (highest to lowest):
    // 1. Static routes
    // 2. Dynamic routes
    // 3. Catch-all routes

    let mut router = Router::new();

    let static_route = Route::from_path("pages/blog/hello.rhtml", "pages");
    let dynamic_route = Route::from_path("pages/blog/[slug].rhtml", "pages");
    let catchall_route = Route::from_path("pages/blog/[...slug].rhtml", "pages");

    // Check priorities
    assert!(static_route.priority < dynamic_route.priority);
    assert!(dynamic_route.priority < catchall_route.priority);

    router.add_route(dynamic_route);
    router.add_route(catchall_route);
    router.add_route(static_route); // Add out of order

    // Static should match first
    let m = router.match_route("/blog/hello").unwrap();
    assert_eq!(m.route.template_path, "pages/blog/hello.rhtml");

    // Dynamic should match
    let m = router.match_route("/blog/world").unwrap();
    assert_eq!(m.route.template_path, "pages/blog/[slug].rhtml");

    // Catch-all should match multiple segments
    let m = router.match_route("/blog/foo/bar/baz").unwrap();
    assert_eq!(m.route.template_path, "pages/blog/[...slug].rhtml");
}

// ============================================================================
// Complex Real-World Scenarios
// ============================================================================

#[test]
fn test_nextjs_complex_dashboard() {
    // Next.js: Complex dashboard with all features
    // - app/dashboard/layout.tsx
    // - app/dashboard/loading.tsx
    // - app/dashboard/error.tsx
    // - app/dashboard/@analytics/page.tsx
    // - app/dashboard/@team/page.tsx
    // - app/dashboard/@notifications/page.tsx
    // - app/dashboard/page.tsx

    let mut router = Router::new();

    router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/loading.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/_error.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@analytics/index.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@team/index.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/@notifications/index.rhtml", "pages"));
    router.add_route(Route::from_path("pages/dashboard/index.rhtml", "pages"));

    // Main page
    assert!(router.match_route("/dashboard").is_some());

    // Layout
    assert!(router.get_layout("/dashboard").is_some());

    // Loading
    assert!(router.get_loading_page("/dashboard").is_some());

    // Error
    assert!(router.get_error_page("/dashboard").is_some());

    // Parallel slots
    let slots = router.get_parallel_routes("/dashboard").unwrap();
    assert_eq!(slots.len(), 3);
    assert!(slots.contains_key("analytics"));
    assert!(slots.contains_key("team"));
    assert!(slots.contains_key("notifications"));
}

#[test]
fn test_nextjs_ecommerce_app() {
    // Next.js: E-commerce app structure
    // - app/(shop)/products/page.tsx
    // - app/(shop)/products/[id]/page.tsx
    // - app/(shop)/cart/page.tsx
    // - app/(marketing)/about/page.tsx
    // - app/(marketing)/contact/page.tsx

    let mut router = Router::new();

    // Shop group
    router.add_route(Route::from_path("pages/(shop)/products.rhtml", "pages"));
    router.add_route(Route::from_path("pages/(shop)/products/[id].rhtml", "pages"));
    router.add_route(Route::from_path("pages/(shop)/cart.rhtml", "pages"));

    // Marketing group
    router.add_route(Route::from_path("pages/(marketing)/about.rhtml", "pages"));
    router.add_route(Route::from_path("pages/(marketing)/contact.rhtml", "pages"));

    // All routes accessible without group names
    assert!(router.match_route("/products").is_some());
    assert!(router.match_route("/products/123").is_some());
    assert!(router.match_route("/cart").is_some());
    assert!(router.match_route("/about").is_some());
    assert!(router.match_route("/contact").is_some());

    // Group names not in URLs
    assert!(router.match_route("/(shop)/products").is_none());
}

#[test]
fn test_nextjs_photo_gallery_with_modals() {
    // Next.js: Photo gallery with modal intercepting
    // - app/photos/page.tsx → Grid view
    // - app/photos/[id]/page.tsx → Full page
    // - app/photos/(.)/[id]/page.tsx → Modal when from grid

    let mut router = Router::new();

    // Grid view
    router.add_route(Route::from_path("pages/photos/index.rhtml", "pages"));

    // Full page view
    router.add_route(Route::from_path("pages/photos/[id].rhtml", "pages"));

    // Modal view (intercept)
    router.add_route(Route::from_path("pages/photos/(.)/[id].rhtml", "pages"));

    // Grid exists
    assert!(router.match_route("/photos").is_some());

    // Full page exists
    let full = router.match_route("/photos/123").unwrap();
    assert_eq!(full.route.template_path, "pages/photos/[id].rhtml");

    // Modal interceptor exists
    let intercept = router.get_intercepting_route("/photos/:id").unwrap();
    assert!(intercept.is_intercepting);
}

// ============================================================================
// Metadata & Advanced Features
// ============================================================================

#[test]
fn test_metadata_and_constraints() {
    // rhtmx extension: Route metadata and parameter constraints
    // Not in Next.js but useful for Rust apps

    let mut router = Router::new();

    let route = Route::from_path("pages/users/[id:int].rhtml", "pages")
        .with_name("user_detail")
        .with_meta("permission", "user.read")
        .with_meta("cache", "60");

    router.add_route(route);

    // Named route exists
    assert!(router.get_route_by_name("user_detail").is_some());

    // Metadata accessible
    let r = router.match_route("/users/123").unwrap();
    assert_eq!(r.route.get_meta("permission"), Some(&"user.read".to_string()));
    assert_eq!(r.route.get_meta("cache"), Some(&"60".to_string()));
}

#[test]
fn test_redirects() {
    // rhtmx extension: Built-in redirects
    // Next.js uses next.config.js for this

    let mut router = Router::new();

    router.add_route(Route::redirect("/old-blog", "/blog", 301));
    router.add_route(Route::redirect("/old-products/:id", "/products/:id", 301));

    let m = router.match_route("/old-blog").unwrap();
    assert!(m.is_redirect());
    assert_eq!(m.redirect_target(), Some("/blog".to_string()));
    assert_eq!(m.redirect_status(), Some(301));

    let m = router.match_route("/old-products/123").unwrap();
    assert!(m.is_redirect());
    assert_eq!(m.redirect_target(), Some("/products/123".to_string()));
}

#[test]
fn test_route_aliases() {
    // rhtmx extension: Route aliases for i18n, legacy URLs
    // Next.js uses middleware for this

    let mut router = Router::new();

    let route = Route::from_path("pages/about.rhtml", "pages")
        .with_aliases(["/about-us", "/company", "/acerca-de"]);

    router.add_route(route);

    // All aliases work
    assert!(router.match_route("/about").is_some());
    assert!(router.match_route("/about-us").is_some());
    assert!(router.match_route("/company").is_some());
    assert!(router.match_route("/acerca-de").is_some());
}

// ============================================================================
// Performance & Functional Programming
// ============================================================================

#[test]
fn test_route_lookup_performance() {
    // All lookups should be O(1) or O(log n)
    let mut router = Router::new();

    // Add 1000 routes
    for i in 0..1000 {
        router.add_route(Route::from_path(&format!("pages/route{}.rhtml", i), "pages"));
    }

    // Match should be fast (sorted Vec, binary search-like behavior)
    let start = std::time::Instant::now();
    for i in 0..100 {
        router.match_route(&format!("/route{}", i));
    }
    let elapsed = start.elapsed();

    // Should complete in reasonable time
    assert!(elapsed.as_millis() < 100, "Routing too slow: {:?}", elapsed);
}

#[test]
fn test_functional_composition() {
    // Demonstrate functional programming patterns
    let route = Route::from_path("pages/users/[id].rhtml", "pages")
        .with_name("user_detail")
        .with_meta("permission", "user.read")
        .with_meta("cache", "60")
        .with_aliases(["/profile/:id", "/member/:id"]);

    assert_eq!(route.name, Some("user_detail".to_string()));
    assert_eq!(route.metadata.len(), 2);
    assert_eq!(route.aliases.len(), 2);
}
