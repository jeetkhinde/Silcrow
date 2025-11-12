# rhtmx-router

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A zero-dependency file-based router for Rust web frameworks.

## Features

- 🚀 **Zero dependencies** - Only uses Rust standard library
- 📁 **File-based routing** - Routes automatically generated from file structure
- 🎯 **Smart prioritization** - Static routes match before dynamic routes
- 🔀 **Dynamic segments** - `/users/:id`, `/posts/:slug`
- 🌟 **Catch-all routes** - `/docs/*path` matches any path depth
- ❓ **Optional parameters** - `/posts/:id?` matches with or without ID
- 🎨 **Layout support** - Special `_layout` files for nested layouts
- ⚠️ **Error pages** - Custom error pages with `_error` files
- 🔤 **Case-insensitive** - Optional case-insensitive matching
- ⚡ **Framework-agnostic** - Works with Axum, Actix, Rocket, Warp, etc.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rhtmx-router = "0.1.0"
```

Or use cargo add:

```bash
cargo add rhtmx-router
```

## Quick Start

### Basic Usage (Hardcoded Paths)

```rust
use rhtmx_router::{Router, Route};

fn main() {
    let mut router = Router::new();

    // Add routes from file paths
    router.add_route(Route::from_path("pages/index.rhtmx", "pages"));
    router.add_route(Route::from_path("pages/about.rhtmx", "pages"));
    router.add_route(Route::from_path("pages/users/[id].rhtmx", "pages"));
    router.add_route(Route::from_path("pages/docs/[...slug].rhtmx", "pages"));

    // Sort routes by priority (call this after adding all routes)
    router.sort_routes();

    // Match incoming requests
    if let Some(route_match) = router.match_route("/users/123") {
        println!("Matched: {}", route_match.route.pattern);
        println!("User ID: {}", route_match.params["id"]);
    }
}
```

### With Configuration (Framework-Agnostic)

The router is framework-agnostic - you can use any configuration system (TOML, YAML, JSON, env vars, etc.):

```rust
use rhtmx_router::{Router, Route};

fn main() {
    let mut router = Router::new();

    // Read configuration from YOUR system
    // (TOML, YAML, JSON, environment variables, hardcoded, etc.)
    let pages_dir = read_config().pages_dir; // ← You provide this

    // Router respects your configuration
    router.add_route(Route::from_path("app/index.rhtmx", &pages_dir));
    router.add_route(Route::from_path("app/users/[id].rhtmx", &pages_dir));

    router.sort_routes();

    // Routes are generated relative to configured directory
    let route = router.match_route("/users/123").unwrap();
    assert_eq!(route.route.pattern, "/users/:id");
}
```

### Example: Using with TOML Configuration

```rust
use rhtmx_router::{Router, Route};
use std::fs;

// Your config structure (can be anything)
#[derive(Deserialize)]
struct Config {
    routing: RoutingConfig,
}

#[derive(Deserialize)]
struct RoutingConfig {
    pages_dir: String,      // e.g., "pages", "app", "routes"
    case_insensitive: bool,
}

fn main() {
    // Load config from your TOML file
    let config_str = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_str).unwrap();

    // Create router with configured options
    let mut router = Router::with_case_insensitive(config.routing.case_insensitive);

    // Use configured directory
    let pages_dir = &config.routing.pages_dir;
    router.add_route(Route::from_path(
        &format!("{}/index.rhtmx", pages_dir),
        pages_dir
    ));

    router.sort_routes();
}
```

**Your `config.toml`:**
```toml
[routing]
pages_dir = "app"          # Use Next.js-style directory
case_insensitive = true    # Case-insensitive URL matching
```

## File Naming Convention

| File Path | Route Pattern | Description |
|-----------|---------------|-------------|
| `pages/index.rhtmx` | `/` | Root page |
| `pages/about.rhtmx` | `/about` | Static route |
| `pages/users/index.rhtmx` | `/users` | Section index |
| `pages/users/[id].rhtmx` | `/users/:id` | Dynamic segment |
| `pages/docs/[...slug].rhtmx` | `/docs/*slug` | Catch-all |
| `pages/posts/[id?].rhtmx` | `/posts/:id?` | Optional param |
| `pages/_layout.rhtmx` | `/` | Root layout |
| `pages/users/_layout.rhtmx` | `/users` | Section layout |
| `pages/_error.rhtmx` | `/` | Root error page |

## Examples

### Basic Routing

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::new();
router.add_route(Route::from_path("pages/about.rhtmx", "pages"));
router.sort_routes();

let result = router.match_route("/about").unwrap();
assert_eq!(result.route.pattern, "/about");
```

### Dynamic Routes

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::new();
router.add_route(Route::from_path("pages/users/[id].rhtmx", "pages"));
router.sort_routes();

let result = router.match_route("/users/42").unwrap();
assert_eq!(result.params["id"], "42");
```

### Catch-all Routes

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::new();
router.add_route(Route::from_path("pages/docs/[...slug].rhtmx", "pages"));
router.sort_routes();

let result = router.match_route("/docs/guide/intro").unwrap();
assert_eq!(result.params["slug"], "guide/intro");
```

### Optional Parameters

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::new();
router.add_route(Route::from_path("pages/posts/[id?].rhtmx", "pages"));
router.sort_routes();

// Matches with parameter
let result = router.match_route("/posts/123").unwrap();
assert_eq!(result.params["id"], "123");

// Matches without parameter
let result = router.match_route("/posts").unwrap();
assert!(result.params.get("id").is_none());
```

### Case-Insensitive Routing

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::with_case_insensitive(true);
router.add_route(Route::from_path("pages/about.rhtmx", "pages"));
router.sort_routes();

// All match the same route
assert!(router.match_route("/about").is_some());
assert!(router.match_route("/About").is_some());
assert!(router.match_route("/ABOUT").is_some());
```

## Architecture: Configuration Flow

This crate is **framework-agnostic** and designed to be used by web frameworks or applications.

```
┌──────────────────────────────────────────────────┐
│  Your Application (Axum, Actix, rhtmx, etc.)    │
│                                                   │
│  ┌─────────────┐                                 │
│  │ config.toml │  ← You choose the format        │
│  │ config.yaml │                                  │
│  │ env vars    │                                  │
│  └──────┬──────┘                                  │
│         │                                         │
│         ↓                                         │
│  Your Config Loader                              │
│  (toml, serde_yaml, env, hardcoded, etc.)       │
│         │                                         │
│         ↓                                         │
│  pages_dir = "app"  ← You extract the values     │
│  case_insensitive = true                         │
│         │                                         │
│         ↓                                         │
│  router.add_route(                               │
│    Route::from_path(path, &pages_dir)  ← Pass it│
│  )                                                │
│                                                   │
└─────────────────┬─────────────────────────────────┘
                  │
                  ↓
        ┌──────────────────────────┐
        │   rhtmx-router crate     │
        │                          │
        │  - Receives parameters   │
        │  - NO config reading     │
        │  - NO TOML/YAML deps     │
        │  - Pure routing logic    │
        └──────────────────────────┘
```

**Key Points:**
- ✅ Router does NOT read config files (stays framework-agnostic)
- ✅ YOU read config using your preferred system
- ✅ YOU pass configured values to the router
- ✅ Router respects your configuration
- ✅ Works with: TOML, YAML, JSON, env vars, hardcoded values, databases, etc.

### Integration with Axum

```rust
use axum::{Router as AxumRouter, routing::get, extract::Path};
use rhtmx_router::{Router, Route};
use std::collections::HashMap;

async fn handle_route(
    Path(path): Path<String>,
) -> String {
    let mut router = Router::new();

    // You decide where pages_dir comes from
    let pages_dir = "pages"; // Could be from config, env var, etc.

    router.add_route(Route::from_path("pages/users/[id].rhtmx", pages_dir));
    router.sort_routes();

    if let Some(route_match) = router.match_route(&format!("/{}", path)) {
        format!("User ID: {}", route_match.params["id"])
    } else {
        "Not found".to_string()
    }
}

#[tokio::main]
async fn main() {
    let app = AxumRouter::new()
        .route("/*path", get(handle_route));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### Integration with rhtmx Framework

If you're using the rhtmx framework (which uses this router):

```rust
use rhtmx_app::Config;  // rhtmx's config system
use rhtmx_router::{Router, Route};

// rhtmx reads rhtmx.toml for you
let config = Config::load("rhtmx.toml")?;

// Pass configured values to router
let mut router = Router::with_case_insensitive(config.routing.case_insensitive);
router.add_route(Route::from_path(
    &format!("{}/index.rhtmx", config.routing.pages_dir),
    &config.routing.pages_dir
));
```

See [rhtmx's configuration guide](https://github.com/jeetkhinde/rhtmx/blob/main/CONFIGURATION.md) for details.

## Route Priority

Routes are matched in order of priority:

1. **Static routes** (priority = 0) - Exact matches like `/about`, `/users/new`
2. **Optional parameters** (priority = depth + params) - `/posts/:id?`
3. **Required dynamic routes** (priority = depth + params + 1) - `/users/:id`
4. **Catch-all routes** (priority = 1000+) - `/docs/*path`

### Example Priority Order

```rust
let static_route = Route::from_path("pages/users/new.rhtmx", "pages");
// Priority: 0 (static always wins)

let optional_route = Route::from_path("pages/users/[id?].rhtmx", "pages");
// Priority: ~3

let dynamic_route = Route::from_path("pages/users/[id].rhtmx", "pages");
// Priority: ~4

let catchall_route = Route::from_path("pages/users/[...rest].rhtmx", "pages");
// Priority: 1000+
```

When matching `/users/new`:
- `static_route` matches first ✅
- Other routes never checked

When matching `/users/123`:
- `static_route` doesn't match
- `optional_route` matches ✅

## Special Files

### Layout Files (`_layout.rhtmx`)

Layout files define nested layouts. The router tracks them separately:

```rust
let mut router = Router::new();
router.add_route(Route::from_path("pages/_layout.rhtmx", "pages"));
router.add_route(Route::from_path("pages/users/_layout.rhtmx", "pages"));

// Get layout for a route
let layout = router.get_layout("/users/123").unwrap();
assert_eq!(layout.pattern, "/users");
```

### Error Pages (`_error.rhtmx`)

Error pages handle 404s and other errors:

```rust
let mut router = Router::new();
router.add_route(Route::from_path("pages/_error.rhtmx", "pages"));
router.add_route(Route::from_path("pages/api/_error.rhtmx", "pages"));

// Get error page for a route
let error = router.get_error_page("/api/users").unwrap();
assert_eq!(error.pattern, "/api"); // Section-specific error page
```

## Performance

- **Route sorting**: O(n log n) at startup
- **Route matching**: O(n) worst case, typically O(1) for static routes
- **Memory**: Minimal overhead, only stores route metadata
- **Zero allocations** during matching (except for parameter extraction)

## Testing

Run tests:

```bash
cargo test
```

Run with coverage:

```bash
cargo test --all-features
```

## Use Cases

- **Web frameworks** - Add file-based routing to any Rust web framework
- **Static site generators** - Map file structure to URLs
- **API gateways** - Route requests based on file structure
- **Documentation sites** - Perfect for nested documentation
- **Content management** - File-based content routing

## Comparison with Other Routers

| Feature | rhtmx-router | matchit | path-tree |
|---------|--------------|---------|-----------|
| File-based | ✅ | ❌ | ❌ |
| Zero deps | ✅ | ❌ | ❌ |
| Catch-all | ✅ | ✅ | ✅ |
| Optional params | ✅ | ❌ | ❌ |
| Layouts | ✅ | ❌ | ❌ |
| Error pages | ✅ | ❌ | ❌ |
| Case insensitive | ✅ | ❌ | ❌ |

## Contributing

Contributions welcome! Please check out the [rhtmx repository](https://github.com/jeetkhinde/rhtmx).

## License

MIT License - see [LICENSE](../LICENSE) file for details.

## Changelog

### 0.1.0 (2025-01-04)

- Initial release
- Static routes
- Dynamic segments
- Catch-all routes
- Optional parameters
- Layout support
- Error page support
- Case-insensitive routing
- Zero dependencies

## Acknowledgments

Part of the [rhtmx project](https://github.com/jeetkhinde/rhtmx) - a Rust-first SSR framework.
