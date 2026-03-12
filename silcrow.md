# **Silcrow.js**

A lightweight client-side runtime for building hypermedia-driven applications. Silcrow handles DOM patching, client-side navigation, response caching, live SSE and WebSocket connections, optimistic updates, and server-driven UI orchestration — all from declarative HTML attributes.  
Silcrow.js is the frontend counterpart to [Pilcrow](https://www.google.com/search?q=readme.md) but operates independently as a standalone library. Any backend that speaks HTTP and returns HTML or JSON can drive it.

## **Table of Contents**

* [Loading](#loading)  
* [Three Systems](#three-systems)  
* [Runtime: Data Binding & DOM Patching](#runtime-data-binding--dom-patching)  
* [Navigator: Client-Side Routing](#navigator-client-side-routing)  
* [Live: SSE, WebSocket & Real-Time Updates](#live-sse-websocket--real-time-updates)  
* [Optimistic Updates](#optimistic-updates)  
* [Toast System](#toast-system)  
* [Events](#events)  
* [Lifecycle](#lifecycle)  
* [API Reference](#api-reference)  
* [Compatibility](#compatibility)

## **Loading**

Silcrow.js is a single self-executing IIFE with no dependencies. Include it in your page:

```html
<script src="/_silcrow/silcrow.js" defer></script>
```

If using Pilcrow on the backend, use the `script_tag()` helper which returns a fingerprinted URL with immutable caching.

Enable debug mode by adding `s-debug` to the body:

```html
<body s-debug>
```

This enables console warnings and throws on template validation errors.

## **Three Systems**

Silcrow.js has three independent systems exposed through a single window.Silcrow API:

1. **Runtime** — reactive data binding and DOM patching via s-bind and s-list attributes
2. **Navigator** — client-side routing, history management, and response caching via s-action attributes
3. **Live** — SSE and WebSocket connections, optimistic updates, and real-time data streaming via s-live attributes

## **Runtime: Data Binding & DOM Patching**

### **Scalar Binding with s-bind**

Bind any element to a data path. The format is s-bind="path" for text content or s-bind="path:property" for element properties.

```html
<h1 s-bind="user.name"></h1>
<input s-bind="user.email:value" />
<img s-bind="user.avatar:src" />
<button s-bind="user.banned:disabled"></button>

```

Patch data into the DOM:

```javascript
Silcrow.patch({
  user: { name: "Alice", email: "a@b.com", avatar: "/img/alice.png", banned: false }
}, "#app");

```

The second argument is a root — either a CSS selector string or a DOM Element. Silcrow only patches bindings within that root.

**Known properties** (value, checked, disabled, selected, src, href, selectedIndex) are set as DOM properties. Everything else is set as an attribute. null or undefined values reset properties to their type default or remove attributes.

**Security:** Binding to event handler attributes (onclick, onload, etc.) is rejected. Text content is set via textContent, never innerHTML.

### **Collection Rendering with s-list**

Render collections of keyed objects into a container. Each item **must** have a `key` property.

```html
<ul s-list="todos" s-template="todo-tpl">
</ul>

<template id="todo-tpl">
  <li s-key=".key">
    <span s-bind=".text"></span>
    <input type="checkbox" s-bind=".done:checked" />
  </li>
</template>

```

**s-list dispatches on the shape of the value you patch:**

| Value shape | Mode | Behavior |
| --- | --- | --- |
| Array `[...]` | **Full sync** | Reconcile entire list — add new items, update existing, remove stale, reorder |
| Keyed object `{key, ...}` | **Merge** | Append or update a single item. All other items in the DOM are untouched. |
| Keyed object with `_remove: true` | **Remove** | Delete the single item matching the key. All other items are untouched. |

**Full sync** (initial load, delete, reorder):

```javascript
Silcrow.patch({
  todos: [
    { key: "1", text: "Buy milk", done: false },
    { key: "2", text: "Write docs", done: true },
  ]
}, "#app");

```

**Merge** (create or update a single item — no need to send the full list):

```javascript
Silcrow.patch({
  todos: { key: "3", text: "Ship it", done: false }
}, "#app");
```

The new item is appended; existing items with keys "1" and "2" are untouched. If an item with key "3" already exists, it is updated in-place.

**Remove** (delete a single item — no need to send the full list):

```javascript
Silcrow.patch({
  todos: { key: "2", _remove: true }
}, "#app");
```

The item with key "2" is removed from the DOM. All other items are untouched. The `_remove` field is a reserved tombstone sentinel — any other fields in the object are ignored.

**Direct targeting:** `s-target` can point directly to the `[s-list]` element (not its parent):

```html
<form s-action="/todos" POST s-target="#todo-list">...</form>
<ul id="todo-list" s-list="todos" s-template="todo-tpl">...</ul>

```

**Local bindings** use a leading dot (.text, .done) — they bind to fields on the individual item, not the global data object.

**Reconciliation:** Silcrow uses keyed reconciliation. Existing DOM nodes are reused by key, new items are created from the template, removed items are deleted, and order is maintained by repositioning. Duplicate keys are rejected.

**Template resolution order:**

```text
1. Item key prefix — if key is `special#3`, Silcrow looks for `<template id="special">`
2. `s-template` attribute on the container
3. Inline <template> child of the container

```

**Template rules:** Templates must contain exactly one element child. Scripts and event handler attributes inside templates are rejected during validation.

**Server-Rendered Lists (Hydration):**
Silcrow seamlessly handles collections that are pre-rendered by the server. If an item exists in the DOM with an `s-key` but was not created dynamically via Silcrow's `<template>` cloning, Silcrow will lazily scan and cache its `[s-bind]` attributes on the first patch. This allows you to serve fully populated HTML on initial load and effortlessly transition to client-side patches.

### **Silcrow.patch(data, root, options?)**

The core patching function. Options:

* invalidate: true — rebuilds the binding map from scratch (use after DOM mutations)
* silent: true — suppresses the silcrow:patched custom event

After each patch, a silcrow:patched event fires on the root with detail.paths listing all bound paths.

### **Silcrow.invalidate(root)**

Clears the cached binding map and template validations for a root. Call this when you've added or removed s-bind / s-list elements dynamically.

### **Silcrow.stream(root)**

Returns a microtask-batched update function. Multiple calls within the same microtask are coalesced — only the last data wins.

```javascript
const update = Silcrow.stream("#dashboard");
update({ count: 1 });
update({ count: 2 });
update({ count: 3 }); // only this patch executes

```

### **Path Resolution**

Dot-separated paths resolve into nested objects: `"user.profile.name"` reads `data.user.profile.name`. Prototype pollution paths (`__proto__`, `constructor`, `prototype`) are blocked and return `undefined`.

## **Navigator: Client-Side Routing**

### **Declarative Navigation with s-action**

Add `s-action` to any element to make it navigate on click:

```html
<a s-action="/dashboard">Dashboard</a>
<button s-action="/api/save" POST>Save</button>
<button s-action="/items/5" DELETE s-target="#item-5">Remove</button>

```

### **Attributes**

| **Attribute** | **Purpose** | **Default** |
| --- | --- | --- |
| `s-action` | URL to request | *(required)* |
| `s-target` | CSS selector — swap response into this element | Closest $$s-key$$ parent, or the triggering element itself |
| `s-html` | Request text/html instead of application/json | JSON |
| `s-skip-history` | Don't push to browser history | Push for full-page GETs |
| `s-preload` | Preload on mouse hover | Off |
| `s-timeout` | Request timeout in ms | 30000 |
| `GET`, `POST`, `PUT`, `PATCH`, `DELETE` | HTTP method (as attribute) | `GET` (or `POST` for forms) |

### **Actions within Lists (s-key Context)**

When building actions inside `s-list` templates, Silcrow provides two ergonomic features to eliminate boilerplate and avoid unnecessary `<form>` wrappers for simple actions:

1. **`{s-key}` Interpolation:** Any `{s-key}` string in your `s-action` or `s-target` attributes is automatically replaced with the value of the closest parent's `s-key` attribute.
2. **Implicit List Targeting:** If you omit the `s-target` attribute, the action will automatically bubble up to target the parent `[s-list]` container (falling back to the `[s-key]` item if orphaned). This perfectly aligns with `s-list` merge behavior: your server can return a single updated JSON object or an HTML fragment, and Silcrow will route it to the list container to append or update the item without needing explicit `s-target` wiring.

This allows you to write perfectly minimal, form-less action buttons inside your collections:

```html
<ul s-list="tasks">
  <template>
    <li s-key=".key">
      <span s-bind=".title"></span>
      <button s-action="/tasks/{s-key}/delete" DELETE>Delete</button>
    </li>
  </template>
</ul>

```

#### **Forms vs. Pure Buttons for Mutations**

Because of {s-key} interpolation and implicit targeting, you have two distinct tools depending on whether your mutation requires a request body. While the HTTP specification builds POST, PUT, and PATCH to carry bodies, it does *not* mandate them.

**1. When you NEED a body → Use a `<form>`** If the user is submitting new data (like typing a task title), you must use a form. Silcrow relies on the form boundary to serialize inputs into a FormData request body.

```html
<form s-action="/tasks/{s-key}/edit" method="PUT">  
  <input type="text" name="title" s-bind=".title" />  
  <button type="submit">Save</button>  
</form>

```

**2. When you DON'T need a body → Use a Pure `<button>`**

If the action is binary and the URL itself contains all the required context (via the ID), you don't need a body. You can use form-less buttons for POST, PUT, and PATCH just like you do for DELETE.

```html
<button s-action="/tasks/{s-key}/toggle" PATCH>Toggle Complete</button>  
<button s-action="/tasks/{s-key}/upvote" POST>Upvote</button>
```

**The Architect's Rule of Thumb:**

* Use `DELETE` (pure button) to destroy a resource.
* Use `POST` / `PUT` / `PATCH` (pure button) to trigger a specific, parameter-less action (like "star", "archive", "toggle").
* Use `<form method="...">` only when sending user input fields.

**Server-Side Example (Axum):**

For a pure button, the backend handler simply extracts the ID from the path and processes the action without expecting a body, returning the updated fragment directly to the targeted `{s-key}` item.

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

### **Forms**

Forms with `s-action` are intercepted automatically. GET forms append FormData as query params. Other methods send FormData as the body.

```html
<form s-action="/search" GET s-target="#results">
  <input name="q" />
  <button>Search</button>
</form>

```

### **Programmatic Navigation**

```javascript
Silcrow.go("/dashboard");
Silcrow.go("/api/items", { method: "POST", body: { name: "New" }, target: "#list" });

```

### **Response Processing**

The navigator reads the Content-Type header to decide how to handle the response:

* **JSON** (application/json) — parsed and passed to Silcrow.patch() on the target element
* **HTML** (text/html) — sanitized and swapped into the target element's innerHTML

For HTML responses, if the response is a full page (`<!DOCTYPE` or `<html>`), Silcrow extracts the `<title>` and the matching `s-target` selector content (or `<body>` as fallback).

**HTML sanitization:**

Silcrow uses the Sanitizer API (`el.setHTML()`) when available. When it isn't, a DOMParser fallback strips all `<script>` elements and event handler attributes (`on*`) before insertion.

### **Server-Driven Headers**

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

### **Caching**

GET responses are cached in-memory for 5 minutes (max 50 entries). Any mutation request (POST, PUT, PATCH, DELETE) clears the entire cache. The server can opt out per-response with the `silcrow-cache: no-cache` header.

```javascript
Silcrow.cache.has("/dashboard");  // check cache
Silcrow.cache.clear("/dashboard"); // clear one entry
Silcrow.cache.clear();             // clear all
```

### **Preloading**

Elements with `s-preload` fire a background fetch on `mouseenter`. The response is cached so the subsequent click is instant.

```html
<a s-action="/settings" s-preload>Settings</a>
```

### **History & Scroll**

Full-page GET navigations push to history.pushState. On popstate (back/forward), Silcrow re-fetches the URL and restores the saved scroll position. Partial updates (those with s-target) skip history by default.

### **Loading States**

During requests, Silcrow adds `silcrow-loading` CSS class and `aria-busy="true"` to the target element. Style it however you want:

```css
.silcrow-loading { opacity: 0.5; pointer-events: none; }
```

### **Abort & Timeout**

Navigating to the same target while a GET is in-flight aborts the previous request. Mutation requests are never aborted. Timeout defaults to 30 seconds and can be set per-element with s-timeout.

## **Live: SSE, WebSocket & Real-Time Updates**

### **Declarative with s-live**

Add `s-live` to any element to automatically open an SSE connection on page load. The attribute value is the SSE endpoint URL:

```html
<div id="feed" s-live="/events/feed">
  <span s-bind="count"></span> items
</div>

```

Silcrow scans for s-live elements during initialization. When the server sends an SSE message, the data is parsed as JSON and piped to Silcrow.patch() on that element.

### **WebSocket with s-live**

Prefix the URL with `ws:` to use WebSocket instead of SSE:

```html
<div id="chat" s-live="ws:/ws/chat">
  <span s-bind="messages"></span>
</div>

```

Without a prefix, s-live defaults to SSE for backward compatibility.

### **Connection Sharing**

When multiple elements connect to the same WebSocket URL, Silcrow opens a single shared connection. Messages with an explicit target selector are applied once to the matching element. Messages without a target fan out to all subscribed elements.

This is automatic — no configuration needed. If you need isolated connections to the same URL (rare), use distinct query parameters: ws:/ws/chat?room=1 vs ws:/ws/chat?room=2.

### **Programmatic with Silcrow.live()**

```javascript
Silcrow.live("#dashboard", "/events/dashboard");

```

Opens an EventSource to the given URL. Every message event is parsed as JSON and passed to Silcrow.patch() on the root element.

### **SSE Message Format**

The server sends standard SSE messages. The data field must be valid JSON:

```sse
data: {"count": 42, "status": "online"}

```

Silcrow also supports named SSE events for specific actions:

| **Event Name** | **Effect** |
| --- | --- |
| `message` (default) | Parsed as JSON, passed to `Silcrow.patch()` on the root |
| `patch` | Parsed and patched. Supports direct payload on root, or `{target, data}` to patch a specific selector |
| `html` | Swaps HTML via `safeSetHTML()`. Supports `{target, html}`; empty `html` clears target content |
| `invalidate` | Calls `Silcrow.invalidate()` on the root (no data needed) |
| `navigate` | `data` field is a URL path — triggers client-side navigation |

```sse
event: navigate
data: /dashboard

event: invalidate
data:

event: patch
data: {"users": [{"key": "1", "name": "Alice"}]}

event: patch
data: {"target":"#dashboard","data":{"count":42}}

event: html
data: {"target":"#slot","html":"<p>Updated</p>"}

```

### **Reconnection**

When an SSE connection drops, Silcrow reconnects automatically with exponential backoff: 1s → 2s → 4s → 8s → ... up to a maximum of 30 seconds. Backoff resets on successful reconnection or on a manual Silcrow.reconnect() call.

### **Silcrow.disconnect(root)**

Pauses the SSE connection for a root. The connection is closed and automatic reconnection is stopped.

```javascript
Silcrow.disconnect("#feed");

```

### **Silcrow.reconnect(root)**

Resumes a disconnected SSE connection. Resets the backoff timer and reconnects immediately.

```javascript
Silcrow.reconnect("#feed");

```

### **Sending Messages (WebSocket only)**

WebSocket connections are bidirectional. Use `Silcrow.send()` to send data to the server:

```javascript
Silcrow.send("#chat", { type: "custom", event: "message", data: { text: "Hello" } });

```

send() is a no-op on SSE connections (SSE is server-to-client only). The connection must be open — if not, a warning is logged.

### **WebSocket Message Format**

WebSocket messages are JSON objects with a `type` field that matches the Rust `WsEvent` enum:

| **Type** | **Fields** | **Effect** |
| --- | --- | --- |
| `patch` | `target`, `data` | Patches JSON data into target element via `Silcrow.patch()` |
| `html` | `target`, `markup` | Swaps HTML into target element via `safeSetHTML()` |
| `invalidate` | `target` | Rebuilds binding maps for target element |
| `navigate` | `path` | Triggers client-side navigation |
| `custom` | `event`, `data` | Dispatches `silcrow:ws:{event}` CustomEvent on `document` |

```json
{"type": "patch", "target": "#stats", "data": {"count": 42}}
{"type": "html", "target": "#slot", "markup": "<p>Updated</p>"}
{"type": "navigate", "path": "/dashboard"}
{"type": "custom", "event": "refresh", "data": {"section": "sidebar"}}

```

## **Optimistic Updates**

### **Silcrow.optimistic(root, data)**

Takes a snapshot of the root element's current DOM state, then immediately patches the data. Use this for instant UI feedback before the server confirms:

```javascript
// User clicks "like" — update immediately
Silcrow.optimistic("#post-42", { likes: currentLikes + 1, liked: true });

// Send to server
Silcrow.go("/api/posts/42/like", { method: "POST", target: "#post-42" });

```

### **Silcrow.revert(root)**

Restores the DOM to the state captured by `Silcrow.optimistic()`. Call this when the server request fails:

```javascript
try {
  await fetch("/api/posts/42/like", { method: "POST" });
} catch (err) {
  Silcrow.revert("#post-42");
  showError("Failed to save");
}

```

revert() restores the element's innerHTML and calls Silcrow.invalidate() to rebuild binding maps since the DOM was replaced.

### **Optimistic + Error Handler Pattern**

Combine optimistic updates with the error handler for a clean pattern:

```javascript
Silcrow.onError((err, { url, target }) => {
  // Revert any optimistic updates on the failed target
  Silcrow.revert(target);
});

```

## **Toast System**

Register a toast handler to receive toast notifications from both JSON payloads and cookie-based HTML responses:

```javascript
Silcrow.onToast((message, level) => {
  showNotification(message, level); // your UI
});

```

**JSON responses:** Toasts are read from the `_toasts` array in the payload, then removed before patching. If the payload was wrapped by the server (non-object root with toasts), Silcrow unwraps it.

**HTML/redirect responses:** Toasts are read from the `silcrow_toasts` cookie (URL-encoded JSON array), then the cookie is immediately cleared.

## **Events**

All events bubble and are dispatched on document (except `silcrow:patched` which fires on the root element).

| **Event** | **Detail** | **Cancelable** | **When** |
| --- | --- | --- | --- |
| `silcrow:navigate` | `{url, method, trigger, target}` | Yes | Before any fetch |
| `silcrow:before-swap` | `{url, target, content, isJSON, proceed}` | Yes | After fetch, before DOM update |
| `silcrow:load` | `{url, target, redirected}` | No | After successful swap |
| `silcrow:error` | `{error, url}` | No | On fetch error or timeout |
| `silcrow:patched` | `{paths}` | No | After `patch()` completes |
| `silcrow:sse` | `{path}` | No | When server sends `silcrow-sse` header |
| `silcrow:live:connect` | `{root, url}` | No | SSE connection opened |
| `silcrow:live:disconnect` | `{root, url, reconnectIn}` | No | SSE connection lost (with backoff ms) |
| `silcrow:optimistic` | `{root, data}` | No | After optimistic patch applied |
| `silcrow:revert` | `{root}` | No | After DOM reverted to snapshot |

**Transition hook:** Listen to `silcrow:before-swap` and call `event.detail.proceed()` manually to control when the DOM update happens (e.g., after a CSS transition). If no listener calls `proceed()`, Silcrow executes it automatically.

## **Lifecycle**

```javascript
// Register handlers (chainable)
Silcrow
  .onToast((msg, level) => { /* ... */ })
  .onRoute(({ url, finalUrl, redirected, method, response, contentType, target }) => {
    // Return false to prevent the default swap
  })
  .onError((err, { url, method, trigger, target }) => {
    // Custom error handling
  });

// Teardown — removes all event listeners, clears caches, closes SSE connections
Silcrow.destroy();

```

## **API Reference**

### **[Runtime](#runtime-data-binding--dom-patching)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.patch(data, root, options?)` | Patch data into bound elements under root |
| `Silcrow.invalidate(root)` | Clear cached binding maps for root |
| `Silcrow.stream(root)` | Returns microtask-batched updater function |

### **[Navigation](#navigator-client-side-routing)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.go(path, options?)` | Programmatic navigation |
| `Silcrow.cache.has(path)` | Check if a path is cached |
| `Silcrow.cache.clear(path?)` | Clear one or all cache entries |

### **[Live (SSE)](#live-sse-websocket--real-time-updates)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.live(root, url)` | Open SSE connection, pipe messages to `patch()` |
| `Silcrow.send(root, data)` | Send data over a WebSocket connection |
| `Silcrow.disconnect(root)` | Pause SSE connection and stop auto-reconnect |
| `Silcrow.reconnect(root)` | Resume SSE connection with reset backoff |

### **[Optimistic Methods](#optimistic-updates)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.optimistic(root, data)` | Snapshot DOM, then patch immediately |
| `Silcrow.revert(root)` | Restore DOM from snapshot, invalidate bindings |

### **[Lifecycle Handlers](#lifecycle)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.onToast(handler)` | Register toast callback (chainable) |
| `Silcrow.onRoute(handler)` | Register route middleware (chainable) |
| `Silcrow.onError(handler)` | Register error handler (chainable) |
| `Silcrow.destroy()` | Teardown all listeners, caches, and SSE connections |

`window.SilcrowNavigate` is available as a backward-compatible alias for `window.Silcrow`.

## **Compatibility**

Silcrow.js requires a modern browser with support for `fetch`, `URL`, `CustomEvent`, `WeakMap`, `queueMicrotask`, `EventSource`, and `<template>`. No polyfills are bundled.
