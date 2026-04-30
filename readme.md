# **Silcrow.js**

A lightweight client-side runtime for building hypermedia-driven applications. Silcrow handles DOM patching, client-side navigation, response caching, live SSE and WebSocket connections, optimistic updates, a headless atom store, and server-driven UI orchestration from declarative HTML attributes.

Silcrow.js is the frontend counterpart to Pilcrow but operates independently as a standalone library. Any backend that speaks HTTP and returns HTML or JSON can drive it.

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

## **Systems**

**Runtime** updates DOM text, properties, visibility, classes, attributes, and keyed fragments through colon bindings, `s-use`, and `s-for`.

**Atoms** provide a framework-agnostic reactive store for route data, stream data, user-named scopes, SSR seeds, and `s-bind` DOM subscriptions.

**Navigator** handles client-side routing, mutations, response caching, history, forms, preloading, `:key` URL interpolation, and server-driven response headers through `s-get`, `s-post`, `s-put`, `s-patch`, and `s-delete`.

**Live** manages SSE and WebSocket connections through `s-sse`, `s-ws`, and `s-wss`, including connection sharing, reconnection, patches, swaps, invalidation, navigation, and custom events.

**Optimistic** stages instant UI updates and can revert them if the server request fails.

## **Reference Documentation**

The canonical Silcrow reference is the structured MCP manifest at `mcp/docs.json`. The MCP server exposes it through `searchDocs`, `getDoc`, `getSection`, `getExamples`, and `analyzeSilcrowUsage`.

Validate the manifest with:

```bash
npm run test:docs
```

## **Compatibility**

Silcrow.js requires a modern browser with support for `fetch`, `URL`, `CustomEvent`, `WeakMap`, `queueMicrotask`, `EventSource`, and `<template>`. No polyfills are bundled.
