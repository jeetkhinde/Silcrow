# Maud! Macro - Summary & Comparison

Overview of the Maud macro with detailed comparison to the html! macro.

## What is Maud?

Maud is a compile-time HTML templating library that uses a Lisp-like syntax for writing HTML. It's a mature, well-tested alternative to JSX-style templating with a focus on:

- **Compact syntax** - minimal visual noise
- **Type safety** - compile-time checking
- **Zero runtime cost** - everything happens at compile time
- **Programmatic style** - natural for loops and conditionals

## Feature Status

| Feature | html! | maud! | Status |
|---------|-------|-------|--------|
| **Basic HTML** | ✅ | ✅ | Both fully supported |
| **Classes/IDs** | `class=""` `id=""` | `.class` `#id` | Different syntax |
| **Attributes** | `attr=""` | `[attr=""]` | Different syntax |
| **Interpolation** | `{expr}` | `(expr)` | Different syntax |
| **Conditionals** | `r-if` | `@if` | Both supported |
| **Loops** | `r-for` | `@for` | Both supported |
| **Type Safety** | ✅ | ✅ | Both type-safe |
| **Compile-time** | ✅ | ✅ | Both compile-time |
| **Zero Cost** | ✅ | ✅ | Both zero-cost |
| **Response Integration** | ✅ | ✅ | Both integrate fully |
| **Layouts** | ✅ | ✅ | Both work with layouts |

## Side-by-Side Comparison

### Basic Elements

```rust
// html!
html! {
    <div>Hello</div>
    <p>Paragraph</p>
}

// maud!
maud! {
    div { "Hello" }
    p { "Paragraph" }
}
```

### Classes and IDs

```rust
// html!
html! {
    <div class="container">
        <h1 id="title">Title</h1>
        <section class="content dark">Content</section>
    </div>
}

// maud!
maud! {
    div.container {
        h1#title { "Title" }
        section.content.dark { "Content" }
    }
}
```

### Attributes

```rust
// html!
html! {
    <a href="/about" target="_blank">About</a>
    <input type="text" placeholder="Name" />
    <button onclick="doIt()" data-id="42">Click</button>
}

// maud!
maud! {
    a[href="/about"][target="_blank"] { "About" }
    input[type="text"][placeholder="Name"]
    button[onclick="doIt()"][data-id="42"] { "Click" }
}
```

### Interpolation

```rust
let name = "Alice";
let count = 5;

// html!
html! {
    <div>
        <h1>{name}</h1>
        <p>{count}</p>
    </div>
}

// maud!
maud! {
    div {
        h1 { (name) }
        p { (count) }
    }
}
```

### Conditionals

```rust
let is_admin = true;

// html!
html! {
    <div r-if="is_admin" class="admin">
        "Admin Panel"
    </div>
}

// maud!
maud! {
    @if is_admin {
        div.admin { "Admin Panel" }
    }
}
```

### Loops

```rust
let items = vec!["A", "B", "C"];

// html!
html! {
    <ul>
        <li r-for="item in items">
            {item}
        </li>
    </ul>
}

// maud!
maud! {
    ul {
        @for item in &items {
            li { (item) }
        }
    }
}
```

## Use Case Matrix

### Choose html! When:

| Scenario | Reason |
|----------|--------|
| Coming from web development | Familiar HTML-like syntax |
| Templates are markup-heavy | JSX style feels natural |
| Copying HTML snippets | Direct HTML input |
| Team knows web stack | Lower learning curve |
| HTML-first mindset | Think in markup |

**Example - Content-Heavy Blog Template:**
```rust
html! {
    <article>
        <header>
            <h1>"Article Title"</h1>
            <time datetime="2024-01-15">"January 15, 2024"</time>
        </header>
        <section class="content">
            <p>"First paragraph"</p>
            <p>"Second paragraph"</p>
            <p>"Third paragraph"</p>
        </section>
        <footer>
            <p class="author">"By Jane Doe"</p>
        </footer>
    </article>
}
```

### Choose maud! When:

| Scenario | Reason |
|----------|--------|
| Heavy loops/conditionals | More compact and readable |
| Programmatic generation | Natural functional style |
| Prefer minimal syntax | Less visual noise |
| Data-driven templates | Loop-heavy patterns |
| Lisp/Scheme background | Familiar syntax |

**Example - Data-Driven List Template:**
```rust
let categories = vec![
    ("Fruits", vec!["Apple", "Banana"]),
    ("Veggies", vec!["Carrot", "Lettuce"]),
];

maud! {
    @for (category, items) in categories {
        section {
            h3 { (category) }
            ul {
                @for item in items {
                    li { (item) }
                }
            }
        }
    }
}
```

## Real-World Examples

### E-Commerce Product Card

**html!:**
```rust
html! {
    <div class="product-card" data-id={product.id}>
        <img src={product.image} alt={product.name} />
        <h3 class="title">{product.name}</h3>
        <p class="description">{product.description}</p>
        <div class="pricing">
            <span class="price">"$" {product.price}</span>
            <button class="btn-primary" onclick="addToCart()">
                "Add to Cart"
            </button>
        </div>
    </div>
}
```

**maud!:**
```rust
maud! {
    div.product-card[data-id=(product.id)] {
        img[src=(product.image)][alt=(product.name)]
        h3.title { (product.name) }
        p.description { (product.description) }
        div.pricing {
            span.price { "$" (product.price) }
            button.btn-primary[onclick="addToCart()"] { "Add to Cart" }
        }
    }
}
```

### User List with Roles

**html!:**
```rust
html! {
    <ul class="user-list">
        <li r-for="user in users">
            <div class="user-item">
                <span class="name">{user.name}</span>
                <span class="role" class=(
                    match user.role {
                        "admin" => "badge-danger",
                        "mod" => "badge-warning",
                        _ => "badge-info"
                    }
                )>
                    {user.role}
                </span>
            </div>
        </li>
    </ul>
}
```

**maud!:**
```rust
maud! {
    ul.user-list {
        @for user in &users {
            li {
                div.user-item {
                    span.name { (user.name) }
                    span.role[class=(
                        match user.role {
                            "admin" => "badge-danger",
                            "mod" => "badge-warning",
                            _ => "badge-info"
                        }
                    )] { (user.role) }
                }
            }
        }
    }
}
```

### Form with Validation

**html!:**
```rust
html! {
    <form class="contact-form">
        <div class="form-group">
            <label for="email">"Email"</label>
            <input type="email" id="email" name="email" />
            <span r-if="errors.contains(&\"email\")" class="error">
                "Email is required"
            </span>
        </div>
        <div class="form-actions">
            <button type="submit" class="btn-primary">"Send"</button>
        </div>
    </form>
}
```

**maud!:**
```rust
maud! {
    form.contact-form {
        div.form-group {
            label[for="email"] { "Email" }
            input[type="email"][id="email"][name="email"]
            @if errors.contains(&"email") {
                span.error { "Email is required" }
            }
        }
        div.form-actions {
            button[type="submit"].btn-primary { "Send" }
        }
    }
}
```

## Syntax Reference Table

| Element | html! | maud! |
|---------|-------|-------|
| Element | `<div>` | `div {` |
| Text | `"text"` | `"text"` |
| Class | `class="name"` | `.name` |
| Multiple classes | `class="a b"` | `.a.b` |
| ID | `id="name"` | `#name` |
| Attribute | `attr="val"` | `[attr="val"]` |
| Dynamic attr | `attr={expr}` | `[attr=(expr)]` |
| Interpolate | `{variable}` | `(variable)` |
| Condition | `r-if="cond"` | `@if cond` |
| Loop | `r-for="x in y"` | `@for x in y` |
| Nesting | `</>`  syntax | `{ }` braces |

## Integration with RHTMX

Both macros integrate seamlessly with RHTMX response builders:

```rust
use rhtmx::{get, post, Ok, OkResponse, maud, html, layouts};

// Both work with Ok()
get!()
fn page1() -> OkResponse {
    Ok().html(html! { <div>"html!"</div> })
}

get!()
fn page2() -> OkResponse {
    Ok().html(maud! { div { "maud!" } })
}

// Both work with layouts
get!()
fn page3() -> OkResponse {
    let content = maud! { div { "Content" } };
    Ok().html(layouts::root::layout(
        content,
        layouts::root::Slots::new("Page")
    ))
}

// Both work with forms
post!()
fn handler(form: FormContext) -> OkResponse {
    let name = form.get("name").unwrap_or("Guest");
    Ok().html(maud! { p { "Hello " (name) } })
}
```

## Migration Path

### From html! to maud!

You can gradually migrate or use both in the same project:

```rust
// Start with html!
get!()
fn old_style() -> OkResponse {
    Ok().html(html! {
        <div class="container">
            <h1>"Title"</h1>
        </div>
    })
}

// Convert to maud!
get!()
fn new_style() -> OkResponse {
    Ok().html(maud! {
        div.container {
            h1 { "Title" }
        }
    })
}

// Mix both when it makes sense
get!()
fn mixed() -> OkResponse {
    let html_part = html! { <header>"Header"</header> };
    let maud_part = maud! { main { "Content" } };
    Ok().html(maud! { html { (html_part) (maud_part) } })
}
```

## Performance Comparison

Both macros compile to equivalent efficient code:

```rust
// html!
html! { <div class="a">"text"</div> }

// maud!
maud! { div.a { "text" } }

// Both compile to roughly:
Html(String::from(r#"<div class="a">text</div>"#))
```

**No performance difference** - both are compile-time code generation.

## Learning Resources

- [Full maud! Guide](./MAUD_MACRO_GUIDE.md)
- [maud! Quick Reference](./MAUD_MACRO_QUICK_REF.md)
- [html! Guide](../html!/HTML_MACRO_GUIDE.md)
- [Official Maud Docs](https://maud.rs/)

## Frequently Asked Questions

**Q: Can I use both in the same project?**
A: Yes! They integrate seamlessly. Use whichever feels right for each template.

**Q: Which is faster?**
A: Both compile to equivalent code - no performance difference.

**Q: Which should new developers learn?**
A: Start with html! if you're from web development, maud! if you prefer compact syntax.

**Q: Can I convert between them?**
A: Yes, mostly automatically. Syntax differences are straightforward.

**Q: Does maud! support HTMX directives?**
A: Yes, use attributes: `[hx-get="/path"][hx-swap="innerHTML"]`

## Summary

| Aspect | html! | maud! |
|--------|-------|-------|
| **Syntax** | HTML-like (JSX) | Lisp-like |
| **Compactness** | Verbose | Compact |
| **Learning curve** | Shallow | Moderate |
| **Best for markup** | ✅ Yes | Good |
| **Best for logic** | OK | ✅ Yes |
| **Performance** | Excellent | Excellent |
| **Community** | Familiar | Niche |

**Bottom line:** Choose based on team preference and use case. Both are excellent choices for RHTMX applications.

## See Also

- [FEATURES.md](../FEATURES.md) - Complete feature list
- [QUICKSTART.md](../QUICKSTART.md) - Getting started
- [LAYOUTS.md](../LAYOUTS.md) - Layout system
