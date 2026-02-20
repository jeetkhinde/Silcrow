# Silcrow

Server-driven UI framework built on Axum and Maud. One Rust crate, one JS file, one `window.Silcrow` API.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
silcrow = { path = "../Silcrow" }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use axum::{
    http::StatusCode,
    response::Html,
    routing::get,
    Json, Router,
};
use silcrow::*;
use silcrow::maud::{html, Markup};

#[tokio::main]
async fn main() {
    let config = SilcrowConfig::load(); // reads silcrow.toml (optional)

    let app = Router::new()
        .route(SILCROW_JS_PATH, get(serve_silcrow_js)) // serves the JS bundle
        .route("/", get(index))
        .route("/api/greet", get(greet));

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn index(req: SilcrowRequest) -> Markup {
    // Default SSR path: plain Maud markup return
    let _ = req;
    page("My App", html! {
        h1 s-bind="message" { "Click the button" }
        button s-action="/api/greet" { "Greet" }
    })
}

async fn greet() -> (StatusCode, Json<serde_json::Value>) {
    // Default JSON path: plain Axum tuple + Json
    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Hello from Silcrow!" })),
    )
}

async fn greet_html() -> (StatusCode, Html<String>) {
    // Explicit HTML fragment path for s-html targets
    (
        StatusCode::OK,
        Html(html! { p { "Hello fragment" } }.into_string()),
    )
}
```

That's it. The JS is embedded in the binary and served automatically. No external files to manage.

---

## Configuration

Create an optional `silcrow.toml` in your project root:

```toml
[project]
name = "my-app"
version = "0.1.0"

[server]
port = 3000
host = "127.0.0.1"
```

Load it with `SilcrowConfig::load()`. All fields have defaults — the file is entirely optional.

---

## Server API

### SilcrowRequest (Axum Extractor)

Use directly as a handler parameter — no manual header parsing needed:

```rust
async fn handler(req: SilcrowRequest) -> Result<impl axum::response::IntoResponse, AppError> {
    if req.wants_html {
        return Ok((
            StatusCode::OK,
            Html(html! { div { "partial update" } }.into_string()),
        ));
    }

    if req.wants_json {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "count": 42 })),
        ));
    }

    Ok(page("Title", html! { body_content }))
}
```

Fields:
- `req.is_silcrow` — `true` when request has `silcrow-target` header
- `req.wants_html` — `true` when `Accept: text/html`
- `req.wants_json` — `true` when `Accept: application/json`

### Response Patterns

Default recommendation: start with plain Axum + Maud return types, then use Silcrow builders when protocol integration helpers are useful.

**Plain JSON response** (default for `s-action` elements):

```rust
(StatusCode::OK, Json(serde_json::json!({
    "name": "Alice",
    "items": [1, 2, 3]
})))
```

**Silcrow JSON convenience builder** (useful for `_toast`, fluent headers/status, or protocol-centric payload shaping):

```rust
JsonOk()
    .set("name", "Alice")           // any Serialize value
    .set("items", vec![1, 2, 3])    // arrays, structs, etc.
    .set_value("raw", json!({}))    // pre-built serde_json::Value
    .toast("Saved!")                 // adds _toast key
    .no_cache()                     // prevent client caching
    .status(StatusCode::CREATED)    // custom status code
```

**Plain HTML fragment response** (for `s-html` elements):

```rust
(StatusCode::OK, Html(html! {
    div.card {
        h2 { "New content" }
        p { "Replaces the target element's innerHTML" }
    }
}.into_string()))
```

**Silcrow HTML convenience builder** (useful when keeping response construction style consistent with Silcrow JSON builders):

```rust
HtmlOk(html! {
    div.card {
        h2 { "New content" }
        p { "Replaces the target element's innerHTML" }
    }
})
```

Also accepts `String` and `&str`:

```rust
HtmlOk("<p>raw html</p>")
```

**Error response:**

```rust
Error()
    .message("Not found")
    .status(StatusCode::NOT_FOUND)

// Or as JSON:
Error()
    .message("Not found")
    .status(StatusCode::NOT_FOUND)
    .json()
```

**Redirect response:**

```rust
Redirect().to("/dashboard")
Redirect().to("/login").status(StatusCode::TEMPORARY_REDIRECT)
```

**Dispatcher pattern with Silcrow builders** (useful in branching logic when you want one fluent API for both formats):

```rust
Ok().html(html! { ... })  // returns HtmlOkResponse
Ok().json()                // returns JsonOkResponse
```

### Layout Helper

`page()` renders a full HTML document with the Silcrow script auto-included:

```rust
page("Page Title", html! {
    h1 { "My App" }
    p { "Content here" }
})
```

Generates:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Page Title</title>
    <script src="/_silcrow/silcrow.js" defer></script>
</head>
<body>
    <h1>My App</h1>
    <p>Content here</p>
</body>
</html>
```

For custom layouts, define a function with the same signature using `script_tag()` and `maud::DOCTYPE`:

```rust
use maud::DOCTYPE;

fn app_layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                (script_tag())
                link rel="stylesheet" href="/style.css";
            }
            body {
                nav { a href="/" { "Home" } }
                main { (body) }
            }
        }
    }
}
```

Use it identically to `page()`:

```rust
app_layout("Dashboard", html! {
    h1 { "Dashboard" }
}).ok()
```

### Serving the JS

Wire the built-in handler into your router:

```rust
use silcrow::{SILCROW_JS_PATH, serve_silcrow_js};

let app = Router::new()
    .route(SILCROW_JS_PATH, get(serve_silcrow_js));
```

The JS is embedded in the binary at compile time via `include_str!`. Served with `Cache-Control: public, max-age=31536000, immutable`.

---

## Client API

Everything lives on `window.Silcrow`. One object, one import.

### HTML Attributes

**Data binding:**

```html
<!-- Bind text content -->
<span s-bind="username">loading...</span>

<!-- Bind to a specific property -->
<input s-bind="email:value" />
<img s-bind="avatar:src" />
<button s-bind="saving:disabled">Save</button>

<!-- Nested paths -->
<span s-bind="user.profile.name">...</span>
```

**Navigation:**

```html
<!-- Click triggers GET, patches response into the element -->
<button s-action="/api/data">Load</button>

<!-- Target a different element -->
<button s-action="/api/data" s-target="#results">Load</button>

<!-- HTTP methods -->
<button s-action="/api/item" DELETE>Delete</button>
<button s-action="/api/item" POST>Create</button>
<button s-action="/api/item" PUT>Update</button>

<!-- Request HTML instead of JSON -->
<button s-action="/components/card" s-html s-target="#container">Load Card</button>

<!-- Skip history push -->
<button s-action="/api/poll" s-skip-history>Refresh</button>

<!-- Custom timeout (ms) -->
<button s-action="/api/slow" s-timeout="60000">Run</button>

<!-- Preload on hover -->
<a s-action="/about" s-preload>About</a>
```

**Forms:**

```html
<!-- Form data sent as POST by default -->
<form s-action="/api/login">
    <input name="email" type="email" />
    <input name="password" type="password" />
    <button type="submit">Login</button>
</form>

<!-- GET form appends data as query params -->
<form s-action="/api/search" method="GET">
    <input name="q" />
    <button type="submit">Search</button>
</form>
```

**Collections:**

```html
<ul s-list="items" s-template="item-tpl">
    <!-- Items rendered here -->
</ul>

<template id="item-tpl">
    <li>
        <span s-bind=".name"></span>
        <span s-bind=".email"></span>
    </li>
</template>
```

Items must have a `key` property. Local bindings use dot prefix (`.name`, `.email`).

Server sends (plain Axum JSON):
```rust
(StatusCode::OK, Json(serde_json::json!({
    "items": [
        {"key": 1, "name": "Alice", "email": "alice@example.com"},
        {"key": 2, "name": "Bob", "email": "bob@example.com"}
    ]
})))
```

Or with Silcrow convenience utilities when desired:
```rust
JsonOk().set("items", vec![
    json!({"key": 1, "name": "Alice", "email": "alice@example.com"}),
    json!({"key": 2, "name": "Bob", "email": "bob@example.com"}),
])
```

**Debug mode:**

```html
<body s-debug>
```

Enables console warnings and throws errors for invalid bindings.

### JavaScript API

**Programmatic patching:**

```javascript
// Patch data into a root element
Silcrow.patch({ count: 42, name: "Alice" }, "#app");

// Options
Silcrow.patch(data, "#app", { invalidate: true }); // rebuild binding maps
Silcrow.patch(data, "#app", { silent: true });      // suppress silcrow:patched event
```

**Programmatic navigation:**

```javascript
// Navigate (GET by default)
Silcrow.go("/about");

// POST with body
Silcrow.go("/api/items", {
    method: "POST",
    body: { name: "New item" }
});

// Partial update
Silcrow.go("/api/sidebar", { target: "#sidebar" });
```

**Cache control:**

```javascript
Silcrow.cache.clear();           // clear all
Silcrow.cache.clear("/api/data"); // clear specific URL
Silcrow.cache.has("/api/data");   // check if cached
```

**Hooks:**

```javascript
// Intercept before render — return false to prevent swap
Silcrow.onRoute(({ url, method, response, contentType, target }) => {
    if (contentType.includes("application/json")) {
        const data = JSON.parse(response);
        if (data._redirect) {
            Silcrow.go(data._redirect);
            return false; // prevent default swap
        }
    }
});

// Global error handler
Silcrow.onError((err, { url, method }) => {
    console.error(`Failed ${method} ${url}:`, err);
});
```

**Streaming / high-frequency updates:**

```javascript
const update = Silcrow.stream("#counter");
// Batches via queueMicrotask — only the last call per tick is applied
update({ count: 1 });
update({ count: 2 });
update({ count: 3 }); // only this one renders
```

**Invalidate cached bindings:**

```javascript
// Force rebuild of binding maps on next patch
Silcrow.invalidate("#app");
```

**Cleanup:**

```javascript
Silcrow.destroy(); // removes all event listeners, clears caches
```

### Events

Listen on `document` or any element (events bubble):

```javascript
// Before fetch starts (cancelable)
document.addEventListener("silcrow:navigate", (e) => {
    console.log("Navigating to", e.detail.url);
    // e.preventDefault() to cancel
});

// After fetch, before DOM swap (cancelable)
document.addEventListener("silcrow:before-swap", (e) => {
    const { content, isJSON, proceed, target } = e.detail;
    // Custom transition:
    target.style.opacity = 0;
    setTimeout(() => {
        proceed(); // execute the swap
        target.style.opacity = 1;
    }, 200);
});

// After successful swap
document.addEventListener("silcrow:load", (e) => {
    console.log("Loaded", e.detail.url);
});

// After patch() applies data bindings
document.addEventListener("silcrow:patched", (e) => {
    console.log("Patched paths:", e.detail.paths);
});

// On error
document.addEventListener("silcrow:error", (e) => {
    console.error(e.detail.error);
});
```

### Loading States

During navigation, the target element gets:

```css
.silcrow-loading {
    /* your loading styles */
}
```

And `aria-busy="true"` for accessibility. Both are removed when the request completes.

---

## Request Flow

```
1. User clicks <button s-action="/api/data">
2. Silcrow JS sends fetch with headers:
   - silcrow-target: true
   - Accept: application/json (or text/html if s-html present)
3. Axum handler receives request
   - SilcrowRequest extractor parses headers automatically
4. Handler returns `(StatusCode::OK, Json(json!({"count": 42})))`
5. Silcrow JS receives JSON, calls patch() on target element
6. DOM updates via s-bind attributes
```

For full-page navigation (no `s-target`), the response replaces `document.body` and pushes browser history.

---

## Project Structure

```
your-app/
├── Cargo.toml
├── silcrow.toml          # optional config
└── src/
    └── main.rs           # your Axum app
```

Silcrow crate structure:

```
Silcrow/
├── Cargo.toml
├── silcrow.toml          # default config
├── silcrow.js            # unified client bundle (embedded at compile time)
└── src/
    ├── lib.rs            # SilcrowRequest extractor, re-exports
    ├── assets.rs          # JS embedding, serve handler, script_tag()
    ├── config.rs          # SilcrowConfig from silcrow.toml
    ├── layout.rs          # page() helper
    └── response.rs        # HtmlOk, JsonOk, Error, Redirect builders
```
