# Simplification Plan

Goal: Strip to the minimum needed for SSR with layouts using an app router. Remove dead code, over-engineering, and incomplete features.

## Current state: 4 crates, ~9,200 lines

---

## Phase 1: Remove dead code and unused features

### rhtmx (core)
- [ ] **validation/** — Remove entire module (87 + 65 = 152 lines). Hardcoded email regexes, `is_blocked_domain()` always returns false, `is_public_domain()` has 4 hardcoded domains. Validation belongs in user code, not the framework core
- [ ] **form_field.rs** (103 lines) — Remove. HTML5 form field metadata struct that's not wired into anything
- [ ] **value.rs** (93 lines) — Remove. Dynamic value type with unused `to_bool()`. Templates use Maud (type-safe Rust), not string interpolation
- [ ] **action_executor.rs** (189 lines) — Remove. Form deserialization with simplistic type detection. Actions should just be Axum handlers
- [ ] **actions.rs** (358 lines) — Remove. Action registry with deprecated methods. Axum already has routing — no need for a parallel action system
- [ ] **renderer.rs** (247 lines) — Remove. String-interpolation renderer (`{{variable}}` style). We use Maud for type-safe templates, not string interpolation
- [ ] **database.rs** (712 lines) — Remove. Hardcoded to a `users` table with basic CRUD. Database access belongs in user app code, not the framework
- [ ] **request_context.rs** — Trim heavily. Remove database pool reference (`sqlx::AnyPool`), cookie parsing, form error tracking. Keep only query params and basic request info (~150 lines → ~80 lines)
- [ ] **config.rs** — Trim. Remove build config (minify, output_dir — no build step). Remove 20+ `default_*` functions, use `Default` trait instead (~330 lines → ~150 lines)
- [ ] **template_loader.rs** — Remove 16 deprecated mutable methods, CSS stubs, and duplicate loading logic (~635 lines → ~300 lines)

**Estimated removal: ~2,500 lines from rhtmx**

### rhtmx-server
- [ ] **example_actions.rs** (281 lines) — Remove entirely. Example CRUD handlers for a `users` table that depends on removed database.rs
- [ ] **action_handlers.rs** (565 lines) — Remove entirely. Manual action handler registry — replaced by standard Axum routing
- [ ] **form_context.rs** (108 lines) — Remove. Every method has `#[allow(dead_code)]`
- [ ] **main.rs** — Simplify to just: config loading, template discovery, Axum server with layout-aware routing, hot reload (~714 lines → ~200 lines)
- [ ] **maud_wrapper.rs** — Keep but trim tests (~160 lines → ~60 lines)

**Estimated removal: ~1,250 lines from rhtmx-server**

### rhtmx-router
- [ ] **lib.rs** (1,927 lines) — Remove Phase 5.x stubs (parallel routes, intercepting routes — never implemented). Remove excessive commented-out code. Split if needed (~1,927 lines → ~1,200 lines)
- [ ] **route/detection.rs** — Simplify. Remove parallel route (@slot) and intercepting route ((..)group) detection since those features are incomplete

**Estimated removal: ~800 lines from rhtmx-router**

### rhtmx-macro
- [ ] Keep as-is. Already clean at 224 lines total (lib.rs + http.rs)

### pages/
- [ ] **examples/maud_demo.rs** (521 lines) — Remove. It's a documentation file disguised as an example
- [ ] Keep `_layout.rhtml`, `index.rhtml`, `test/_layout.rhtml`, `Users/UsersStat.rhtml` as reference

**Estimated removal: ~520 lines from pages**

---

## Phase 2: Simplify dependencies

After code removal:
- [ ] Remove `sqlx` from workspace — no database in framework core
- [ ] Remove `once_cell` — only used by removed renderer.rs validators
- [ ] Remove `uuid`, `chrono` — only used by removed database.rs
- [ ] Remove `regex` — only used by removed renderer.rs and validators
- [ ] Remove `urlencoding` — check if still needed after request_context trim

Remaining workspace deps should be: tokio, serde, serde_json, anyhow, tracing, tracing-subscriber, toml, maud, axum, tower, tower-http, tower-livereload, notify

---

## Phase 3: Verify

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] Server starts and serves pages with layouts

---

## Expected result

**Before:** ~9,200 lines across 4 crates, 35 files
**After:** ~3,100 lines across 4 crates, ~15 files

What remains is *only*:
1. **rhtmx** — Config loading, template discovery, layout resolution, response helpers
2. **rhtmx-router** — File-based routing with nested layouts and dynamic params
3. **rhtmx-macro** — HTTP handler macros
4. **rhtmx-server** — Axum server with hot reload, layout-aware template serving
