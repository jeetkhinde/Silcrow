mod hot_reload;
mod maud_wrapper;

use axum::{
    body::Bytes,
    extract::{Query as AxumQuery, State},
    http::{HeaderMap, Method},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use rhtmx::{Config, FormData, HandlerFn, QueryParams, RequestContext, TemplateLoader};
use crate::hot_reload::{create_watcher, ChangeType};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_livereload::LiveReloadLayer;
use tracing::info;

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    template_loader: Arc<RwLock<TemplateLoader>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("rhtmx starting...");

    let config = Config::load_default().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}, using defaults", e);
        Config::default()
    });

    println!("Port: {}, Pages: {}", config.server.port, config.routing.pages_dir);

    let hot_reload_enabled = std::env::var("HOT_RELOAD")
        .map(|v| v.parse::<bool>().unwrap_or(config.dev.hot_reload))
        .unwrap_or(config.dev.hot_reload);

    // Discover routes from .rs files in pages directory
    let mut loader = TemplateLoader::with_config(
        &config.routing.pages_dir,
        &config.routing.components_dir,
        config.routing.case_insensitive,
    );
    match loader.discover_routes() {
        Result::Ok(_) => {
            println!("Discovered {} routes", loader.count());
            for route in loader.list_routes() {
                println!("  {} -> page", route);
            }
        }
        Err(e) => {
            eprintln!("Failed to discover routes: {}", e);
            // Not fatal — routes can be registered programmatically
        }
    }

    // Register compiled Maud handlers for routes
    register_default_handlers(&mut loader);

    let template_loader = Arc::new(RwLock::new(loader));

    // Hot reload
    if hot_reload_enabled {
        println!("Hot reload: enabled (recompile required for .rs page changes)");
        match create_watcher() {
            Ok(watcher) => {
                let mut reload_rx = watcher.subscribe();

                tokio::spawn(async move {
                    let _watcher = watcher;
                    while let Ok(file_change) = reload_rx.recv().await {
                        let ChangeType::SourceCode = file_change.change_type;
                        info!("Source code changed: {:?} - restart server for changes", file_change.path);
                    }
                });
            }
            Err(e) => eprintln!("Failed to create file watcher: {}", e),
        }
    }

    let state = AppState { template_loader: template_loader.clone() };

    let app = Router::new()
        .route("/", get(index_handler).post(index_handler).put(index_handler).delete(index_handler))
        .route("/*path", get(template_handler).post(template_handler).put(template_handler).delete(template_handler))
        .with_state(state);

    let app = if hot_reload_enabled {
        app.layer(LiveReloadLayer::new())
    } else {
        app
    };

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

/// Register default Maud-based handlers
fn register_default_handlers(loader: &mut TemplateLoader) {
    let index_handler: HandlerFn = Arc::new(|_ctx| {
        Box::pin(async {
            let markup = maud::html! {
                (maud::DOCTYPE)
                html lang="en" {
                    head {
                        meta charset="UTF-8";
                        title { "RHTMX" }
                    }
                    body {
                        h1 { "Welcome to RHTMX" }
                        p { "Rust + HTMX with Maud compile-time templates." }
                    }
                }
            };
            Html(markup.into_string()).into_response()
        })
    });
    loader.register_route("/", index_handler);
}

async fn index_handler(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    query: AxumQuery<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let ctx = create_request_context(method, "/".to_string(), query.0, headers, body);
    dispatch_route(&state, "/", ctx).await
}

async fn template_handler(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
    method: Method,
    headers: HeaderMap,
    query: AxumQuery<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let route = format!("/{}", path);
    let ctx = create_request_context(method, route.clone(), query.0, headers, body);
    dispatch_route(&state, &route, ctx).await
}

fn create_request_context(
    method: Method,
    path: String,
    query_params: std::collections::HashMap<String, String>,
    headers: HeaderMap,
    body: Bytes,
) -> RequestContext {
    let query = QueryParams::new(query_params);

    let form = if method == Method::POST || method == Method::PUT || method == Method::DELETE {
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(ct) = content_type.to_str() {
                if ct.contains("application/json") {
                    serde_json::from_slice::<JsonValue>(&body)
                        .map(FormData::from_json)
                        .unwrap_or_else(|_| FormData::new())
                } else if ct.contains("application/x-www-form-urlencoded") {
                    let form_str = String::from_utf8_lossy(&body);
                    let fields = form_str.split('&')
                        .filter_map(|pair| {
                            pair.split_once('=').map(|(k, v)| {
                                (
                                    urlencoding::decode(k).unwrap_or_default().to_string(),
                                    urlencoding::decode(v).unwrap_or_default().to_string(),
                                )
                            })
                        })
                        .collect();
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

    RequestContext::new(method, path, query, form, headers)
}

/// Dispatch a request to the appropriate compiled handler
async fn dispatch_route(state: &AppState, route: &str, ctx: RequestContext) -> Response {
    let loader = state.template_loader.read().await;

    // Content negotiation: JSON response
    if ctx.accepts_json() {
        let data = serde_json::json!({
            "route": route,
            "method": ctx.method.as_str(),
            "query": ctx.query.as_map(),
            "form": ctx.form.as_map(),
        });
        return Json(data).into_response();
    }

    // Try to match route via the router
    let matched_pattern = loader
        .router()
        .match_route(route)
        .map(|m| m.route.pattern.clone());

    // Look up the handler: first try matched pattern, then direct route
    let handler = matched_pattern
        .as_deref()
        .and_then(|pattern| loader.get_handler(pattern))
        .or_else(|| loader.get_handler(route))
        .cloned();

    drop(loader);

    match handler {
        Some(handler) => handler(ctx).await,
        None => error_response(404, "Page Not Found", &format!("Route '{}' not found", route)),
    }
}

fn error_response(status: u16, title: &str, message: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{title}</title></head>
<body>
  <h1>{status} {title}</h1>
  <p>{message}</p>
  <a href="/">Go Home</a>
</body>
</html>"#,
        status = status, title = title, message = message
    );
    (
        axum::http::StatusCode::from_u16(status).unwrap(),
        Html(html),
    ).into_response()
}
