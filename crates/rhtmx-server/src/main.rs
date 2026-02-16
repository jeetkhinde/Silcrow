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
use rhtmx::{Config, FormData, QueryParams, Renderer, RequestContext, TemplateLoader, Value};
use crate::hot_reload::{create_watcher, ChangeType};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_livereload::LiveReloadLayer;
use tracing::{error, info};

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

    // Load templates
    let mut loader = TemplateLoader::with_config(
        &config.routing.pages_dir,
        &config.routing.components_dir,
        config.routing.case_insensitive,
    );
    match loader.load_all() {
        Ok(_) => {
            println!("Loaded {} templates", loader.count());
            for route in loader.list_routes() {
                println!("  {} -> template", route);
            }
        }
        Err(e) => {
            eprintln!("Failed to load templates: {}", e);
            std::process::exit(1);
        }
    }

    let template_loader = Arc::new(RwLock::new(loader));

    // Hot reload
    if hot_reload_enabled {
        println!("Hot reload: enabled");
        match create_watcher() {
            Ok(watcher) => {
                let loader_clone = template_loader.clone();
                let mut reload_rx = watcher.subscribe();

                tokio::spawn(async move {
                    let _watcher = watcher;
                    while let Ok(file_change) = reload_rx.recv().await {
                        match file_change.change_type {
                            ChangeType::Template | ChangeType::Component => {
                                info!("Reloading template: {:?}", file_change.path);
                                let mut loader = loader_clone.write().await;
                                if let Err(e) = loader.reload_template(&file_change.path) {
                                    error!("Failed to reload template: {}", e);
                                }
                            }
                            ChangeType::SourceCode => {
                                info!("Source code changed - restart server for changes");
                            }
                        }
                    }
                });
            }
            Err(e) => eprintln!("Failed to create file watcher: {}", e),
        }
    }

    let state = AppState { template_loader: template_loader.clone() };

    let mut app = Router::new()
        .route("/", get(index_handler).post(index_handler).put(index_handler).delete(index_handler))
        .route("/*path", get(template_handler).post(template_handler).put(template_handler).delete(template_handler))
        .with_state(state);

    if hot_reload_enabled {
        app = app.layer(LiveReloadLayer::new());
    }

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    query: AxumQuery<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let ctx = create_request_context(method, "/".to_string(), query.0, headers, body);
    render_route(&state, "/", ctx).await
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
    render_route(&state, &route, ctx).await
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

async fn render_route(state: &AppState, route: &str, ctx: RequestContext) -> Response {
    let loader = state.template_loader.read().await;

    // Match route
    let route_match = match loader.router().match_route(route) {
        Some(m) => m,
        None => {
            if loader.get(route).is_some() {
                drop(loader);
                return render_route_direct(state, route, ctx).await;
            }
            drop(loader);
            return error_response(404, "Page Not Found", &format!("Route '{}' not found", route));
        }
    };

    let page_template = loader.get(&route_match.route.pattern).or_else(|| loader.get(route));
    let page_template = match page_template {
        Some(t) => t.clone(),
        None => return error_response(404, "Template Not Found", &format!("Template for '{}' not found", route)),
    };

    let layout_template = match loader.get_layout_for_route(&route_match.route.pattern) {
        Some(t) => t.clone(),
        None => return error_response(500, "Layout Not Found", "Missing _layout in pages directory"),
    };

    let loader_arc = Arc::new((*loader).clone());
    drop(loader);

    let mut renderer = Renderer::with_loader(loader_arc);

    // Set route parameters
    for (name, value) in &route_match.params {
        renderer.set_var(name, Value::String(value.clone()));
    }

    setup_request_vars(&mut renderer, &ctx);

    // Content negotiation
    if ctx.accepts_json() {
        let data = serde_json::json!({
            "route": route,
            "method": ctx.method.as_str(),
            "query": ctx.query.as_map(),
            "form": ctx.form.as_map(),
        });
        return Json(data).into_response();
    }

    if ctx.wants_partial() {
        match renderer.render_partial(&page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    } else {
        match renderer.render_with_layout(&layout_template.content, &page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    }
}

async fn render_route_direct(state: &AppState, route: &str, ctx: RequestContext) -> Response {
    let loader = state.template_loader.read().await;

    let page_template = match loader.get(route) {
        Some(t) => t.clone(),
        None => return error_response(404, "Page Not Found", &format!("Route '{}' not found", route)),
    };

    let layout_template = match loader.get_layout() {
        Some(t) => t.clone(),
        None => return error_response(500, "Layout Not Found", "Missing _layout in pages directory"),
    };

    let loader_arc = Arc::new((*loader).clone());
    drop(loader);

    let mut renderer = Renderer::with_loader(loader_arc);
    setup_request_vars(&mut renderer, &ctx);

    if ctx.accepts_json() {
        let data = serde_json::json!({
            "route": route,
            "method": ctx.method.as_str(),
            "query": ctx.query.as_map(),
            "form": ctx.form.as_map(),
        });
        return Json(data).into_response();
    }

    if ctx.wants_partial() {
        match renderer.render_partial(&page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    } else {
        match renderer.render_with_layout(&layout_template.content, &page_template.content) {
            Ok(result) => Html(result.html).into_response(),
            Err(e) => error_response(500, "Render Error", &format!("{}", e)),
        }
    }
}

fn setup_request_vars(renderer: &mut Renderer, ctx: &RequestContext) {
    renderer.set_var("request_method", Value::String(ctx.method.as_str().to_string()));
    renderer.set_var("request_path", Value::String(ctx.path.clone()));

    let mut query_map = std::collections::HashMap::new();
    for (key, value) in ctx.query.as_map() {
        query_map.insert(key.clone(), Value::String(value.clone()));
        renderer.set_var(format!("query_{}", key), Value::String(value.clone()));
    }
    renderer.set_var("query", Value::Object(query_map));

    let mut form_map = std::collections::HashMap::new();
    for (key, value) in ctx.form.as_map() {
        form_map.insert(key.clone(), Value::String(value.clone()));
        renderer.set_var(format!("form_{}", key), Value::String(value.clone()));
    }
    renderer.set_var("form", Value::Object(form_map));

    renderer.set_var("is_htmx", Value::Bool(ctx.is_htmx()));
    renderer.set_var("wants_partial", Value::Bool(ctx.wants_partial()));
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
