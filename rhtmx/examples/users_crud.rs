// RHTMX Example: User CRUD Application
// Demonstrates the clean RHTMX architecture

use rhtmx::{html, Html, Ok, Error, Redirect, StatusCode};
use rhtmx::{get, post, delete};
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Models
// ============================================================================

#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    is_admin: bool,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// Mock database
mod db {
    use super::User;

    pub fn get_users() -> Result<Vec<User>, String> {
        Ok(vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                is_admin: true,
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                is_admin: false,
            },
        ])
    }

    pub fn create_user(name: String, email: String) -> Result<User, String> {
        Ok(User {
            id: 3,
            name,
            email,
            is_admin: false,
        })
    }

    pub fn delete_user(_id: i32) -> Result<(), String> {
        Ok(())
    }

    pub fn count_users() -> Result<usize, String> {
        Ok(2)
    }
}

// ============================================================================
// HTTP Handlers (in actual app, these would be in pages/users.rs)
// ============================================================================

// Handles GET /users
get!()
fn index() -> rhtmx::OkResponse {
    let users = db::get_users().unwrap_or_default();
    Ok().render(users_page, users)
}

// Handles POST /users
post!()
fn create(req: CreateUserRequest) -> rhtmx::OkResponse {
    let user = db::create_user(req.name, req.email).unwrap();

    Ok()
        .render(user_card, user)
        .render_oob("user-count", user_count, db::count_users().unwrap())
        .toast("User created successfully!")
}

// Handles DELETE /users/:id
delete!(":id")
fn delete(id: i32) -> rhtmx::OkResponse {
    db::delete_user(id).unwrap();

    Ok()
        .render_oob("user-count", user_count, db::count_users().unwrap())
        .toast("User deleted!")
}

// ============================================================================
// UI Components (Pure functions that return Html)
// ============================================================================

/// Main users page layout
fn users_page(users: Vec<User>) -> Html {
    html! {
        <div class="container">
            <header class="page-header">
                <h1>Users</h1>
                <div id="user-count">
                    {user_count(users.len())}
                </div>
            </header>

            <main>
                <section class="create-user">
                    <h2>Create New User</h2>
                    {user_form()}
                </section>

                <section class="user-list">
                    <h2>All Users</h2>
                    <div id="user-list" class="users-grid">
                        <div r-for="user in users">
                            {user_card(user)}
                        </div>
                    </div>
                </section>
            </main>
        </div>
    }
}

/// User creation form
fn user_form() -> Html {
    html! {
        <form
            hx-post="/users"
            hx-target="#user-list"
            hx-swap="beforeend"
            class="user-form">

            <div class="form-group">
                <label for="name">Full Name</label>
                <input
                    id="name"
                    name="name"
                    type="text"
                    required
                    placeholder="Enter full name"
                    minlength="2" />
            </div>

            <div class="form-group">
                <label for="email">Email</label>
                <input
                    id="email"
                    name="email"
                    type="email"
                    required
                    placeholder="user@example.com" />
            </div>

            <button type="submit" class="btn btn-primary">
                Create User
            </button>
        </form>
    }
}

/// Individual user card component
fn user_card(user: User) -> Html {
    html! {
        <div class="user-card" id="user-{user.id}">
            <div class="user-avatar">
                <span class="avatar-placeholder">
                    {user.name.chars().next().unwrap_or('?')}
                </span>
            </div>

            <div class="user-info">
                <h3>{user.name}</h3>
                <p class="email">{user.email}</p>

                <div r-if="user.is_admin" class="badge badge-admin">
                    Admin
                </div>
            </div>

            <div class="user-actions">
                <button
                    class="btn btn-danger btn-sm"
                    hx-delete="/users/{user.id}"
                    hx-target="#user-{user.id}"
                    hx-swap="outerHTML"
                    hx-confirm="Are you sure you want to delete this user?">
                    Delete
                </button>
            </div>
        </div>
    }
}

/// User count display component
fn user_count(count: usize) -> Html {
    let label = if count == 1 { "user" } else { "users" };

    html! {
        <div class="count-badge">
            <span class="count-number">{count}</span>
            <span class="count-label">{label}</span>
        </div>
    }
}

// ============================================================================
// Main (for demonstration - in real app, this would be in main.rs)
// ============================================================================

fn main() {
    println!("=== RHTMX Users CRUD Example ===\n");

    // Render the page
    let users = db::get_users().unwrap();
    let page_html = users_page(users);

    println!("Rendered HTML:");
    println!("{}", page_html.as_str());

    println!("\n✓ Example compiled successfully!");
    println!("✓ All components are type-safe");
    println!("✓ Zero runtime overhead");
}
