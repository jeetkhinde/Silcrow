# Silcrow CLI (`sc`)

Scaffolds and enforces Clean Architecture for Rust microservices built on Silcrow, Axum, and Maud.

## Install

```bash
cargo install --path crates/silcrow-cli
```

Binary name is `sc`.

## Quick Start

```bash
# 1. Create a new service
sc new order-service
cd order-service

# 2. Add a domain aggregate (entity + repo trait + Postgres stub)
sc gen aggregate Order

# 3. Add a usecase
sc gen usecase CreateOrder

# 4. Add an endpoint (handler + route auto-registered)
sc gen endpoint CreateOrder --method POST --path /orders

# 5. Add a Maud template
sc gen template OrderForm

# 6. Add a migration
sc gen migration create_orders_table

# 7. Validate architecture
sc validate
```

After step 2, `main.rs` is automatically wired — `AppState` gets the repository field, construction, and initialization. After step 4, the route is registered in `routes::routes()`.

Set your database URL and run:

```bash
export DATABASE_URL=postgres://user:pass@localhost/db
cargo run
```

---

## Commands

### `sc new <name>`

Scaffolds a complete service project with all four Clean Architecture layers.

```bash
sc new billing-service
```

Creates:

```
billing-service/
├── Cargo.toml          # silcrow, axum, sqlx, maud, thiserror, tracing, tokio
├── silcrow.toml        # server config (host, port)
├── migrations/
└── src/
    ├── main.rs         # AppState, DI wiring, router setup
    ├── routes.rs       # all route registrations (SC:ROUTES marker)
    ├── prelude.rs      # re-exports all layer preludes
    ├── domain/
    │   ├── mod.rs
    │   ├── prelude.rs
    │   ├── entities/
    │   ├── repositories/
    │   └── errors.rs   # DomainError enum
    ├── application/
    │   ├── mod.rs
    │   ├── prelude.rs
    │   ├── usecases/
    │   └── errors.rs   # ApplicationError + From<DomainError>
    ├── infrastructure/
    │   ├── mod.rs
    │   ├── prelude.rs
    │   └── repositories/
    └── presentation/
        ├── mod.rs      # IntoResponse for ApplicationError
        ├── prelude.rs
        ├── handlers/
        └── templates/
```

Generated `main.rs` includes Silcrow JS serving, `SilcrowConfig::load()`, sqlx migration runner, and tracing setup.

---

### `sc gen aggregate <Name>`

Creates a domain entity, repository trait, and Postgres repository stub. Wires into `AppState` automatically.

Name must be PascalCase.

```bash
sc gen aggregate Invoice
```

Creates three files:

**`src/domain/entities/invoice.rs`** — Serializable entity struct:

```rust
use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
}
```

**`src/domain/repositories/invoice_repository.rs`** — Pure trait, no framework deps:

```rust
use crate::prelude::*;

pub trait InvoiceRepository: Send + Sync {
    fn find_by_id(&self, id: i64) -> impl Future<Output = Result<Option<Invoice>, DomainError>> + Send;
    fn save(&self, entity: &Invoice) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn delete(&self, id: i64) -> impl Future<Output = Result<(), DomainError>> + Send;
}
```

**`src/infrastructure/repositories/invoice_repository_impl.rs`** — sqlx Postgres stub:

```rust
use crate::prelude::*;
use sqlx::PgPool;

pub struct PgInvoiceRepository {
    pool: PgPool,
}

impl InvoiceRepository for PgInvoiceRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<Invoice>, DomainError> {
        todo!("Implement find_by_id")
    }
    // ...
}
```

Also updates:

- `domain/entities/mod.rs` — adds `pub mod invoice;`
- `domain/repositories/mod.rs` — adds `pub mod invoice_repository;`
- `infrastructure/repositories/mod.rs` — adds `pub(crate) mod invoice_repository_impl;`
- `domain/prelude.rs` — appends `pub use crate::domain::entities::invoice::Invoice;` and `pub use crate::domain::repositories::invoice_repository::InvoiceRepository;`
- `infrastructure/prelude.rs` — appends `pub use crate::infrastructure::repositories::invoice_repository_impl::PgInvoiceRepository;`
- `src/main.rs` — adds `invoice_repo` field to `AppState`, constructs `PgInvoiceRepository`

---

### `sc gen usecase <Name>`

Creates a usecase struct with constructor injection. Generic over the repository trait — the concrete type is resolved in `main.rs`.

```bash
sc gen usecase CreateInvoice
```

**`src/application/usecases/create_invoice.rs`:**

```rust
use crate::prelude::*;
use std::sync::Arc;
use tracing::instrument;

pub struct CreateInvoice<R> {
    repo: Arc<R>,
}

impl<R> CreateInvoice<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

impl<R: Send + Sync> CreateInvoice<R> {
    #[instrument(skip(self), name = "CreateInvoice::execute")]
    pub async fn execute(&self) -> Result<(), ApplicationError> {
        // TODO: implement usecase logic using self.repo
        Ok(())
    }
}
```

Also updates:

- `application/usecases/mod.rs` — adds `pub mod create_invoice;`
- `application/prelude.rs` — appends `pub use crate::application::usecases::create_invoice::CreateInvoice;`

---

### `sc gen endpoint <Name> --method <METHOD> --path <path>`

Creates an Axum handler and registers the route automatically.

```bash
sc gen endpoint CreateInvoice --method POST --path /invoices
```

**`src/presentation/handlers/create_invoice.rs`:**

```rust
use crate::prelude::*;
use std::sync::Arc;
use axum::extract::State;
use axum::{Json, http::StatusCode};
use silcrow::SilcrowRequest;
use tracing::instrument;
use crate::AppState;

#[instrument(skip(state, _req), name = "handler::create_invoice")]
pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    _req: SilcrowRequest,
) -> Result<(StatusCode, Json<serde_json::Value>), ApplicationError> {
    // TODO: construct usecase from state, call execute
    Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))))
}
```

Also updates:

- `presentation/handlers/mod.rs` — adds `pub(crate) mod create_invoice;`
- `presentation/prelude.rs` — appends `pub use crate::presentation::handlers::create_invoice::create_invoice;`
- `src/routes.rs` — inserts `.route("/invoices", axum::routing::post(presentation::handlers::create_invoice::create_invoice))`

Supported methods: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`. Default is `GET`.

---

### `sc gen template <Name>`

Creates a Maud SSR template in the presentation layer.

```bash
sc gen template InvoiceList
```

**`src/presentation/templates/invoice_list.rs`:**

```rust
use crate::prelude::*;
use maud::{html, Markup};

pub fn invoice_list() -> Markup {
    html! {
        div class="invoice-list" {
            h2 { "InvoiceList" }
            // TODO: add template content
        }
    }
}
```

Also updates:

- `presentation/templates/mod.rs` — adds `pub(crate) mod invoice_list;`
- `presentation/prelude.rs` — appends `pub use crate::presentation::templates::invoice_list::invoice_list;`

---

### `sc gen migration <name>`

Creates a timestamped sqlx migration file. Name should be snake_case.

```bash
sc gen migration create_invoices_table
```

Creates `migrations/20260218143022_create_invoices_table.sql`:

```sql
-- Add migration script here
```

Run migrations with `sqlx migrate run` or they run automatically on startup via `sqlx::migrate!()` in the generated `main.rs`.

---

### `sc validate`

Scans all `.rs` files for architectural boundary violations. Checks both `use` and `pub use` statements — including re-exports inside `prelude.rs` files — against the layer dependency rules.

```bash
sc validate
```

On a clean project:

```
Architecture check PASSED. No violations found.
```

With violations:

```
VIOLATION [domain] src/domain/entities/order.rs:14
  use sqlx::PgPool;
  → Domain cannot use framework crates (sqlx, axum, maud, silcrow)

VIOLATION [domain] src/domain/prelude.rs:5
  pub use crate::infrastructure::repositories::order_repository_impl::PgOrderRepository;
  → Domain cannot import from other layers

Found 2 violation(s). Architecture check FAILED.
```

Exits with code 1 on violations. Add to CI:

```yaml
- run: sc validate
```

---

## Architecture Rules

The CLI enforces strict layer boundaries. These are checked at two levels: Rust module visibility (compile-time) and `sc validate` (static analysis).

### Layer dependency matrix

| Layer | Can import from | Visibility |
|---|---|---|
| **domain** | nothing (pure Rust, no framework crates) | `pub mod` |
| **application** | `domain` only | `pub mod` |
| **infrastructure** | `domain` only (implements repo traits) | `pub(crate) mod` |
| **presentation** | `application`, `silcrow`, `axum`, `maud` | `pub(crate) mod` |
| **main.rs** | all layers (sole DI wiring point) | — |

### What `sc validate` flags

Both `use` and `pub use` lines are checked, so violations in `prelude.rs` re-exports are caught alongside normal imports.

| Source layer | Flagged imports |
|---|---|
| `domain/` | Any `use`/`pub use crate::infrastructure`, `crate::presentation`, `crate::application` |
| `domain/` | Any `use sqlx`, `use axum`, `use maud`, `use silcrow` |
| `application/` | Any `use`/`pub use crate::infrastructure`, `crate::presentation` |
| `application/` | Any `use axum`, `use maud`, `use silcrow` |
| `infrastructure/` | Any `use`/`pub use crate::presentation`, `crate::application` |
| `presentation/` | Any `use`/`pub use crate::infrastructure` |
| `presentation/` | Any `use sqlx` |

---

## Prelude System

Every scaffolded service has a two-level prelude hierarchy that accumulates project types automatically as you run `sc gen` commands.

### Structure

```
src/prelude.rs                  ← root aggregator (re-exports all layer preludes)
src/domain/prelude.rs           ← domain entity + repo trait re-exports
src/application/prelude.rs      ← usecase re-exports
src/infrastructure/prelude.rs   ← repository implementation re-exports
src/presentation/prelude.rs     ← handler + template re-exports
```

**`src/prelude.rs`** (generated by `sc new`, never modified by gen commands):

```rust
pub use crate::domain::prelude::*;
pub use crate::application::prelude::*;
pub use crate::infrastructure::prelude::*;
pub use crate::presentation::prelude::*;
```

### How layer preludes grow

Each `sc gen` command appends the newly created type to the appropriate layer prelude immediately after writing the file:

| Command | Appended to |
|---|---|
| `sc gen aggregate Order` | `domain/prelude.rs` — entity + repo trait; `infrastructure/prelude.rs` — Postgres impl |
| `sc gen usecase CreateOrder` | `application/prelude.rs` — usecase struct |
| `sc gen endpoint CreateOrder ...` | `presentation/prelude.rs` — handler fn |
| `sc gen template OrderList` | `presentation/prelude.rs` — template fn |

### Usage in generated files

Every generated file includes `use crate::prelude::*;` as its project-types import. External crates (`sqlx`, `serde`, `axum`, `maud`, etc.) are still imported explicitly:

```rust
use crate::prelude::*;   // project types (entities, errors, usecases, ...)
use sqlx::PgPool;        // external — stays explicit
```

### Prelude boundary rules

`sc validate` checks that prelude files do not re-export across forbidden boundaries. Manually adding a `pub use` to the wrong layer prelude is caught the same way as a normal import violation:

- `domain/prelude.rs` must not re-export from `crate::infrastructure`, `crate::presentation`, or `crate::application`
- `application/prelude.rs` must not re-export from `crate::infrastructure` or `crate::presentation`

---

## Error Flow

Errors propagate across layers with type-safe mapping:

```
DomainError  →  ApplicationError  →  HTTP Response
 (domain/)       (application/)       (presentation/)
```

**`domain/errors.rs`** — pure business errors:

```rust
pub enum DomainError {
    NotFound(String),
    Validation(String),
    Internal(String),
}
```

**`application/errors.rs`** — converts domain errors via `From`:

```rust
pub enum ApplicationError {
    NotFound(String),
    Validation(String),
    Internal(String),
}

impl From<DomainError> for ApplicationError { /* ... */ }
impl From<sqlx::Error> for ApplicationError { /* ... */ }
```

**`presentation/mod.rs`** — converts to HTTP responses using Silcrow builders:

```rust
impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        match self {
            ApplicationError::NotFound(msg) =>
                Error().message(msg).status(StatusCode::NOT_FOUND).ok(),
            ApplicationError::Validation(msg) =>
                Error().message(msg).status(StatusCode::UNPROCESSABLE_ENTITY).ok(),
            ApplicationError::Internal(msg) =>
                Error().message(msg).status(StatusCode::INTERNAL_SERVER_ERROR).ok(),
        }
    }
}
```

Handlers should prefer plain Axum + Maud return types such as `Markup`, `(StatusCode, Html<_>)`, or `(StatusCode, Json<_>)`, usually wrapped in `Result<_, ApplicationError>`.
Use Silcrow's `JsonOkResponse` / `HtmlOkResponse` when protocol-specific conveniences (like `_toast`) are needed.

---

## SC Markers

The generated `main.rs` contains marker comments that `sc gen aggregate` and `sc gen endpoint` use to insert code at the right locations:

```rust
pub struct AppState {
    pub db: PgPool,
    pub order_repo: Arc<...>,     // ← inserted by sc gen aggregate
    // SC:REPO_FIELDS             // ← new repo fields go above this line
}

// ...

let order_repo = Arc::new(PgOrderRepository::new(db.clone()));  // ← inserted
// SC:REPO_INIT                    // ← new repo init goes above this line

let state = Arc::new(AppState {
    db,
    order_repo,                    // ← inserted
    // SC:STATE_FIELDS             // ← new state fields go above this line
});
```

For routes in `src/routes.rs`:

```rust
pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/orders", axum::routing::post(presentation::handlers::create_order::create_order))  // ← inserted
    // SC:ROUTES                   // ← new routes go above this line
}
```

Do not remove the `// SC:` comments — they are required for future `sc gen` commands to work. New entries are always inserted above the marker.

---

## Naming Conventions

| Input | Conversion | Example |
|---|---|---|
| Aggregate name | PascalCase → snake_case file | `Order` → `order.rs`, `order_repository.rs` |
| Usecase name | PascalCase → snake_case file | `CreateOrder` → `create_order.rs` |
| Endpoint name | PascalCase → snake_case handler fn | `CreateOrder` → `create_order()` |
| Template name | PascalCase → snake_case file + fn | `OrderList` → `order_list.rs`, `order_list()` |
| Migration name | used as-is (should be snake_case) | `create_orders_table` |

All names passed to `sc gen` (except `migration`) must be PascalCase — starting with an uppercase letter, alphanumeric only.

---

## Stack

Generated services depend on:

| Crate | Version | Used in |
|---|---|---|
| `silcrow` | 0.1 | presentation (Silcrow request extractor, optional response conveniences, JS serving) |
| `axum` | 0.7 | presentation (routing, handlers, extractors) |
| `sqlx` | 0.8 | infrastructure (Postgres queries, migrations) |
| `maud` | 0.26 | presentation (SSR templates) |
| `serde` | 1 | domain (entity serialization) |
| `thiserror` | 2 | domain + application (error types) |
| `anyhow` | 1 | main.rs (top-level error handling) |
| `tracing` | 0.1 | application + presentation (instrumentation) |
| `tokio` | 1 | main.rs (async runtime) |

Requires Rust 1.82+ (native async fn in traits).
