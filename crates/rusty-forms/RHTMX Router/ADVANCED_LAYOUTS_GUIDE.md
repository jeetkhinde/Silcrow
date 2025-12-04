# Advanced Layouts Guide

Complete guide to advanced layout patterns and layout hierarchy control in RHTMX Router.

## Table of Contents

1. [Layout Hierarchy Overview](#layout-hierarchy-overview)
2. [The `_nolayout` Convention](#the-_nolayout-convention)
3. [LayoutOption Enum](#layoutoption-enum)
4. [Layout Hierarchy Skipping](#layout-hierarchy-skipping)
5. [Named Layouts](#named-layouts)
6. [Intercepting Routes](#intercepting-routes)
7. [Using Slots with Other Frameworks](#using-slots-with-other-frameworks)
8. [Dynamic Sidebar Loading](#dynamic-sidebar-loading)

---

## Layout Hierarchy Overview

RHTMX Router uses a **hierarchical layout system** where layouts are automatically inherited from parent directories. This creates a chain of layouts that can be customized at different levels.

### Default Behavior: Layout Inheritance

Layouts are discovered by walking up the directory hierarchy:

```
pages/
├── _layout.rsx              # Root layout (applies to all pages)
├── page.rsx                # Uses root layout
└── dashboard/
    ├── _layout.rsx          # Dashboard layout (overrides root)
    ├── page.rsx            # Uses dashboard layout
    └── admin/
        ├── _layout.rsx      # Admin layout (overrides dashboard)
        └── users.rsx        # Uses admin layout
```

When matching a route, the router walks up the hierarchy:

```rust
router.get_layout("/dashboard/admin/users")
// Checks in order:
// 1. /dashboard/admin → FOUND (uses admin layout)
// 2. /dashboard → FOUND (but uses admin's instead)
// 3. / → FOUND (but uses admin's instead)
// Result: Admin layout
```

---

## The `_nolayout` Convention

The `_nolayout` marker file tells the router to **stop layout inheritance** at that point.

### Use Cases

1. **HTMX Partials**: Return HTML fragments without the full page layout
2. **Standalone Pages**: Pages that shouldn't inherit layouts (print pages, raw content)
3. **API Response Pages**: Return only content, no layout wrapper
4. **Modal/Overlay Content**: Content for dynamic insertion

### Creating a No-Layout Zone

Create an empty file named `_nolayout` in the directory where you want to stop layout inheritance:

```
pages/
├── _layout.rsx           # Root layout
├── dashboard/
│   ├── _layout.rsx       # Dashboard layout
│   ├── _nolayout           # 👈 No layout marker
│   └── api/
│       └── users.json.rsx    # No layout applied
```

Any route under `dashboard/` will **not use layouts**, even if parent layouts exist.

### Router Behavior

```rust
let mut router = Router::new();

// Add layouts
router.add_route(Route::from_path("pages/_layout.rsx", "pages"));
router.add_route(Route::from_path("pages/dashboard/_layout.rsx", "pages"));
router.add_route(Route::from_path("pages/dashboard/_nolayout", "pages"));

// Result:
router.get_layout("/dashboard/api/users")    // None (blocked by _nolayout)
router.get_layout("/other/page")             // Some(root layout)
router.get_layout("/dashboard/settings")     // None (under _nolayout)
```

### Detecting in Code

```rust
use rhtmx_router::{Route, Router};

let route = Route::from_path("pages/dashboard/_nolayout", "pages");
assert!(route.is_nolayout_marker);

// In router context:
router.add_route(Route::from_path("pages/dashboard/_nolayout", "pages"));
let is_no_layout_zone = router.nolayout_patterns.contains("/dashboard");
```

### Builder Method (Programmatic)

You can also explicitly set no layout via the builder:

```rust
let route = Route::from_path("pages/dashboard/api/users.rsx", "pages")
    .with_no_layout();  // Explicitly remove layout

assert_eq!(route.layout_option, LayoutOption::None);
```

---

## LayoutOption Enum

The `LayoutOption` enum provides fine-grained control over layout resolution:

```rust
pub enum LayoutOption {
    Inherit,              // Default - walk up hierarchy
    None,                 // No layout
    Root,                 // Root layout only
    Named(String),        // Specific named layout
    Pattern(String),      // Specific path pattern
}
```

### Usage Examples

#### 1. Inherit (Default)

```rust
use rhtmx_router::{Route, LayoutOption};

let route = Route::from_path("pages/users/profile.rsx", "pages");
assert_eq!(route.layout_option, LayoutOption::Inherit);

// Inheritance walks up: /users/profile → /users → /
```

#### 2. None - No Layout

```rust
let route = Route::from_path("pages/api/data.rsx", "pages")
    .with_no_layout();

// Router will not apply any layout to this route
```

#### 3. Root - Skip Intermediate Layouts

```rust
let route = Route::from_path("pages/dashboard/admin/report.rsx", "pages")
    .with_root_layout();

// Even if /dashboard and /dashboard/admin have layouts,
// this route uses ONLY the root (/) layout
```

#### 4. Named - Use Specific Layout

```rust
let route = Route::from_path("pages/vendors/settings.rsx", "pages")
    .with_named_layout("vendor");

// Router looks for _layout.vendor.rsx instead of walking hierarchy
```

#### 5. Pattern - Use Layout at Specific Path

```rust
let route = Route::from_path("pages/api/v2/users.rsx", "pages")
    .with_layout_pattern("/api");

// Uses layout from /api, ignoring /api/v2
```

---

## Layout Hierarchy Skipping

### Problem: Skip Immediate Parent, Use Grandparent Layout

You want to skip the immediate parent layout but accept the grandparent or root layout.

**Directory Structure:**

```
pages/
├── _layout.rsx                    # Root layout (main site)
├── dashboard/
│   ├── _layout.rsx               # Dashboard layout (sidebar + content)
│   └── print/
│       ├── report.rsx            # Should use root, not dashboard!
│       └── invoice.rsx
```

**Problem:** By default, `/dashboard/print/report` uses dashboard layout.

**Solution 1: Use `Root` LayoutOption**

```rust
let route = Route::from_path("pages/dashboard/print/report.rsx", "pages")
    .with_root_layout();

// Forces root layout only, skips dashboard layout
```

**Solution 2: Use `_nolayout` with Builder Method**

```rust
let route = Route::from_path("pages/dashboard/print/report.rsx", "pages")
    .with_no_layout();

// Returns no layout at all (for HTMX partials or API endpoints)
```

**Solution 3: Use `Pattern` LayoutOption**

```rust
let route = Route::from_path("pages/dashboard/print/report.rsx", "pages")
    .with_layout_pattern("/");  // Explicit root path

// Uses layout at /, skips /dashboard
```

### Complete Example

```rust
use rhtmx_router::{Router, Route, LayoutOption};

let mut router = Router::new();

// Add layouts
router.add_route(Route::from_path("pages/_layout.rsx", "pages"));
router.add_route(Route::from_path("pages/dashboard/_layout.rsx", "pages"));

// Add routes with different layout options
router.add_route(
    Route::from_path("pages/dashboard/settings.rsx", "pages")
        .with_layout_option(LayoutOption::Inherit)  // Uses dashboard layout
);

router.add_route(
    Route::from_path("pages/dashboard/print/report.rsx", "pages")
        .with_root_layout()  // Skips dashboard, uses root only
);

// Verify behavior
let settings_layout = router.get_layout("/dashboard/settings");
assert_eq!(settings_layout.unwrap().pattern, "/dashboard");

let report_layout = router.get_layout("/dashboard/print/report");
assert_eq!(report_layout.unwrap().pattern, "/");  // Root only!
```

### Comparison of Approaches

| Approach | Skip Parent | Use Root | No Layout | Use Case |
|----------|-----------|----------|-----------|----------|
| `Inherit` | ❌ | Uses if no intermediate | ❌ | Normal pages |
| `None` | ✅ | ❌ | ✅ | HTMX partials, APIs |
| `Root` | ✅ | ✅ | ❌ | Print pages, special layouts |
| `Named` | ✅ | ❌ | ❌ | Alternative layouts |
| `Pattern` | ✅ | Customizable | ❌ | Specific path layouts |

---

## Named Layouts

Named layouts let you create multiple layout options and select them by name.

### Creating Named Layouts

File naming convention: `_layout.{name}.rsx`

```
pages/
├── _layout.rsx              # Default layout
├── _layout.admin.rsx        # Admin layout
├── _layout.public.rsx       # Public layout
├── dashboard/
│   ├── admin/
│   │   └── users.rsx        # Will use admin layout
│   └── public/
│       └── faq.rsx          # Will use public layout
```

### Selecting Named Layouts

```rust
use rhtmx_router::{Route, LayoutOption};

// Route 1: Uses admin layout
let admin_route = Route::from_path("pages/dashboard/admin/users.rsx", "pages")
    .with_named_layout("admin");

// Route 2: Uses public layout
let public_route = Route::from_path("pages/dashboard/public/faq.rsx", "pages")
    .with_named_layout("public");

// Route 3: Uses default layout
let default_route = Route::from_path("pages/dashboard/settings.rsx", "pages");
```

### Detecting Named Layouts

```rust
let route = Route::from_path("pages/_layout.admin.rsx", "pages");
assert!(route.is_layout);
assert_eq!(route.layout_name, Some("admin".to_string()));
```

---

## Intercepting Routes

Intercepting routes allow you to create modal, overlay, or dialog content that doesn't navigate, but renders alongside the current page (Next.js parallel routes pattern).

### Syntax

Intercepting routes use parentheses to indicate interception level:

```
(.)      → Same directory level
(..)     → One level up
(...)    → Two levels up (from root relative paths)
(....)   → Two levels up
```

### Example: Modal Dialogs

```
pages/
├── dashboard/
│   ├── layout.rsx
│   ├── page.rsx
│   └── (.) users/
│       ├── modal.rsx        # Shows as modal when /dashboard/users/modal
│       └── details.rsx
```

### Detecting Intercepting Routes

```rust
use rhtmx_router::{Route, InterceptLevel};

let route = Route::from_path("pages/dashboard/(.)users/modal.rsx", "pages");
assert!(route.is_intercepting);
assert_eq!(route.intercept_level, Some(InterceptLevel::SameLevel));
assert_eq!(route.intercept_target, Some("users".to_string()));
```

### Using with Layout Control

```rust
let modal_route = Route::from_path("pages/dashboard/(.)users/modal.rsx", "pages")
    .with_no_layout();  // Modal content without layout

// Combine intercepting with layout options
let intercept_with_root = Route::from_path("pages/dashboard/(.)users/modal.rsx", "pages")
    .with_root_layout();  // Modal uses root layout only
```

---

## Using Slots with Other Frameworks

RHTMX slots are **plain Rust structs with a builder pattern** - they work with any web framework!

### With Rocket Web Framework

```rust
#[macro_use]
extern crate rocket;

use rhtmx::{html, Html};
use rhtmx::layouts;

#[get("/")]
fn index() -> Html {
    let content = html! {
        <div class="hero">
            <h1>"Welcome"</h1>
            <p>"Built with Rocket + RHTMX"</p>
        </div>
    };

    layouts::root::layout(
        content,
        layouts::root::Slots::new("Home")
            .description("Welcome to our Rocket app")
    )
}

#[get("/admin/dashboard")]
fn admin_dashboard() -> Html {
    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
            <p>"Admin statistics"</p>
        </div>
    };

    layouts::admin::layout(
        content,
        layouts::admin::Slots::new("Admin Dashboard")
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, admin_dashboard])
}
```

### With Actix-Web Framework

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use rhtmx::{html, Html};
use rhtmx::layouts;

async fn index() -> HttpResponse {
    let content = html! {
        <div class="hero">
            <h1>"Welcome"</h1>
            <p>"Built with Actix-Web + RHTMX"</p>
        </div>
    };

    let page = layouts::root::layout(
        content,
        layouts::root::Slots::new("Home")
            .description("Welcome to our Actix app")
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(page.0)  // .0 unwraps the Html wrapper
}

async fn admin_dashboard() -> HttpResponse {
    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
        </div>
    };

    let page = layouts::admin::layout(
        content,
        layouts::admin::Slots::new("Dashboard")
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(page.0)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/admin/dashboard", web::get().to(admin_dashboard))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### With Warp Framework

```rust
use warp::Filter;
use rhtmx::{html, Html};
use rhtmx::layouts;

#[tokio::main]
async fn main() {
    let index = warp::path::end()
        .map(|| {
            let content = html! {
                <div class="hero">
                    <h1>"Welcome"</h1>
                </div>
            };

            layouts::root::layout(
                content,
                layouts::root::Slots::new("Home")
            ).0
        });

    let admin = warp::path!("admin" / "dashboard")
        .map(|| {
            let content = html! {
                <div class="dashboard">
                    <h1>"Dashboard"</h1>
                </div>
            };

            layouts::admin::layout(
                content,
                layouts::admin::Slots::new("Dashboard")
            ).0
        });

    let routes = index.or(admin);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

### Key Points

1. **Slots are just Rust structs** - no framework-specific magic
2. **Builder pattern works everywhere** - `.new()` and `.field()`
3. **Layout function returns Html** - works with any response type
4. **Unwrap with `.0`** - The `Html` wrapper is a newtype (`Html(String)`)

---

## Dynamic Sidebar Loading

Load different sidebar content dynamically based on user role, settings, or context.

### Approach 1: Dynamic Sidebar Builder

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;

// Sidebar configuration based on user role
enum UserRole {
    Admin,
    Editor,
    Viewer,
}

fn build_sidebar(role: UserRole) -> Html {
    match role {
        UserRole::Admin => html! {
            <nav class="sidebar">
                <a href="/admin/users">"Users"</a>
                <a href="/admin/settings">"Settings"</a>
                <a href="/admin/logs">"Audit Logs"</a>
            </nav>
        },
        UserRole::Editor => html! {
            <nav class="sidebar">
                <a href="/dashboard/posts">"Posts"</a>
                <a href="/dashboard/media">"Media"</a>
            </nav>
        },
        UserRole::Viewer => html! {
            <nav class="sidebar">
                <a href="/dashboard">"Dashboard"</a>
                <a href="/dashboard/reports">"Reports"</a>
            </nav>
        },
    }
}

pub fn admin_page(content: Html, role: UserRole) -> Html {
    let sidebar = build_sidebar(role);

    admin::layout(
        content,
        admin::Slots::new("Admin")
            .sidebar(sidebar)
    )
}

// Usage in handler:
let user_role = UserRole::Admin;
let page = admin_page(content, user_role);
```

### Approach 2: Database-Driven Sidebar

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;

struct NavItem {
    label: String,
    href: String,
    icon: Option<String>,
}

async fn load_sidebar_for_user(user_id: i32) -> Vec<NavItem> {
    // Query database for user's navigation items
    // based on permissions, settings, etc.
    sqlx::query_as::<_, NavItem>(
        "SELECT label, href, icon FROM user_nav WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_all(&db)
    .await
    .unwrap_or_default()
}

fn render_sidebar_html(items: Vec<NavItem>) -> Html {
    let mut nav_items = String::new();

    for item in items {
        let icon_html = item.icon
            .map(|i| format!(r#"<span class="icon">{}</span>"#, i))
            .unwrap_or_default();

        nav_items.push_str(&format!(
            r#"<li><a href="{}">{}{}</a></li>"#,
            item.href, icon_html, item.label
        ));
    }

    Html(format!(
        r#"<nav class="sidebar"><ul>{}</ul></nav>"#,
        nav_items
    ))
}

// In handler:
async fn admin_dashboard(user_id: i32) -> Html {
    let nav_items = load_sidebar_for_user(user_id).await;
    let sidebar = render_sidebar_html(nav_items);

    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
        </div>
    };

    admin::layout(
        content,
        admin::Slots::new("Dashboard")
            .sidebar(sidebar)
    )
}
```

### Approach 3: Sidebar with Context/State

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;
use std::sync::Arc;

pub struct AppContext {
    user_permissions: Vec<String>,
    sidebar_sections: Vec<String>,
}

fn build_sidebar_from_context(ctx: Arc<AppContext>) -> Html {
    let mut sidebar_html = String::from(r#"<nav class="sidebar">"#);

    for section in &ctx.sidebar_sections {
        // Only show if user has permission
        if ctx.user_permissions.contains(&format!("view:{}", section)) {
            sidebar_html.push_str(&format!(
                r#"<a href="/admin/{}" class="nav-link">{}</a>"#,
                section, section
            ));
        }
    }

    sidebar_html.push_str("</nav>");
    Html(sidebar_html)
}

pub fn render_admin_page(
    content: Html,
    ctx: Arc<AppContext>,
) -> Html {
    let sidebar = build_sidebar_from_context(ctx);

    admin::layout(
        content,
        admin::Slots::new("Admin")
            .sidebar(sidebar)
    )
}
```

### Approach 4: Conditional Slots Builder

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;

pub struct SidebarConfig {
    show_users: bool,
    show_settings: bool,
    show_audit: bool,
}

impl SidebarConfig {
    pub fn build(&self) -> Html {
        let mut items = Vec::new();

        if self.show_users {
            items.push(r#"<a href="/admin/users">"Users"</a>"#);
        }
        if self.show_settings {
            items.push(r#"<a href="/admin/settings">"Settings"</a>"#);
        }
        if self.show_audit {
            items.push(r#"<a href="/admin/logs">"Audit Logs"</a>"#);
        }

        let sidebar_html = format!(
            r#"<nav class="sidebar">{}</nav>"#,
            items.join("")
        );

        Html(sidebar_html)
    }
}

// Usage:
let config = SidebarConfig {
    show_users: true,
    show_settings: true,
    show_audit: user.is_super_admin,
};

let page = admin::layout(
    content,
    admin::Slots::new("Admin")
        .sidebar(config.build())
);
```

### Complete Real-World Example

```rust
use rhtmx::{html, Html, get, Ok, OkResponse};
use rhtmx::layouts::admin;

#[derive(Clone)]
struct User {
    id: i32,
    name: String,
    role: String,
    permissions: Vec<String>,
}

fn build_sidebar_for_user(user: &User) -> Html {
    let can_manage_users = user.permissions.contains(&"manage:users".to_string());
    let can_manage_settings = user.permissions.contains(&"manage:settings".to_string());

    let mut nav_html = String::from(r#"<nav class="sidebar"><ul>"#);

    nav_html.push_str(r#"<li><a href="/admin">Dashboard</a></li>"#);

    if can_manage_users {
        nav_html.push_str(r#"<li><a href="/admin/users">Users</a></li>"#);
    }

    if can_manage_settings {
        nav_html.push_str(r#"<li><a href="/admin/settings">Settings</a></li>"#);
    }

    nav_html.push_str("</ul></nav>");
    Html(nav_html)
}

#[get("/admin")]
fn admin_dashboard(user: User) -> OkResponse {
    let content = html! {
        <div class="dashboard">
            <h1>"Welcome, " {user.name}</h1>
            <p>"Role: " {user.role}</p>
        </div>
    };

    let sidebar = build_sidebar_for_user(&user);

    Ok().html(admin::layout(
        content,
        admin::Slots::new("Admin Dashboard")
            .sidebar(sidebar)
    ))
}
```

---

## Best Practices Summary

### Layout Decisions Flowchart

1. **Inheriting from parent?** → Use `LayoutOption::Inherit` (default)
2. **No layout needed?** → Use `LayoutOption::None` or `_nolayout` file
3. **Skip immediate parent?** → Use `LayoutOption::Root` or `LayoutOption::Pattern`
4. **Need specific layout?** → Use `LayoutOption::Named("name")`
5. **HTMX partial?** → Use `LayoutOption::None`
6. **Modal/Dialog?** → Use `with_no_layout()` or `with_root_layout()`

### Documentation Checklist

- ✅ Document `_nolayout` convention in README
- ✅ Add builder method examples to API docs
- ✅ Create advanced patterns guide
- ✅ Show framework integration examples
- ✅ Provide dynamic sidebar patterns

---

## See Also

- [LayoutOption API](lib.rs) - Technical reference
- [Router README](README.md) - Basic routing guide
- [RHTMX LAYOUTS.md](../rhtmx/docs/LAYOUTS.md) - Layout rendering with slots
