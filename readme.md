# **Silcrow.js**

A lightweight client-side runtime for building hypermedia-driven applications. Silcrow handles DOM patching, client-side navigation, response caching, live SSE and WebSocket connections, optimistic updates, and server-driven UI orchestration — all from declarative HTML attributes.  
Silcrow.js is the frontend counterpart to [Pilcrow](https://www.google.com/search?q=readme.md) but operates independently as a standalone library. Any backend that speaks HTTP and returns HTML or JSON can drive it.

## **Table of Contents**

* [Loading](#loading)  
* [Three Systems](#three-systems)  
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

1. **[Runtime](docs/runtime.md)** — Reactive data binding via colon-shorthands (`:text`, `:value`, `:show`), spread binding with `s-use`, and fragment-aware `s-for` loops with keyed reconciliation. Data flows through middleware → toast extraction → smart unwrapping before patching.

2. **[Navigator](docs/navigator.md)** — Client-side routing, history management, and response caching via `s-action` attributes. Supports implicit targeting, `:key` interpolation, server-driven headers, preloading, and form serialization.

3. **[Live](docs/live.md)** — SSE and WebSocket connections via `s-sse` and `s-ws` attributes. Hub-based connection sharing, automatic reconnection with exponential backoff, and structured message formats for patches, HTML swaps, invalidation, and navigation.

**[Optimistic Updates](docs/optimistic.md)** — Snapshot & revert for instant UI feedback. Works with the Live and Navigator systems.

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
  .use((data) => { /* global transform */ return data; })
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

### **[Runtime](docs/runtime.md)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.use(fn)` | Register a global data transformer |
| `Silcrow.patch(data, root, options?)` | Process middleware, unwrap data, and apply `s-use` and `:` bindings |
| `Silcrow.invalidate(root)` | Clear cached binding maps for root |
| `Silcrow.stream(root)` | Returns microtask-batched updater function |

### **[Navigation](docs/navigator.md)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.go(path, options?)` | Programmatic navigation |
| `Silcrow.cache.has(path)` | Check if a path is cached |
| `Silcrow.cache.clear(path?)` | Clear one or all cache entries |

### **[Live (SSE & WebSocket)](docs/live.md)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.live(root, url)` | Open an SSE connection, pipe messages to `patch()` |
| `Silcrow.send(data, root)` | Send data over an established WebSocket connection |
| `Silcrow.disconnect(root)` | Pause a live connection (SSE or WS) and stop auto-reconnect |
| `Silcrow.reconnect(root)` | Resume a disconnected connection with reset backoff |

### **[Optimistic](docs/optimistic.md)**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.optimistic(data, root)` | Snapshot DOM, then patch immediately |
| `Silcrow.revert(root)` | Restore DOM from snapshot, invalidate bindings |

### **Lifecycle Handlers**

| **Method** | **Description** |
| --- | --- |
| `Silcrow.onToast(handler)` | Register toast callback (chainable) |
| `Silcrow.onRoute(handler)` | Register route middleware (chainable) |
| `Silcrow.onError(handler)` | Register error handler (chainable) |
| `Silcrow.destroy()` | Teardown all listeners, caches, and SSE connections |

## **Compatibility**

Silcrow.js requires a modern browser with support for `fetch`, `URL`, `CustomEvent`, `WeakMap`, `queueMicrotask`, `EventSource`, and `<template>`. No polyfills are bundled.
