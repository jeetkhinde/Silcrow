# Layout Control - Quick Reference

One-page cheat sheet for all layout configuration options in RHTMX Router.

## File-Based Convention

### _nolayout Marker
```
pages/
├── _layout.rsx
└── api/
    ├── _nolayout     ← Blocks all layouts in this directory
    └── users.rsx   → No layout applied
```

### Named Layouts
```
pages/
├── _layout.rsx              # Default
├── _layout.admin.rsx        # Alternative
└── admin/
    └── dashboard.rsx
```

---

## Builder Methods (Programmatic)

### Basic LayoutOption Methods

| Method | Result | Use Case |
|--------|--------|----------|
| `.with_no_layout()` | No layout | HTMX partials, API responses |
| `.with_root_layout()` | Root layout only | Print pages, special layouts |
| `.with_named_layout("x")` | Named layout | Alternative layouts |
| `.with_layout_pattern("/x")` | Specific path layout | API versioning, sections |

### Full LayoutOption Control

```rust
use rhtmx_router::LayoutOption;

LayoutOption::Inherit        // (default) Walk up hierarchy
LayoutOption::None           // No layout
LayoutOption::Root           // Root only
LayoutOption::Named("name")  // Specific layout
LayoutOption::Pattern("/x")  // Specific path
```

---

## Quick Decision Tree

```
START
  │
  ├─ Need layout? ─────────────────────┐
  │  NO                               YES
  │                                    │
  │  ├─ HTMX partial?                 ├─ Skip immediate parent?
  │  │  YES → with_no_layout()        │  YES → with_root_layout()
  │  │                                │  NO → keep Inherit (default)
  │  ├─ API endpoint?                 │
  │  │  YES → with_no_layout()        ├─ Alternative layout?
  │  │                                │  YES → with_named_layout("x")
  │  ├─ Modal/dialog?                 │
  │  │  YES → with_no_layout()        └─ DONE ✓
  │  │
  │  └─ Raw content only?
  │     YES → with_no_layout()
  │
  └─ DONE ✓
```

---

## Real-World Scenarios

### Print Page
```rust
// File: pages/dashboard/print/report.rsx
Route::from_path("pages/dashboard/print/report.rsx", "pages")
    .with_root_layout()  // Skips dashboard sidebar
```

### API Endpoint
```rust
// File: pages/api/v2/users.rsx
Route::from_path("pages/api/v2/users.rsx", "pages")
    .with_no_layout()  // No HTML wrapper
```

### HTMX Partial
```rust
// File: pages/dashboard/users/list.rsx
Route::from_path("pages/dashboard/users/list.rsx", "pages")
    .with_no_layout()  // Returns fragment only
```

### Admin Section
```rust
// File: pages/admin/dashboard.rsx
Route::from_path("pages/admin/dashboard.rsx", "pages")
    .with_named_layout("admin")  // Uses _layout.admin.rsx
```

### Modal/Dialog
```rust
// File: pages/dashboard/(.)users/modal.rsx
Route::from_path("pages/dashboard/(.)users/modal.rsx", "pages")
    .with_no_layout()  // Modal content without layout
```

---

## Layout Resolution Examples

### Example 1: Default Inheritance
```
Directory: /dashboard/admin/settings

Checks in order:
1. /dashboard/admin/settings  → Not found
2. /dashboard/admin           → Not found
3. /dashboard                 → FOUND! Use /dashboard layout
```

### Example 2: Skip to Root
```rust
route.with_root_layout()

Directory: /dashboard/admin/settings

Skips: /dashboard/admin, /dashboard
Uses: / (root layout)
```

### Example 3: No Layout
```rust
route.with_no_layout()

Directory: /dashboard/admin/settings

Result: No layout, just content
```

### Example 4: Named Layout
```rust
route.with_named_layout("admin")

Directory: /dashboard/settings

Looks for: _layout.admin.rsx
Uses: That layout regardless of file location
```

---

## Builder Method Chaining

Combine multiple builder methods:

```rust
Route::from_path("pages/users/profile.rsx", "pages")
    .with_named_layout("vendor")
    .with_meta("title", "User Profile")
    .with_meta("permission", "users.read")
    .with_alias("/user")
    .with_alias("/profile")
```

---

## Router Queries

```rust
let mut router = Router::new();

// Get layout for path
let layout = router.get_layout("/dashboard/admin");

// Get error page
let error = router.get_error_page("/api/users");

// Match route
let route_match = router.match_route("/users/123");

// Check for nolayout zone
let is_no_layout = router.nolayout_patterns.contains("/api");
```

---

## Slots Integration

### With Root Layout
```rust
use rhtmx::layouts::root;

let page = root::layout(
    content,
    root::Slots::new("Page Title")
        .description("Meta description")
        .header(custom_header)
        .footer(custom_footer)
);
```

### With Admin Layout
```rust
use rhtmx::layouts::admin;

let page = admin::layout(
    content,
    admin::Slots::new("Dashboard")
        .sidebar(sidebar_html)
        .breadcrumbs(breadcrumbs_html)
);
```

### Dynamic Sidebar Example
```rust
let sidebar = match user.role {
    "admin" => admin_sidebar(),
    "user" => user_sidebar(),
    _ => default_sidebar(),
};

admin::layout(content, admin::Slots::new("Title").sidebar(sidebar))
```

---

## Common Patterns

### Multi-Level Hierarchy
```
pages/
├── _layout.rsx           # Level 0
├── dashboard/
│   ├── _layout.rsx       # Level 1 (overrides root)
│   ├── admin/
│   │   ├── _layout.rsx   # Level 2 (overrides dashboard)
│   │   └── users.rsx     # Uses level 2
│   └── settings.rsx      # Uses level 1
```

### Multiple Layout Options
```
pages/
├── _layout.rsx           # Default
├── _layout.admin.rsx     # For admin pages
├── _layout.public.rsx    # For public pages
└── admin/
    └── dashboard.rsx     # Route applies named layout
```

### No-Layout Zone
```
pages/
├── _layout.rsx
└── api/
    ├── _nolayout           # Marker
    ├── users.rsx         # No layout
    └── posts.rsx         # No layout
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Layout applies when it shouldn't | Use `with_no_layout()` or `_nolayout` file |
| Wrong layout applies | Check hierarchy, use `with_root_layout()` or `with_named_layout()` |
| Missing sidebar/header | Ensure slots are passed to layout function |
| HTMX swap includes layout | Use `with_no_layout()` on route |
| Print page has sidebar | Use `with_root_layout()` to skip intermediate |

---

## Documentation Links

- **[ADVANCED_LAYOUTS_GUIDE.md](ADVANCED_LAYOUTS_GUIDE.md)** - Comprehensive guide (350+ lines)
- **[SLOTS_FRAMEWORK_INTEGRATION.md](../rhtmx/docs/SLOTS_FRAMEWORK_INTEGRATION.md)** - Framework examples (400+ lines)
- **[README.md](README.md#builder-methods-for-layout-control)** - Main reference
- **[API Docs](lib.rs)** - Detailed rustdoc

---

## Copy-Paste Templates

### Template 1: Basic Route
```rust
Route::from_path("pages/about.rsx", "pages")
```

### Template 2: No Layout Route
```rust
Route::from_path("pages/api/data.rsx", "pages")
    .with_no_layout()
```

### Template 3: Alternative Layout
```rust
Route::from_path("pages/admin/dashboard.rsx", "pages")
    .with_named_layout("admin")
```

### Template 4: Print/Special Page
```rust
Route::from_path("pages/dashboard/print/report.rsx", "pages")
    .with_root_layout()
```

### Template 5: Intercepting Route
```rust
Route::from_path("pages/dashboard/(.)users/modal.rsx", "pages")
    .with_no_layout()
```

---

## Performance Notes

- **Inherit (default)**: O(n) where n = depth of hierarchy (lazy evaluation)
- **Root**: O(1) - direct lookup to root
- **Named**: O(1) - hash lookup by name
- **Pattern**: O(1) - direct path lookup
- **None**: No layout lookup needed (fastest)

For most applications (< 1000 routes), hierarchy depth is 3-5 levels, so all options are performant.

---

Last updated: RHTMX Router v0.1.0
