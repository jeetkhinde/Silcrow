# RHTMX

**Rust + HTMX Framework with Compile-Time HTML Generation**

RHTMX is a modern web framework that combines Rust's type safety with HTMX's simplicity. It features compile-time HTML generation with zero runtime overhead, making it perfect for building fast, type-safe web applications.

## ✨ Key Features

- **🚀 Compile-Time HTML** - All templates compiled to Rust code, zero runtime overhead
- **🔒 Type-Safe** - Full type checking at compile time
- **⚡ Zero-Cost Abstractions** - No runtime template engine
- **🎯 Simple API** - Clean, intuitive response builders
- **🔄 R-Directives** - Powerful control flow (`r-for`, `r-if`)
- **📁 File-Based Routing** - Automatic route generation from file structure
- **🎨 Scoped CSS** - Optional `css!` macro for component styles
- **🔗 HTMX Native** - First-class HTMX support with OOB updates and toasts

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rhtmx = "0.1"
```

### Basic Example

```rust
use rhtmx::{html, Html, Ok, get};

// HTTP Handler
get!()
fn index() -> rhtmx::OkResponse {
    let users = vec!["Alice", "Bob", "Charlie"];
    Ok().render(user_list, users)
}

// UI Component
fn user_list(users: Vec<&str>) -> Html {
    html! {
        <div class="users">
            <h1>Users</h1>
            <ul>
                <li r-for="user in users">{user}</li>
            </ul>
        </div>
    }
}
```

## 📖 Core Concepts

### 1. HTTP Handlers with Macros

Use HTTP verb macros (`get!`, `post!`, `put!`, `patch!`, `delete!`) to define route handlers:

```rust
// GET /users
get!()
fn index() -> OkResponse {
    Ok().render(users_page, get_users())
}

// POST /users
post!()
fn create(req: CreateUserRequest) -> OkResponse {
    let user = db::create_user(req)?;
    Ok().render(user_card, user)
        .toast("User created!")
}

// DELETE /users/:id
delete!(":id")
fn delete(id: i32) -> OkResponse {
    db::delete_user(id)?;
    Ok().toast("Deleted!")
}
```

**Learn more:** See [HTTP Handlers Guide](./docs/http/HTTP_HANDLERS_GUIDE.md) for complete documentation on routing, path parameters, query parameters, and response builders.

### 2. Pure UI Functions

UI is defined as pure functions that return `Html`:

```rust
fn user_card(user: User) -> Html {
    html! {
        <div class="card">
            <h3>{user.name}</h3>
            <p>{user.email}</p>
        </div>
    }
}
```

### 3. R-Directives for Control Flow

#### r-for (Loops)

```rust
html! {
    <div r-for="item in items">
        <p>{item.name}</p>
    </div>
}

// With index
html! {
    <li r-for="(i, item) in items">
        {i + 1}: {item}
    </li>
}
```

#### r-if (Conditionals)

```rust
html! {
    <div r-if="user.is_admin">
        Admin Panel
    </div>
}
```

### 4. Response Builders

#### Ok() - Success Response

```rust
Ok()
    .render(component, data)           // Main content
    .render_oob("id", component, data) // OOB update
    .toast("Success!")                  // Toast notification
    .status(StatusCode::CREATED)       // Custom status
```

#### Error() - Error Response

```rust
Error()
    .render(error_component, errors)
    .status(StatusCode::BAD_REQUEST)
```

#### Redirect() - Redirect Response

```rust
Redirect()
    .to("/dashboard")
    .toast("Welcome!")
```

## 🎯 Complete Example

```rust
// File: pages/users.rs
use rhtmx::{html, Html, Ok, get, post, delete};

// ============================================================================
// Handlers
// ============================================================================

get!()
fn index() -> rhtmx::OkResponse {
    let users = db::get_users()?;
    Ok().render(users_page, users)
}

post!()
fn create(req: CreateUserRequest) -> rhtmx::OkResponse {
    let user = db::create_user(req)?;

    Ok()
        .render(user_card, user.clone())
        .render_oob("user-count", user_count, db::count_users()?)
        .toast("User created!")
}

delete!(":id")
fn delete(id: i32) -> rhtmx::OkResponse {
    db::delete_user(id)?;

    Ok()
        .render_oob("user-count", user_count, db::count_users()?)
        .toast("User deleted!")
}

// ============================================================================
// UI Components
// ============================================================================

fn users_page(users: Vec<User>) -> Html {
    html! {
        <div class="container">
            <header>
                <h1>Users</h1>
                <div id="user-count">{user_count(users.len())}</div>
            </header>

            <main>
                {user_form()}

                <div id="user-list">
                    <div r-for="user in users">
                        {user_card(user)}
                    </div>
                </div>
            </main>
        </div>
    }
}

fn user_form() -> Html {
    html! {
        <form hx-post="/users" hx-target="#user-list" hx-swap="beforeend">
            <input name="name" required placeholder="Name" />
            <input name="email" type="email" required placeholder="Email" />
            <button type="submit">Create User</button>
        </form>
    }
}

fn user_card(user: User) -> Html {
    html! {
        <div class="user-card" id="user-{user.id}">
            <h3>{user.name}</h3>
            <p>{user.email}</p>

            <div r-if="user.is_admin" class="badge">
                Admin
            </div>

            <button
                hx-delete="/users/{user.id}"
                hx-target="#user-{user.id}"
                hx-swap="outerHTML"
                hx-confirm="Delete this user?">
                Delete
            </button>
        </div>
    }
}

fn user_count(count: usize) -> Html {
    html! {
        <div class="count">Total: <strong>{count}</strong></div>
    }
}
```

## 🏗️ Project Structure

```
my-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── lib.rs
└── pages/              # File-based routing
    ├── index.rs        # → /
    ├── users.rs        # → /users
    └── admin/
        └── dashboard.rs # → /admin/dashboard
```

## 🎨 Architecture

### Clean Separation of Concerns

```rust
// ✓ Handlers = Logic
get!()
fn index() -> OkResponse {
    let data = fetch_data();
    Ok().render(page, data)
}

// ✓ Components = Pure UI
fn page(data: Data) -> Html {
    html! { <div>{data.value}</div> }
}
```

### Type-Safe All the Way

```rust
// Compiler enforces:
// - Function returns Html
// - Props match expected type
// - All expressions are valid

fn user_card(user: &User) -> Html {
    html! {
        <div>{user.name}</div>  // ✓ Checked at compile time
    }
}
```

## 🔧 How It Works

### Compile Time

```rust
html! {
    <div r-for="user in users">
        <p>{user.name}</p>
    </div>
}
```

**↓ Expands to:**

```rust
let mut __html = String::new();
for user in users {
    __html.push_str("<div>");
    __html.push_str("<p>");
    __html.push_str(&format!("{}", user.name));
    __html.push_str("</p>");
    __html.push_str("</div>");
}
rhtmx::Html(__html)
```

**Result:**
- Fast execution (native Rust loops)
- Type-checked (compiler validates `user.name`)
- Zero overhead (no runtime interpretation)

## 📊 Performance

- **Zero runtime overhead** - All templates compiled to Rust
- **No parsing** - HTML parsed at compile time
- **Optimal code** - Efficient string building
- **Small binary** - No runtime template engine

## 🆚 Comparison

| Feature | RHTMX | Leptos | Yew | Actix+Tera |
|---------|-------|--------|-----|------------|
| **Compile-time HTML** | ✅ | ✅ | ✅ | ❌ |
| **Zero runtime cost** | ✅ | ✅ | ✅ | ❌ |
| **Type-safe templates** | ✅ | ✅ | ✅ | ❌ |
| **HTMX-first** | ✅ | ❌ | ❌ | ➖ |
| **Simple syntax** | ✅ | ➖ | ➖ | ✅ |
| **Learning curve** | Low | Medium | Medium | Low |

## 🎓 Learn More

- **HTTP Handlers**: [Complete Guide](./docs/http/HTTP_HANDLERS_GUIDE.md) · [Quick Ref](./docs/http/HTTP_HANDLERS_QUICK_REF.md) · [Summary](./docs/http/HTTP_HANDLERS_SUMMARY.md)
- **Features**: [FEATURES.md](./docs/FEATURES.md) - Complete feature reference
- **Quick Start**: [QUICKSTART.md](./QUICKSTART.md) - Get started in 5 minutes
- **Examples**: See `examples/` directory
- **Migration Guide**: [From RHTML](./MIGRATION.md)

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## 📄 License

MIT License - see [LICENSE](./LICENSE) for details.

---

**RHTMX** - Fast, type-safe, simple. 🚀
