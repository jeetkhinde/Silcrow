# RHTMX Quick Start Guide

Get started with RHTMX in 5 minutes!

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rhtmx = { path = "../rhtmx" }  # Or use the crates.io version when published
```

## Your First RHTMX App

### Step 1: Create a Simple Handler

```rust
use rhtmx::{html, Html, Ok, get};

#[get]
fn hello() -> rhtmx::OkResponse {
    Ok().render(hello_page, "World")
}

fn hello_page(name: &str) -> Html {
    html! {
        <div>
            <h1>Hello, {name}!</h1>
        </div>
    }
}
```

### Step 2: Understanding the Parts

```rust
// 1. HTTP Handler - Handles routing and logic
#[get]  // Maps to GET request
fn hello() -> rhtmx::OkResponse {
    // Your business logic here
    let data = "World";

    // Return a response with rendered HTML
    Ok().render(hello_page, data)
}

// 2. UI Component - Pure function that returns HTML
fn hello_page(name: &str) -> Html {
    // html! macro generates compile-time HTML
    html! {
        <div>
            <h1>Hello, {name}!</h1>
        </div>
    }
}
```

## Key Concepts

### 1. HTTP Verb Macros

```rust
#[get]          // GET requests
#[post]         // POST requests
#[put(":id")]   // PUT with path parameter
#[delete(":id")] // DELETE with path parameter
```

### 2. Response Builders

```rust
// Success response
Ok()
    .render(component, data)
    .toast("Success!")

// Error response
Error()
    .render(error_component, errors)
    .status(StatusCode::BAD_REQUEST)

// Redirect
Redirect()
    .to("/dashboard")
    .toast("Redirecting...")
```

### 3. R-Directives

```rust
// Loops
html! {
    <div r-for="item in items">
        {item}
    </div>
}

// Conditionals
html! {
    <div r-if="show_admin">
        Admin Panel
    </div>
}
```

### 4. Expression Interpolation

```rust
html! {
    <h1>{user.name}</h1>
    <p>{format!("Age: {}", user.age)}</p>
    <div id="user-{user.id}">Content</div>
}
```

## Complete Mini Example

```rust
use rhtmx::{html, Html, Ok, Error, get, post, StatusCode};
use serde::Deserialize;

// Data model
#[derive(Deserialize)]
struct Todo {
    id: i32,
    text: String,
    done: bool,
}

// GET /todos - List all todos
#[get]
fn list_todos() -> rhtmx::OkResponse {
    let todos = vec![
        Todo { id: 1, text: "Learn RHTMX".to_string(), done: true },
        Todo { id: 2, text: "Build app".to_string(), done: false },
    ];

    Ok().render(todos_page, todos)
}

// POST /todos - Create new todo
#[post]
fn create_todo(text: String) -> rhtmx::OkResponse {
    let todo = Todo {
        id: 3,
        text,
        done: false,
    };

    Ok()
        .render(todo_item, todo)
        .toast("Todo created!")
}

// UI Components

fn todos_page(todos: Vec<Todo>) -> Html {
    html! {
        <div class="todos">
            <h1>My Todos</h1>

            <form hx-post="/todos">
                <input name="text" placeholder="New todo..." />
                <button>Add</button>
            </form>

            <ul id="todo-list">
                <li r-for="todo in todos">
                    {todo_item(todo)}
                </li>
            </ul>
        </div>
    }
}

fn todo_item(todo: Todo) -> Html {
    html! {
        <div class="todo">
            <input type="checkbox" checked={todo.done} />
            <span>{todo.text}</span>
        </div>
    }
}
```

## Next Steps

1. **Read the full README**: `README.md`
2. **Check examples**: `examples/users_crud.rs`
3. **Learn HTMX**: https://htmx.org/
4. **Start building!**

## Common Patterns

### 1. Form Handling

```rust
#[post]
fn submit_form(req: FormData) -> rhtmx::OkResponse {
    // Validate
    if req.name.is_empty() {
        return Error()
            .message("Name is required")
            .status(StatusCode::BAD_REQUEST);
    }

    // Process
    let result = process(req);

    // Respond
    Ok()
        .render(success_message, result)
        .toast("Submitted!")
}
```

### 2. OOB Updates

```rust
#[post]
fn add_item(item: Item) -> rhtmx::OkResponse {
    save(item.clone());

    Ok()
        .render(item_card, item)              // Main content
        .render_oob("counter", count_badge, get_count()) // Update counter
}
```

### 3. Conditional Content

```rust
fn dashboard(user: &User) -> Html {
    html! {
        <div>
            <h1>Dashboard</h1>

            <div r-if="user.is_admin">
                <h2>Admin Section</h2>
            </div>

            <div r-if="!user.is_admin">
                <h2>User Section</h2>
            </div>
        </div>
    }
}
```

## Tips

✅ **DO:**
- Keep UI functions pure (no side effects)
- Extract complex logic before the `html!` block
- Use r-directives for loops and conditionals
- Compose components by calling functions

❌ **DON'T:**
- Put business logic in UI functions
- Use manual string concatenation
- Forget to use `#[get]`, `#[post]`, etc. macros
- Mix concerns (keep handlers and UI separate)

---

Happy coding with RHTMX! 🚀
