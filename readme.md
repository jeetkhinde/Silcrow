# **Silcrow.js**

A lightweight client-side runtime for building hypermedia-driven applications. Silcrow handles DOM patching, client-side navigation, response caching, live SSE and WebSocket connections, optimistic updates, and server-driven UI orchestration — all from declarative HTML attributes.  
Silcrow.js is the frontend counterpart to Pilcrow but operates independently as a standalone library. Any backend that speaks HTTP and returns HTML or JSON can drive it.

## **Table of Contents**

* [Loading](#loading)  
* [Three Systems](#three-systems)  
* [Reference Documentation](#reference-documentation)
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

2. **[Navigator](docs/navigator.md)** — Client-side routing, history management, and response caching via verb attributes (`s-get`, `s-post`, `s-put`, `s-patch`, `s-delete`). Supports implicit targeting, `:key` interpolation, server-driven headers, preloading, and form serialization.

3. **[Live](docs/live.md)** — SSE and WebSocket connections via `s-sse` and `s-ws` attributes. Hub-based connection sharing, automatic reconnection with exponential backoff, and structured message formats for patches, HTML swaps, invalidation, and navigation.

**[Optimistic Updates](docs/optimistic.md)** — Snapshot & revert for instant UI feedback. Works with the Live and Navigator systems.

## **Reference Documentation**

Silcrow's complete feature set is documented in the following reference guides:

* **[HTML Attributes & Keywords](docs/attributes.md)** — Data-binding, fetch triggers, loops, and visual indicators.
* **[HTTP Headers](docs/http-headers.md)** — Server-sent headers for orchestrating UI patches, invalidations, navigation, and live streams.
* **[Events](docs/events.md)** — All custom lifecycle events dispatched and consumed by the runtime.
* **[JavaScript API](docs/javascript-api.md)** — The public methods on `window.Silcrow` to execute logic programmatically.

## **Compatibility**

Silcrow.js requires a modern browser with support for `fetch`, `URL`, `CustomEvent`, `WeakMap`, `queueMicrotask`, `EventSource`, and `<template>`. No polyfills are bundled.
