# RHTMX Features Guide

Complete reference for all RHTMX features and directives.

## Table of Contents

1. [R-Directives](#r-directives)
   - [r-for](#r-for---loops)
   - [r-if / r-else-if / r-else](#r-if--r-else-if--r-else---conditionals)
   - [r-match / r-when / r-default](#r-match--r-when--r-default---pattern-matching)
2. [css! Macro](#css-macro---scoped-styles)
3. [HTTP Verb Macros](#http-verb-macros)
4. [Response Builders](#response-builders)
5. [Form Validation](#form-validation)
   - [Basic Usage](#basic-usage)
   - [Available Validators](#available-validators)
   - [Validation with HTMX](#validation-with-htmx)
6. [Complete Examples](#complete-examples)

---

## R-Directives

### r-for - Loops

Iterate over collections with compile-time loop generation.

#### Basic Iteration

```rust
fn user_list(users: Vec<User>) -> Html {
    html! {
        <ul>
            <li r-for="user in users">
                {user.name}
            </li>
        </ul>
    }
}
```

**Generated Code:**
```rust
for user in users {
    __html.push_str("<li>");
    __html.push_str(&format!("{}", user.name));
    __html.push_str("</li>");
}
```

#### With Index

```rust
fn numbered_list(items: Vec<String>) -> Html {
    html! {
        <ol>
            <li r-for="(i, item) in items">
                {i + 1}". " {item}
            </li>
        </ol>
    }
}
```

**Key Features:**
- ✅ Compile-time loop expansion
- ✅ Zero runtime overhead
- ✅ Type-safe iteration
- ✅ Support for enumerate()

---

### r-if / r-else-if / r-else - Conditionals

Conditional rendering based on boolean expressions.

#### Simple Conditional

```rust
fn admin_panel(user: &User) -> Html {
    html! {
        <div>
            <div r-if="user.is_admin">
                <h2>"Admin Controls"</h2>
                <button>"Manage Users"</button>
            </div>
        </div>
    }
}
```

#### If-Else Chain

```rust
fn score_display(score: i32) -> Html {
    html! {
        <div>
            <div r-if="score >= 90" class="grade-a">
                "Excellent!"
            </div>

            <div r-else-if="score >= 75" class="grade-b">
                "Good job!"
            </div>

            <div r-else-if="score >= 60" class="grade-c">
                "Average"
            </div>

            <div r-else class="grade-f">
                "Needs improvement"
            </div>
        </div>
    }
}
```

**Generated Code:**
```rust
if score >= 90 {
    __html.push_str("<div class=\"grade-a\">Excellent!</div>");
} else if score >= 75 {
    __html.push_str("<div class=\"grade-b\">Good job!</div>");
} else if score >= 60 {
    __html.push_str("<div class=\"grade-c\">Average</div>");
} else {
    __html.push_str("<div class=\"grade-f\">Needs improvement</div>");
}
```

**Supported Operators:**
- `==`, `!=` - Equality
- `>`, `<`, `>=`, `<=` - Comparison
- `&&`, `||` - Logical AND/OR
- `!` - Negation

**Key Features:**
- ✅ Compile-time conditional generation
- ✅ Full Rust expression support
- ✅ Type-checked conditions
- ✅ Zero runtime overhead

---

### r-match / r-when / r-default - Pattern Matching

Rust pattern matching in HTML templates.

#### Enum Matching

```rust
#[derive(PartialEq)]
enum UserStatus {
    Active,
    Pending,
    Suspended,
}

fn status_badge(status: UserStatus) -> Html {
    html! {
        <div r-match="status">
            <span r-when="UserStatus::Active" class="badge-active">
                "✓ Active"
            </span>
            <span r-when="UserStatus::Pending" class="badge-pending">
                "⏳ Pending"
            </span>
            <span r-when="UserStatus::Suspended" class="badge-suspended">
                "⛔ Suspended"
            </span>
            <span r-default class="badge-unknown">
                "Unknown"
            </span>
        </div>
    }
}
```

**Generated Code:**
```rust
match status {
    UserStatus::Active => {
        __html.push_str("<span class=\"badge-active\">✓ Active</span>");
    }
    UserStatus::Pending => {
        __html.push_str("<span class=\"badge-pending\">⏳ Pending</span>");
    }
    UserStatus::Suspended => {
        __html.push_str("<span class=\"badge-suspended\">⛔ Suspended</span>");
    }
    _ => {
        __html.push_str("<span class=\"badge-unknown\">Unknown</span>");
    }
}
```

#### Option Matching

```rust
fn user_display(user: Option<User>) -> Html {
    html! {
        <div r-match="user">
            <div r-when="Some(u)">
                "User: " {u.name}
            </div>
            <div r-default>
                "No user logged in"
            </div>
        </div>
    }
}
```

**Key Features:**
- ✅ Full Rust pattern matching
- ✅ Enum support
- ✅ Option/Result support
- ✅ Default fallback with r-default
- ✅ Compile-time exhaustiveness checking

---

## css! Macro - Scoped Styles

Define component-scoped CSS with automatic prefixing.

### Basic Usage

```rust
fn styled_card(title: &str) -> Html {
    css! {
        scope: "card",
        .card {
            border: 1px solid #ccc;
            border-radius: 8px;
            padding: 1rem;
        }
        .card:hover {
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .title {
            font-size: 1.5rem;
            font-weight: bold;
        }
    }

    html! {
        <div class="card" data-scope="card">
            <h3 class="title">{title}</h3>
        </div>
    }
}
```

**Generated CSS:**
```css
[data-scope="card"] .card {
    border: 1px solid #ccc;
    border-radius: 8px;
    padding: 1rem;
}
[data-scope="card"] .card:hover {
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}
[data-scope="card"] .title {
    font-size: 1.5rem;
    font-weight: bold;
}
```

### Pseudo-Classes

```rust
css! {
    scope: "button",
    :hover {
        background: #007bff;
    }
    :active {
        transform: scale(0.98);
    }
}

// Generates: [data-scope="button"]:hover { ... }
```

### Parent Selector (&)

```rust
css! {
    scope: "component",
    &.active {
        background: green;
    }
    & > .child {
        margin: 0;
    }
}

// Generates:
// [data-scope="component"].active { ... }
// [data-scope="component"] > .child { ... }
```

**Key Features:**
- ✅ Automatic scope prefixing
- ✅ Unique data-scope attributes
- ✅ No CSS conflicts between components
- ✅ Supports pseudo-classes and pseudo-elements
- ✅ Parent selector (&) support

---

## HTTP Verb Macros

Define route handlers with HTTP verb macros (`get!`, `post!`, `put!`, `patch!`, `delete!`).

**📖 Complete documentation:** See [HTTP Handlers Guide](./http/HTTP_HANDLERS_GUIDE.md) for detailed routing guide, path parameters, query parameters, response builders, and best practices.

### Available Macros

- `get!` - GET requests
- `post!` - POST requests
- `put!` - PUT requests
- `patch!` - PATCH requests
- `delete!` - DELETE requests

### Basic Handler

```rust
get!()
fn list_users() -> OkResponse {
    let users = db::get_users()?;
    Ok().render(users_page, users)
}
```

### With Path Parameters

```rust
get!(":id")
fn get_user(id: i32) -> OkResponse {
    let user = db::get_user(id)?;
    Ok().render(user_detail, user)
}

delete!(":id")
fn delete_user(id: i32) -> OkResponse {
    db::delete_user(id)?;
    Ok().toast("User deleted!")
}
```

### With Query Parameters

```rust
get!("partial=stats")
fn user_stats() -> OkResponse {
    let stats = calculate_stats()?;
    Ok().render(stats_component, stats)
}
```

**Key Features:**
- ✅ Clean, declarative routing
- ✅ Path parameter support
- ✅ Query parameter support
- ✅ Type-safe handlers
- ✅ Works with file-based routing

---

## Response Builders

### Ok() - Success Response

```rust
post!()
fn create_user(req: CreateUserRequest) -> OkResponse {
    let user = db::create_user(req)?;

    Ok()
        .render(user_card, user)                    // Main content
        .render_oob("user-count", count, 10)        // OOB update
        .toast("User created successfully!")         // Toast notification
        .status(StatusCode::CREATED)                 // Custom status code
        .header("X-Custom-Header", "value")          // Custom header
}
```

**Methods:**
- `.render(component, props)` - Render main content
- `.render_oob(target, component, props)` - Out-of-band update
- `.toast(message)` - Show toast notification
- `.status(code)` - Set HTTP status code
- `.header(name, value)` - Add custom header

### Error() - Error Response

```rust
post!()
fn create_user(req: CreateUserRequest) -> Result<OkResponse, ErrorResponse> {
    let errors = validate(&req);

    if !errors.is_empty() {
        return Err(
            Error()
                .render(validation_errors, errors)
                .status(StatusCode::BAD_REQUEST)
                .message("Validation failed")
        );
    }

    // ... create user
}
```

**Methods:**
- `.render(component, props)` - Render error UI
- `.message(text)` - Set error message
- `.status(code)` - Set HTTP status code
- `.header(name, value)` - Add custom header

### Redirect() - Redirect Response

```rust
post!()
fn login(req: LoginRequest) -> RedirectResponse {
    if authenticate(&req) {
        Redirect()
            .to("/dashboard")
            .toast("Welcome back!")
            .status(StatusCode::SEE_OTHER)
    } else {
        Redirect()
            .to("/login")
            .toast("Invalid credentials")
    }
}
```

**Methods:**
- `.to(url)` - Set redirect URL
- `.toast(message)` - Show toast notification
- `.status(code)` - Set HTTP status code (301, 302, 303, 307, 308)

---

## Complete Examples

### CRUD with All Features

```rust
use rhtmx::{html, css, Html, Ok, Error, get, post, delete};

#[derive(Clone, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

struct Task {
    id: i32,
    title: String,
    completed: bool,
    priority: Priority,
}

// GET /tasks
get!()
fn list_tasks() -> OkResponse {
    let tasks = db::get_tasks()?;
    Ok().render(tasks_page, tasks)
}

// POST /tasks
post!()
fn create_task(req: CreateTaskRequest) -> OkResponse {
    let task = db::create_task(req)?;

    Ok()
        .render(task_card, task)
        .render_oob("task-count", task_count, db::count_tasks()?)
        .toast("Task created!")
}

// DELETE /tasks/:id
delete!(":id")
fn delete_task(id: i32) -> OkResponse {
    db::delete_task(id)?;

    Ok()
        .render_oob("task-count", task_count, db::count_tasks()?)
        .toast("Task deleted!")
}

// UI Components

fn tasks_page(tasks: Vec<Task>) -> Html {
    css! {
        scope: "tasks",
        .tasks-grid {
            display: grid;
            gap: 1rem;
        }
        .task-card {
            border: 1px solid #ddd;
            padding: 1rem;
            border-radius: 4px;
        }
        .task-card:hover {
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
    }

    html! {
        <div class="tasks-container" data-scope="tasks">
            <header>
                <h1>"Tasks"</h1>
                <div id="task-count">{task_count(tasks.len())}</div>
            </header>

            <div class="tasks-grid">
                <div r-for="task in tasks">
                    {task_card(task)}
                </div>
            </div>
        </div>
    }
}

fn task_card(task: Task) -> Html {
    html! {
        <div class="task-card" id="task-{task.id}">
            <h3>{task.title}</h3>

            <div r-if="task.completed" class="status">
                "✓ Completed"
            </div>
            <div r-else class="status">
                "○ Pending"
            </div>

            <div r-match="task.priority">
                <span r-when="Priority::High" class="badge-high">
                    "High Priority"
                </span>
                <span r-when="Priority::Medium" class="badge-medium">
                    "Medium Priority"
                </span>
                <span r-when="Priority::Low" class="badge-low">
                    "Low Priority"
                </span>
                <span r-default>
                    "No Priority"
                </span>
            </div>

            <button hx-delete="/tasks/{task.id}">
                "Delete"
            </button>
        </div>
    }
}

fn task_count(count: usize) -> Html {
    html! {
        <div class="count">
            "Total: " <strong>{count}</strong>
        </div>
    }
}
```

---

## Best Practices

### 1. Keep UI Functions Pure

```rust
// ✓ Good: Pure UI function
fn user_card(user: &User) -> Html {
    html! {
        <div>{user.name}</div>
    }
}

// ✗ Bad: Side effects in UI
fn user_card(user: &User) -> Html {
    db::log_view(user.id); // Side effect!
    html! {
        <div>{user.name}</div>
    }
}
```

### 2. Extract Complex Logic

```rust
// ✓ Good: Logic before template
fn user_card(user: &User) -> Html {
    let display_name = format!("{} (@{})", user.name, user.username);
    let badge_class = if user.verified { "verified" } else { "" };

    html! {
        <div>
            <h3>{display_name}</h3>
            <span class={badge_class}>"Badge"</span>
        </div>
    }
}
```

### 3. Use r-directives for Control Flow

```rust
// ✓ Good: Use r-for
html! {
    <div r-for="item in items">{item}</div>
}

// ✗ Bad: Manual iteration
html! {
    {items.iter().map(|item| html! {
        <div>{item}</div>
    }).collect::<Vec<_>>()}
}
```

### 4. Scope Your CSS

```rust
// ✓ Good: Scoped CSS
fn component() -> Html {
    css! {
        scope: "my-component",
        .title { color: blue; }
    }

    html! {
        <div data-scope="my-component">
            <h1 class="title">"Title"</h1>
        </div>
    }
}
```

---

## Form Validation

RHTMX includes a powerful compile-time validation system using `#[derive(Validate)]`.

### Basic Usage

```rust
use rhtmx::{Validate, ValidateTrait};
use serde::Deserialize;

#[derive(Validate, Deserialize)]
struct CreateUserRequest {
    #[min_length(3)]
    #[max_length(50)]
    name: String,

    #[email]
    email: String,

    #[password("strong")]
    password: String,

    #[min(18)]
    #[max(120)]
    age: i32,

    bio: Option<String>,  // Optional fields
}

// Validate the struct
fn handle_request(req: CreateUserRequest) {
    match req.validate() {
        Ok(()) => {
            // Validation passed, process request
            println!("Valid request!");
        }
        Err(errors) => {
            // Validation failed, errors is HashMap<String, String>
            for (field, error) in errors {
                println!("{}: {}", field, error);
            }
        }
    }
}
```

### Available Validators

#### Email Validators

```rust
#[derive(Validate)]
struct EmailForm {
    // Basic email format validation
    #[email]
    email: String,

    // Reject public domains (gmail, yahoo, etc.)
    #[email]
    #[no_public_domains]
    work_email: String,

    // Block specific domains
    #[email]
    #[blocked_domains("competitor.com", "spam.com")]
    business_email: String,
}
```

#### Password Validators

```rust
#[derive(Validate)]
struct PasswordForm {
    // Predefined patterns
    #[password("strong")]  // 8+ chars, upper, lower, digit, special
    strong_password: String,

    #[password("medium")]  // 8+ chars, upper, lower, digit
    medium_password: String,

    #[password("basic")]   // 6+ chars minimum
    basic_password: String,

    // Custom regex pattern
    #[password(r"^(?=.*[A-Z])(?=.*[a-z])(?=.*\d).{12,}$")]
    custom_password: String,
}
```

**Password Pattern Definitions:**

| Pattern | Requirements |
|---------|-------------|
| `"strong"` | 8+ chars, uppercase, lowercase, digit, special char (@$!%*?&) |
| `"medium"` | 8+ chars, uppercase, lowercase, digit |
| `"basic"` | 6+ chars minimum |
| Custom regex | Any regex pattern |

#### Numeric Validators

```rust
#[derive(Validate)]
struct NumericForm {
    #[min(18)]
    age: i32,

    #[max(100)]
    score: i32,

    #[range(1, 10)]
    rating: i32,
}
```

#### String Validators

```rust
#[derive(Validate)]
struct StringForm {
    #[min_length(3)]
    username: String,

    #[max_length(100)]
    bio: String,

    #[length(3, 50)]
    display_name: String,

    #[regex(r"^\d{3}-\d{3}-\d{4}$")]
    phone: String,

    #[url]
    website: String,
}
```

#### General Validators

```rust
#[derive(Validate)]
struct GeneralForm {
    // Required for Option<T> fields
    #[required]
    required_field: Option<String>,

    // Don't trim whitespace (for code, poems, etc.)
    #[allow_whitespace]
    content: String,

    // Optional field (no validation if empty)
    notes: Option<String>,
}
```

### Complete Validator Reference

| Validator | Type | Description | Example |
|-----------|------|-------------|---------|
| `#[email]` | String | Valid email format | `user@example.com` |
| `#[no_public_domains]` | String | Reject gmail, yahoo, etc. | Use with `#[email]` |
| `#[blocked_domains(...)]` | String | Block specific domains | `#[blocked_domains("spam.com")]` |
| `#[password("pattern")]` | String | Password strength | `#[password("strong")]` |
| `#[min(n)]` | Numeric | Minimum value | `#[min(18)]` |
| `#[max(n)]` | Numeric | Maximum value | `#[max(120)]` |
| `#[range(min, max)]` | Numeric | Value range | `#[range(1, 10)]` |
| `#[min_length(n)]` | String | Minimum length | `#[min_length(3)]` |
| `#[max_length(n)]` | String | Maximum length | `#[max_length(50)]` |
| `#[length(min, max)]` | String | Length range | `#[length(3, 50)]` |
| `#[regex(r"pattern")]` | String | Custom regex | `#[regex(r"^\d+$")]` |
| `#[url]` | String | Valid URL | `https://example.com` |
| `#[required]` | Option<T> | Required field | For optional fields |
| `#[allow_whitespace]` | String | Preserve whitespace | For code blocks, etc. |

### Validation with HTMX

```rust
use rhtmx::{Validate, ValidateTrait, Ok, Error};

#[derive(Validate, Deserialize)]
struct CreateUserRequest {
    #[min_length(3)]
    name: String,

    #[email]
    email: String,
}

#[post]
fn create_user(req: CreateUserRequest) -> Result<OkResponse, ErrorResponse> {
    // Validate the request
    if let Err(errors) = req.validate() {
        // Return errors to be displayed in the form
        return Err(Error()
            .message("Validation failed")
            .render(validation_errors, errors));
    }

    // Process valid request
    let user = db::create_user(req)?;
    Ok()
        .render(user_card, user)
        .toast("User created!")
}

fn validation_errors(errors: HashMap<String, String>) -> Html {
    html! {
        <div class="errors">
            <div r-for="(field, error) in errors">
                <p class="error">{field}": "{error}</p>
            </div>
        </div>
    }
}
```

### Whitespace Handling

**Default Behavior:**

| Type | Behavior |
|------|----------|
| `String` | Trimmed, must be non-empty after trimming |
| `Option<String>` | Allows empty/whitespace |
| `#[allow_whitespace] String` | Preserves whitespace, but required |

**Example:**

```rust
#[derive(Validate)]
struct PostForm {
    // Auto-trimmed, non-empty
    title: String,

    // Optional, allows empty
    description: Option<String>,

    // Preserve whitespace (for code, poems)
    #[allow_whitespace]
    content: String,
}
```

### Custom Error Messages

The validation system provides default error messages:

- Email: `"Invalid email address"`
- Password (strong): `"Password must be at least 8 characters with uppercase, lowercase, digit, and special character"`
- Min: `"Must be at least {min}"`
- Max: `"Must be at most {max}"`
- Min length: `"Must be at least {n} characters"`
- URL: `"Invalid URL"`

### Combining Validators

You can stack multiple validators on a single field:

```rust
#[derive(Validate)]
struct AdvancedForm {
    #[min_length(3)]
    #[max_length(50)]
    #[regex(r"^[a-zA-Z0-9_]+$")]
    username: String,

    #[email]
    #[no_public_domains]
    #[blocked_domains("competitor.com")]
    email: String,

    #[min(18)]
    #[max(120)]
    age: i32,
}
```

### Performance

- **Compile-time code generation** - Validation logic is generated at compile time
- **Zero runtime overhead** - No reflection or dynamic dispatch
- **Type-safe** - All validations are type-checked by Rust compiler
- **No dependencies on validation libraries** - Built-in validators use only `regex` crate

---

## Performance Notes

- **All directives compile to native Rust** - Zero runtime overhead
- **Type checking at compile time** - Catch errors before running
- **Optimal code generation** - Efficient string building
- **No runtime template engine** - Small binary size
- **Compile-time validation** - Form validation with zero runtime cost

---

For more examples, see:
- `examples/users_crud.rs` - Basic CRUD
- `examples/complete_features.rs` - All features demo
- `examples/validation_demo.rs` - Complete validation examples

For quick start, see `QUICKSTART.md`
