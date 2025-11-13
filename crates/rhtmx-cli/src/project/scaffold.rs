use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Create a new RHTMX project structure
pub fn create_project(project_path: &Path, theme: Option<&str>) -> Result<()> {
    // Create root directory
    fs::create_dir_all(project_path)
        .context("Failed to create project directory")?;

    // Create directory structure
    create_directories(project_path)?;

    // Create configuration files
    create_config_files(project_path, theme)?;

    // Create initial pages
    create_initial_pages(project_path)?;

    // Create components directory (empty for now)
    fs::create_dir_all(project_path.join("components"))?;

    // Create static assets directory
    fs::create_dir_all(project_path.join("static/css"))?;
    fs::create_dir_all(project_path.join("static/js"))?;

    // Create .gitignore
    create_gitignore(project_path)?;

    // Create README
    create_readme(project_path)?;

    Ok(())
}

fn create_directories(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "pages",
        "components",
        "static",
        "src",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))
            .with_context(|| format!("Failed to create {} directory", dir))?;
    }

    Ok(())
}

fn create_config_files(project_path: &Path, theme: Option<&str>) -> Result<()> {
    // Create rhtmx.toml
    let rhtmx_config = generate_rhtmx_config(theme);
    fs::write(project_path.join("rhtmx.toml"), rhtmx_config)
        .context("Failed to create rhtmx.toml")?;

    // Create Cargo.toml
    let cargo_config = generate_cargo_toml(project_path)?;
    fs::write(project_path.join("Cargo.toml"), cargo_config)
        .context("Failed to create Cargo.toml")?;

    // Create src/main.rs
    let main_rs = generate_main_rs();
    fs::write(project_path.join("src/main.rs"), main_rs)
        .context("Failed to create src/main.rs")?;

    Ok(())
}

fn create_initial_pages(project_path: &Path) -> Result<()> {
    // Ensure directories exist
    fs::create_dir_all(project_path.join("pages"))?;
    fs::create_dir_all(project_path.join("static/css"))?;

    // Create _layout.rhtmx
    let layout_content = r#"// pages/_layout.rhtmx

pub struct LayoutSlots {
    pub content: impl Render,       // Required - auto-filled
    pub title: &str,                // Required - must provide
    pub description: Option<&str>,  // Optional
}

#[layout]
pub fn layout(slots: LayoutSlots) {
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>{slots.title}</title>
        {slots.description.map(|d| <meta name="description" content={d} />)}
        <link rel="stylesheet" href="/static/css/styles.css" />
    </head>
    <body>
        <nav>
            <a href="/">Home</a>
        </nav>
        <main>
            {slots.content}
        </main>
        <footer>
            <p>Built with RHTMX</p>
        </footer>
    </body>
    </html>
}
"#;

    fs::write(project_path.join("pages/_layout.rhtmx"), layout_content)
        .context("Failed to create _layout.rhtmx")?;

    // Create index.rhtmx
    let index_content = r#"// pages/index.rhtmx

slot! {
    title: "Welcome to RHTMX",
    description: "A Rust + HTMX framework"
}

#[webpage]
pub fn index() {
    <div class="container">
        <h1>Welcome to RHTMX</h1>
        <p>Your new RHTMX project is ready!</p>

        <div class="features">
            <h2>Features</h2>
            <ul>
                <li>⚡ File-based routing</li>
                <li>🔄 Real-time sync with HTMX</li>
                <li>🎨 Component-based architecture</li>
                <li>🔥 Hot reload in development</li>
            </ul>
        </div>

        <div class="next-steps">
            <h2>Next Steps</h2>
            <ol>
                <li>Edit <code>pages/index.rhtmx</code> to customize this page</li>
                <li>Create new pages in the <code>pages/</code> directory</li>
                <li>Add components in <code>components/</code></li>
                <li>Run <code>rhtmx dev</code> to start the dev server</li>
            </ol>
        </div>
    </div>
}
"#;

    fs::write(project_path.join("pages/index.rhtmx"), index_content)
        .context("Failed to create index.rhtmx")?;

    // Create a basic CSS file
    let css_content = r#"/* static/css/styles.css */

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}

nav {
    background: #2c3e50;
    padding: 1rem 2rem;
}

nav a {
    color: white;
    text-decoration: none;
    font-weight: 500;
}

main {
    max-width: 1200px;
    margin: 2rem auto;
    padding: 0 2rem;
}

.container {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

h1 {
    color: #2c3e50;
    margin-bottom: 1rem;
}

h2 {
    color: #34495e;
    margin: 1.5rem 0 1rem;
}

.features, .next-steps {
    margin-top: 2rem;
}

ul, ol {
    margin-left: 2rem;
}

li {
    margin: 0.5rem 0;
}

code {
    background: #f4f4f4;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: 'Courier New', monospace;
}

footer {
    text-align: center;
    padding: 2rem;
    color: #7f8c8d;
}
"#;

    fs::write(project_path.join("static/css/styles.css"), css_content)
        .context("Failed to create styles.css")?;

    Ok(())
}

fn create_gitignore(project_path: &Path) -> Result<()> {
    let gitignore = r#"# Rust
/target/
**/*.rs.bk
*.pdb

# RHTMX
/.rhtmx/          # Merged files (theme + user)
/.themes/         # Downloaded themes cache
/dist/            # Build output

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local
"#;

    fs::write(project_path.join(".gitignore"), gitignore)
        .context("Failed to create .gitignore")?;

    Ok(())
}

fn create_readme(project_path: &Path) -> Result<()> {
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("rhtmx-project");

    let readme = format!(r#"# {}

A web application built with [RHTMX](https://github.com/jeetkhinde/RHTMX) - a Rust + HTMX framework.

## Getting Started

### Development

Start the development server with hot reload:

```bash
rhtmx dev
```

Visit http://localhost:3000

### Building

Build for production:

```bash
# Server-Side Rendering (default)
rhtmx build --mode=ssr

# Static Site Generation
rhtmx build --mode=ssg

# Incremental Static Regeneration
rhtmx build --mode=isr
```

## Project Structure

```
{}
├── pages/              # File-based routes
│   ├── _layout.rhtmx  # Root layout
│   └── index.rhtmx    # Home page
├── components/         # Reusable components
├── static/            # Static assets (CSS, JS, images)
├── src/               # Rust source code
├── rhtmx.toml        # RHTMX configuration
└── Cargo.toml        # Rust dependencies
```

## Features

- ⚡ **File-based routing** - Create pages by adding files to `pages/`
- 🔄 **Real-time sync** - HTMX integration with server-sent events
- 🎨 **Component-based** - Reusable UI components
- 🔥 **Hot reload** - Instant updates during development
- ✅ **Unified validation** - Single source of truth for client & server
- 🎭 **Theme support** - Use and create themes

## Learn More

- [RHTMX Documentation](https://github.com/jeetkhinde/RHTMX)
- [File-based Routing Guide](https://github.com/jeetkhinde/RHTMX/tree/main/crates/rhtmx-router)

## License

MIT
"#, project_name, project_name);

    fs::write(project_path.join("README.md"), readme)
        .context("Failed to create README.md")?;

    Ok(())
}

fn generate_rhtmx_config(theme: Option<&str>) -> String {
    let theme_section = if let Some(theme_name) = theme {
        format!(r#"
[theme]
name = "{}"
# Uncomment and set the source for your theme
# [theme.source]
# type = "local"  # or "git"
# path = "../{}"
"#, theme_name, theme_name)
    } else {
        String::new()
    };

    format!(r#"# RHTMX Configuration File

[project]
name = "rhtmx-app"
version = "0.1.0"
{}
[server]
port = 3000
host = "127.0.0.1"
workers = 4

[routing]
pages_dir = "pages"
components_dir = "components"
case_insensitive = true
trailing_slash = false

[build]
output_dir = "dist"
static_dir = "static"
minify_html = false
minify_css = false

[dev]
hot_reload = true
port = 3000
open_browser = false
watch_paths = ["pages", "components", "static"]
"#, theme_section)
}

fn generate_cargo_toml(project_path: &Path) -> Result<String> {
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("rhtmx-app");

    Ok(format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
# RHTMX Framework
rhtmx = {{ git = "https://github.com/jeetkhinde/RHTMX" }}
rhtmx-router = {{ git = "https://github.com/jeetkhinde/RHTMX" }}
rhtmx-server = {{ git = "https://github.com/jeetkhinde/RHTMX" }}

# Web Framework
axum = "0.7"
tokio = {{ version = "1.0", features = ["full"] }}
tower = "0.4"
tower-http = {{ version = "0.5", features = ["fs"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Utilities
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
"#, project_name))
}

fn generate_main_rs() -> String {
    r#"use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting RHTMX server...");

    // Build router
    let app = Router::new()
        .route("/", get(|| async { "Hello from RHTMX!" }));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✅ Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
"#.to_string()
}
