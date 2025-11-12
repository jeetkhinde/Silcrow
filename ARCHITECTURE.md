# RHTMX Codebase Architecture Analysis

## 1. What is RHTMX?

RHTMX is a **Rust + HTMX framework** for building type-safe, full-stack web applications with:

- **Compile-time HTML generation** using Rust macros (`html!` and `maud!`)
- **Zero runtime overhead** - all templates compiled to native Rust code
- **Type-safe rendering** - full type checking at compile time
- **HTMX-first design** - built specifically to work with HTMX for dynamic updates
- **File-based routing** - automatic route generation from directory structure
- **Built-in database support** - SQLx for database operations (currently SQLite)

### Core Philosophy
- Combine Rust's type safety with HTMX's simplicity
- Server-side rendering (SSR) only - no client-side JavaScript framework
- Pure functions for UI components
- Server state management via HTTP handlers

---

## 2. Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│          Browser (HTMX Client)                      │
│  - Sends HTMX requests (HX-Request header)          │
│  - Receives HTML responses                          │
│  - No state management (server-driven)              │
└────────────┬────────────────────────────────────────┘
             │
             │ HTTP (GET, POST, PUT, DELETE, PATCH)
             │
┌────────────▼────────────────────────────────────────┐
│      RHTMX Server (Rust/Axum)                       │
├─────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────┐    │
│  │ Routing Layer (rhtmx-router)                │    │
│  │ - File-based routing                        │    │
│  │ - Dynamic parameters (:id, [...slug])       │    │
│  │ - Pattern matching                          │    │
│  └─────────────────────────────────────────────┘    │
│                      ▼                              │
│  ┌─────────────────────────────────────────────┐    │
│  │ HTTP Handler Layer                          │    │
│  │ - get!, post!, put!, patch!, delete! macros │    │
│  │ - Request validation                        │    │
│  │ - Type-safe deserialization                 │    │
│  └─────────────────────────────────────────────┘    │
│                      ▼                              │
│  ┌─────────────────────────────────────────────┐    │
│  │ Business Logic Layer                        │    │
│  │ - Database queries (SQLx)                   │    │
│  │ - Data transformations                      │    │
│  │ - Business rules                            │    │
│  └─────────────────────────────────────────────┘    │
│                      ▼                              │
│  ┌─────────────────────────────────────────────┐    │
│  │ Rendering Layer (rhtmx-renderer)            │    │
│  │ - Template processing                       │    │
│  │ - Directive evaluation (r-for, r-if, etc)   │    │
│  │ - Expression interpolation                  │    │
│  │ - CSS scoping                               │    │
│  └─────────────────────────────────────────────┘    │
│                      ▼                              │
│  ┌─────────────────────────────────────────────┐    │
│  │ Response Builders                           │    │
│  │ - Ok() - Success response                   │    │
│  │ - Error() - Error response                  │    │
│  │ - Redirect() - Redirect response            │    │
│  │ - OOB updates (Out-of-band updates)         │    │
│  │ - Toast notifications (HX-Trigger)          │    │
│  └─────────────────────────────────────────────┘    │
│                      ▼                              │
│  ┌─────────────────────────────────────────────┐    │
│  │ Data Persistence                            │    │
│  │ - SQLite database                           │    │
│  │ - SQLx connection pool                      │    │
│  │ - Async SQL operations                      │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

### Crate Structure

```
RHTMX (workspace)
├── rhtmx/                    # Main framework crate
│   ├── src/
│   │   ├── lib.rs           # Public API
│   │   ├── main.rs          # Application server
│   │   ├── config.rs        # Configuration management
│   │   ├── database.rs      # SQLite database layer
│   │   ├── request_context.rs # HTTP request context
│   │   ├── renderer.rs      # Template rendering
│   │   ├── html.rs          # HTML response builders
│   │   ├── template_loader.rs # Template discovery
│   │   ├── hot_reload.rs    # Development hot reload
│   │   ├── action_handlers.rs # Route action registry
│   │   ├── action_executor.rs # Execute actions
│   │   ├── validation/      # Form validation
│   │   └── ...
│   └── docs/                # Documentation
│
├── rhtmx-macro/             # Procedural macros
│   └── src/
│       ├── html.rs         # html! macro implementation
│       ├── http_verbs.rs   # get!, post!, etc macros
│       └── ...
│
├── rhtmx-parser/            # Template parser
│   └── src/
│       ├── directive.rs    # r-for, r-if, r-match
│       ├── expression.rs   # Expression evaluation
│       ├── css.rs          # CSS parsing
│       └── ...
│
├── rhtmx-router/            # File-based router
│   └── src/
│       └── lib.rs          # Route matching logic
│
├── pages/                   # Application pages (templates)
│   ├── _layout.rhtml       # Root layout
│   ├── index.rhtml         # Home page
│   └── ...
│
├── src/                     # Example/demo application
│   ├── main.rs            # Demo server
│   ├── database.rs        # Example DB operations
│   └── ...
│
└── rhtmx.toml             # Configuration file
```

---

## 3. Key Components Deep Dive

### 3.1 Routing System

**Technology**: File-based routing with priority matching

**How it works**:
- Files in `pages/` directory automatically become routes
- Route pattern is determined by file path
- Dynamic segments use `[id]` syntax → `:id` parameter
- Catch-all routes use `[...slug]` syntax → `*slug` parameter

**Example mappings**:
```
pages/index.rs                 → GET /
pages/users/index.rs           → GET /users
pages/users/[id].rs            → GET /users/:id
pages/blog/[...slug].rs        → GET /blog/*slug
pages/_layout.rs               → Default layout for all pages
```

**Priority system** (smart route ordering):
1. Static routes (e.g., `/about`) - priority 0
2. Optional parameters (e.g., `/posts/:id?`) - lower priority
3. Required dynamic routes (e.g., `/users/:id`) - medium priority
4. Catch-all routes (e.g., `/docs/*path`) - priority 1000+

**Implementation**: `/home/user/RHTMX/rhtmx-router/src/lib.rs`

---

### 3.2 HTTP Handler Macros

**Technology**: Procedural macros that generate route handler code

**Available macros**:
- `get!()` - GET requests
- `post!()` - POST requests  
- `put!()` - PUT requests
- `patch!()` - PATCH requests
- `delete!()` - DELETE requests

**Features**:
- Path parameter extraction (`:id`, `:user_id`, etc)
- Query parameter support
- Type-safe request deserialization
- Automatic form parsing (JSON, URL-encoded)
- Error handling with `?` operator

**Example**:
```rust
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

post!()
fn create_user(req: CreateUserRequest) -> OkResponse {
    let user = db::create_user(req)?;
    Ok()
        .render(user_card, user)
        .toast("User created!")
}
```

**Implementation**: `/home/user/RHTMX/rhtmx-macro/src/`

---

### 3.3 Response Builders

**Technology**: Fluent API for building HTTP responses

**Response types**:

#### Ok() - Success Response
```rust
Ok()
    .render(component, data)           // Main content
    .render_oob("id", component, data) // Out-of-band update
    .toast("Success!")                  // Toast notification (HX-Trigger)
    .header("X-Custom", "value")        // Custom headers
    .status(StatusCode::CREATED)        // Custom HTTP status
```

#### Error() - Error Response
```rust
Error()
    .render(error_component, errors)
    .status(StatusCode::BAD_REQUEST)
    .message("Validation failed")
```

#### Redirect() - Redirect Response
```rust
Redirect()
    .to("/dashboard")
    .toast("Welcome back!")
    .status(StatusCode::SEE_OTHER)
```

**Special Features**:
- **OOB Updates**: Update multiple DOM elements with HTMX `hx-swap="outerHTML"`
- **Toast Messages**: Triggers browser notifications via `HX-Trigger` header
- **Out-of-band swaps**: Update parts of the page not directly triggered by request

**Implementation**: `/home/user/RHTMX/rhtmx/src/html.rs`

---

### 3.4 Template System

**Technology**: Rust procedural macros for compile-time HTML generation

#### html! Macro
- Generates HTML at compile time
- Type checks all expressions
- Zero runtime overhead
- Converts to Rust string building

#### maud! Macro
- Alternative template syntax
- More concise HTML syntax
- Better support for complex templates

**Key directives**:

**r-for (Loops)**:
```rust
html! {
    <div r-for="user in users">
        <p>{user.name}</p>
    </div>
}
```

**r-if (Conditionals)**:
```rust
html! {
    <div r-if="user.is_admin">
        Admin Panel
    </div>
}
```

**r-match (Pattern Matching)**:
```rust
html! {
    <div r-match="status">
        <span r-when="Active" class="badge-active">Active</span>
        <span r-when="Pending" class="badge-pending">Pending</span>
        <span r-default>Unknown</span>
    </div>
}
```

**css! Macro (Scoped CSS)**:
```rust
css! {
    scope: "card",
    .card {
        border: 1px solid #ccc;
        padding: 1rem;
    }
}
```

**Implementation**: 
- `/home/user/RHTMX/rhtmx-macro/src/html.rs`
- `/home/user/RHTMX/rhtmx-parser/src/`

---

### 3.5 Data Persistence Layer

**Technology**: SQLx (async Rust SQL toolkit)

**Features**:
- Compile-time query verification (optional)
- Connection pooling
- SQLite support (default)
- Support for other databases (PostgreSQL, MySQL)
- Type-safe parameter binding
- Async/await support with Tokio

**User Model Example**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub bio: Option<String>,
    pub username: String,
}
```

**Database Operations**:
```rust
pub async fn get_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error>
pub async fn create_user(pool: &SqlitePool, ...) -> Result<User, sqlx::Error>
pub async fn update_user(pool: &SqlitePool, id: i32, ...) -> Result<Option<User>>
pub async fn delete_user(pool: &SqlitePool, id: i32) -> Result<bool, sqlx::Error>
pub async fn search_users(pool: &SqlitePool, filter: Option<String>) -> Result<Vec<User>>
```

**Implementation**: `/home/user/RHTMX/src/database.rs`

---

### 3.6 Request Context

**Technology**: Request metadata container

**Contains**:
- HTTP method (GET, POST, etc)
- Request path and routing parameters
- Query parameters
- Form data (POST/PUT body)
- HTTP headers
- Parsed cookies
- Database connection pool reference
- HTMX-specific headers (HX-Request, HX-Target, HX-Trigger)

**Helper methods**:
```rust
pub fn is_get(&self) -> bool
pub fn is_post(&self) -> bool
pub fn accepts_json(&self) -> bool
pub fn wants_partial(&self) -> bool
pub fn is_htmx(&self) -> bool
pub fn htmx_target(&self) -> Option<String>
pub fn htmx_trigger(&self) -> Option<String>
```

**Implementation**: `/home/user/RHTMX/src/request_context.rs`

---

## 4. Existing SSE/WebSocket Support

### Current Status: **NO SUPPORT**

**Findings**:
- No WebSocket implementation in dependencies
- No Server-Sent Events (SSE) support
- No streaming endpoints
- No real-time communication mechanism

**Current communication pattern**:
- Traditional HTTP request/response only
- HTMX sends requests, server responds with HTML
- Polling-based updates (HTMX `hx-trigger="every 5s"`)

**Tech stack limitations**:
- Axum supports WebSockets (via `axum::extract::ws`)
- Tokio provides async channels for real-time communication
- No existing WebSocket handlers in RHTMX codebase

---

## 5. IndexedDB Integration

### Current Status: **NO SUPPORT**

**Findings**:
- Zero IndexedDB implementation
- No browser-side data persistence
- No client-side state management
- No sync mechanisms

**What would be needed**:
1. **Server-side data versioning** - track changes
2. **Sync API endpoint** - provide data snapshots
3. **Client-side JavaScript** - IndexedDB operations (outside Rust)
4. **Change detection** - identify what changed
5. **Conflict resolution** - handle concurrent edits
6. **Incremental updates** - delta sync for large datasets

---

## 6. Architecture Patterns

### 6.1 State Management

**Pattern**: Server-Driven State Management

**How it works**:
- Server is source of truth
- State stored in database (SQLite)
- No client-side state beyond current page
- Each request gets full data context
- Database pool injected into request context

**State Flow**:
```
Browser Request → Server Handler
    ↓
Get data from DB
    ↓
Transform data
    ↓
Render component with data
    ↓
Send HTML response
    ↓
HTMX updates DOM
```

### 6.2 Reactive/Reactive Patterns

**Pattern**: HTML-Driven Reactivity (HTMX)

**How it works**:
- HTML elements have HTMX attributes
- `hx-get="/api/data"` - fetch new data
- `hx-post="/api/action"` - submit action
- `hx-target="#target"` - swap into element
- `hx-swap="innerHTML"` - how to swap HTML
- `hx-trigger="click"` - what triggers request

**Example**:
```html
<div id="user-list">
    <!-- Initial content -->
</div>

<form hx-post="/users" hx-target="#user-list" hx-swap="beforeend">
    <input name="name" placeholder="Name" />
    <button type="submit">Add User</button>
</form>
```

**HTMX Interactions**:
- `hx-get` - GET request
- `hx-post` - POST request
- `hx-put` - PUT request
- `hx-patch` - PATCH request
- `hx-delete` - DELETE request
- `hx-trigger` - event trigger
- `hx-swap` - swap strategy (innerHTML, outerHTML, beforeend, etc)
- `hx-confirm` - confirmation dialog
- `hx-target` - target element for swap

### 6.3 Form Handling

**Pattern**: Server-Side Validation

**Features**:
- `#[derive(Validate)]` macro for declarative validation
- Compile-time validation code generation
- Email validation, password strength, numeric ranges, string length
- Custom regex validators
- Error messages returned in response

**Available validators**:
```rust
#[email]                        // Email format
#[password("strong")]           // Password strength
#[min(18)]                      // Minimum number
#[max_length(50)]               // String length
#[regex(r"^[a-z0-9]+$")]       // Custom pattern
#[required]                     // Required for Option<T>
```

---

## 7. Framework Dependencies

**Core Dependencies**:
```toml
axum = "0.7"                    # Web framework
tokio = "1.0"                   # Async runtime
sqlx = "0.7"                    # Database toolkit
serde = "1.0"                   # Serialization
serde_json = "1.0"              # JSON
toml = "0.8"                    # TOML config
regex = "1.10"                  # Regex (for validators)
maud = "0.26"                   # HTML templating
notify = "6.1"                  # File watching (hot reload)
tower-livereload = "0.9.6"      # Browser reload
```

**Notable absences**:
- No WebSocket library
- No EventSource (SSE) library
- No IndexedDB library (wouldn't work in Rust anyway - JS-only)
- No database migration library
- No ORM (raw SQL with sqlx)

---

## 8. Data Flow Examples

### Example 1: Simple GET Request

```
Browser: GET /users
    ↓
Server: Route to /users handler
    ↓
Handler: SELECT * FROM users
    ↓
Database: Returns Vec<User>
    ↓
Renderer: users_page(users) → render users_list component
    ↓
Response: HTML with user list
    ↓
Browser: Display users
```

### Example 2: Create with OOB Update

```
Browser: POST /users with form data (hx-post)
    ↓
Server: Parse CreateUserRequest
    ↓
Handler: Validate & INSERT INTO users
    ↓
Database: Returns new User
    ↓
Response:
    - Main: render(user_card, new_user)
    - OOB: render_oob("user-count", count_badge, get_count())
    - Toast: .toast("User created!")
    ↓
HTMX: 
    - Append new user card to user-list
    - Update user-count element
    - Show toast notification
```

### Example 3: Search with Partial Rendering

```
Browser: hx-get="/users?search=john"
    ↓
Server: SELECT * FROM users WHERE name LIKE '%john%'
    ↓
Handler: Filter users, render partial
    ↓
Response: Just the filtered users HTML (no layout)
    ↓
HTMX: Swap into target element
```

---

## 9. Configuration System

**Config file**: `rhtmx.toml` (optional, uses defaults if missing)

**Sections**:
- `[project]` - Name, version, author
- `[server]` - Port, host, worker threads
- `[routing]` - Pages directory, components directory, case sensitivity
- `[build]` - Output directory, minification options
- `[dev]` - Hot reload, port, browser opening, watch paths

**Load priority**:
1. Check for `rhtmx.toml`
2. Fall back to defaults
3. Environment variables override config file

**Implementation**: `/home/user/RHTMX/src/config.rs`

---

## 10. Development Features

### Hot Reload

**Technology**: File watching + template reloading

**How it works**:
1. File watcher monitors pages/ and components/ directories
2. When template changes detected, reload in memory
3. Browser auto-refresh (via tower-livereload)
4. No server restart needed

**Enabled by**: 
- `notify` crate for file watching
- `tower-livereload` for browser refresh
- Arc<RwLock<TemplateLoader>> for thread-safe updates

**Implementation**: `/home/user/RHTMX/src/hot_reload.rs`

---

## 11. Known Limitations

1. **No WebSocket/SSE** - Only HTTP request/response
2. **No IndexedDB** - Server-driven state only
3. **No client-side JavaScript framework** - Pure HTMX/HTML
4. **No real-time synchronization** - Polling only
5. **No offline support** - Requires server connection
6. **SQLite only** - No built-in migration tooling
7. **Single database** - No multi-database support
8. **No authentication framework** - Users implement themselves
9. **No authorization/middleware** - Route-level only
10. **No built-in caching** - Application must implement

---

## 12. Design Philosophy Comparison

| Aspect | RHTMX | Traditional SPA | NextJS |
|--------|-------|-----------------|--------|
| Rendering | Server-side | Client-side | Hybrid (SSR+SPA) |
| State | Server database | Client memory | Hybrid |
| Network | HTTP request/response | WebSocket/HTTP | HTTP/API routes |
| JavaScript | Minimal (HTMX) | Heavy (React/Vue) | Medium (Next) |
| Database | Server-connected | API-connected | Server-connected |
| Real-time | Polling | WebSocket | API-driven |
| Learning curve | Low | Medium | Medium-High |
| Type safety | Full (Rust) | Partial (TypeScript) | Good (TypeScript) |

---

## 13. Current Project State

**Current branch**: `claude/sync-indexeddb-from-server-011CV4BeUCVaL5Pg9woSikEm`

**Recent commits**:
- b47aafc - Implemented maud! macro
- 0354e97 - Updated httpverb fn macro docs
- 3d6e002 - Updated HTTPVerbs
- 63d68cb - Changing name to RHTMX and added .gitIgnore
- 9f3b1e8 - Organised Docs

**Status**: Active development, framework features being built out

---

## Summary for Data Sync Design

For implementing **IndexedDB data sync from server**:

1. **Server-side preparation**:
   - Implement version tracking on data entities
   - Create sync API endpoint (GET /api/sync?lastVersion=X)
   - Return JSON snapshots with version numbers
   - Implement conflict resolution strategy

2. **Client-side implementation** (JavaScript - outside RHTMX):
   - Use IndexedDB for offline storage
   - Implement sync service worker
   - Pull changes from server periodically
   - Push local changes when connection available

3. **RHTMX integration**:
   - Add sync endpoint to HTTP handlers
   - Return JSON responses (not HTML) for sync API
   - Use RequestContext to access database
   - Include version/timestamp metadata

4. **Challenge**: Bridging Rust server with JavaScript client
   - RHTMX can provide the server-side sync API
   - Client needs separate JavaScript layer
   - Consider using WebSocket eventually for real-time

