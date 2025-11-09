# Maud! Macro - Quick Reference

Fast lookup for maud! syntax and common patterns.

## Basic Syntax Cheat Sheet

```rust
use rhtmx::maud;

// Element
div { "content" }

// Classes
div.container { }
div.row.active { }

// IDs
h1#main-title { }

// Combined
section.hero.dark#hero { }

// Attributes
a[href="/about"] { "Link" }
input[type="text"][placeholder="Name"]

// Interpolate expressions
let name = "Alice";
p { (name) }
p { (2 + 2) }

// Conditionals
@if condition { "yes" }
@if x > 5 { "big" } @else { "small" }

// Loops
@for item in items { (item) }

// Text (raw, safe)
p { "This is safe" }

// HTML-safe interpolation
p { (user_input) }  // Auto-escaped
```

## Quick Examples

### Basic Page Structure

```rust
get!()
fn home() -> OkResponse {
    let title = "Home";

    let html = maud! {
        div.container {
            h1 { (title) }
            p { "Welcome!" }
        }
    };

    Ok().html(html)
}
```

### List Rendering

```rust
let items = vec!["Apple", "Banana", "Cherry"];

maud! {
    ul {
        @for item in &items {
            li { (item) }
        }
    }
}
```

### Form with Validation

```rust
let errors = vec!["Email required", "Password too short"];

maud! {
    form {
        input[type="email"][placeholder="Email"]

        @if !errors.is_empty() {
            div.errors {
                @for error in &errors {
                    p.error { (error) }
                }
            }
        }

        button { "Submit" }
    }
}
```

### Card Component

```rust
fn card(title: &str, content: &str) -> String {
    maud! {
        div.card {
            h3 { (title) }
            p { (content) }
        }
    }.into_string()
}
```

### Conditional Rendering

```rust
let is_admin = true;

maud! {
    @if is_admin {
        div.admin-panel { "Admin controls" }
    } @else {
        div.user-panel { "User area" }
    }
}
```

### Nested Loops

```rust
let groups = vec![
    ("Team A", vec!["Alice", "Bob"]),
    ("Team B", vec!["Carol", "Dave"]),
];

maud! {
    @for (group, members) in &groups {
        section.group {
            h2 { (group) }
            ul {
                @for member in members {
                    li { (member) }
                }
            }
        }
    }
}
```

### With Layouts

```rust
get!()
fn users_page() -> OkResponse {
    let users = vec!["Alice", "Bob"];

    let content = maud! {
        div.users {
            @for user in users {
                div.user-item { (user) }
            }
        }
    };

    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Users")
    ))
}
```

## Syntax Comparison

| Task | html! | maud! |
|------|-------|-------|
| **Element** | `<div>` | `div {` |
| **Class** | `class="btn"` | `.btn` |
| **ID** | `id="main"` | `#main` |
| **Attribute** | `attr="val"` | `[attr="val"]` |
| **Interpolate** | `{var}` | `(var)` |
| **If** | `r-if` directive | `@if` |
| **Loop** | `r-for` directive | `@for` |
| **Text** | `"text"` | `"text"` |

## Common Mistakes

| Mistake | Error | Fix |
|---------|-------|-----|
| Using `{var}` | Parse error | Use `(var)` |
| Omitting space before `{` | Parse error | `div { }` not `div{` |
| Wrong quote style | Parse error | Use `[attr="val"]` |
| Nested class syntax | Doesn't work | Use combined: `div.a.b` |

## Attributes Reference

```rust
// Data attributes
div[data-id="42"][data-type="user"] { }

// Standard HTML
input[type="text"][name="username"][value="alice"]
button[onclick="func()"][disabled=(true)]
a[href="/page"][target="_blank"]

// Dynamic
let disabled = true;
button[disabled=(disabled)] { "Click" }

// Conditional
div[class=(if active { "active" } else { "" })] { }
```

## Control Flow

```rust
// If
@if condition { "yes" }

// If-Else
@if x > 5 { "big" } @else { "small" }

// If-Else-If
@if x > 10 { "huge" } @else if x > 5 { "big" } @else { "small" }

// For loop
@for item in items { (item) }

// For with destructuring
@for (i, item) in items.iter().enumerate() { (i) ": " (item) }

// For with range
@for i in 0..5 { (i) }

// Match (limited support)
div { (match status {
    "ok" => "Success",
    _ => "Error"
}) }
```

## Response Integration

```rust
use rhtmx::{get, Ok, OkResponse, maud, layouts};

// Direct HTML
get!()
fn page() -> OkResponse {
    Ok().html(maud! { div { "Content" } })
}

// With Layout
get!()
fn page2() -> OkResponse {
    let content = maud! { div { "Content" } };
    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Page")
    ))
}

// With Form Data
post!()
fn submit(form: FormContext) -> OkResponse {
    let name = form.get("name").unwrap_or("Guest");
    Ok().html(maud! { p { "Hello " (name) } })
}
```

## Performance Tips

- ✅ Use `@for` loops - efficient
- ✅ Compose with functions - DRY
- ✅ Interpolate expressions directly - `(value)`
- ✅ Let compiler optimize - it's smart
- ❌ Avoid creating intermediate HTML strings
- ❌ Don't build HTML with format!()

## See Also

- [Full Maud! Guide](./MAUD_MACRO_GUIDE.md)
- [html! Quick Reference](../html!/HTML_MACRO_QUICK_REF.md)
- [Maud Official Docs](https://maud.rs/)
