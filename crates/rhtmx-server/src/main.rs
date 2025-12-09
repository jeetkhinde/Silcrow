// Module declarations
mod hot_reload;
mod action_handlers;
mod example_actions;
mod maud_wrapper;
mod form_context;

use axum::{
    body::Bytes,
    extract::{Query as AxumQuery, State},
    http::{HeaderMap, Method},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use rhtmx::{
    database, Config, FormData,
    QueryParams, Renderer, RequestContext, TemplateLoader, Value,
};
use crate::hot_reload::{create_watcher, ChangeType};
use crate::action_handlers::{ActionHandlerRegistry, register_built_in_handlers};
use serde_json::Value as JsonValue;
use sqlx::AnyPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_livereload::LiveReloadLayer;
use tracing::{error, info};

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    template_loader: Arc<RwLock<TemplateLoader>>,
    action_registry: Arc<ActionHandlerRegistry>,
    db: Option<Arc<AnyPool>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 rhtmx App Starting...");

    // Load configuration
    let config = Config::load_default().unwrap_or_else(|e| {
        eprintln!("⚠️  Failed to load config: {}", e);
        eprintln!("   Using default configuration...");
        Config::default()
    });

    println!("⚙️  Configuration:");
    println!("   - Port: {}", config.server.port);
    println!("   - Pages directory: {}", config.routing.pages_dir);
    println!(
        "   - Components directory: {}",
        config.routing.components_dir
    );
    println!(
        "   - Case-insensitive routing: {}",
        config.routing.case_insensitive
    );

    // Check if hot reload is enabled (default: true for development)
    let hot_reload_enabled = std::env::var("HOT_RELOAD")
        .map(|v| v.parse::<bool>().unwrap_or(config.dev.hot_reload))
        .unwrap_or(config.dev.hot_reload);

    // Load all templates with configuration from rhtmx.toml
    let mut loader = TemplateLoader::with_config(
        &config.routing.pages_dir,
        &config.routing.components_dir,
        config.routing.case_insensitive,
    );
    match loader.load_all() {
        Ok(_) => {
            println!("✅ Loaded {} templates", loader.count());
            println!("📋 Routes:");
            for route in loader.list_routes() {
                println!("   {} -> template", route);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to load templates: {}", e);
            std::process::exit(1);
        }
    }

    // Wrap loader in Arc<RwLock> for thread-safe updates
    let template_loader = Arc::new(RwLock::new(loader));

    // Setup hot reload if enabled
    if hot_reload_enabled {
        println!("🔄 Hot Reload: ENABLED");

        // Create file watcher and spawn template reload task
        match create_watcher() {
            Ok(watcher) => {
                let loader_clone = template_loader.clone();
                let mut reload_rx = watcher.subscribe();

                tokio::spawn(async move {
                    let _watcher = watcher; // Keep watcher alive

                    while let Ok(file_change) = reload_rx.recv().await {
                        match file_change.change_type {
                            ChangeType::Template | ChangeType::Component => {
                                info!("🔄 Reloading template: {:?}", file_change.path);

                                let mut loader = loader_clone.write().await;
                                if let Err(e) = loader.reload_template(&file_change.path) {
                                    error!("❌ Failed to reload template: {}", e);
                                } else {
                                    info!("✅ Template reloaded successfully");
                                }
                            }
                            ChangeType::SourceCode => {
                                info!("⚠️  Source code changed - restart server for changes to take effect");
                            }
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("⚠️  Failed to create file watcher: {}", e);
                eprintln!("   Continuing without hot reload...");
            }
        }
    } else {
        println!("🔄 Hot Reload: DISABLED");
    }

    // Initialize database (optional)
    // Load DATABASE_URL from .env file or environment variable
    dotenvy::dotenv().ok();
    let db_pool = if let Ok(database_url) = std::env::var("DATABASE_URL") {
        let db_type = if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            "PostgreSQL"
        } else if database_url.starts_with("mysql://") {
            "MySQL"
        } else {
            "SQLite"
        };

        println!("📁 Initializing {} database...", db_type);
        println!("   Database: {}", if db_type == "SQLite" { &database_url } else { "***" });

        match database::init_db(&database_url).await {
            Ok(pool) => {
                println!("✅ Database initialized successfully");
                Some(Arc::new(pool))
            }
            Err(e) => {
                eprintln!("❌ Failed to initialize database: {}", e);
                eprintln!("   Make sure DATABASE_URL is set correctly in .env or environment");
                std::process::exit(1);
            }
        }
    } else {
        println!("📁 Database configuration not found");
        println!("   Set DATABASE_URL in .env or environment to enable database");
        println!("   Application running without database support");
        None
    };

    // Setup action handler registry
    let mut action_registry = ActionHandlerRegistry::new();
    register_built_in_handlers(&mut action_registry);

    // Setup application state
    let state = AppState {
        template_loader: template_loader.clone(),
        action_registry: Arc::new(action_registry),
        db: db_pool,
    };

    // Build router with support for all HTTP methods
    let mut app = Router::new()
        .route(
            "/",
            get(index_handler)
                .post(index_handler)
                .put(index_handler)
                .delete(index_handler),
        )
        .route(
            "/*path",
            get(template_handler)
                .post(template_handler)
                .put(template_handler)
                .delete(template_handler),
        )
        .with_state(state);

    // Add LiveReloadLayer if hot reload is enabled
    // tower-livereload has built-in file watching and will trigger browser reloads automatically
    if hot_reload_enabled {
        app = app.layer(LiveReloadLayer::new());
    }

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("✅ Server running at http://localhost:3000");
    if hot_reload_enabled {
        println!("🔥 Hot reload enabled - edit templates and watch them update!");
    }
    println!("🎯 Try visiting: http://localhost:3000/\n");

    axum::serve(listener, app).await.unwrap();
}

/// Handler for home page "/"
async fn index_handler(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    query: AxumQuery<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let request_context = create_request_context(
        method,
        "/".to_string(),
        query.0,
        headers,
        body,
        state.db.clone(),
    )
    .await;
    render_route(&state, "/", request_context).await
}

/// Handler for all other routes
async fn template_handler(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
    method: Method,
    headers: HeaderMap,
    query: AxumQuery<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let route = format!("/{}", path);
    let request_context = create_request_context(
        method,
        route.clone(),
        query.0,
        headers,
        body,
        state.db.clone(),
    )
    .await;
    render_route(&state, &route, request_context).await
}

/// Create request context from Axum extractors
async fn create_request_context(
    method: Method,
    path: String,
    query_params: std::collections::HashMap<String, String>,
    headers: HeaderMap,
    body: Bytes,
    db: Option<Arc<AnyPool>>,
) -> RequestContext {
    // Create query params
    let query = QueryParams::new(query_params);

    // Parse form data based on content-type
    let form = if method == Method::POST || method == Method::PUT || method == Method::DELETE {
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(ct) = content_type.to_str() {
                if ct.contains("application/json") {
                    // Parse as JSON
                    if let Ok(json) = serde_json::from_slice::<JsonValue>(&body) {
                        FormData::from_json(json)
                    } else {
                        FormData::new()
                    }
                } else if ct.contains("application/x-www-form-urlencoded") {
                    // Parse as form
                    let form_str = String::from_utf8_lossy(&body);
                    let mut fields = std::collections::HashMap::new();
                    for pair in form_str.split('&') {
                        if let Some((key, value)) = pair.split_once('=') {
                            let key = urlencoding::decode(key).unwrap_or_default().to_string();
                            let value = urlencoding::decode(value).unwrap_or_default().to_string();
                            fields.insert(key, value);
                        }
                    }
                    FormData::from_fields(fields)
                } else {
                    FormData::new()
                }
            } else {
                FormData::new()
            }
        } else {
            FormData::new()
        }
    } else {
        FormData::new()
    };

    RequestContext::new(method, path, query, form, headers, db)
}

/// Render a route with layout
async fn render_route(state: &AppState, route: &str, request_context: RequestContext) -> Response {
    // Check if there's an action handler for this route and method
    // Try with query parameters first (more specific match)
    let method_str = request_context.method.as_str();
    if let Some(handler) = state.action_registry.find_with_query(
        route,
        method_str,
        request_context.query.as_map(),
    ) {
        // Execute the action handler instead of rendering the template
        return handler(request_context).await.into_response();
    }

    let loader = state.template_loader.read().await;

    // Use the router to match the route
    let route_match = match loader.router().match_route(route) {
        Some(m) => m,
        None => {
            // Try direct template lookup as fallback
            if loader.get(route).is_some() {
                drop(loader);
                return render_route_direct(state, route, request_context).await;
            }
            drop(loader);
            return custom_error_response(
                state,
                404,
                "Page Not Found",
                &format!("Route '{}' not found", route),
                Some(route),
            )
            .await;
        }
    };

    // Try to get template by pattern
    let page_template = loader
        .get(&route_match.route.pattern)
        .or_else(|| loader.get(route));

    let page_template = match page_template {
        Some(t) => t.clone(),
        None => {
            return error_response(
                404,
                "Template Not Found",
                &format!("Template for route '{}' not found", route),
            );
        }
    };

    // Get the appropriate layout (section-specific or root)
    let layout_template = match loader.get_layout_for_route(&route_match.route.pattern) {
        Some(t) => t.clone(),
        None => {
            return error_response(
                500,
                "Layout Not Found",
                "Missing _layout.rhtmx in pages directory",
            );
        }
    };

    // Create Arc wrapper for the locked loader to pass to renderer
    let loader_arc = Arc::new((*loader).clone());
    drop(loader);

    // Create a new renderer for this request with component access
    let mut renderer = Renderer::with_loader(loader_arc);

    // Collect CSS from layout and page templates
    renderer.collect_template_css(&layout_template.scoped_css);
    renderer.collect_template_css(&page_template.scoped_css);

    // Set route parameters as variables
    for (param_name, param_value) in &route_match.params {
        renderer.set_var(param_name, Value::String(param_value.clone()));
    }

    // Set request context data as variables
    setup_request_context(&mut renderer, &request_context);

    // Set up demo data based on route (for backward compatibility)
    setup_demo_data(&mut renderer, route, &route_match.params);

    // Check if client wants JSON response (content negotiation)
    if request_context.accepts_json() {
        // Return JSON response (you can customize this to return actual data)
        let response_data = serde_json::json!({
            "route": route,
            "method": request_context.method.as_str(),
            "query": request_context.query.as_map(),
            "form": request_context.form.as_map(),
        });
        return Json(response_data).into_response();
    }

    // Check if this is a partial request (HTMX request or ?partial=true)
    let wants_partial = request_context.wants_partial();

    if wants_partial {
        // Render as partial (without layout)
        match renderer.render_partial(&page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    } else {
        // Render the page with default layout (full HTML response)
        match renderer.render_with_layout(&layout_template.content, &page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    }
}

/// Render a route directly (fallback for old-style routes)
async fn render_route_direct(
    state: &AppState,
    route: &str,
    request_context: RequestContext,
) -> Response {
    let loader = state.template_loader.read().await;

    let page_template = match loader.get(route) {
        Some(t) => t.clone(),
        None => {
            return error_response(
                404,
                "Page Not Found",
                &format!("Route '{}' not found", route),
            );
        }
    };

    let layout_template = match loader.get_layout() {
        Some(t) => t.clone(),
        None => {
            return error_response(
                500,
                "Layout Not Found",
                "Missing _layout.rhtmx in pages directory",
            );
        }
    };

    let loader_arc = Arc::new((*loader).clone());
    drop(loader);

    let mut renderer = Renderer::with_loader(loader_arc);

    // Collect CSS from layout and page templates
    renderer.collect_template_css(&layout_template.scoped_css);
    renderer.collect_template_css(&page_template.scoped_css);

    // Set request context data as variables
    setup_request_context(&mut renderer, &request_context);

    setup_demo_data(&mut renderer, route, &std::collections::HashMap::new());

    // Check if client wants JSON response (content negotiation)
    if request_context.accepts_json() {
        let response_data = serde_json::json!({
            "route": route,
            "method": request_context.method.as_str(),
            "query": request_context.query.as_map(),
            "form": request_context.form.as_map(),
        });
        return Json(response_data).into_response();
    }

    // Check if this is a partial request (HTMX request or ?partial=true)
    let wants_partial = request_context.wants_partial();

    if wants_partial {
        // Render as partial (without layout)
        match renderer.render_partial(&page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    } else {
        // Render the page with default layout (full HTML response)
        match renderer.render_with_layout(&layout_template.content, &page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    }
}

/// Setup request context data as template variables
fn setup_request_context(renderer: &mut Renderer, ctx: &RequestContext) {
    // Set HTTP method
    renderer.set_var(
        "request_method",
        Value::String(ctx.method.as_str().to_string()),
    );

    // Set path
    renderer.set_var("request_path", Value::String(ctx.path.clone()));

    // Set query parameters as an object
    let mut query_map = std::collections::HashMap::new();
    for (key, value) in ctx.query.as_map() {
        query_map.insert(key.clone(), Value::String(value.clone()));
    }
    renderer.set_var("query", Value::Object(query_map.clone()));

    // Also set individual query params
    for (key, value) in ctx.query.as_map() {
        renderer.set_var(format!("query_{}", key), Value::String(value.clone()));
    }

    // Set form data as an object
    let mut form_map = std::collections::HashMap::new();
    for (key, value) in ctx.form.as_map() {
        form_map.insert(key.clone(), Value::String(value.clone()));
    }
    renderer.set_var("form", Value::Object(form_map.clone()));

    // Also set individual form fields
    for (key, value) in ctx.form.as_map() {
        renderer.set_var(format!("form_{}", key), Value::String(value.clone()));
    }

    // Set cookies as an object
    let mut cookies_map = std::collections::HashMap::new();
    for (key, value) in &ctx.cookies {
        cookies_map.insert(key.clone(), Value::String(value.clone()));
    }
    renderer.set_var("cookies", Value::Object(cookies_map));

    // Set request info
    renderer.set_var("is_get", Value::Bool(ctx.is_get()));
    renderer.set_var("is_post", Value::Bool(ctx.is_post()));
    renderer.set_var("is_put", Value::Bool(ctx.is_put()));
    renderer.set_var("is_delete", Value::Bool(ctx.is_delete()));
    renderer.set_var("accepts_json", Value::Bool(ctx.accepts_json()));

    // Set partial/HTMX info
    renderer.set_var("wants_partial", Value::Bool(ctx.wants_partial()));
    renderer.set_var("is_htmx", Value::Bool(ctx.is_htmx()));
    if let Some(target) = ctx.htmx_target() {
        renderer.set_var("htmx_target", Value::String(target.to_string()));
    }
    if let Some(trigger) = ctx.htmx_trigger() {
        renderer.set_var("htmx_trigger", Value::String(trigger.to_string()));
    }
}

/// Setup demo data for specific routes
fn setup_demo_data(
    renderer: &mut Renderer,
    route: &str,
    _params: &std::collections::HashMap<String, String>,
) {
    if route == "/loops" {
        // Example 1: Fruits array
        renderer.set_var(
            "fruits",
            Value::Array(vec![
                Value::String("Apple".to_string()),
                Value::String("Banana".to_string()),
                Value::String("Cherry".to_string()),
                Value::String("Dragon Fruit".to_string()),
            ]),
        );

        // Example 2: Colors array
        renderer.set_var(
            "colors",
            Value::Array(vec![
                Value::String("Red".to_string()),
                Value::String("Green".to_string()),
                Value::String("Blue".to_string()),
                Value::String("Yellow".to_string()),
            ]),
        );

        // Example 3: Tasks array
        renderer.set_var(
            "tasks",
            Value::Array(vec![
                Value::String("Implement r-for directive".to_string()),
                Value::String("Create demo page".to_string()),
                Value::String("Test the feature".to_string()),
                Value::String("Write documentation".to_string()),
            ]),
        );

        // Example 4: Numbers array
        renderer.set_var(
            "numbers",
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
                Value::Number(7.0),
                Value::Number(8.0),
            ]),
        );
    } else if route == "/match" {
        // Example 1: User role
        renderer.set_var("user_role", Value::String("admin".to_string()));

        // Example 2: Order status
        renderer.set_var("order_status", Value::String("shipped".to_string()));

        // Example 3: Payment method
        renderer.set_var("payment_method", Value::String("card".to_string()));

        // Example 4: Theme
        renderer.set_var("theme", Value::String("dark".to_string()));
    }
}

/// Create a custom error response using _error.rhtmx if available
async fn custom_error_response(
    state: &AppState,
    status: u16,
    title: &str,
    message: &str,
    route_pattern: Option<&str>,
) -> Response {
    let loader = state.template_loader.read().await;

    // Try to get custom error page for the route
    let error_template = if let Some(pattern) = route_pattern {
        loader
            .get_error_page_for_route(pattern)
            .or_else(|| loader.get_error_page())
    } else {
        loader.get_error_page()
    };

    if let Some(error_page) = error_template {
        // Custom error page found - render it
        let error_page_clone = error_page.clone();
        let loader_arc = Arc::new((*loader).clone());
        drop(loader);

        let mut renderer = Renderer::with_loader(loader_arc);

        // Set error variables
        renderer.set_var("status", Value::Number(status as f64));
        renderer.set_var("title", Value::String(title.to_string()));
        renderer.set_var("message", Value::String(message.to_string()));

        // Render error page (without layout, as error pages should be standalone)
        match renderer.render_partial(&error_page_clone.content) {
            Ok(result) => {
                return (
                    axum::http::StatusCode::from_u16(status).unwrap(),
                    Html(result.html),
                )
                    .into_response();
            }
            Err(_) => {
                // Fall through to default error response
                return error_response(status, title, message);
            }
        }
    } else {
        drop(loader);
    }

    // Fall back to default error response
    error_response(status, title, message)
}

/// Create a default error response (fallback when no custom error page exists)
fn error_response(status: u16, title: &str, message: &str) -> Response {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>{title}</title>
            <script src="https://cdn.tailwindcss.com"></script>
        </head>
        <body class="bg-gray-100">
            <div class="min-h-screen flex items-center justify-center">
                <div class="bg-white rounded-lg shadow-lg p-8 max-w-md">
                    <h1 class="text-4xl font-bold text-red-600 mb-4">{status}</h1>
                    <h2 class="text-2xl font-semibold text-gray-800 mb-2">{title}</h2>
                    <p class="text-gray-600">{message}</p>
                    <a href="/" class="mt-4 inline-block bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                        Go Home.
                    </a>
                </div>
            </div>
        </body>
        </html>
        "#,
        status = status,
        title = title,
        message = message
    );

    (
        axum::http::StatusCode::from_u16(status).unwrap(),
        Html(html),
    )
        .into_response()
}
