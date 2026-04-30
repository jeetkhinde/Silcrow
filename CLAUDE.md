# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Setup

```bash
npm install          # also installs mcp/server deps via postinstall
```

## Build Commands

```bash
npm run build        # Bundle src/ -> dist/silcrow.js + dist/silcrow.min.js
npm run watch        # Rebuild on source changes
npm run build:docs   # Compatibility alias: validate canonical mcp/docs.json
npm run test:docs    # Validate canonical mcp/docs.json
npm run mcp          # Start the MCP server (stdio transport)
npm run mcp:reload   # Send SIGHUP to reload docs without restarting
npm run mcp:redoc    # test:docs then mcp:reload in one step
npm run mcp:test     # Run all MCP server tests
```

The build (`build.js`) concatenates source files in strict dependency order, wraps them in a strict-mode IIFE, and produces both unminified and minified bundles with Terser at ES2020 target.

## Architecture

Silcrow is a zero-dependency, single-file client-side library for hypermedia-driven UIs. It ships as an IIFE via `<script src="silcrow.js" defer>` and exposes `window.Silcrow`. The source is split into orthogonal systems that are concatenated at build time.

### Source dependency order (critical for build)

```
debug.js -> url-safety.js -> safety.js -> toasts.js
-> patcher.js -> atoms.js -> live.js -> ws.js -> navigator.js -> optimistic.js -> index.js
```

### Main systems

**Runtime** (`src/patcher.js`)  
DOM patching and reactive data binding. Drives colon-shorthand attributes (`:text`, `:value`, `:show`, `:class`), spread binding via `s-use`, and keyed list reconciliation via `s-for`. Protects against prototype pollution by blocking `__proto__`, `constructor`, and `prototype` keys.

**Atoms** (`src/atoms.js`)  
Framework-agnostic reactive store used by route, stream, and user-named scopes. Exposes `Silcrow.prefetch`, `submit`, `subscribe`, `snapshot`, and `publish`, plus `s-bind` for vanilla DOM subscriptions.

**Navigator** (`src/navigator.js`)  
Client-side routing and history management. Driven by HTTP verb attributes: `s-get`, `s-post`, `s-put`, `s-patch`, `s-delete`. Handles form serialization, `:key` URL interpolation, implicit target resolution, response caching, mouseenter preloading, and server-driven response headers.

**Live** (`src/live.js` + `src/ws.js`)  
Real-time communication. `s-sse` drives SSE connections; `s-ws` and `s-wss` drive bidirectional WebSocket. Both use hub-based connection sharing and exponential backoff reconnection. Incoming messages may carry patches, HTML swaps, invalidation signals, navigation instructions, or custom events.

**Optimistic** (`src/optimistic.js`)  
Snapshot/revert for instant UI feedback. Works alongside Navigator and Live to stage changes before server confirmation and roll back on failure.

### Bootstrap (`src/index.js`)

Auto-initializes on `DOMContentLoaded`. Registers global event listeners (click, submit, popstate, mouseenter), seeds atoms from SSR data, initializes live elements and `s-bind` subscriptions, and starts a MutationObserver for live/atom cleanup. This is the only place where the public `window.Silcrow` API surface is assembled.

## MCP Server

`mcp/server/` is a production MCP server that exposes the canonical Silcrow docs as a queryable API.

**Canonical docs source:** `mcp/docs.json`. Edit this structured JSON directly when docs change. The old Markdown docs pipeline has been removed to prevent source drift.

**Tools:** `searchDocs(query)`, `getDoc(id)`, `getSection(id)`, `getExamples(topic)`, `analyzeSilcrowUsage(code)`

**Architecture:**
- `lib/docs-loader.js` — loads `mcp/docs.json` once at startup, builds two Fuse.js indices (docs + flattened sections), exposes a frozen singleton store
- `lib/search-index.js` — Fuse index construction; docs weighted title 35% / tags 25% / summary 20%
- `lib/silcrow-catalog.js` — maps every known `s-*` attribute, `:binding`, and `Silcrow.*` API method to a doc ID; keep aligned with `src/index.js`
- `lib/code-parser.js` — regex extractor for Silcrow constructs in HTML/JS (returns `Map<name, count>`)
- `lib/analysis-rules.js` — static analysis rules for unknown attrs, deprecated patterns, missing `:key`, multiple HTTP verbs, security notes, and unknown APIs
- `tools/` — one file per MCP tool; each exports `definition` (MCP schema) and `handler(args, store)`

**Docs path override** (for multi-project reuse):
```bash
node mcp/server/server.js --docs /path/to/other/docs.json
# or
SILCROW_DOCS_PATH=/path/to/other/docs.json npm run mcp
```

When `mcp/docs.json` changes, run `npm run test:docs` before reloading the server.

## Key conventions

- Features are attribute-first: behavior is declared in HTML, not configured in JS.
- Runtime, Atoms, Navigator, Live, and Optimistic are designed to be used independently.
- Dist files are committed (`dist/silcrow.js`, `dist/silcrow.min.js`); always rebuild before committing source changes.
- Keep `mcp/docs.json`, `mcp/server/lib/silcrow-catalog.js`, and `src/index.js` aligned whenever public APIs change.
