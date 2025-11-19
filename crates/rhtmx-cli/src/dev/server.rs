#[cfg(feature = "dev-server")]
use anyhow::Result;
#[cfg(feature = "dev-server")]
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
#[cfg(feature = "dev-server")]
use colored::Colorize;
#[cfg(feature = "dev-server")]
use rhtmx::TemplateLoader;
#[cfg(feature = "dev-server")]
use std::path::Path;
#[cfg(feature = "dev-server")]
use std::sync::Arc;
#[cfg(feature = "dev-server")]
use tokio::sync::RwLock;
#[cfg(feature = "dev-server")]
use tower_livereload::LiveReloadLayer;

#[cfg(feature = "dev-server")]
#[derive(Clone)]
struct AppState {
    template_loader: Arc<RwLock<TemplateLoader>>,
}

/// Start the development server
#[cfg(feature = "dev-server")]
pub async fn start_dev_server(merged_path: &Path, port: u16) -> Result<()> {
    println!();
    println!("{}", "🚀 Starting RHTMX development server...".green().bold());
    println!();

    // Get paths for merged directories
    let pages_dir = merged_path.join("pages");
    let components_dir = merged_path.join("components");

    // Verify directories exist
    if !pages_dir.exists() {
        anyhow::bail!("Pages directory not found: {}", pages_dir.display());
    }

    println!("  {} Pages: {}", "📂".cyan(), pages_dir.display());
    println!("  {} Components: {}", "📦".cyan(), components_dir.display());

    // Load templates
    let mut loader = TemplateLoader::with_config(
        pages_dir.to_str().unwrap(),
        components_dir.to_str().unwrap(),
        true, // case_insensitive
    );

    match loader.load_all() {
        Ok(_) => {
            println!("  {} Loaded {} templates", "✓".green(), loader.count());

            if loader.count() > 0 {
                println!();
                println!("{}", "Routes:".cyan().bold());
                for route in loader.list_routes() {
                    println!("  {} {}", "→".green(), route);
                }
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to load templates: {}", e);
        }
    }

    // Wrap loader for thread-safe access
    let template_loader = Arc::new(RwLock::new(loader));

    // Create app state
    let state = AppState { template_loader };

    // Build router
    let mut app = Router::new()
        .route("/", get(index_handler).post(index_handler))
        .route("/*path", get(template_handler).post(template_handler))
        .with_state(state);

    // Add live reload layer
    app = app.layer(LiveReloadLayer::new());

    println!();
    println!("{}", "✅ Server ready!".green().bold());
    println!();
    println!("  {} {}", "URL:".cyan(), format!("http://localhost:{}", port).bold());
    println!("  {} Hot reload enabled - edit files and watch them update!",  "🔥".yellow());
    println!();
    println!("  {} Press Ctrl+C to stop", "ℹ".cyan());
    println!();

    // Start server
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "dev-server")]
async fn index_handler(State(state): State<AppState>) -> Response {
    render_route(&state, "/").await
}

#[cfg(feature = "dev-server")]
async fn template_handler(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let route = format!("/{}", path);
    render_route(&state, &route).await
}

#[cfg(feature = "dev-server")]
async fn render_route(state: &AppState, route: &str) -> Response {
    let loader = state.template_loader.read().await;

    // Get page template
    let page_template = match loader.get(route) {
        Some(t) => t.clone(),
        None => {
            drop(loader);
            return error_response(
                404,
                "Page Not Found",
                &format!("Route '{}' not found", route),
            );
        }
    };

    // Get layout template
    let layout_template = match loader.get_layout() {
        Some(t) => t.clone(),
        None => {
            drop(loader);
            return error_response(
                500,
                "Layout Not Found",
                "Missing _layout.rhtml in pages directory",
            );
        }
    };

    drop(loader);

    // Create renderer (simplified version for CLI dev server)
    let mut renderer = rhtmx::Renderer::new();

    // Render with layout
    match renderer.render_with_layout(&layout_template.content, &page_template.content) {
        Ok(result) => Html(result.html).into_response(),
        Err(e) => error_response(500, "Render Error", &format!("{}", e)),
    }
}

#[cfg(feature = "dev-server")]
fn error_response(status: u16, title: &str, message: &str) -> Response {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>{title}</title>
            <style>
                body {{
                    font-family: system-ui, -apple-system, sans-serif;
                    background: #f5f5f5;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    min-height: 100vh;
                    margin: 0;
                }}
                .error-container {{
                    background: white;
                    border-radius: 8px;
                    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                    padding: 2rem;
                    max-width: 500px;
                }}
                h1 {{
                    color: #e53e3e;
                    margin: 0 0 1rem;
                    font-size: 3rem;
                }}
                h2 {{
                    color: #2d3748;
                    margin: 0 0 1rem;
                    font-size: 1.5rem;
                }}
                p {{
                    color: #4a5568;
                    line-height: 1.6;
                }}
                a {{
                    display: inline-block;
                    margin-top: 1rem;
                    padding: 0.5rem 1rem;
                    background: #3182ce;
                    color: white;
                    text-decoration: none;
                    border-radius: 4px;
                }}
                a:hover {{
                    background: #2c5282;
                }}
            </style>
        </head>
        <body>
            <div class="error-container">
                <h1>{status}</h1>
                <h2>{title}</h2>
                <p>{message}</p>
                <a href="/">Go Home</a>
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

/// Fallback for when dev-server feature is disabled
#[cfg(not(feature = "dev-server"))]
pub async fn start_dev_server(_merged_path: &std::path::Path, _port: u16) -> anyhow::Result<()> {
    anyhow::bail!(
        "Dev server not available. Rebuild with --features dev-server or use default features."
    )
}
