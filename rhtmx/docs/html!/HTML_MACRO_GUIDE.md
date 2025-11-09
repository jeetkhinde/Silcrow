# HTML! Macro Guide

The `html!` macro is RHTML's compile-time HTML generation system. It provides JSX-like syntax with powerful r-directives for control flow, all compiled to efficient Rust code.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Response Builders](#response-builders)
3. [R-Directives](#r-directives)
4. [Expression Interpolation](#expression-interpolation)
5. [Complete Examples](#complete-examples)

---

## Basic Usage

### Simple HTML Generation

```rust
use rhtml::html;
use rhtml::Html;

fn simple_card() {
    html! {
        <div class="card">
            <h1>Hello, World!</h1>
            <p>This is a simple example</p>
        </div>
    }
}
```

**Generated HTML:**
```html
<div class="card">
    <h1>Hello, World!</h1>
    <p>This is a simple example</p>
</div>
```

---

## Response Builders

RHTML provides three convenient response builders for HTTP handlers:

### Ok() - Success Responses

```rust
use rhtml::{Ok, Html, html};

#[post]
fn create_user(req: CreateUserRequest) {
    let user = db.create_user(req)?;

    Ok()
        .render(user_card, &user)        // Render a component
        .toast("User created!")           // Show toast notification
        .status(StatusCode::CREATED)      // Custom status code
}

fn user_card(user: &User) -> Html {
    html! {
        <div class="user-card">
            <h3>{user.name}</h3>
            <p>{user.email}</p>
        </div>
    }
}
```

### Error() - Error Responses

```rust
use rhtml::{Error, Html, html};

#[post]
fn create_user(req: CreateUserRequest) -> Result<OkResponse, ErrorResponse> {
    if let Err(errors) = validate(&req) {
        return Err(
            Error()
                .render(validation_errors, errors)
                .status(StatusCode::BAD_REQUEST)
        );
    }

    // ... create user
}

fn validation_errors(errors: Vec<String>) -> Html {
    html! {
        <div class="errors">
            <h3>Please fix the following:</h3>
            <ul>
                <li r-for="error in errors">{error}</li>
            </ul>
        </div>
    }
}
```

### Redirect() - Redirects

```rust
use rhtml::Redirect;

#[post]
fn login(req: LoginRequest) -> RedirectResponse {
    if authenticate(&req) {
        Redirect()
            .to("/dashboard")
            .toast("Welcome back!")
    } else {
        Redirect()
            .to("/login")
            .toast("Invalid credentials")
    }
}
```

---

## R-Directives

R-directives are special attributes that control flow and rendering at compile-time.

### r-for - Loops

Loop through collections with the `r-for` directive:

#### Simple Iteration

```rust
fn users_list(users: Vec<User>) -> Html {
    html! {
        <div class="users-list">
            <h2>Users</h2>
            <div r-for="user in users" class="user-item">
                <p>{user.name} - {user.email}</p>
            </div>
        </div>
    }
}
```

**Compiles to:**
```rust
let mut __html = String::new();
__html.push_str("<div class=\"users-list\">");
__html.push_str("<h2>Users</h2>");
for user in users {
    __html.push_str("<div class=\"user-item\">");
    __html.push_str("<p>");
    __html.push_str(&format!("{}", user.name));
    __html.push_str(" - ");
    __html.push_str(&format!("{}", user.email));
    __html.push_str("</p>");
    __html.push_str("</div>");
}
__html.push_str("</div>");
rhtml::Html(__html)
```

#### With Index

```rust
fn numbered_list(items: Vec<String>) -> Html {
    html! {
        <ol>
            <li r-for="(i, item) in items">
                {i + 1}: {item}
            </li>
        </ol>
    }
}
```

**Output:**
```html
<ol>
    <li>1: First item</li>
    <li>2: Second item</li>
    <li>3: Third item</li>
</ol>
```

### r-if - Conditionals

Conditionally render elements based on boolean expressions:

```rust
fn dashboard(user: &User) -> Html {
    html! {
        <div>
            <h1>Dashboard</h1>

            <div r-if="user.is_admin" class="admin-panel">
                <h2>Admin Controls</h2>
                <button>Manage Users</button>
            </div>

            <div r-if="!user.is_admin" class="user-panel">
                <h2>User Dashboard</h2>
                <p>Welcome, {user.name}!</p>
            </div>
        </div>
    }
}
```

**Compiles to:**
```rust
let mut __html = String::new();
__html.push_str("<div>");
__html.push_str("<h1>Dashboard</h1>");

if user.is_admin {
    __html.push_str("<div class=\"admin-panel\">");
    __html.push_str("<h2>Admin Controls</h2>");
    __html.push_str("<button>Manage Users</button>");
    __html.push_str("</div>");
}

if !user.is_admin {
    __html.push_str("<div class=\"user-panel\">");
    __html.push_str("<h2>User Dashboard</h2>");
    __html.push_str("<p>Welcome, ");
    __html.push_str(&format!("{}", user.name));
    __html.push_str("!</p>");
    __html.push_str("</div>");
}

__html.push_str("</div>");
rhtml::Html(__html)
```

---

## Expression Interpolation

Insert Rust expressions directly into HTML using `{expr}` syntax:

### Basic Interpolation

```rust
fn user_greeting(user: &User) -> Html {
    html! {
        <div>
            <h1>Hello, {user.name}!</h1>
            <p>Your email is {user.email}</p>
            <p>Account age: {user.created_at.format("%Y-%m-%d")}</p>
        </div>
    }
}
```

### In Attributes

```rust
fn user_card(user: &User) -> Html {
    html! {
        <div
            class="user-card"
            id="user-{user.id}"
            data-role={if user.is_admin { "admin" } else { "user" }}>

            <img src={user.avatar_url} alt="Avatar" />
            <h3>{user.name}</h3>
        </div>
    }
}
```

### Complex Expressions

```rust
fn stats_dashboard(stats: &Stats) -> Html {
    let growth_rate = (stats.new_users as f64 / stats.total_users as f64) * 100.0;

    html! {
        <div class="stats">
            <div class="stat">
                <h3>Total Users</h3>
                <p class="number">{stats.total_users}</p>
            </div>
            <div class="stat">
                <h3>Growth Rate</h3>
                <p class="number">{format!("{:.2}%", growth_rate)}</p>
            </div>
        </div>
    }
}
```

---

## Complete Examples

### Example 1: CRUD User Interface

```rust
use rhtml::{Ok, Error, html, Html};

// File: pages/Users.rhtml

// GET /users - List all users
#[get]
fn index() -> OkResponse {
    let users = db::get_users()?;
    Ok().render(users_page, users)
}

// POST /users - Create new user
#[post]
fn create(req: CreateUserRequest) -> Result<OkResponse, ErrorResponse> {
    // Validate
    if let Err(errors) = validate(&req) {
        return Err(
            Error()
                .render(validation_errors, errors)
                .status(StatusCode::BAD_REQUEST)
        );
    }

    // Create user
    let user = db.create_user(req)?;

    Ok()
        .render(user_card, &user)
        .render_oob("user-count", user_count, db.count_users()?)
        .toast("User created successfully!")
}

// DELETE /users/:id - Delete user
#[delete(":id")]
fn delete(id: i32) -> OkResponse {
    db.delete_user(id)?;

    Ok()
        .render_oob("user-count", user_count, db.count_users()?)
        .toast("User deleted!")
}

// UI Components

fn users_page(users: Vec<User>) -> Html {
    html! {
        <div class="container">
            <header>
                <h1>Users</h1>
                <div id="user-count">
                    {user_count(users.len())}
                </div>
            </header>

            <main>
                {user_form()}

                <div id="user-list" class="users-grid">
                    <div r-for="user in users">
                        {user_card(&user)}
                    </div>
                </div>
            </main>
        </div>
    }
}

fn user_form() -> Html {
    html! {
        <form
            hx-post="/users"
            hx-target="#user-list"
            hx-swap="beforeend"
            class="user-form">

            <input
                name="name"
                required
                placeholder="Full Name"
                minlength="2" />

            <input
                name="email"
                type="email"
                required
                placeholder="email@example.com" />

            <button type="submit">Create User</button>
        </form>
    }
}

fn user_card(user: &User) -> Html {
    html! {
        <div class="user-card" id="user-{user.id}">
            <div class="user-info">
                <h3>{user.name}</h3>
                <p>{user.email}</p>

                <span r-if="user.is_admin" class="badge admin">
                    Admin
                </span>
            </div>

            <div class="actions">
                <button
                    hx-delete="/users/{user.id}"
                    hx-target="#user-{user.id}"
                    hx-swap="outerHTML"
                    hx-confirm="Are you sure?">
                    Delete
                </button>
            </div>
        </div>
    }
}

fn user_count(count: usize) -> Html {
    html! {
        <div class="count">
            Total: <strong>{count}</strong>
        </div>
    }
}

fn validation_errors(errors: Vec<String>) -> Html {
    html! {
        <div class="errors">
            <h3>Validation Errors:</h3>
            <ul>
                <li r-for="error in errors" class="error">
                    {error}
                </li>
            </ul>
        </div>
    }
}
```

### Example 2: Nested Components with Data

```rust
fn dashboard(user: &User, stats: &DashboardStats) -> Html {
    html! {
        <div class="dashboard">
            <header>
                <h1>Welcome, {user.name}!</h1>
                <nav>
                    <a href="/dashboard">Dashboard</a>
                    <a href="/settings">Settings</a>
                    <a href="/logout">Logout</a>
                </nav>
            </header>

            <main>
                <section class="stats-grid">
                    <div r-for="stat in &stats.metrics" class="stat-card">
                        <h3>{stat.label}</h3>
                        <p class="value">{stat.value}</p>
                        <span class="trend" class:positive={stat.change > 0}>
                            {format!("{:+.1}%", stat.change)}
                        </span>
                    </div>
                </section>

                <section class="recent-activity">
                    <h2>Recent Activity</h2>
                    {activity_list(&stats.activities)}
                </section>
            </main>
        </div>
    }
}

fn activity_list(activities: &[Activity]) -> Html {
    html! {
        <ul class="activity-list">
            <li r-for="(i, activity) in activities.iter().enumerate()"
                class="activity-item">

                <span class="time">{activity.timestamp.format("%H:%M")}</span>
                <span class="message">{activity.message}</span>
            </li>
        </ul>
    }
}
```

---

## Key Benefits

### 1. **Compile-Time Safety**
- Syntax errors caught at compile time
- Type checking for all expressions
- No runtime parsing overhead

### 2. **Zero Runtime Cost**
- All directives expanded to native Rust loops/conditionals
- Efficient string building
- Optimal binary size

### 3. **Great Developer Experience**
- JSX-like syntax familiar to web developers
- Clean, readable templates
- Full Rust language support in expressions

### 4. **HTMX Integration**
- Natural HTMX attribute syntax
- OOB updates with `.render_oob()`
- Toast notifications built-in

---

## Migration from Current RHTML

### Before (Runtime)
```rust
#[component]
pub fn get_users() {
    <div r-for="user in users">
        <p>{user.name}</p>
    </div>
}
```

### After (Compile-Time)
```rust
#[get]
fn index() -> OkResponse {
    let users = db::get_users()?;
    Ok().render(users_list, users)
}

fn users_list(users: Vec<User>) -> Html {
    html! {
        <div r-for="user in users">
            <p>{user.name}</p>
        </div>
    }
}
```

**Benefits:**
- Clear separation: handlers do logic, functions do UI
- Type-safe props
- Compile-time optimization
- Better testability

---

## Tips and Best Practices

### 1. Keep UI Functions Pure
```rust
// ✓ Good: Pure UI function
fn user_card(user: &User) -> Html {
    html! { <div>{user.name}</div> }
}

// ✗ Bad: Side effects in UI
fn user_card(user: &User) -> Html {
    db.log_view(user.id); // Side effect!
    html! { <div>{user.name}</div> }
}
```

### 2. Extract Complex Logic
```rust
// ✓ Good: Logic outside template
fn user_card(user: &User) -> Html {
    let display_name = format!("{} (@{})", user.name, user.username);
    let badge_class = if user.is_verified { "verified" } else { "" };

    html! {
        <div class="user-card">
            <h3>{display_name}</h3>
            <span class={badge_class}>Badge</span>
        </div>
    }
}
```

### 3. Compose Components
```rust
fn layout(content: Html, user: &User) -> Html {
    html! {
        <html>
            <head><title>My App</title></head>
            <body>
                {header(user)}
                <main>{content}</main>
                {footer()}
            </body>
        </html>
    }
}
```

---

## Future Enhancements

Planned features for future releases:

1. **r-else-if and r-else** directives
2. **r-match** for pattern matching
3. **Component slots** for flexible layouts
4. **Scoped CSS** with `css!` macro integration
5. **Automatic HTMX attribute helpers**

---

For more information, see the [RHTML Documentation](./README.md).
