# Silcrow.js

A lightweight, zero-dependency client-side runtime for hypermedia-driven applications. Silcrow handles DOM patching, client-side navigation, response caching, live SSE and WebSocket connections, optimistic updates, a headless atom store, and server-driven UI orchestration — all from declarative HTML attributes.

Silcrow.js is the frontend counterpart to Pilcrow but operates independently. Any backend that speaks HTTP and returns HTML or JSON can drive it.

---

## Installation

Silcrow.js is a single self-executing IIFE. Include it in your page:

```html
<script src="/_silcrow/silcrow.js" defer></script>
```

If using Pilcrow on the backend, use the `script_tag()` helper which returns a fingerprinted URL with immutable caching.

Enable debug mode by adding `s-debug` to the body:

```html
<body s-debug>
```

This enables console warnings and throws on template validation errors.

### Browser support

Modern browsers with support for `fetch`, `URL`, `CustomEvent`, `WeakMap`, `queueMicrotask`, `EventSource`, and `<template>`. No polyfills are bundled.

---

## Quick Start

A counter that talks to a server with no JavaScript:

```html
<div id="counter">
  <span :text="count">0</span>
  <button s-post="/increment" s-target="#counter">+</button>
</div>
```

The server responds with `{"count": 1}` and Silcrow patches `:text="count"` automatically.

---

## Core Concepts

Silcrow is **attribute-first**: behavior is declared in HTML, not configured in JS. Five subsystems work together but are usable independently.

### 1. Runtime — DOM patching

Bind data to the DOM with colon-prefixed attributes. The server returns JSON, Silcrow patches matching bindings.

```html
<div id="user">
  <h1 :text="name">Loading…</h1>
  <img :src="avatar" alt="">
  <span :class="{ active: online }">●</span>
  <button :disabled="busy">Save</button>
</div>
```

```js
Silcrow.patch({ name: "Ada", avatar: "/a.png", online: true, busy: false }, "#user");
```

**Spread binding** with `s-use` applies an entire object:

```html
<input s-use="ui.search">
<!-- patches { value, placeholder, disabled, ... } in one go -->
```

**Keyed lists** with `s-for` and `:key` reconcile efficiently:

```html
<ul>
  <template s-for="item in todos" :key="item.id">
    <li :key>
      <span :text="item.title"></span>
      <button s-delete="/todos/:key">×</button>
    </li>
  </template>
</ul>
```

The server can patch the whole list (`{ todos: [...] }`) or a single item (`{ todos: { id: 7, title: "updated" } }`) — Silcrow merges in place. Send `{ id: 7, _remove: true }` to delete.

### 2. Navigator — routing and mutations

HTTP verb attributes drive navigation and mutations. Forms and clicks are intercepted, requests are sent with `silcrow-target: true`, responses (HTML or JSON) are swapped or patched into the target.

```html
<a s-get="/dashboard">Dashboard</a>
<button s-post="/logout">Log out</button>

<form s-post="/posts" s-target="#feed">
  <input name="title">
  <button>Publish</button>
</form>

<a s-get="/products" s-preload>Products</a>  <!-- prefetches on hover -->
```

Targets resolve in this order: explicit `s-target` → nearest `[:key]` block (for `s-for` items) → triggering element. Use `:key` inside URLs for interpolation: `s-delete="/todos/:key"`.

GET responses are cached for 5 minutes (max 50 entries); mutations bust the cache. Override per-response with `silcrow-cache: no-cache`.

**Server-driven side effects** via response headers:

| Header | Effect |
|---|---|
| `silcrow-patch` | JSON envelope `{target, data}` patched into element |
| `silcrow-invalidate` | CSS selector to re-scan bindings |
| `silcrow-navigate` | Trigger a client navigation |
| `silcrow-retarget` | Override the swap target for this response |
| `silcrow-push` | Override the URL pushed to history |
| `silcrow-trigger` | JSON `{eventName: detail, ...}` dispatched on document |
| `silcrow-sse` / `silcrow-ws` | Open a live connection after this response |
| `silcrow-cache` | `no-cache` to skip caching |
| `silcrow-full-reload` | `true` to make top-level boosted GETs fall back to browser navigation |

### 3. Live — SSE and WebSockets

Open a connection declaratively. Both protocols share a hub-based pool with exponential backoff and automatic reconnection.

```html
<!-- SSE: server-pushed updates -->
<div s-sse="/events/dashboard">
  <span :text="users.online">…</span>
</div>

<!-- WebSocket: bidirectional -->
<div s-ws="/chat">
  <ul><template s-for="msg in messages" :key="msg.id"><li :key :text="msg.text"></li></template></ul>
</div>
```

Both protocols accept structured messages with these types:

```json
{ "type": "patch", "target": "#feed", "data": { "items": [...] } }
{ "type": "html",  "target": "#sidebar", "markup": "<nav>…</nav>" }
{ "type": "invalidate", "target": "#feed" }
{ "type": "navigate", "path": "/login" }
{ "type": "custom", "event": "user-joined", "data": { "name": "Ada" } }
```

Custom events are dispatched as `silcrow:sse:<event>` or `silcrow:ws:<event>`.

Send messages back over WebSocket:

```js
Silcrow.send({ type: "message", text: "hello" }, "#chat");
```

Same-origin only. SSE requires `http(s):`, WebSockets require `ws(s):` — cross-origin URLs are rejected.

### 4. Atoms — headless reactive store

Atoms are the canonical sink for network-sourced data. The DOM patcher is one consumer; React, Solid, Vue, and Svelte adapters subscribe via `Silcrow.subscribe`. Structural sharing keeps `Object.is` stable for unchanged subtrees, so React 19's `useSyncExternalStore` and `use()` are safe.

Three scope namespaces:

| Scope | Source | Example |
|---|---|---|
| `route:<path>` | Top-level GET JSON responses, prefetches | `route:/dashboard` |
| `stream:<url>` | SSE/WS messages | `stream:/events/dashboard` |
| `<custom>` | User-published data | `cart`, `user`, etc. |

```js
// Subscribe (returns unsubscribe fn)
const unsub = Silcrow.subscribe("route:/dashboard", data => render(data));

// Read current value
const snap = Silcrow.snapshot("cart");

// Publish to a custom scope (deep-merges)
Silcrow.publish("cart", { items: [...] });

// Identity-stable promise for React's use()
const dataPromise = Silcrow.prefetch("/dashboard");

// Async submit (for useActionState)
const { ok, status, data } = await Silcrow.submit("/posts", { title: "Hello" });
```

**Vanilla DOM binding** — subscribe an element to a scope without JS:

```html
<div s-bind="cart">
  <span :text="items.length"></span>
</div>
```

**SSR hydration** — seed atoms before boot to skip the first roundtrip:

```html
<script>
  window.__silcrow_seed = {
    "/dashboard": { users: { online: 42 } }
  };
</script>
<script src="/_silcrow/silcrow.js" defer></script>
```

### 5. Optimistic — instant UI feedback

Snapshot the DOM, apply a guess, revert if the server disagrees:

```js
Silcrow.optimistic({ todos: [...current, draft] }, "#list");

try {
  await Silcrow.submit("/todos", draft);
} catch {
  Silcrow.revert("#list");
}
```

---

## Toasts

Server can push notifications via JSON `_toasts` arrays or a `silcrow_toasts` cookie. Register a handler once:

```js
Silcrow.onToast((message, level) => showToast(message, level));
```

Server responses like `{ _toasts: [{message: "Saved", level: "success"}], data: {...} }` strip toasts before patching the rest.

---

## Events

Listen for lifecycle events on `document`:

| Event | When |
|---|---|
| `silcrow:navigate` | Before a navigation request (cancelable) |
| `silcrow:before-swap` | Before swapping content (cancelable; `detail.proceed()` available) |
| `silcrow:patched` | After a patch completes |
| `silcrow:load` | After navigation finishes |
| `silcrow:error` | On request failure or timeout |
| `silcrow:live:connect` / `:disconnect` | Live connection state changes |
| `silcrow:optimistic` / `:revert` | Optimistic update lifecycle |
| `silcrow:sse:<event>` / `silcrow:ws:<event>` | Server custom events |

---

## Middleware

Transform every patch payload before it reaches the DOM. Must be registered before `init()` runs (i.e. before `DOMContentLoaded` fires):

```js
Silcrow.use(data => {
  if (data.timestamp) data.timestamp = new Date(data.timestamp).toLocaleString();
  return data;
});
```

```js
Silcrow.onRoute(async ctx => { /* return false to halt swap */ });
Silcrow.onError((err, ctx) => reportToSentry(err));
```

---

## Security

Silcrow defends against common XSS vectors automatically:

- **Sanitization** — `safeSetHTML` strips `<script>`, `<style>`, `<iframe>`, inline event handlers, `style` attributes, `srcdoc`, and unsafe URL protocols. Uses native `Element.setHTML()` where available.
- **URL validation** — `href`, `src`, `action`, `formaction`, `xlink:href`, `poster`, `cite`, and `background` reject `javascript:`, `vbscript:`, `file:`, and unsafe `data:` URLs. `data:image/*;base64` allowed only on `<img src>`.
- **`target="_blank"` hardening** — `rel="noopener noreferrer"` is forced on all `_blank` links.
- **Prototype pollution** — `__proto__`, `constructor`, and `prototype` keys are blocked in patch payloads, atom merges, and path resolution.
- **Same-origin Live** — SSE and WebSocket connections to other origins are refused.

---

## Reference Documentation

The complete directive, attribute, header, event, and API surface is in [`docs/silcrow-api.md`](docs/silcrow-api.md).

---

## Build

```bash
npm install
npm run build       # src/silcrow.js -> dist/silcrow.js + dist/silcrow.min.js
npm run watch       # rebuild on src/ changes
```

`src/silcrow.js` is the canonical source. `dist/silcrow.js` and `dist/silcrow.min.js` are generated build artifacts.

---

## License

See LICENSE.
