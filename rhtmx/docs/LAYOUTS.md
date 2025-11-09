# RHTMX Layout System

Simple, type-safe layouts using plain Rust functions with a slot-based pattern.

## Overview

RHTMX provides a clean layout system that:
- ✅ Uses regular Rust functions (no complex macros)
- ✅ Type-safe slot contracts via structs
- ✅ Builder pattern for easy customization
- ✅ Supports root layouts and page-level layouts
- ✅ Works seamlessly with file-based routing and #[get], #[post] handlers
- ✅ Zero runtime overhead - pure compile-time

## Quick Start

### 1. Basic Usage

```rust
use rhtmx::{html, Html, layouts, Ok, OkResponse};

#[get]
fn index() -> OkResponse {
    let content = html! {
        <div class="hero">
            <h1>"Welcome to RHTMX"</h1>
            <p>"Build fast web apps with Rust + HTMX"</p>
        </div>
    };

    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Home Page")
    ))
}
```

### 2. With Custom Slots

```rust
#[get]
fn about() -> OkResponse {
    let content = html! {
        <article>
            <h1>"About Us"</h1>
            <p>"Learn more about our company."</p>
        </article>
    };

    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("About - My App")
            .description("Learn about our company and mission")
    ))
}
```

### 3. HTMX Partials (No Layout)

For HTMX partial updates, return HTML without a layout:

```rust
#[get]
fn user_list() -> OkResponse {
    let users = get_users();

    let html = html! {
        <div id="users">
            <div r-for="user in users">
                <p>{user.name}</p>
            </div>
        </div>
    };

    Ok().html(html)  // No layout - HTMX will swap this into the page
}
```

## Available Layouts

### Root Layout (`layouts::root`)

The main layout for most pages.

**Slots:**
```rust
pub struct Slots {
    pub title: String,                 // Required - page title
    pub description: Option<String>,   // Optional - meta description
    pub header: Option<Html>,          // Optional - custom header (uses default nav if not provided)
    pub footer: Option<Html>,          // Optional - custom footer
    pub head_extra: Option<Html>,      // Optional - additional <head> content
}
```

**Features:**
- Full HTML document with `<!DOCTYPE html>`
- Includes HTMX script (unpkg CDN)
- Responsive viewport meta tag
- Default navigation and footer
- Customizable via slots

**Example:**
```rust
#[get]
fn homepage() -> OkResponse {
    let content = html! {
        <div class="container">
            <h1>"Welcome"</h1>
        </div>
    };

    let custom_header = html! {
        <nav class="navbar">
            <a href="/">"Home"</a>
            <a href="/products">"Products"</a>
            <a href="/about">"About"</a>
        </nav>
    };

    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Homepage")
            .description("Welcome to our website")
            .header(custom_header)
    ))
}
```

### Admin Layout (`layouts::admin`)

Two-column layout with sidebar for admin pages.

**Slots:**
```rust
pub struct Slots {
    pub title: String,                // Required - page title
    pub sidebar: Option<Html>,        // Optional - custom sidebar (uses default if not provided)
    pub breadcrumbs: Option<Html>,    // Optional - breadcrumb navigation
}
```

**Features:**
- Two-column flexbox layout
- Dark sidebar with navigation
- Breadcrumbs support
- Includes basic admin styling

**Example:**
```rust
#[get]
fn admin_dashboard() -> OkResponse {
    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
            <div class="stats">
                <div class="stat-card">
                    <h3>"Users"</h3>
                    <p>"1,234"</p>
                </div>
            </div>
        </div>
    };

    let breadcrumbs = html! {
        <nav>
            <a href="/admin">"Admin"</a>
            " / "
            <span>"Dashboard"</span>
        </nav>
    };

    Ok().html(layouts::admin::layout(
        content,
        layouts::admin::Slots::new("Dashboard")
            .breadcrumbs(breadcrumbs)
    ))
}
```

## Builder Pattern

All slot structs support a builder pattern:

```rust
Slots::new("Page Title")
    .description("Meta description")
    .header(custom_header_html)
    .footer(custom_footer_html)
    .head_extra(extra_meta_tags)
```

## Creating Custom Layouts

You can create your own layouts following the same pattern:

```rust
// src/layouts/blog.rs

use crate::Html;

#[derive(Clone)]
pub struct Slots {
    pub title: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
}

impl Slots {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            author: None,
            published_date: None,
        }
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn published_date(mut self, date: impl Into<String>) -> Self {
        self.published_date = Some(date.into());
        self
    }
}

pub fn layout(content: Html, slots: Slots) -> Html {
    let author_html = if let Some(author) = slots.author {
        format!(r#"<meta name="author" content="{}" />"#, author)
    } else {
        String::new()
    };

    Html(format!(r#"<!DOCTYPE html>
<html>
    <head>
        <title>{}</title>
        {}
    </head>
    <body>
        <article>
            <header>
                <h1>{}</h1>
                {}
            </header>
            <main>{}</main>
        </article>
    </body>
</html>"#,
        slots.title,
        author_html,
        slots.title,
        slots.published_date.map(|d| format!("<time>{}</time>", d)).unwrap_or_default(),
        content.0
    ))
}
```

Then register it in `src/layouts/mod.rs`:

```rust
pub mod root;
pub mod admin;
pub mod blog;  // Add your custom layout

pub use blog::layout as blog_layout;
```

## File Structure

```
rhtmx/
└── src/
    └── layouts/
        ├── mod.rs       # Re-exports all layouts
        ├── root.rs      # Root layout (default for most pages)
        ├── admin.rs     # Admin layout (sidebar + content)
        └── blog.rs      # Your custom layouts...
```

## Best Practices

### 1. Use Root Layout for Most Pages

The root layout is suitable for 90% of pages:

```rust
Ok().html(layouts::root::layout(content, Slots::new("Page Title")))
```

### 2. No Layout for HTMX Partials

When returning HTML fragments for HTMX swaps, skip the layout:

```rust
#[get]
fn load_more_items() -> OkResponse {
    let html = html! {
        <div r-for="item in items">
            <p>{item.name}</p>
        </div>
    };

    Ok().html(html)  // No layout!
}
```

### 3. Use Admin Layout for Admin Sections

Keep admin pages visually consistent:

```rust
#[get]
fn admin_users() -> OkResponse {
    Ok().html(layouts::admin::layout(content, Slots::new("Users")))
}
```

### 4. Custom Layouts for Special Pages

Create custom layouts for landing pages, blogs, documentation, etc.

### 5. Builder Pattern for Readability

```rust
// ✓ Good: Clear and readable
Slots::new("About")
    .description("Learn about us")
    .header(custom_nav)

// ✗ Avoid: Constructing struct directly
Slots {
    title: "About".into(),
    description: Some("Learn about us".into()),
    header: Some(custom_nav),
    footer: None,
    head_extra: None,
}
```

## Advanced Patterns

### Nested Layouts

You can nest layouts by passing the output of one layout as content to another:

```rust
let inner_content = html! { <p>"Inner content"</p> };
let section_layout = layouts::section::layout(inner_content, section_slots);
let final_page = layouts::root::layout(section_layout, root_slots);
```

### Conditional Layouts

Choose layouts based on user state:

```rust
fn render_page(content: Html, user: Option<User>) -> Html {
    match user {
        Some(user) if user.is_admin => {
            layouts::admin::layout(content, Slots::new("Dashboard"))
        }
        _ => {
            layouts::root::layout(content, Slots::new("Public"))
        }
    }
}
```

### Layout Helpers

Create helper functions for common layout configurations:

```rust
fn with_auth_layout(content: Html, title: impl Into<String>) -> Html {
    layouts::root::layout(
        content,
        Slots::new(title)
            .header(auth_header())
            .footer(auth_footer())
    )
}
```

## Performance

- **Zero Runtime Overhead**: Layouts are just functions that concatenate strings at compile time
- **No Template Engine**: Everything is pure Rust code
- **Type Safe**: Compiler checks all slot usage
- **Efficient**: String building is optimized by the Rust compiler

## Comparison with Old RHTML

| Feature | Old RHTML | RHTMX |
|---------|-----------|-------|
| Layout files | `_layout.rhtml` runtime templates | Plain Rust functions |
| Slot definition | `LayoutSlots` struct (parsed at runtime) | Rust structs (compile-time) |
| Slot passing | `slot!` macro | Builder pattern |
| Discovery | File-based scanning | Explicit imports |
| Performance | Runtime template parsing | Compile-time only |
| Type safety | Runtime validation | Compile-time validation |

## Examples

See `examples/layouts_demo.rs` for comprehensive examples showing:
- Basic root layout usage
- Custom slot configuration
- Admin layout with breadcrumbs
- HTMX partials without layouts
- Real handler patterns

Run the demo:
```bash
cargo run --example layouts_demo
```

## See Also

- [FEATURES.md](./FEATURES.md) - All RHTMX features
- [QUICKSTART.md](./QUICKSTART.md) - Getting started guide
- [examples/](./examples/) - Working code examples
