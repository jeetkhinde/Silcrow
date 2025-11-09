# RHTMX HTTP Handler Macros - Complete Guide

Declarative HTTP route handlers using clean, type-safe Rust macros.

## Overview

RHTMX provides HTTP verb macros for defining route handlers. The route is determined by the file location in your `pages/` directory, and the macro specifies the HTTP method and optional path/query parameters.

**Key Features:**
- ✅ Clean, declarative routing with `get!`, `post!`, `put!`, `patch!`, `delete!`
- ✅ Path parameter support (`:id`)
- ✅ Query parameter support
- ✅ Type-safe request/response handling
- ✅ Works seamlessly with file-based routing
- ✅ Chainable response builders
- ✅ Compile-time route discovery

## Available Macros

| Macro | HTTP Method | Usage |
|-------|-------------|-------|
| `get!` | GET | Fetch resource data |
| `post!` | POST | Create new resource |
| `put!` | PUT | Replace entire resource |
| `patch!` | PATCH | Partial resource update |
| `delete!` | DELETE | Remove resource |

## Quick Start

### Basic Handler

```rust
use rhtmx::{OkResponse, Ok};

get!()
fn list_users() -> OkResponse {
    let users = db::get_users()?;
    Ok().render(users_page, users)
```

### With Path Parameters

```rust
get!(":id")
fn get_user(id: i32) -> OkResponse {
    let user = db::get_user(id)?;
    Ok().render(user_detail, user)

delete!(":id")
fn delete_user(id: i32) -> OkResponse {
    db::delete_user(id)?;
    Ok().toast("User deleted!")
```

### With Query Parameters

```rust
get!("partial=stats")
fn user_stats() -> OkResponse {
    let stats = calculate_stats()?;
    Ok().render(stats_component, stats)
```

## File-Based Routing

The route is determined by where you place the handler in your `pages/` directory:

```
pages/
├── users/
│   ├── index.rs        # #[get] → GET /users
│   ├── index.rs        # #[post] → POST /users
│   ├── [id].rs         # #[get] → GET /users/:id
│   └── [id].rs         # #[delete] → DELETE /users/:id
├── blog/
│   ├── [slug].rs       # #[get] → GET /blog/:slug
│   └── [slug].rs       # #[patch] → PATCH /blog/:slug
└── index.rs            # #[get] → GET /
```

Multiple handlers in the same file must have different HTTP methods:

```rust
// pages/users/[id].rs

get!()
fn get_user(id: i32) -> OkResponse {
        // GET /users/:id
    }

post!()
fn update_user(id: i32, req: UpdateUserRequest) -> OkResponse {
        // POST /users/:id
    }
```

## Request Handling

### Type-Safe Requests

Request body is automatically deserialized from the function signature:

```rust
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

post!()
fn create_user(req: CreateUserRequest) -> OkResponse {
        let user = db::create_user(req.name, req.email)?;
        Ok().render(user_card, user)
    }
```

### Query Parameters

Query string parameters can be captured:

```rust
get!("partial=comments")
fn load_comments(post_id: i32) -> OkResponse {
    let comments = db::get_comments(post_id)?;
    Ok().render(comments_list, comments)
```

### Path Parameters

Dynamic path segments are extracted from the function parameters:

```rust
get!(":user_id/posts/:post_id")
fn get_post(user_id: i32, post_id: i32) -> OkResponse {
    let post = db::get_post(user_id, post_id)?;
    Ok().render(post_detail, post)
```

## Response Builders

Responses are built using chainable builder methods:

### Ok() - Success Response

```rust
post!()
fn create_user(req: CreateUserRequest) -> OkResponse {
    let user = db::create_user(req)?;

    Ok()
        .render(user_card, user)                    // Main content
        .render_oob("user-count", count, 10)        // Out-of-band update
        .toast("User created successfully!")         // Toast notification
        .status(StatusCode::CREATED)                 // Custom status code
        .header("X-Custom-Header", "value")          // Custom header
}
```

**Available Methods:**

| Method | Purpose |
|--------|---------|
| `.render(component, props)` | Render main page content |
| `.render_oob(target, component, props)` | Out-of-band update (separate element) |
| `.toast(message)` | Show toast notification to user |
| `.status(code)` | Set HTTP status code (default: 200) |
| `.header(name, value)` | Add custom response header |
| `.html(html)` | Send raw HTML (no component) |

### Error() - Error Response

```rust
use rhtmx::{Error, ErrorResponse};

post!()
fn create_user(req: CreateUserRequest) -> Result<OkResponse, ErrorResponse> {
        let errors = validate(&req);

        if !errors.is_empty() {
            return Err(
                Error()
                    .render(validation_errors, errors)
                    .status(StatusCode::BAD_REQUEST)
                    .message("Validation failed")
            );
        }

        let user = db::create_user(req)?;
        Ok(user)
    }
```

**Available Methods:**

| Method | Purpose |
|--------|---------|
| `.render(component, props)` | Render error content |
| `.message(msg)` | Set error message |
| `.status(code)` | Set HTTP status code (default: 500) |
| `.header(name, value)` | Add custom response header |

### Redirect() - Redirect Response

```rust
use rhtmx::{Redirect, RedirectResponse};

post!()
fn login_user(req: LoginRequest) -> Result<RedirectResponse, ErrorResponse> {
        let user = authenticate(&req)?;

        // Redirect to dashboard after successful login
        Ok(Redirect("/dashboard").see_other())
    }
```

## Common Patterns

### HTMX Partial Updates (No Layout)

For HTMX-driven partial updates, skip the layout wrapper:

```rust
get!()
fn load_more_items(page: i32) -> OkResponse {
    let items = db::get_items(page)?;

    Ok().html(html! {
        <div r-for="item in items" class="item">
            <p>{item.name}</p>
        </div>
    })  // No layout - HTMX swaps this into the page
}
```

### Full Page Render with Layout

For full page navigation, use a layout:

```rust
use rhtmx::layouts;

get!()
fn index() -> OkResponse {
        let users = db::get_users()?;

        let content = html! {
            <div class="container">
                // ... render users
            </div>
        };

        Ok().html(layouts::root::layout(
            content,
            layouts::root::Slots::new("Users")
        ))
    }
```

### Error Handling with Result

```rust
get!(":id")
fn get_user(id: i32) -> Result<OkResponse, ErrorResponse> {
    let user = db::get_user(id)
        .map_err(|e| Error().message(format!("User not found: {}", e)))?;

    Ok(Ok().render(user_detail, user))
```

### Out-of-Band (OOB) Updates

Update multiple parts of the page in a single request:

```rust
post!()
fn update_user(id: i32, req: UpdateUserRequest) -> OkResponse {
    let user = db::update_user(id, req)?;
    let stats = db::get_stats()?;

    Ok()
        .render(user_detail, user)                     // Main update
        .render_oob("stats-panel", stats_widget, stats) // Side update
        .toast("User updated!")
```

### Conditional Responses

```rust
post!()
fn toggle_status(id: i32) -> OkResponse {
    let item = db::toggle_status(id)?;

    if item.is_active {
        Ok()
            .render(active_badge, ())
            .toast("Activated!")
    } else {
        Ok()
            .render(inactive_badge, ())
            .toast("Deactivated!")
    }
```

## Macro Definition Location

These macros are defined in the RHTMX macro crate:

**Macro Definitions:** `rhtmx-macro/src/lib.rs` (lines 159-248)
```rust
#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn put(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn patch(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn delete(args: TokenStream, input: TokenStream) -> TokenStream { ... }
```

**Implementation:** `rhtmx-macro/src/http.rs`

All macros delegate to the `http_handler()` function which:
- Parses the macro input and function signature
- Generates metadata modules for compile-time route discovery
- Preserves the original function signature and body
- Creates module constants for HTTP method and handler name

## Real-World Example

```rust
// pages/users/index.rs

use rhtmx::{html, Ok, OkResponse, Error, ErrorResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// GET /users - List all users
get!()
fn index() -> OkResponse {
        let users = db::get_users().unwrap_or_default();
        Ok().render(users_page, users)
    }

// POST /users - Create new user
post!()
fn create(req: CreateUserRequest) -> Result<OkResponse, ErrorResponse> {
        if req.name.is_empty() {
            return Err(Error().message("Name is required"));
        }

        let user = db::create_user(req.name, req.email)?;

        Ok()
            .render(user_card, user)
            .render_oob("user-count", user_count, db::count_users()?)
            .toast("User created!")
    }

// pages/users/[id].rs

// GET /users/:id - Get single user
get!()
fn show(id: i32) -> Result<OkResponse, ErrorResponse> {
        let user = db::get_user(id)
            .ok_or_else(|| Error().message("User not found"))?;

        Ok().render(user_detail, user)
    }

// POST /users/:id - Update user
post!()
fn update(id: i32, req: UpdateUserRequest) -> Result<OkResponse, ErrorResponse> {
        let user = db::update_user(id, req)?;
        Ok()
            .render(user_card, user)
            .toast("User updated!")
    }

// DELETE /users/:id - Delete user
delete!()
fn delete(id: i32) -> OkResponse {
        db::delete_user(id).ok();

        Ok()
            .render_oob("user-count", user_count, db::count_users().unwrap_or(0))
            .toast("User deleted!")
    }
```

## Best Practices

### 1. Use Proper HTTP Methods

- **GET** - Retrieving data (safe, idempotent)
- **POST** - Creating new resources
- **PUT** - Replacing entire resources
- **PATCH** - Partial updates
- **DELETE** - Removing resources

```rust
// ✓ Good
post!()
fn create_item(req: CreateRequest) -> OkResponse { ... }

get!(":id")
fn get_item(id: i32) -> OkResponse { ... }

// ✗ Avoid using GET for mutations
get!("delete/:id") {  // Wrong!
    fn delete_item(id: i32) -> OkResponse { ... }
```

### 2. Return Result for Fallible Operations

```rust
// ✓ Good - explicit error handling
get!(":id")
fn get_user(id: i32) -> Result<OkResponse, ErrorResponse> {
    let user = db::get_user(id)?;
    Ok(Ok().render(user_detail, user))

// ✗ Avoid - panics on error
get!(":id")
fn get_user(id: i32) -> OkResponse {
    let user = db::get_user(id).unwrap();  // Panics if not found!
    Ok().render(user_detail, user)
```

### 3. Use Type-Safe Requests

```rust
// ✓ Good - type-safe with validation
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

post!()
fn create(req: CreateUserRequest) -> OkResponse { ... }

// ✗ Avoid - stringly-typed
post!()
fn create(raw_body: String) -> OkResponse { ... }
```

### 4. Use Builder Pattern for Responses

```rust
// ✓ Good - clear and chainable
Ok()
    .render(component, props)
    .toast("Success!")
    .status(StatusCode::CREATED)

// ✗ Avoid - manual struct construction
OkResponse {
    content: Some(html),
    toast: Some("Success!".to_string()),
    status: StatusCode::CREATED,
    ...
}
```

### 5. Return Full Pages or HTMX Partials Based on Context

```rust
// ✓ Good - return full page with layout for initial request
get!()
fn index() -> OkResponse {
        Ok().html(layouts::root::layout(content, slots))
    }

// ✓ Good - return partial (no layout) for HTMX swaps
get!("partial=items")
fn load_more() -> OkResponse {
        Ok().html(html! { <div>...</div> })
    }
```

### 6. Use OOB Updates for Multi-Element Changes

```rust
// ✓ Good - update multiple elements atomically
post!()
fn update() -> OkResponse {
        Ok()
            .render(main_content, new_content)
            .render_oob("sidebar", sidebar, new_sidebar)
            .render_oob("stats", stats, new_stats)
    }

// ✗ Avoid - multiple separate requests
// Client would need to make 3 separate HTMX requests
```

## Comparison with Traditional Frameworks

| Aspect | Traditional | RHTMX |
|--------|-----------|-------|
| Route definition | Separate router configuration | File location determines route |
| Routing syntax | `router.get("/users/:id", handler)` | File: `pages/users/[id].rs` + `get!` |
| Request/response | Manual struct creation | Type-safe macros + builders |
| Type safety | Runtime validation | Compile-time checks |
| File organization | Routes separated from handlers | Handlers organized by route |
| Response building | Manual string concatenation | Chainable builder methods |

## See Also

- [LAYOUTS.md](../LAYOUTS.md) - Layout system for full page renders
- [FEATURES.md](../FEATURES.md) - All RHTMX features
- [QUICKSTART.md](../QUICKSTART.md) - Getting started guide
- [examples/](../examples/) - Complete working examples

## Advanced Topics

### Custom Response Headers

```rust
get!()
fn get_file() -> OkResponse {
    Ok()
        .html(file_content)
        .header("Content-Type", "application/pdf")
        .header("Content-Disposition", "attachment; filename=\"file.pdf\"")
```

### Custom Status Codes

```rust
use http::StatusCode;

post!()
fn create_item(req: CreateRequest) -> OkResponse {
        let item = db::create_item(req)?;

        Ok()
            .render(item_card, item)
            .status(StatusCode::CREATED)  // 201 instead of 200
    }
```

### Streaming Responses

For large data responses, use the underlying HTML type:

```rust
get!()
fn export_data() -> OkResponse {
    let data = db::get_large_dataset()?;
    let csv = generate_csv(data);

    Ok()
        .html(csv)
        .header("Content-Type", "text/csv")
        .header("Content-Disposition", "attachment; filename=\"data.csv\"")
```
