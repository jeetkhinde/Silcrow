# Actions, Validators, and Form Helpers

This document describes the comprehensive validation and action system for RHTML.

## Table of Contents

1. [Actions System](#actions-system)
2. [Validation Attributes](#validation-attributes)
3. [Form Helpers](#form-helpers)
4. [Whitespace Handling](#whitespace-handling)
5. [Complete Examples](#complete-examples)

---

## Actions System

Actions are functions that start with HTTP verb prefixes: `get_`, `post_`, `put_`, `patch_`, `delete_`.

### Basic Action Structure

```rust
// pages/users/page.rhtml

// GET /users
fn get_users() -> Result<Vec<User>, String> {
    Ok(db.get_users()?)
}

// GET /users?filter=active&page=2
struct SearchUsersRequest {
    #[query] filter: Option<String>,
    #[query] page: Option<i32>,
}

fn get_users(req: SearchUsersRequest) -> Result<Vec<User>, String> {
    let users = db.search_users(req.filter, req.page.unwrap_or(1))?;
    Ok(users)
}

// POST /users
fn post_user(req: CreateUserRequest) -> Result<User, String> {
    let user = db.create_user(req)?;
    Ok(user)
        .toast("User created!")
        .oob("user-count", db.count_users()?)
}

// PUT /users/:id
fn put_user(id: i32, req: UpdateUserRequest) -> Result<User, String> {
    let user = db.update_user(id, req)?;
    Ok(user).toast("User updated!")
}

// PATCH /users/:id
fn patch_user(id: i32, req: PatchUserRequest) -> Result<User, String> {
    let user = db.patch_user(id, req)?;
    Ok(user).toast("User updated!")
}

// DELETE /users/:id
fn delete_user(id: i32) -> Result<Empty, String> {
    db.delete_user(id)?;
    Ok(Empty::new()
        .toast("User deleted!")
        .oob("user-count", db.count_users()?))
}
```

---

## Validation Attributes

### Email Validators

```rust
#[derive(Validate)]
struct RegisterRequest {
    // Basic email validation
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

### Password Validators

```rust
#[derive(Validate)]
struct CreateUserRequest {
    // Predefined patterns
    #[password("strong")]  // 8+ chars, upper, lower, number, special
    password: String,

    #[password("medium")]  // 8+ chars, upper, lower, number
    alt_password: String,

    #[password("basic")]   // 6+ chars
    simple_password: String,

    // Custom regex pattern
    #[password(r"^(?=.*[A-Z])(?=.*[a-z])(?=.*\d)(?=.*[@$!%*?&]).{12,}$")]
    strong_password: String,
}
```

**Password Pattern Definitions:**

- **strong**: `^(?=.*[A-Z])(?=.*[a-z])(?=.*\d)(?=.*[@$!%*?&]).{8,}$`
  - At least 8 characters
  - At least one uppercase letter
  - At least one lowercase letter
  - At least one number
  - At least one special character (@$!%*?&)

- **medium**: `^(?=.*[A-Z])(?=.*[a-z])(?=.*\d).{8,}$`
  - At least 8 characters
  - At least one uppercase letter
  - At least one lowercase letter
  - At least one number

- **basic**: `^.{6,}$`
  - At least 6 characters

### Numeric Validators

```rust
#[derive(Validate)]
struct UserRequest {
    #[min(18)]
    age: i32,

    #[max(100)]
    score: i32,

    #[range(1, 10)]
    rating: i32,
}
```

### String Validators

```rust
#[derive(Validate)]
struct ProfileRequest {
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

### All Available Validators

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[email]` | Valid email format | `user@example.com` |
| `#[no_public_domains]` | Reject gmail, yahoo, etc. | - |
| `#[blocked_domains("a.com", "b.com")]` | Block specific domains | - |
| `#[password("strong\|medium\|basic")]` | Predefined password patterns | - |
| `#[password(r"regex")]` | Custom password regex | - |
| `#[min(n)]` | Minimum numeric value | `#[min(18)]` |
| `#[max(n)]` | Maximum numeric value | `#[max(120)]` |
| `#[range(min, max)]` | Numeric range | `#[range(1, 10)]` |
| `#[min_length(n)]` | Minimum string length | `#[min_length(3)]` |
| `#[max_length(n)]` | Maximum string length | `#[max_length(50)]` |
| `#[length(min, max)]` | String length range | `#[length(3, 50)]` |
| `#[regex(r"pattern")]` | Custom regex validation | `#[regex(r"^\d+$")]` |
| `#[url]` | Valid URL format | - |
| `#[allow_whitespace]` | Keep whitespace (don't trim) | - |
| `#[required]` | Required for Option<T> | - |
| `#[query]` | Parse from query params | - |
| `#[form]` | Parse from form data | - |

---

## Form Helpers

### Empty Response

For actions that don't return content (e.g., DELETE):

```rust
fn delete_user(id: i32) -> Result<Empty, String> {
    db.delete_user(id)?;
    Ok(Empty::new()
        .toast("User deleted!")
        .oob("user-count", db.count_users()?))
}
```

### Toast Notifications

Add toast messages to any response:

```rust
fn post_user(req: CreateUserRequest) -> Result<User, String> {
    let user = db.create_user(req)?;
    Ok(user).toast("User created successfully!")
}
```

The toast message is sent via HX-Trigger header:

```json
{
  "showToast": {
    "message": "User created successfully!"
  }
}
```

### Out-of-Band (OOB) Updates

Update multiple parts of the page:

```rust
fn post_user(req: CreateUserRequest) -> Result<User, String> {
    let user = db.create_user(req)?;
    Ok(user)
        .toast("User created!")
        .oob("user-count", db.count_users()?)
        .oob("user-stats", format!("Total: {}", db.count_users()?))
}
```

OOB updates are rendered as:

```html
<div id="user-count" hx-swap-oob="true">42</div>
<div id="user-stats" hx-swap-oob="true">Total: 42</div>
```

---

## Whitespace Handling

### Default Behavior

| Type | Behavior | Example |
|------|----------|---------|
| `String` | Trimmed, non-empty | `"  hello  "` → `"hello"` ✅<br>`"   "` → Error ❌ |
| `Option<String>` | Allows empty | `"   "` → `Some("")` ✅<br>`None` → `None` ✅ |
| `#[allow_whitespace] String` | Keep spaces, but required | `"  hello  "` → `"  hello  "` ✅<br>`""` → Error ❌ |

### Examples

```rust
#[derive(Validate)]
struct CreatePostRequest {
    // Auto-trimmed, must be non-empty after trimming
    title: String,

    // Optional, allows empty/whitespace
    description: Option<String>,

    // Preserve whitespace (for poems, code, etc.)
    #[allow_whitespace]
    content: String,
}
```

**Form submission:**

```json
{
  "title": "  My Title  ",      // → "My Title"
  "description": "   ",          // → Some("")
  "content": "  Code block  "    // → "  Code block  " (preserved)
}
```

---

## Complete Examples

### Simple CRUD

```rust
// pages/users/page.rhtml

use rhtml_macro::Validate;
use serde::{Deserialize, Serialize};

#[derive(Validate, Deserialize)]
struct CreateUserRequest {
    name: String,

    #[email]
    #[no_public_domains]
    email: String,

    #[password("strong")]
    password: String,

    #[min(18)] #[max(120)]
    age: i32,

    bio: Option<String>,
}

#[derive(Validate, Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
    age: Option<i32>,
}

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    age: i32,
}

// ===== ACTIONS =====

// LIST: GET /users
fn get_users() -> Result<Vec<User>, String> {
    Ok(db.get_users()?)
}

// CREATE: POST /users
fn post_user(req: CreateUserRequest) -> Result<User, String> {
    let user = db.create_user(req)?;
    Ok(user)
        .toast("User created!")
        .oob("user-count", db.count_users()?)
}

// UPDATE: PUT /users/:id
fn put_user(id: i32, req: UpdateUserRequest) -> Result<User, String> {
    let user = db.update_user(id, req)?;
    Ok(user).toast("User updated!")
}

// PARTIAL UPDATE: PATCH /users/:id
fn patch_user(id: i32, req: UpdateUserRequest) -> Result<User, String> {
    let user = db.patch_user(id, req)?;
    Ok(user).toast("Changes saved!")
}

// DELETE: DELETE /users/:id
fn delete_user(id: i32) -> Result<Empty, String> {
    db.delete_user(id)?;
    Ok(Empty::new()
        .toast("User deleted!")
        .oob("user-count", db.count_users()?))
}

#[webpage]
pub fn page() {
    <div class="container">
        <h1>Users</h1>

        <div id="user-count">
            Total: {db.count_users()}
        </div>

        <!-- Create Form -->
        <form hx-post="/users" hx-target="#user-list">
            <input name="name" required />
            <input name="email" type="email" required />
            <input name="password" type="password" required />
            <input name="age" type="number" required />
            <textarea name="bio"></textarea>
            <button>Create User</button>
        </form>

        <!-- User List -->
        <div id="user-list">
            <div r-for="user in db.get_users()">
                <div class="user-card">
                    <h3>{user.name}</h3>
                    <p>{user.email}</p>
                    <button hx-delete="/users/{user.id}">Delete</button>
                </div>
            </div>
        </div>
    </div>
}
```

### Validation Error Rendering

When validation fails, the framework automatically re-renders the form with errors:

```html
<form hx-post="/users/register" hx-target="this" hx-swap="outerHTML">
    <input name="name" value="" />
    <span class="error">Name is required</span>

    <input name="email" value="test@gmail.com" />
    <span class="error">Public email domains not allowed</span>

    <input name="password" value="weak" />
    <span class="error">Password must be at least 8 characters with uppercase, lowercase, number, and special character</span>

    <input name="age" value="15" />
    <span class="error">Must be at least 18</span>

    <button>Register</button>
</form>
```

### Bulk Operations

```rust
#[derive(Validate, Deserialize)]
struct BulkDeleteRequest {
    #[form]
    ids: Vec<i32>,
}

fn delete_users(req: BulkDeleteRequest) -> Result<Empty, String> {
    db.delete_users(req.ids)?;
    Ok(Empty::new()
        .toast(format!("{} users deleted", req.ids.len()))
        .oob("user-count", db.count_users()?))
}
```

---

## Usage in Templates

### Accessing Validation Errors

```html
<form hx-post="/users">
    <input name="email" value="{form_email}" />
    <span r-if="form.has_error('email')" class="error">
        {form.get_error('email')}
    </span>
</form>
```

### Conditional Rendering Based on Validation

```html
<div r-if="form.has_errors()" class="alert alert-danger">
    Please fix the errors below
</div>
```

---

## Implementation Details

### How It Works

1. **Request arrives** with form data
2. **FormData parses** and automatically trims string values
3. **Validation runs** using `#[derive(Validate)]` trait
4. **If validation fails**:
   - Errors are stored in `FormData.validation_errors`
   - Form is re-rendered with errors and original values
5. **If validation succeeds**:
   - Action function executes
   - Response is sent with optional toast/OOB updates

### Type Safety

All validation happens at compile-time (macros) and runtime (validators):

```rust
// Compile-time: Macro generates validation code
#[derive(Validate)]
struct User {
    #[email] email: String,
}

// Runtime: Validation runs on form submission
let result = form.parse::<User>();
match result {
    Ok(user) => { /* validated user */ },
    Err(errors) => { /* show errors */ },
}
```

---

## Best Practices

1. **Always use `#[derive(Validate)]`** on request structs
2. **Use `Option<T>` for optional fields** (allows empty values)
3. **Use `#[allow_whitespace]`** only when you need to preserve formatting
4. **Prefer specific validators** (`#[email]`, `#[url]`) over regex when possible
5. **Use toast messages** to provide user feedback
6. **Use OOB updates** to keep UI in sync without full page reload

---

## Future Enhancements

- [ ] Custom validator functions
- [ ] Async validators (e.g., check if email exists)
- [ ] Cross-field validation (e.g., password confirmation)
- [ ] File upload validation
- [ ] Array/nested object validation
- [ ] Conditional validation rules
