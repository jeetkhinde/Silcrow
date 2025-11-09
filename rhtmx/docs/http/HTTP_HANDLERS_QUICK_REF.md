# HTTP Handlers - Quick Reference

## Import

```rust
use rhtmx::{get, post, put, patch, delete, Ok, Error, Redirect, StatusCode};
use serde::Deserialize;
```

## Basic Handler

```rust
get! {
    fn index() -> OkResponse {
    Ok().render(users_page, data)
}

#[post]
fn create(req: CreateRequest) -> OkResponse {
    Ok().render(component, data)
        .toast("Created!")
}

delete!(":id") {
    fn delete(id: i32) -> OkResponse {
    Ok().toast("Deleted!")
}
```

## Available Macros

| Macro | HTTP Method |
|-------|------------|
| `get!` | GET |
| `post!` | POST |
| `#[put(":id")]` | PUT |
| `#[patch(":id")]` | PATCH |
| `#[delete(":id")]` | DELETE |

## Path Parameters

```rust
get!(":id") {
    fn get_user(id: i32) -> OkResponse {
    let user = db::get_user(id)?;
    Ok().render(user_detail, user)
}

get!(":user_id/posts/:post_id") {
    fn get_post(user_id: i32, post_id: i32) -> OkResponse {
    let post = db::get_post(user_id, post_id)?;
    Ok().render(post_page, post)
}
```

## Query Parameters

```rust
get!("partial=items") {
    fn load_items(page: i32) -> OkResponse {
    let items = db::get_items(page)?;
    Ok().html(html! { <div r-for="item in items">{item.name}</div> })
}
```

## Type-Safe Requests

```rust
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[post]
fn create(req: CreateUserRequest) -> OkResponse {
    let user = db::create_user(req.name, req.email)?;
    Ok().render(user_card, user)
}
```

## Response Builders

### Ok() - Success

```rust
Ok()
    .render(component, props)         // Main content
    .render_oob("id", comp, props)    // Out-of-band update
    .toast("Success!")                 // Toast notification
    .status(StatusCode::CREATED)       // Custom status
    .header("X-Custom", "value")       // Custom header
    .html(html)                        // Raw HTML
```

### Error() - Error Response

```rust
Error()
    .render(error_comp, errors)
    .message("Failed!")
    .status(StatusCode::BAD_REQUEST)
```

### Redirect() - Redirect

```rust
Redirect("/dashboard")
    .toast("Redirecting...")
    .status(StatusCode::SEE_OTHER)
```

## Return Types

```rust
// Simple success
fn handler() -> OkResponse {
    Ok().render(comp, data)
}

// With error handling
fn handler() -> Result<OkResponse, ErrorResponse> {
    let data = fetch_data()?;
    Ok(Ok().render(comp, data))
}

// With redirect
fn handler() -> Result<RedirectResponse, ErrorResponse> {
    validate()?;
    Ok(Redirect("/success"))
}
```

## File-Based Routing

```
pages/
├── index.rs           → GET /
├── users/
│   ├── index.rs       → GET /users, POST /users
│   └── [id].rs        → GET /users/:id, DELETE /users/:id
└── blog/
    └── [slug].rs      → GET /blog/:slug
```

**Multiple handlers in one file:**

```rust
// pages/users/[id].rs
#[get]
fn get_user(id: i32) -> OkResponse { ... }  // GET /users/:id

#[post]
fn update_user(id: i32, req: UpdateRequest) -> OkResponse { ... }  // POST /users/:id
```

## Common Patterns

### HTMX Partial (No Layout)

```rust
get! {
    fn load_more() -> OkResponse {
    let items = db::get_items()?;
    Ok().html(html! {
        <div r-for="item in items">{item_card(item)}</div>
    })
}
```

### Full Page (With Layout)

```rust
get! {
    fn index() -> OkResponse {
    let content = html! { <div>...</div> };
    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Page Title")
    ))
}
```

### OOB Updates

```rust
post! {
    fn update() -> OkResponse {
    Ok()
        .render(main, data)              // Main update
        .render_oob("sidebar", sidebar, sidebar_data)  // Side update
        .render_oob("counter", counter, count)
}
```

### Error Response

```rust
post! {
    fn create(req: CreateRequest) -> Result<OkResponse, ErrorResponse> {
    if req.name.is_empty() {
        return Err(Error()
            .message("Name is required")
            .status(StatusCode::BAD_REQUEST));
    }

    let item = db::create(req)?;
    Ok(Ok().render(item_card, item).toast("Created!"))
}
```

### Form Validation

```rust
post! {
    fn submit(req: FormData) -> Result<OkResponse, ErrorResponse> {
    let errors = validate(&req);

    if !errors.is_empty() {
        return Err(Error()
            .render(validation_errors, errors)
            .status(StatusCode::BAD_REQUEST));
    }

    process(&req)?;
    Ok(Ok().toast("Submitted!"))
}
```

## Tips

- Use `Result<OkResponse, ErrorResponse>` for fallible operations
- Return `OkResponse` for simple success cases
- Use `.render()` for component + data, `.html()` for raw HTML
- Use `.render_oob()` to update multiple elements atomically
- Keep handlers pure - extract logic before rendering
- Use proper HTTP methods (GET, POST, PUT, PATCH, DELETE)
- For HTMX partials, skip the layout wrapper

## Macro Definition

**Location:** `rhtmx-macro/src/lib.rs` (lines 159-248)

**Implementation:** `rhtmx-macro/src/http.rs`

All macros delegate to the `http_handler()` function which handles:
- Route metadata generation
- Method assignment
- Compile-time route discovery
