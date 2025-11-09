# Maud! Macro - Complete Guide

Learn how to use the `maud!` macro in RHTMX for compact, programmatic HTML generation with Lisp-like syntax.

## Overview

The `maud!` macro provides seamless integration with the [Maud](https://maud.rs/) templating engine, allowing you to write HTML with a compact Lisp-like syntax. It's an excellent alternative to the `html!` macro when you prefer:

- **More compact syntax** - less visual noise, more signal
- **Programmatic generation** - easier to loop and conditionally render
- **Functional style** - natural Rust integration

## Quick Comparison

**html! (JSX-style):**
```rust
let name = "Alice";
html! {
    <div class="card" id="user-1">
        <h3 class="title">{name}</h3>
        <p class="description">"User profile"</p>
    </div>
}
```

**maud! (Lisp-style):**
```rust
let name = "Alice";
maud! {
    div.card#user-1 {
        h3.title { (name) }
        p.description { "User profile" }
    }
}
```

Both produce identical HTML!

## Basic Syntax

### Elements and Content

```rust
maud! {
    div { "Hello" }                    // <div>Hello</div>
    p { "Paragraph text" }             // <p>Paragraph text</p>
    span { "Inline" }                  // <span>Inline</span>
}
```

### Classes and IDs

Classes use dot notation, IDs use hash notation:

```rust
maud! {
    div.container { }                  // <div class="container"></div>
    div.row.active { }                 // <div class="row active"></div>
    h1#main-title { }                  // <h1 id="main-title"></h1>
    section.hero.dark#hero { }         // <section class="hero dark" id="hero"></section>
}
```

### Attributes

Use square bracket syntax:

```rust
maud! {
    a[href="/about"] { "About" }
    input[type="text"][placeholder="Name"]
    img[src="/logo.png"][alt="Logo"]
    button[onclick="doSomething()"][data-id="42"] { "Click me" }
}
```

### Combining Classes, IDs, and Attributes

```rust
maud! {
    div.card.featured#main[data-role="featured"] {
        h2 { "Featured Item" }
    }
}

// Produces:
// <div class="card featured" id="main" data-role="featured">
//     <h2>Featured Item</h2>
// </div>
```

## Expressions and Interpolation

Use parentheses to interpolate Rust expressions:

```rust
let name = "Alice";
let age = 30;
let items = vec!["Item 1", "Item 2"];

maud! {
    h1 { (name) }                      // Interpolate variable
    p { (age * 2) }                    // Interpolate expression
    span { (items.len()) }             // Call methods
    p { (format!("Age: {}", age)) }    // Format strings
}
```

### String Interpolation vs Raw Strings

```rust
maud! {
    p { "Plain text is safe" }         // Raw text (no HTML)
    p { (var) }                        // Interpolated (user input - safe)
    p { (html_var) }                   // Still safe, Maud escapes by default
}
```

## Control Flow

### Conditionals with @if

```rust
let is_admin = true;

maud! {
    @if is_admin {
        div.admin-panel { "Admin controls" }
    }
}
```

### If-Else

```rust
let user_role = "guest";

maud! {
    @if user_role == "admin" {
        p { "Admin area" }
    } @else if user_role == "moderator" {
        p { "Moderator area" }
    } @else {
        p { "Guest area" }
    }
}
```

### Loops with @for

```rust
let items = vec!["Apple", "Banana", "Cherry"];

maud! {
    ul {
        @for item in &items {
            li { (item) }
        }
    }
}

// Produces:
// <ul>
//     <li>Apple</li>
//     <li>Banana</li>
//     <li>Cherry</li>
// </ul>
```

### Loops with Index

```rust
let users = vec!["Alice", "Bob", "Carol"];

maud! {
    @for (i, user) in users.iter().enumerate() {
        div {
            (i + 1) ": " (user)
        }
    }
}
```

### Complex Loops

```rust
let groups = vec![
    ("Admins", vec!["Alice", "Bob"]),
    ("Users", vec!["Carol", "Dave"]),
];

maud! {
    @for (group_name, members) in groups {
        h3 { (group_name) }
        ul {
            @for member in members {
                li { (member) }
            }
        }
    }
}
```

## HTML Attributes

### Common Patterns

```rust
let checked = true;
let href = "/home";

maud! {
    input[type="checkbox"][checked=(checked)]
    a[href=(href)] { "Home" }
    button[disabled=(true)] { "Disabled" }
    img[src="/logo.png"][alt="Logo"][class="logo"]
}
```

### Data Attributes

```rust
let user_id = 42;

maud! {
    div[data-id=(user_id)][data-type="user"] {
        "User data"
    }
}

// Produces:
// <div data-id="42" data-type="user">User data</div>
```

### Dynamic Attributes

```rust
let attrs = vec![("data-x", "1"), ("data-y", "2")];

maud! {
    div {
        @for (key, val) in attrs {
            (format!(r#" {}="{}""#, key, val))
        }
    }
}
```

## Working with Response Builders

### With Ok() Response

```rust
use rhtmx::{get, Ok, OkResponse, maud};

get!()
fn dashboard() -> OkResponse {
    let user_name = "Alice";
    let message = "Welcome!";

    let html = maud! {
        div.dashboard {
            h1 { (user_name) }
            p { (message) }
        }
    };

    Ok().html(html)
}
```

### In Layouts

```rust
use rhtmx::{get, Ok, OkResponse, maud, layouts};

get!()
fn users() -> OkResponse {
    let users_list = vec!["Alice", "Bob", "Carol"];

    let content = maud! {
        ul.users {
            @for user in users_list {
                li { (user) }
            }
        }
    };

    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Users")
    ))
}
```

### With Query/Form Parameters

```rust
use rhtmx::{post, Ok, OkResponse, FormContext, maud};

post!()
fn create_user(form: FormContext) -> OkResponse {
    let name = form.get("name").unwrap_or("Unknown");

    let html = maud! {
        div.success {
            p { "Created user: " (name) }
        }
    };

    Ok().html(html)
}
```

## Advanced Patterns

### Reusable Components

```rust
fn user_card(name: &str, email: &str) -> String {
    maud! {
        div.card.user {
            h3 { (name) }
            p { (email) }
        }
    }.into_string()  // Convert Html to String if needed
}

// Use in another template:
let users = vec![
    ("Alice", "alice@example.com"),
    ("Bob", "bob@example.com"),
];

maud! {
    div.users-grid {
        @for (name, email) in users {
            (user_card(name, email))
        }
    }
}
```

### Conditional Classes

```rust
let is_active = true;
let error = Some("Invalid input");

maud! {
    div.form-group {
        input[class=(
            if is_active { "input active" } else { "input" }
        )]
    }

    @if let Some(err) = error {
        div.error-message { (err) }
    }
}
```

### Nested Templates

```rust
let header = maud! {
    header {
        h1 { "My Site" }
    }
};

let footer = maud! {
    footer {
        p { "Copyright 2024" }
    }
};

let page = maud! {
    html {
        head { title { "Page" } }
        body {
            (header)
            main { "Content" }
            (footer)
        }
    }
};
```

### HTML in Variables

```rust
let message = "<script>alert('xss')</script>";

maud! {
    div {
        // Safe - Maud escapes by default
        p { (message) }

        // Produces: <script>alert('xss')</script> (escaped)
    }
}
```

## Performance Considerations

### Compile-Time vs Runtime

The `maud!` macro is **compile-time only** - there's no runtime overhead:

```rust
// This compiles down to efficient string building
maud! {
    div.card {
        h3 { "Title" }
        p { "Content" }
    }
}

// Is roughly equivalent to:
// Html(r#"<div class="card"><h3>Title</h3><p>Content</p></div>"#.to_string())
```

### Memory Efficiency

Maud generates efficient string building code:

```rust
// Bad - creates multiple intermediate strings
let html = format!("{}{}{}", html1, html2, html3);

// Good - Maud builds once
maud! {
    div { (html1) (html2) (html3) }
}
```

## Comparison with html! Macro

| Feature | html! | maud! |
|---------|-------|-------|
| **Syntax Style** | HTML-like (JSX) | Lisp-like |
| **Class Syntax** | `class="..."` | `.classname` |
| **ID Syntax** | `id="..."` | `#idname` |
| **Interpolation** | `{variable}` | `(variable)` |
| **Familiarity** | Web developers | Scheme/Lisp users |
| **Markup Density** | Verbose | Compact |
| **Learning Curve** | Shallow | Moderate |
| **Best For** | Markup-heavy | Programmatic generation |

### When to Use html!

- Coming from web development background
- Templates are markup-heavy
- Prefer familiar HTML syntax
- Working with HTML snippets

### When to Use maud!

- Prefer compact syntax
- Heavy use of loops and conditionals
- Programmatic HTML generation
- Want minimal visual noise

## Common Patterns

### Blog Post Card

```rust
#[derive(Clone)]
struct Post {
    id: i32,
    title: String,
    excerpt: String,
    author: String,
    date: String,
}

let post = Post {
    id: 1,
    title: "Getting Started".to_string(),
    excerpt: "Learn the basics...".to_string(),
    author: "Alice".to_string(),
    date: "2024-01-15".to_string(),
};

maud! {
    article.post-card[data-id=(post.id)] {
        h2 { (post.title) }
        p.excerpt { (post.excerpt) }
        footer.meta {
            span.author { "By " (post.author) }
            time[datetime=(post.date)] { (post.date) }
        }
    }
}
```

### Form with Validation

```rust
#[derive(Clone)]
struct FormField {
    name: String,
    label: String,
    value: String,
    error: Option<String>,
}

let field = FormField {
    name: "email".to_string(),
    label: "Email".to_string(),
    value: "user@example.com".to_string(),
    error: None,
};

maud! {
    div.form-group[class=(
        if field.error.is_some() { "has-error" } else { "" }
    )] {
        label[for=(field.name)] { (field.label) }
        input[
            type="email"
            name=(field.name)
            value=(field.value)
            class=(if field.error.is_some() { "input-error" } else { "" })
        ]
        @if let Some(err) = &field.error {
            span.error-text { (err) }
        }
    }
}
```

### Navigation Menu

```rust
let menu_items = vec![
    ("Home", "/", true),
    ("About", "/about", false),
    ("Contact", "/contact", false),
];

maud! {
    nav.navbar {
        ul.nav-menu {
            @for (label, url, is_active) in menu_items {
                li[class=(if is_active { "active" } else { "" })] {
                    a[href=(url)] { (label) }
                }
            }
        }
    }
}
```

## Tips and Tricks

### Use Maud for Complex Templates

```rust
// Good - Maud shines with nested loops and conditionals
maud! {
    @for group in groups {
        section.group {
            h2 { (group.name) }
            @if group.items.is_empty() {
                p { "No items" }
            } @else {
                ul {
                    @for item in &group.items {
                        li { (item) }
                    }
                }
            }
        }
    }
}
```

### Mix with html! if Needed

```rust
// Start with maud! for overall structure
let content = maud! {
    div.container {
        @for item in items {
            // Can mix with other templating as needed
            (render_complex_item(item))
        }
    }
};
```

### Use Variables for Common Elements

```rust
let btn_classes = "btn btn-primary";
let input_classes = "form-input";

maud! {
    form {
        input[class=(input_classes)][type="text"]
        button[class=(btn_classes)] { "Submit" }
    }
}
```

## Troubleshooting

### Syntax Errors

**Error:** `Unexpected token`

**Solution:** Check spacing and braces:
```rust
// Wrong
div.class{ content }

// Right
div.class { content }
```

**Error:** Interpolation not working

**Solution:** Use parentheses, not braces:
```rust
// Wrong
div { {variable} }

// Right
div { (variable) }
```

### HTML Escaping

Maud escapes HTML by default for safety:

```rust
let user_input = "<script>alert('xss')</script>";

maud! {
    div { (user_input) }
}

// Produces safe escaped output:
// <div>&lt;script&gt;alert('xss')&lt;/script&gt;</div>
```

## See Also

- [Maud Official Docs](https://maud.rs/)
- [html! Macro Guide](../html!/HTML_MACRO_GUIDE.md)
- [FEATURES.md](../FEATURES.md) - All RHTMX features
- [QUICKSTART.md](../QUICKSTART.md) - Getting started guide
