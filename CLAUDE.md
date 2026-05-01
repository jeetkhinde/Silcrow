# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

> **Before writing any Silcrow code, read [`docs/silcrow-api.md`](docs/silcrow-api.md).** It contains the complete directive, attribute, header, event, and API surface. Treat it as authoritative — do not infer behavior from the source when the reference covers it.

---

## Project at a glance

Silcrow.js is a zero-dependency, single-file client-side library for hypermedia-driven UIs. It ships as an IIFE via `<script src="silcrow.js" defer>` and exposes `window.Silcrow`. The canonical source is `src/silcrow.js` — a self-contained strict-mode IIFE.

**Five subsystems, usable independently:**

- **Runtime** — DOM patching via colon bindings (`:text`, `:class`, etc.), `s-use` spread, `s-for` keyed lists
- **Atoms** — framework-agnostic reactive store with `route:`, `stream:`, and custom scopes; powers React/Solid/Vue/Svelte adapters
- **Navigator** — client-side routing via `s-get`/`s-post`/`s-put`/`s-patch`/`s-delete`, response caching, history, preloading
- **Live** — SSE (`s-sse`) and WebSocket (`s-ws`/`s-wss`) with hub-based connection sharing and exponential backoff
- **Optimistic** — DOM snapshot/revert for instant UI feedback

Bootstrap auto-runs on `DOMContentLoaded`: registers global listeners (click, submit, popstate, mouseenter), seeds atoms from SSR data, initializes live elements and `s-bind` subscriptions, starts a MutationObserver for cleanup, and assembles the public `window.Silcrow` API. Middleware registration is locked after init.

---

## Commands

```bash
npm install

npm run build       # src/silcrow.js -> dist/silcrow.js + dist/silcrow.min.js
npm run watch       # rebuild on src/ changes
```

`build.js` reads `src/silcrow.js` and emits both `dist/silcrow.js` (unminified) and `dist/silcrow.min.js` (Terser, ES2020).

**Edit `src/silcrow.js` as the single source of truth.** Do not edit anything in `dist/` — it is generated.

---

## Repo layout

```
src/
  silcrow.js            ← canonical source (edit here)
dist/
  silcrow.js            ← generated, committed
  silcrow.min.js        ← generated, committed
docs/
  silcrow-api.md        ← full API surface — READ FIRST
build.js
CLAUDE.md
README.md
```

---

## Conventions

- **Attribute-first.** Behavior is declared in HTML, not configured in JS. New features should expose an attribute before exposing a JS method.
- **Subsystem independence.** Runtime, Atoms, Navigator, Live, and Optimistic must each remain usable without the others.
- **Single source of truth.** `src/silcrow.js` is canonical. The files in `dist/` are generated build artifacts. When you add or remove a public API (any `s-*` attribute, `:binding`, response header, event, or `Silcrow.*` method), update `docs/silcrow-api.md` in the same change.
- **Security defaults are not optional.** Sanitization, URL protocol validation, prototype-pollution blocking, same-origin Live, and `_blank` rel hardening are part of the contract. Do not bypass them for convenience.
- **No dependencies in `src/silcrow.js`.** It must remain a self-contained IIFE.

---

## Known issues

### WebSocket path is broken — avoid until fixed

`registerLiveState` and `unregisterLiveState` are called in four places in `src/silcrow.js` (in `openWsLive`, `unsubscribeWs`, the SSE→WS switching path, and the MutationObserver cleanup) but are **never defined**. Any code path that touches WebSocket subscription will throw a `ReferenceError`.

**When working on Silcrow:**

- Do not write examples, tests, or docs that exercise `s-ws`, `s-wss`, or `Silcrow.send` until these helpers are added.
- If asked to fix the WS path, the missing functions need to register/unregister a state object in `liveConnections` and `liveConnectionsByUrl` — mirror the SSE bookkeeping in `openLive`/`unsubscribeSse`.
- SSE (`s-sse`) is unaffected and works.

This bug is **not** documented in the user-facing README or `docs/silcrow-api.md` — keep it that way until fixed.

---

## When in doubt

1. Read `docs/silcrow-api.md` for the surface.
2. Check `src/silcrow.js` for actual behavior.
