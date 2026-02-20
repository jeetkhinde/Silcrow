Question:Show me an example of Axum, with Maud libraries.
Gemini Answer:
Here is a clean, modern example of using the Axum web framework alongside the Maud templating engine.

### 1. Dependencies (`Cargo.toml`)

The critical step here is enabling the `axum` feature flag on the `maud` crate. This tells Maud to implement the necessary Axum traits for its types.

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
maud = { version = "0.26", features = ["axum"] }

```

### 2. Application Code (`main.rs`)

Here is a complete, working example using the latest Axum `0.7` syntax:

```rust
use axum::{routing::get, Router};
use maud::{html, Markup, DOCTYPE};

#[tokio::main]
async fn main() {
    // Build the Axum router and attach the route
    let app = Router::new()
        .route("/", get(index));

    // Bind the listener to a local port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on http://127.0.0.1:3000");
    
    // Start the server
    axum::serve(listener, app).await.unwrap();
}

/// Handler that returns Maud's Markup directly
async fn index() -> Markup {
    let framework_name = "Axum";
    
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "Rust HTML Generation" }
            }
            body {
                h1 { "Hello from " (framework_name) " and Maud!" }
                p { "This HTML is verified and compiled at build time." }
            }
        }
    }
}

```

### How the Integration Works Under the Hood

When designing a domain-specific language for HTML generation, one of the cleverest parts of Maud's architecture is how seamlessly it hooks into existing web frameworks.

By enabling the `axum` feature, Maud automatically implements Axum's `IntoResponse` trait for its internal `PreEscaped<String>` type (which is aliased as `Markup`).

* **Zero Boilerplate:** The `html!` macro expands directly into this `Markup` type at compile time.
* **Direct Returns:** Because `Markup` implements `IntoResponse`, Axum automatically knows how to unpack the struct, attach the correct `Content-Type: text/html; charset=utf-8` HTTP headers, and stream the resulting bytes to the client. There is no need for manual string conversion or explicit header management in your route handlers.

Would you like me to break down exactly how to implement Axum's `IntoResponse` trait for your own custom HTML types?

Question: how you combine small partials to make big page? just need small example for understanding
like Dashboard example or header footer, content examples
header footer stays same and internal content changes.

Gemini Answer:
In Maud, partials are simply standard Rust functions that return a `Markup` struct. Because of this, you don't need special template syntax to include them—you just call the function directly inside your `html!` block or pass `Markup` around as a variable.

Here is a clean example of how to build a consistent layout where the header and footer stay the same, but the internal content changes.

### 1. Define the Partials

First, create standard functions for your reusable chunks.

```rust
use maud::{html, Markup, DOCTYPE};

// A reusable header that takes a title
fn header(page_title: &str) -> Markup {
    html! {
        header {
            h1 { (page_title) }
            nav {
                a href="/" { "Home" }
                " | "
                a href="/dashboard" { "Dashboard" }
                " | "
                a href="/settings" { "Settings" }
            }
        }
    }
}

// A reusable, static footer
fn footer() -> Markup {
    html! {
        footer {
            hr;
            p { "© 2026 My App. All rights reserved." }
        }
    }
}

```

### 2. Create the Base Layout Wrapper

Next, create a function that represents your entire page shell. It will accept the dynamic `inner_content` as an argument of type `Markup`.

```rust
fn base_layout(page_title: &str, inner_content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { (page_title) " - My App" }
            }
            body {
                // 1. Call the header partial
                (header(page_title))

                // 2. Inject the dynamic page content here
                main {
                    (inner_content)
                }

                // 3. Call the footer partial
                (footer())
            }
        }
    }
}

```

### 3. Build Specific Pages

Now, you can build any number of pages by defining just their specific internal content and passing it into the `base_layout` function.

```rust
// The Dashboard Page
pub fn dashboard_page() -> Markup {
    let content = html! {
        h2 { "Welcome back!" }
        div class="stats" {
            p { "Active Servers: 4" }
            p { "System Health: Optimal" }
        }
    };

    // Wrap the content in the base layout
    base_layout("Dashboard", content)
}

// The Settings Page
pub fn settings_page() -> Markup {
    let content = html! {
        h2 { "User Settings" }
        form method="POST" action="/update" {
            label for="username" { "Username:" }
            input type="text" id="username" name="username";
            button type="submit" { "Save" }
        }
    };

    base_layout("Settings", content)
}

```

### Why This Design Pattern is Powerful

When designing an HTML macro or DSL, treating partials as plain Rust functions is a massive advantage over traditional string-based templates (like Jinja or Handlebars).

It means you get full Rust compiler support for your HTML components. You can pass structs, enums, or closures into your layout functions, enforce type safety on the arguments (like requiring a valid `User` struct to render a profile header), and easily test individual components in isolation.

Would you like an example of how to pass a struct (like a `User` or a database record) into one of these partials to render a list of items?

Question: excellent. How do we write routers in axum? how do we group them or use different layouts. or maybe handles error some one fn not able to produce required html?
Gemini Answer:
Gemini said
Axum is fantastic for building out modular, robust web servers, especially because it leans heavily into Rust's type system.

Here is how you can tackle grouping routes, managing different layouts, and handling errors gracefully by rendering Maud HTML when things go wrong.

1. Grouping Routes (Nesting)

Axum allows you to group related routes using the nest method. This is perfect for organizing your application, such as putting all admin-related routes under an /admin prefix.

Rust
use axum::{routing::get, Router};
use maud::{html, Markup};

pub fn build_router() -> Router {
    // 1. Group Admin Routes
    let admin_routes = Router::new()
        .route("/dashboard", get(admin_dashboard))
        .route("/settings", get(admin_settings));

    // 2. Main Router
    Router::new()
        .route("/", get(public_home))
        // 3. Nest the admin routes. 
        // `admin_dashboard` is now accessible at "/admin/dashboard"
        .nest("/admin", admin_routes)
}

async fn public_home() -> Markup { /* ... */ html!{ p { "Home" } } }
async fn admin_dashboard() -> Markup { /* ... */ html!{ p { "Dashboard" } } }
async fn admin_settings() -> Markup { /* ... */ html!{ p { "Settings" } } }
2. Using Different Layouts

Because Maud templates are just plain Rust functions, you don't configure layouts at the Axum router level. You simply call the specific layout function you need directly inside your handler.

This keeps your code incredibly flexible. You can have a public_layout for your landing pages and an admin_layout for your dashboard.

Rust
// A public-facing route
async fn public_home() -> Markup {
    let content = html! { p { "Welcome to our site!" } };
    
    // Wrap with the public shell
    public_layout("Home", content) 
}

// An admin-facing route
async fn admin_dashboard() -> Markup {
    let content = html! { p { "Server status: Online" } };
    
    // Wrap with the admin shell
    admin_layout("Dashboard", content) 
}
3. Handling Errors with HTML Pages

In Axum, every handler must return a type that implements the IntoResponse trait. If a function might fail (like a database query dropping), you should return a Result<Markup, AppError>.

You can implement IntoResponse on your custom AppError enum so that Axum automatically knows how to convert the error into an HTTP status code and a beautiful Maud error page.

Rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// 1. Define your custom application errors
pub enum AppError {
    NotFound,
    InternalError(String),
}

// 2. Tell Axum how to render this error into an HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Map the error variant to a specific status code and message
        let (status, error_message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Page not found".to_string()),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        // Render a Maud HTML error page right here
        let error_page = html! {
            h1 { "Error " (status.as_u16()) }
            p { (error_message) }
            a href="/" { "Go back home" }
        };

        // A tuple of (StatusCode, T: IntoResponse) automatically implements IntoResponse!
        (status, error_page).into_response()
    }
}

// 3. A handler that can fail
async fn fragile_handler() -> Result<Markup, AppError> {
    let operation_successful = false; // Simulate a failure

    if operation_successful {
        // Returns 200 OK with this HTML
        Ok(html! { p { "The operation worked!" } })
    } else {
        // Returns a 500 status code with the HTML defined in our AppError implementation
        Err(AppError::InternalError("Failed to connect to the database.".into()))
    }
}
By centralizing your error rendering inside the IntoResponse implementation, your actual route handlers stay clean and focused strictly on business logic, automatically short-circuiting to standard, styled error pages whenever the ? operator encounters an issue.

Handling Exceptions Like a Pro in Axum
If you want to dive deeper into custom error management, this walkthrough provides a great visual breakdown of mapping different application states to specific HTTP responses.

