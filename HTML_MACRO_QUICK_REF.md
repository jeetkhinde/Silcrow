# html! Macro - Quick Reference

## Import
```rust
use rhtml::{html, Html, Ok, Error, Redirect};
```

## Basic Syntax

### Simple HTML
```rust
html! {
    <div class="card">
        <h1>Title</h1>
        <p>Content</p>
    </div>
}
```

### With Props
```rust
fn user_card(user: &User) -> Html {
    html! {
        <div>{user.name}</div>
    }
}
```

## R-Directives

### r-for (Loop)
```rust
// Simple
<div r-for="user in users">{user.name}</div>

// With index
<li r-for="(i, item) in items">{i}: {item}</li>
```

### r-if (Conditional)
```rust
<div r-if="user.is_admin">Admin Panel</div>
<div r-if="count > 0">Has items</div>
```

## Expressions

### In Content
```rust
<p>{user.name}</p>
<p>{format!("Hello {}", name)}</p>
```

### In Attributes
```rust
<div id="user-{user.id}" class={class_name}>
<img src={user.avatar} />
```

## Response Builders

### Ok() - Success
```rust
Ok()
    .render(component, props)
    .toast("Success!")
    .render_oob("id", component, props)
    .status(StatusCode::OK)
```

### Error() - Error
```rust
Error()
    .render(errors_component, errors)
    .message("Failed")
    .status(StatusCode::BAD_REQUEST)
```

### Redirect() - Redirect
```rust
Redirect()
    .to("/dashboard")
    .toast("Welcome!")
    .status(StatusCode::SEE_OTHER)
```

## HTTP Handlers

```rust
#[get]
fn index() -> OkResponse {
    Ok().render(page, data)
}

#[post]
fn create(req: Request) -> OkResponse {
    Ok().render(component, data)
        .toast("Created!")
}

#[delete(":id")]
fn delete(id: i32) -> OkResponse {
    Ok().toast("Deleted!")
}
```

## Common Patterns

### CRUD List
```rust
fn users_list(users: Vec<User>) -> Html {
    html! {
        <div r-for="user in users">
            {user_card(&user)}
        </div>
    }
}
```

### Form
```rust
html! {
    <form hx-post="/users">
        <input name="name" required />
        <button>Submit</button>
    </form>
}
```

### Conditional Classes
```rust
html! {
    <div class={if active { "active" } else { "" }}>
        Content
    </div>
}
```

### OOB Updates
```rust
Ok()
    .render(main_content, data)
    .render_oob("sidebar", sidebar, sidebar_data)
    .render_oob("counter", counter, count)
```

## Tips

- UI functions should be pure (no side effects)
- Extract complex logic before the html! block
- Use r-for for loops, not manual iteration
- Use r-if for conditionals
- Compose components by calling functions
- `.render()` enforces Html return type
