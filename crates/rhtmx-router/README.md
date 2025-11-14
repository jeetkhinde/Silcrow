# RHTMX Router

`rhtmx-router` is a file-system-based router for Rust web applications, designed for the RHTMX framework. It automatically generates URL routes from your file structure in the `pages` directory, supporting static, dynamic, and catch-all routes with a priority system for correct matching.

## Features

- **File-System-Based Routing**: Convention over configuration. Create files and folders in your `pages` directory to define your application's routes.
- **Multiple Route Types**: Supports static paths, dynamic segments, optional parameters, and catch-all routes.
  - Static routes: `/about`, `/contact`
  - Dynamic routes: `/users/[id]` → `/users/:id`
  - Optional parameters: `/posts/[id?]` → `/posts/:id?` (matches with or without parameter)
  - Catch-all routes: `/docs/[...slug]` → `/docs/*slug`
- **Hierarchical Layouts**: Define shared layouts for different sections of your site using `_layout.rhtmx` files. The router automatically finds the most specific layout for any given route.
- **Hierarchical Error Pages**: Specify custom error pages with `_error.rhtmx` that act as error boundaries for their directory and subdirectories.
- **Intelligent Route Prioritization**: Automatically sorts routes by priority to ensure correct matching: static > optional params > required dynamic > catch-all.
- **Case-Sensitive/Insensitive Routing**: Choose between exact case matching or case-insensitive URLs.
- **Parameter Extraction**: Automatically extracts dynamic parameters as a HashMap.
- **Zero External Dependencies**: Built entirely with Rust's standard library.

## How it Works

The router scans a `pages` directory and converts the file paths into route patterns.

### File-to-Route Mapping

- `pages/index.rhtmx` → `/`
- `pages/about.rhtmx` → `/about`
- `pages/users/index.rhtmx` → `/users`
- `pages/users/[id].rhtmx` → `/users/:id` (dynamic route)
- `pages/posts/[id?].rhtmx` → `/posts/:id?` (optional parameter)
- `pages/docs/[...slug].rhtmx` → `/docs/*slug` (catch-all route)
- `pages/users/_layout.rhtmx` → Layout for `/users` and subdirectories
- `pages/_error.rhtmx` → Error page for entire app

### Special Files

- **`_layout.rhtmx`**: Provides a layout for its directory and all subdirectories.
- **`_error.rhtmx`**: Acts as an error boundary for its directory and subdirectories.

### Layout and Error Page Inheritance

When looking for a layout or error page for a specific route, the router follows a fallback chain to find the most specific one:

1.  **Exact Match**: Looks for `_layout.rhtmx` or `_error.rhtmx` in the same directory as the matched route file.
2.  **Parent Section**: If not found, it searches in the parent directory, and so on, up to the root `pages` directory.
3.  **Root**: Finally, it falls back to the `_layout.rhtmx` or `_error.rhtmx` in the root of the `pages` directory.

For example, for a route `/dashboard/settings/profile`, it would look for a layout in this order:
1. `pages/dashboard/settings/_layout.rhtmx`
2. `pages/dashboard/_layout.rhtmx`
3. `pages/_layout.rhtmx`

### Route Matching Priority

The router automatically assigns a priority to each route to resolve ambiguity. Routes are sorted and matched in this order:

1. **Static routes** (e.g., `/about`) — priority: 0 (highest, most specific)
2. **Optional dynamic routes** (e.g., `/posts/:id?`) — priority: depth
3. **Required dynamic routes** (e.g., `/users/:id`) — priority: depth + 1
4. **Catch-all routes** (e.g., `/docs/*slug`) — priority: 1000+ (lowest, least specific)

**Priority Formula:**

```text
static route:        priority = 0
optional params:     priority = dynamic_count + depth
required dynamic:    priority = dynamic_count + depth + 1
catch-all:           priority = 1000 + depth
```

Lower values = higher priority (matched first).

### Optional vs. Required Parameters

Optional parameters (e.g., `/posts/[id?]`) match both with and without the parameter:

- `/posts` ✓ (matches, `id` is absent)
- `/posts/123` ✓ (matches, `id = "123"`)
- `/posts/123/comments` ✓ (matches, `id = "123"`)

The router intelligently distinguishes between optional parameters and the next static segment to avoid ambiguity.

### Catch-All Routes

Catch-all routes capture one or more remaining path segments:

- `/docs` ✓ (matches, `slug = ""`)
- `/docs/guide` ✓ (matches, `slug = "guide"`)
- `/docs/guide/intro` ✓ (matches, `slug = "guide/intro"`)

### Trailing Slash and Case Sensitivity

- **Trailing Slashes**: Automatically normalized. `/about/` matches the pattern `/about`.
- **Case Sensitivity**: When case-insensitive mode is enabled, only static segments are affected. Dynamic segment values preserve their original case.

## Usage

Here's a basic example of how to use the `Router`. In a real RHTMX application, this is handled by the `TemplateLoader`.

```rust
use rhtmx_router::{Route, Router};

fn main() {
    // 1. Create a new router
    let mut router = Router::with_case_insensitive(false); // Case-sensitive

    // 2. Create routes from file paths
    // In a real app, you would scan a directory.
    let pages_dir = "pages";
    let route_paths = vec![
        "pages/index.rhtmx",
        "pages/about.rhtmx",
        "pages/users/_layout.rhtmx",
        "pages/users/[id].rhtmx",
        "pages/docs/[...slug].rhtmx",
    ];

    for path in route_paths {
        let route = Route::from_path(path, pages_dir);
        println!(
            "Discovered Route: pattern='{}', priority={}, is_layout={}, file='{}'",
            route.pattern, route.priority, route.is_layout, route.template_path
        );
        router.add_route(route);
    }

    // 3. Sort routes by priority
    // This is crucial for correct matching.
    router.sort_routes();

    // 4. Match an incoming request path
    let request_path = "/users/123";
    if let Some(matched) = router.match_route(request_path) {
        println!("\nMatched '{}' to pattern '{}'", request_path, matched.pattern);
        println!("Params: {:?}", matched.params);

        // Find the appropriate layout for the matched route
        // The `get_layout` method takes the request path and finds the best layout.
        if let Some(layout_route) = router.get_layout(matched.pattern) {
            println!("Using layout with pattern: '{}'", layout_route.pattern);
        } else {
            println!("No specific layout found for this route.");
        }
    } else {
        println!("\nNo match found for '{}'", request_path);
    }

    let request_path_about = "/about";
    if let Some(matched) = router.match_route(request_path_about) {
        println!("\nMatched '{}' to pattern '{}'", request_path_about, matched.pattern);
    }
}
```

### `Route` Struct

The `Route` struct contains information about a single route.

```rust
pub struct Route {
    pub pattern: String,           // The URL pattern, e.g., "/users/:id"
    pub template_path: String,     // Original file path to the template
    pub params: Vec<String>,       // Extracted parameter names
    pub priority: usize,           // Calculated priority for sorting (lower = higher)
    pub is_layout: bool,           // True if it's a _layout.rhtmx file
    pub is_error_page: bool,       // True if it's an _error.rhtmx file
    pub has_catch_all: bool,       // True if pattern contains catch-all parameter
    pub optional_params: Vec<String>, // Names of optional parameters
}
```

#### `Route` Methods

- `Route::from_path(file_path: &str, pages_dir: &str) -> Self`: Creates a route from a file path.
- `matches(&self, path: &str) -> Option<HashMap<String, String>>`: Matches a path (case-sensitive) and returns extracted parameters.
- `matches_with_options(&self, path: &str, case_insensitive: bool) -> Option<HashMap<String, String>>`: Matches with optional case sensitivity.
- `layout_pattern(&self) -> Option<String>`: Gets the parent directory pattern for layout inheritance.

### `Router` Struct

The `Router` manages the collection of routes, layouts, and error pages.

#### Constructor Methods

- `Router::new() -> Self`: Creates a case-sensitive router.
- `Router::with_case_insensitive(case_insensitive: bool) -> Self`: Creates a router with specified case sensitivity.

#### Route Management

- `add_route(&mut self, route: Route)`: Adds a route. Automatically categorizes it as a layout, error page, or regular route.
- `remove_route(&mut self, pattern: &str) -> bool`: Removes a route by its pattern. Returns true if found and removed.
- `sort_routes(&mut self)`: Sorts routes by priority. **Must be called after adding all routes and before matching.**
- `set_case_insensitive(&mut self, case_insensitive: bool)`: Sets case sensitivity mode.

#### Route Matching

- `match_route(&self, path: &str) -> Option<RouteMatch>`: Finds the best-matching route for a given request path and extracts parameters into a HashMap.

#### Layout and Error Page Resolution

- `get_layout(&self, pattern: &str) -> Option<&Route>`: Finds the most specific layout for a route pattern using inheritance chain: exact → parent section → root.
- `get_error_page(&self, pattern: &str) -> Option<&Route>`: Finds the most specific error page for a route pattern using the same inheritance chain.

#### Query Methods

- `routes(&self) -> &[Route]`: Returns a slice of all regular routes.
- `layouts(&self) -> &HashMap<String, Route>`: Returns a reference to the layouts map.
- `error_pages(&self) -> &HashMap<String, Route>`: Returns a reference to the error pages map.

### `RouteMatch` Struct

The result of a successful route match.

```rust
pub struct RouteMatch {
    pub route: Route,                      // The matched route
    pub params: HashMap<String, String>,   // Extracted parameter values
}
```
