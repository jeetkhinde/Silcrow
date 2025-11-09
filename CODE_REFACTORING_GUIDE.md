# RHTML Code Refactoring & Quality Guide
**Version:** v0.1.0-alpha
**Target:** v0.2.0 cleanup sprint
**Effort:** 3-4 weeks for full implementation

---

## 1. REFACTORING PRIORITIES

### Priority 1: Reduce main.rs Complexity (791 lines → 150 lines)

**Current State:**
```rust
// src/main.rs - 791 lines containing:
// ✗ HTTP server setup
// ✗ Request handlers
// ✗ Template loading
// ✗ Hot reload
// ✗ Database setup
// ✗ Action handler registration
// ✗ Static file serving
```

**Proposed Structure:**

#### Create `src/startup.rs`:
```rust
// File: src/startup.rs
use crate::AppState;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn initialize_app(config: &Config) -> Result<AppState, Box<dyn Error>> {
    // Load templates
    let loader = create_template_loader(&config)?;
    let template_loader = Arc::new(RwLock::new(loader));

    // Initialize database
    let db = initialize_database(&config).await?;

    // Register action handlers
    let action_registry = Arc::new(register_action_handlers());

    // Setup hot reload if enabled
    if should_enable_hot_reload(&config) {
        setup_hot_reload(&template_loader).await?;
    }

    Ok(AppState {
        template_loader,
        action_registry,
        db,
    })
}

fn create_template_loader(config: &Config) -> Result<TemplateLoader, Error> {
    let mut loader = TemplateLoader::with_config(
        &config.routing.pages_dir,
        &config.routing.components_dir,
        config.routing.case_insensitive,
    );
    loader.load_all()?;
    Ok(loader)
}

async fn initialize_database(config: &Config) -> Result<SqlitePool, Error> {
    // Database initialization logic
}

async fn setup_hot_reload(loader: &Arc<RwLock<TemplateLoader>>) -> Result<(), Error> {
    // Hot reload logic
}

fn should_enable_hot_reload(config: &Config) -> bool {
    std::env::var("HOT_RELOAD")
        .map(|v| v.parse::<bool>().unwrap_or(config.dev.hot_reload))
        .unwrap_or(config.dev.hot_reload)
}

fn register_action_handlers() -> ActionHandlerRegistry {
    let mut registry = ActionHandlerRegistry::new();
    register_built_in_handlers(&mut registry);
    registry
}
```

#### Create `src/handlers/mod.rs`:
```rust
// File: src/handlers/mod.rs
pub mod page;
pub mod api;
pub mod static_files;
pub mod error;

pub use self::page::handle_page_request;
pub use self::api::handle_api_request;
pub use self::static_files::handle_static;
pub use self::error::handle_404;
```

#### Create `src/handlers/page.rs`:
```rust
// File: src/handlers/page.rs
use crate::AppState;
use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn handle_page_request(
    State(state): State<AppState>,
    uri: String,
    headers: HeaderMap,
) -> Result<Html<String>, Error> {
    // Page rendering logic
    // Currently scattered in main.rs
}
```

#### Simplified main.rs:
```rust
// File: src/main.rs
mod handlers;
mod startup;

use axum::Router;
use rhtml_app::Config;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    template_loader: Arc<RwLock<TemplateLoader>>,
    action_registry: Arc<ActionHandlerRegistry>,
    db: SqlitePool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    println!("🚀 RHTML App Starting...");

    let config = Config::load_default()
        .unwrap_or_else(|e| {
            eprintln!("⚠️  Failed to load config: {}", e);
            Config::default()
        });

    print_startup_info(&config);

    let app_state = startup::initialize_app(&config)
        .await
        .expect("Failed to initialize app");

    let app = build_router(app_state);

    let addr = format!("0.0.0.0:{}", config.server.port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    println!("✨ Server running on http://localhost:{}", config.server.port);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/*path", axum::routing::get(handlers::page::handle_page_request))
        .route("/*path", axum::routing::post(handlers::api::handle_api_request))
        .fallback(handlers::error::handle_404)
        .with_state(state)
}

fn print_startup_info(config: &Config) {
    println!("⚙️  Configuration:");
    println!("   - Port: {}", config.server.port);
    println!("   - Pages directory: {}", config.routing.pages_dir);
    println!("   - Case-insensitive routing: {}", config.routing.case_insensitive);
}
```

**Benefits:**
- ✅ Easier to understand main flow
- ✅ Better separation of concerns
- ✅ Simpler testing
- ✅ Clearer error handling paths

---

### Priority 2: Create Custom Error Type

**Current Issues:**
```rust
// Scattered error handling:
pub fn load_routes() -> Result<Vec<Route>, Box<dyn Error>> { }
pub fn parse_template() -> Result<String, anyhow::Error> { }
pub fn load_config() -> Result<Config, toml::de::Error> { }
```

**Proposed Solution - `src/error.rs`:**
```rust
use std::fmt;

/// All errors that can occur in RHTML
#[derive(Debug)]
pub enum RhtmlError {
    // Template errors
    TemplateNotFound {
        path: String,
    },
    TemplateParseError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },

    // Configuration errors
    ConfigError {
        file: String,
        message: String,
    },

    // Database errors
    DatabaseError {
        query: String,
        error: String,
    },
    DatabaseConnectionError(String),

    // Validation errors
    ValidationError {
        field: String,
        message: String,
    },
    ValidationErrors(Vec<ValidationError>),

    // File system errors
    FileNotFound(String),
    FileReadError(String),

    // Action errors
    ActionNotFound {
        route: String,
        method: String,
    },
    ActionDeserializationError {
        field: String,
        expected: String,
        got: String,
    },

    // Request errors
    InvalidQuery {
        param: String,
        reason: String,
    },
    MalformedFormData(String),

    // Internal errors
    InternalError(String),
    FeatureNotYetImplemented(String),
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for RhtmlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RhtmlError::TemplateNotFound { path } => {
                write!(f, "Template not found: {}", path)
            }
            RhtmlError::TemplateParseError { file, line, column, message } => {
                write!(f, "Parse error in {} at {}:{}: {}", file, line, column, message)
            }
            RhtmlError::ConfigError { file, message } => {
                write!(f, "Configuration error in {}: {}", file, message)
            }
            RhtmlError::DatabaseError { query, error } => {
                write!(f, "Database error in query '{}': {}", query, error)
            }
            RhtmlError::DatabaseConnectionError(msg) => {
                write!(f, "Failed to connect to database: {}", msg)
            }
            RhtmlError::ValidationError { field, message } => {
                write!(f, "Validation error in field '{}': {}", field, message)
            }
            RhtmlError::ValidationErrors(errors) => {
                write!(f, "Validation failed for {} fields:", errors.len())?;
                for error in errors {
                    write!(f, "\n  - {}: {}", error.field, error.message)?;
                }
                Ok(())
            }
            _ => write!(f, "RHTML Error: {:?}", self),
        }
    }
}

impl std::error::Error for RhtmlError {}

/// Result type for RHTML operations
pub type Result<T> = std::result::Result<T, RhtmlError>;

// Conversions from external error types
impl From<std::io::Error> for RhtmlError {
    fn from(err: std::io::Error) -> Self {
        RhtmlError::FileReadError(err.to_string())
    }
}

impl From<sqlx::Error> for RhtmlError {
    fn from(err: sqlx::Error) -> Self {
        RhtmlError::DatabaseError {
            query: "unknown".to_string(),
            error: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for RhtmlError {
    fn from(err: serde_json::Error) -> Self {
        RhtmlError::InternalError(err.to_string())
    }
}

impl From<toml::de::Error> for RhtmlError {
    fn from(err: toml::de::Error) -> Self {
        RhtmlError::ConfigError {
            file: "rhtml.toml".to_string(),
            message: err.to_string(),
        }
    }
}
```

**Usage Examples:**
```rust
// Clear, type-safe error handling
let template = loader.load("users")
    .map_err(|e| RhtmlError::TemplateNotFound {
        path: "users".to_string()
    })?;

// Validation errors
let errors = vec![
    ValidationError {
        field: "email".to_string(),
        message: "Invalid email format".to_string()
    },
];
return Err(RhtmlError::ValidationErrors(errors));

// Pattern matching for error handling
match result {
    Err(RhtmlError::TemplateNotFound { path }) => {
        println!("Help: Check that {} exists in pages/", path);
    }
    Err(RhtmlError::ValidationError { field, message }) => {
        println!("Field {} failed: {}", field, message);
    }
    Err(e) => println!("Error: {}", e),
    Ok(value) => { /* success */ }
}
```

---

### Priority 3: Improve Request Context

**Current Structure:**
```rust
pub struct RequestContext {
    pub method: Method,
    pub query: QueryParams,
    pub form: FormData,
    pub headers: HeaderMap,
    pub cookies: HashMap<String, String>,
    pub path: String,
    pub db: Arc<SqlitePool>,
}
```

**Issues:**
- `Method` is good, but need type-safe access methods
- `QueryParams` and `FormData` need better APIs
- No support for Extensions (for middleware data)
- Cookies are basic strings

**Improved Version - `src/request_context.rs`:**

```rust
use axum::extract::rejection::JsonRejection;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::body::Body;
use cookie::Cookie;

/// Enhanced request context
#[derive(Clone)]
pub struct RequestContext {
    method: Method,
    path: String,
    query: QueryParams,
    form: FormData,
    headers: HeaderMap,
    cookies: CookieJar,
    extensions: Arc<Extensions>,
    db: Arc<SqlitePool>,
}

impl RequestContext {
    /// Create new request context
    pub fn new(method: Method, path: String, db: Arc<SqlitePool>) -> Self {
        Self {
            method,
            path,
            query: QueryParams::default(),
            form: FormData::default(),
            headers: HeaderMap::new(),
            cookies: CookieJar::new(),
            extensions: Arc::new(Extensions::new()),
            db,
        }
    }

    // Safe method accessors
    pub fn is_get(&self) -> bool { self.method == Method::GET }
    pub fn is_post(&self) -> bool { self.method == Method::POST }
    pub fn is_put(&self) -> bool { self.method == Method::PUT }
    pub fn is_delete(&self) -> bool { self.method == Method::DELETE }
    pub fn is_patch(&self) -> bool { self.method == Method::PATCH }

    // Safe header access
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)?.to_str().ok()
    }

    pub fn headers(&self) -> &HeaderMap { &self.headers }

    // Safe query access with parsing
    pub fn query<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.query.get_parsed(key)
    }

    pub fn query_str(&self, key: &str) -> Option<&str> {
        self.query.get(key)
    }

    pub fn query_all(&self, key: &str) -> Vec<&str> {
        self.query.get_all(key)
    }

    // Safe form access
    pub fn form<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        self.form.deserialize()
    }

    // Safe cookie access
    pub fn cookie(&self, name: &str) -> Option<&Cookie<'_>> {
        self.cookies.get(name)
    }

    pub fn cookie_value(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(|c| c.value())
    }

    // HTMX detection
    pub fn is_htmx_request(&self) -> bool {
        self.headers.get("HX-Request")
            .and_then(|h| h.to_str().ok())
            .map(|s| s == "true")
            .unwrap_or(false)
    }

    pub fn htmx_target(&self) -> Option<&str> {
        self.header("HX-Target")
    }

    pub fn htmx_trigger(&self) -> Option<&str> {
        self.header("HX-Trigger")
    }

    // Extensions (for middleware data)
    pub fn extensions(&self) -> &Extensions { &self.extensions }
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        Arc::get_mut(&mut self.extensions).expect("extensions Arc has multiple owners")
    }

    // Database access
    pub fn db(&self) -> &SqlitePool { &self.db }

    // Builder pattern
    pub fn with_query(mut self, params: QueryParams) -> Self {
        self.query = params;
        self
    }

    pub fn with_form(mut self, form: FormData) -> Self {
        self.form = form;
        self
    }

    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }
}

/// Type-safe query parameters
#[derive(Clone, Default)]
pub struct QueryParams {
    params: HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key)?.first().map(|s| s.as_str())
    }

    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.params
            .get(key)
            .map(|vals| vals.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn get_parsed<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key)?.parse().ok()
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.params.entry(key).or_default().push(value);
    }
}

/// Type-safe form data
#[derive(Clone, Default)]
pub struct FormData {
    fields: HashMap<String, String>,
    files: HashMap<String, Vec<u8>>,
}

impl FormData {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|s| s.as_str())
    }

    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        let json = serde_json::to_value(&self.fields)?;
        serde_json::from_value(json)
    }

    pub fn file(&self, key: &str) -> Option<&[u8]> {
        self.files.get(key).map(|v| v.as_slice())
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.fields.insert(key, value);
    }

    pub fn insert_file(&mut self, key: String, data: Vec<u8>) {
        self.files.insert(key, data);
    }
}

/// For storing middleware data
pub struct Extensions(HashMap<std::any::TypeId, Box<dyn std::any::Any>>);

impl Extensions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.0.insert(std::any::TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&std::any::TypeId::of::<T>())
            .and_then(|b| b.downcast_ref())
    }
}
```

---

## 2. CODE QUALITY IMPROVEMENTS

### Improvement 1: Add Comprehensive Tests

**Missing Test Coverage:**
```rust
// tests/integration/mod.rs
mod request_context;
mod action_executor;
mod validation;
mod database;
mod rendering;
mod routing;

// tests/unit/mod.rs
mod template_loader;
mod directive_parsing;
mod css_scoping;
```

**Example - Template Rendering Tests:**
```rust
// tests/integration/rendering.rs
#[tokio::test]
async fn test_simple_page_render() {
    let renderer = Renderer::new();
    let template = "Hello, {name}!";
    let context = json!({"name": "World"});

    let result = renderer.render(template, &context).await.unwrap();
    assert_eq!(result, "Hello, World!");
}

#[tokio::test]
async fn test_conditional_rendering() {
    let renderer = Renderer::new();
    let template = r#"
        <div r-if="show">Visible</div>
        <div r-else>Hidden</div>
    "#;

    let context = json!({"show": true});
    let result = renderer.render(template, &context).await.unwrap();
    assert!(result.contains("Visible"));
    assert!(!result.contains("Hidden"));
}

#[tokio::test]
async fn test_loop_rendering() {
    let renderer = Renderer::new();
    let template = "<li r-for=\"item in items\">{item}</li>";
    let context = json!({"items": ["a", "b", "c"]});

    let result = renderer.render(template, &context).await.unwrap();
    assert!(result.contains("<li>a</li>"));
    assert!(result.contains("<li>b</li>"));
    assert!(result.contains("<li>c</li>"));
}

#[tokio::test]
async fn test_error_on_missing_variable() {
    let renderer = Renderer::new();
    let template = "Hello, {missing}!";
    let context = json!({"name": "World"});

    let result = renderer.render(template, &context).await;
    assert!(result.is_err());
}
```

### Improvement 2: Error Recovery Patterns

**Add Graceful Degradation:**
```rust
/// Render with fallbacks
pub async fn render_safe(
    &self,
    template: &str,
    context: &Value,
) -> String {
    match self.render(template, context).await {
        Ok(html) => html,
        Err(e) => {
            // Log error for debugging
            tracing::error!("Template render error: {}", e);

            // Return helpful error page in development
            if cfg!(debug_assertions) {
                format!(
                    "<div style='border: 1px solid red; padding: 10px;'>\
                    <h3>Template Error</h3>\
                    <pre>{}</pre>\
                    </div>",
                    html_escape::encode_text(&e.to_string())
                )
            } else {
                // Return minimal error in production
                "<div>Content unavailable</div>".to_string()
            }
        }
    }
}
```

---

## 3. PERFORMANCE OPTIMIZATIONS

### Optimization 1: Reduce Lock Contention

**Current (problematic):**
```rust
lazy_static::lazy_static! {
    static ref TEMPLATE_CACHE: Mutex<HashMap<String, String>> =
        Mutex::new(HashMap::new());
}

// Every access acquires lock
let mut cache = TEMPLATE_CACHE.lock().unwrap();
let template = cache.get("users");
```

**Improved - Use DashMap:**
```rust
use dashmap::DashMap;

pub struct TemplateCache {
    cache: DashMap<String, Arc<CompiledTemplate>>,
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<CompiledTemplate>> {
        self.cache.get(key).map(|entry| entry.clone())
    }

    pub fn insert(&self, key: String, template: Arc<CompiledTemplate>) {
        self.cache.insert(key, template);
    }

    pub fn clear(&self) {
        self.cache.clear();
    }
}
```

### Optimization 2: Compile Templates Once

**Proposed - `src/compiler.rs`:**
```rust
/// Pre-compiled template ready for rendering
pub struct CompiledTemplate {
    // Pre-parsed AST
    directives: Vec<Directive>,
    // Pre-compiled expressions
    expressions: HashMap<String, Expression>,
    // Pre-computed CSS scopes
    styles: String,
}

impl CompiledTemplate {
    pub async fn render(&self, context: &Value) -> Result<String> {
        // Much faster than parsing on every render
        let mut output = String::new();
        self.render_into(&mut output, context).await?;
        Ok(output)
    }

    async fn render_into(&self, out: &mut String, context: &Value) -> Result<()> {
        // Render pre-parsed directives
        for directive in &self.directives {
            self.render_directive(directive, out, context).await?;
        }
        Ok(())
    }
}
```

---

## 4. TESTING STRATEGY

### Test Coverage Goals
- **Unit tests:** 70%+ coverage
- **Integration tests:** All critical paths
- **Property-based tests:** Template rendering edge cases

### Test Organization
```
tests/
├── common/
│   ├── mod.rs              # Test utilities
│   ├── fixtures.rs         # Reusable test data
│   └── assertions.rs       # Custom assertions
├── integration/
│   ├── mod.rs
│   ├── routing_test.rs     # Route matching
│   ├── rendering_test.rs   # Template rendering
│   ├── actions_test.rs     # Action execution
│   ├── validation_test.rs  # Validation pipeline
│   └── database_test.rs    # Database operations
└── unit/
    ├── mod.rs
    ├── request_context_test.rs
    ├── form_data_test.rs
    ├── query_params_test.rs
    └── error_test.rs
```

---

## 5. CHECKLIST FOR IMPLEMENTATION

- [ ] Create custom error type (`src/error.rs`)
- [ ] Extract startup logic (`src/startup.rs`)
- [ ] Create handlers module (`src/handlers/`)
- [ ] Refactor main.rs to use modules
- [ ] Improve RequestContext API
- [ ] Add 50+ unit tests
- [ ] Add 30+ integration tests
- [ ] Update error handling throughout
- [ ] Add docstrings to public APIs
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Check formatting: `cargo fmt --check`
- [ ] Run full test suite
- [ ] Measure code coverage: `cargo tarpaulin`

---

## Summary

These refactorings will:
✅ Reduce complexity and cognitive load
✅ Improve error handling
✅ Enable better testing
✅ Support future extensibility
✅ Match Rust best practices
✅ Better IDE support and navigation
✅ Clearer error messages to users
✅ Easier debugging

**Estimated Time Investment:** 3-4 weeks
**Expected Payoff:** Significantly improved code quality and maintainability
