# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Setup

```bash
npm install          # also installs mcp/server/ deps via postinstall
```

## Build Commands

```bash
npm run build        # Bundle src/ → dist/silcrow.js + dist/silcrow.min.js
npm run watch        # Rebuild on source changes
npm run build:docs   # Generate mcp/generated/docs.json from docs/*.md
npm run test:docs    # Validate HTML/JSON examples embedded in docs
npm run mcp          # Start the MCP server (stdio transport)
npm run mcp:reload   # Send SIGHUP to reload docs without restarting
npm run mcp:redoc    # build:docs then mcp:reload in one step
npm run mcp:test     # Run all MCP server tests (66 assertions)
```

The build (`build.js`) concatenates source files in strict dependency order, wraps them in a strict-mode IIFE, and produces both unminified (58 KB) and minified (25 KB) bundles with Terser at ES2020 target. There is no traditional test framework — correctness testing is documentation-example-based via `test:docs`.

## Architecture

Silcrow is a zero-dependency, single-file client-side library for hypermedia-driven UIs. It ships as an IIFE via `<script src="silcrow.js" defer>` and exposes `window.Silcrow`. The source is split into four orthogonal systems that are concatenated at build time.

### Source dependency order (critical for build)

```
debug.js → url-safety.js → safety.js → toasts.js
→ patcher.js → live.js → ws.js → navigator.js → optimistic.js → index.js
```

### The four systems

**Runtime** (`src/patcher.js`)  
DOM patching and reactive data binding. Drives colon-shorthand attributes (`:text`, `:value`, `:show`, `:class`, `:attr:*`), spread binding via `s-use`, and keyed list reconciliation via `s-for`. Uses WeakMap instance caching. Protects against prototype pollution by blocking `__proto__`, `constructor`, `prototype` keys.

**Navigator** (`src/navigator.js`)  
Client-side routing and history management. Driven by HTTP verb attributes: `s-get`, `s-post`, `s-put`, `s-patch`, `s-delete`. Handles form serialization, `:key` URL interpolation, implicit target resolution (nearest parent loop or `s-target`), response caching (5-min TTL, max 50 entries), and mouseenter preloading. Server-driven UI orchestration via custom response headers.

**Live** (`src/live.js` + `src/ws.js`)  
Real-time communication. `s-sse` drives SSE connections; `s-ws` drives bidirectional WebSocket. Both use hub-based connection sharing and exponential backoff reconnection (max 30s). Incoming messages may carry patches, HTML swaps, invalidation signals, or navigation instructions. Cross-origin protection and protocol validation are enforced.

**Optimistic** (`src/optimistic.js`)  
Snapshot/revert for instant UI feedback. Works alongside Navigator and Live to stage changes before server confirmation and roll back on failure.

### Bootstrap (`src/index.js`)

Auto-initializes on `DOMContentLoaded`. Registers global event listeners (click, submit, popstate, mouseenter) and a MutationObserver for live element cleanup. Locks the middleware pipeline after init. This is the only place where the public `window.Silcrow` API surface is assembled.

### Supporting modules

- `src/safety.js` — HTML sanitization, forbidden tag stripping, attribute and protocol validation
- `src/url-safety.js` — Protocol whitelist, srcset validation
- `src/toasts.js` — Toast extraction from JSON payloads or cookies
- `src/debug.js` — Conditional logging; enabled by `s-debug` attribute on `<body>`

## MCP Server

`mcp/server/` is a production MCP server that exposes the Silcrow docs as a queryable API. It is registered in `.mcp.json` and auto-approved in `.claude/settings.local.json`.

**Tools:** `searchDocs(query)`, `getDoc(id)`, `getSection(id)`, `getExamples(topic)`, `analyzeSilcrowUsage(code)`

**Architecture:**
- `lib/docs-loader.js` — loads `mcp/generated/docs.json` once at startup, builds two Fuse.js indices (docs + flattened sections), exposes a frozen singleton store
- `lib/search-index.js` — Fuse index construction; docs weighted title 35% / tags 25% / summary 20%
- `lib/silcrow-catalog.js` — ground-truth maps of every known `s-*` attribute, `:binding`, and `Silcrow.*` API method → doc ID
- `lib/code-parser.js` — regex extractor for Silcrow constructs in HTML/JS (returns `Map<name, count>`)
- `lib/analysis-rules.js` — 7 static analysis rules (unknown attrs, deprecated, `s-for` missing `:key`, multiple HTTP verbs, `s-html` XSS note, `s-ws` non-TLS, unknown API)
- `tools/` — one file per MCP tool; each exports `definition` (MCP schema) and `handler(args, store)`

**Docs path override** (for multi-project reuse):
```bash
node mcp/server/server.js --docs /path/to/other/docs.json
# or
SILCROW_DOCS_PATH=/path/to/other/docs.json npm run mcp
```

When `docs/*.md` files change, run `npm run build:docs` before restarting the server.

## Documentation

`docs/` contains the authoritative reference. Always start with `docs/INDEX.md` (per `.clinerules`) which maps features to source files and docs. The docs are also compiled into `mcp/generated/docs.json` for the MCP server (`mcp/`).

## Key conventions

- Features are attribute-first — behavior is declared in HTML, not configured in JS.
- The three main systems (Runtime, Navigator, Live) are designed to be used independently.
- Dist files are committed (`dist/silcrow.js`, `dist/silcrow.min.js`); always rebuild before committing source changes.
