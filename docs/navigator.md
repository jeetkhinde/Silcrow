# **Navigator: Client-Side Routing**

## **Declarative Navigation with s-action**

Add `s-action` to any element to enable client-side navigation or mutations. Silcrow standardizes on the colon-prefix (`:`) for dynamic URL parameters.

```html
<a s-action="/dashboard">Dashboard</a>
<button s-action="/api/save" POST>Save</button>
<button s-action="/items/5" DELETE s-target="#item-5">Remove</button>

```

```html
<a s-action="/dashboard">Dashboard</a>
<button s-action="/tasks/:key/complete" POST>Complete</button>
<button s-action="/items/:key" DELETE s-target="#notifications">Remove</button>
```

## The `:key` Placeholder**

When an action is placed inside an `s-for` loop, Silcrow provides automatic context discovery. Any `:key` placeholder in an `s-action` or `s-target` is automatically replaced with the stable ID of the nearest loop block.

* **Discovery**: The Navigator looks for the nearest printed `:key` attribute in the DOM.

* **Symmetry**: This matches the `:key` used in your templates, creating a "One Way" mental model for dynamic data.

### **Implicit Targeting**

If you omit the `s-target` attribute, Silcrow intelligently resolves the swap target:

1. **Container Swap**: If inside an `s-for` loop, it targets the parent container. This allows the server to return a single JSON object for a "Merge" patch.

2. **Self Swap**: If no loop context is found, it targets the triggering element itself.

### **Form Mutations vs. Pure Buttons**

Silcrow eliminates the need for `<form>` wrappers for simple, binary actions.

* **Pure Buttons**: Use for actions where the URL contains all required state (e.g., `POST`, `PATCH`, `DELETE` via `/:key`).

* **Forms**: Use only when sending user input (e.g., text fields, file uploads). Silcrow automatically serializes the form into a `FormData` body.

```html
<button s-action="/tasks/:key/star" POST>Star Task</button>

<form s-action="/tasks/:key/rename" PATCH>
  <input name="new_name" placeholder="Enter name..." />
  <button type="submit">Rename</button>
</form>
```

## **Attributes**

| **Attribute** | **Purpose** | **Default** |
| --- | --- | --- |
| `s-action` | URL to request (supports `/:key` interpolation) | *(required)* |
| `s-target` | CSS selector for the swap target | Closest loop block or self |
| `s-html` | Force request to expect `text/html` | `application/json` |
| `s-skip-history` | Don't push to browser history | Push for full-page GETs |
| `s-preload` | Preload on `mouseenter` | Off |
| `s-timeout` | Request timeout in ms | 30000 |
| `GET`, `POST`, `PUT`, `PATCH`, `DELETE` | HTTP method (as attribute) | `GET` (or `POST` for forms) |

### **Forms vs. Pure Buttons for Mutations**

Because of `:key` interpolation and implicit targeting, you have two distinct tools depending on whether your mutation requires a request body. While the HTTP specification builds POST, PUT, and PATCH to carry bodies, it does *not* mandate them.

**1. When you NEED a body → Use a `<form>`** If the user is submitting new data (like typing a task title), you must use a form. Silcrow relies on the form boundary to serialize inputs into a FormData request body.

```html
<form s-action="/tasks/:key/edit" method="PUT">  
  <input type="text" name="title" :value=".title" /> 
  <button type="submit">Save</button>  
</form>

```

**2. When you DON'T need a body → Use a Pure `<button>`**

If the action is binary and the URL itself contains all the required context (via the ID), you don't need a body. You can use form-less buttons for POST, PUT, and PATCH just like you do for DELETE.

```html
<button s-action="/tasks/:key/toggle" PATCH>Toggle Complete</button>  
<button s-action="/tasks/:key/upvote" POST>Upvote</button>
```

**The Architect's Rule of Thumb:**

* Use `DELETE` (pure button) to destroy a resource.
* Use `POST` / `PUT` / `PATCH` (pure button) to trigger a specific, parameter-less action (like "star", "archive", "toggle").
* Use `<form method="...">` only when sending user input fields.

**Server-Side Example (Axum):**

For a pure button, the backend handler simply extracts the ID from the path and processes the action without expecting a body, returning the updated fragment directly to the targeted `:key` item.

```rust
use axum::extract::Path;
use pilcrow::{SilcrowRequest, respond!, html, json, ResponseExt};
use axum::response::Response;

pub async fn upvote_task(
    req: SilcrowRequest,
    Path(id): Path<i64>,
) -> Result<Response, Response> {
    // 1. Process the parameter-less action using the ID from the URL
    let updated_task = db.upvote(id).await.unwrap();

    // 2. Return the updated fragment (Silcrow implicit targeting swaps this in)
    respond!(req, {
        html => html(render_task(&updated_task)).with_toast("Upvoted!", "success"),
        json => json(&updated_task),
    })
}

```

## **Forms**

Forms with `s-action` are intercepted automatically. GET forms append FormData as query params. Other methods send FormData as the body.

```html
<form s-action="/search" GET s-target="#results">
  <input name="q" />
  <button>Search</button>
</form>

```

## **Programmatic Navigation**

```javascript
Silcrow.go("/dashboard");
Silcrow.go("/api/items", { method: "POST", body: { name: "New" }, target: "#list" });

```

## **Response Processing**

The navigator reads the Content-Type header to decide how to handle the response:

* **JSON** (application/json) — parsed and passed to Silcrow.patch() on the target element
* **HTML** (text/html) — sanitized and swapped into the target element's innerHTML

For HTML responses, if the response is a full page (`<!DOCTYPE` or `<html>`), Silcrow extracts the `<title>` and the matching `s-target` selector content (or `<body>` as fallback).

**HTML sanitization:**

Silcrow uses the Sanitizer API (`el.setHTML()`) when available. When it isn't, a DOMParser fallback strips all `<script>` elements and event handler attributes (`on*`) before insertion.

## **Server-Driven Headers**

The backend can control Silcrow's behavior through response headers. These are split into two phases: headers processed during the fetch, and side-effect headers executed after the main swap.

**During fetch:**

| **Header** | **Effect** |
| --- | --- |
| `silcrow-trigger` | Fire custom DOM events. JSON object `{"event-name": detail}` or a plain event name string. |
| `silcrow-retarget` | CSS selector — override where the response is swapped into. |
| `silcrow-push` | Override the URL pushed to browser history. |
| `silcrow-cache` | Set to `no-cache` to prevent this response from being cached. |

**After swap (side effects):**

| **Header** | **Effect** |
| --- | --- |
| `silcrow-patch` | JSON `{"target": "#el", "data": {...}}` — patches data into a secondary element via `Silcrow.patch()`. |
| `silcrow-invalidate` | CSS selector — rebuilds binding maps for the target element via `Silcrow.invalidate()`. |
| `silcrow-navigate` | URL path — triggers a client-side navigation after the swap completes. |
| `silcrow-sse` | URL path — dispatches a `silcrow:sse` event signaling the client to open an SSE connection. |
| `silcrow-ws` | URL path — dispatches a `silcrow:ws` event signaling the client to open a WebSocket connection. |

Side-effect headers execute in order: patch → invalidate → navigate → sse/ws. This lets a single response update the primary target, patch a secondary counter, rebuild a sidebar, and trigger a follow-up navigation.

## **Caching**

GET responses are cached in-memory for 5 minutes (max 50 entries). Any mutation request (POST, PUT, PATCH, DELETE) clears the entire cache. The server can opt out per-response with the `silcrow-cache: no-cache` header.

```javascript
Silcrow.cache.has("/dashboard");  // check cache
Silcrow.cache.clear("/dashboard"); // clear one entry
Silcrow.cache.clear();             // clear all
```

## **Preloading**

Elements with `s-preload` fire a background fetch on `mouseenter`. The response is cached so the subsequent click is instant.

```html
<a s-action="/settings" s-preload>Settings</a>
```

## **History & Scroll**

Full-page GET navigations push to history.pushState. On popstate (back/forward), Silcrow re-fetches the URL and restores the saved scroll position. Partial updates (those with s-target) skip history by default.

## **Loading States**

During requests, Silcrow adds `silcrow-loading` CSS class and `aria-busy="true"` to the target element. Style it however you want:

```css
.silcrow-loading { opacity: 0.5; pointer-events: none; }
```

## **Abort & Timeout**

Navigating to the same target while a GET is in-flight aborts the previous request. Mutation requests are never aborted. Timeout defaults to 30 seconds and can be set per-element with s-timeout.
