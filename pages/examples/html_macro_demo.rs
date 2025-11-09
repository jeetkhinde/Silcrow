// File: examples/html_macro_demo.rs
// Purpose: Demonstrate the html! macro functionality

use rhtml::html::*;
use rhtml_macro::html;

// User struct for examples
#[derive(Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
    is_admin: bool,
}

/// Example 1: Simple HTML generation
fn simple_card() {
    let result = html! {
        <div class="card">
            <h1>Hello, World!</h1>
            <p>This is a simple example</p>
        </div>
    };

    println!("Simple card:\n{}", result.as_str());
}

/// Example 2: Expression interpolation
fn user_card(user: &User) {
    let result = html! {
        <div class="user-card" id="user-{user.id}">
            <h3>{user.name}</h3>
            <p>{user.email}</p>
        </div>
    };

    println!("User card:\n{}", result.as_str());
}

/// Example 3: r-for directive
fn users_list(users: Vec<User>) {
    let result = html! {
        <div class="users-list">
            <h2>Users</h2>
            <div r-for="user in users" class="user-item">
                <p>{user.name} - {user.email}</p>
            </div>
        </div>
    };

    println!("Users list:\n{}", result.as_str());
}

/// Example 4: r-for with index
fn numbered_list(items: Vec<String>) {
    let result = html! {
        <ol>
            <li r-for="(i, item) in items">
                {i}: {item}
            </li>
        </ol>
    };

    println!("Numbered list:\n{}", result.as_str());
}

/// Example 5: r-if directive
fn conditional_content(user: &User) {
    let result = html! {
        <div>
            <h1>Dashboard</h1>
            <div r-if="user.is_admin" class="admin-panel">
                <p>Admin Controls</p>
            </div>
        </div>
    };

    println!("Conditional content:\n{}", result.as_str());
}

/// Example 6: Using with Ok() response
fn create_user_response(user: User) -> OkResponse {
    Ok()
        .render(user_card, &user)
        .toast("User created successfully!")
}

/// Example 7: Using with Error() response
fn validation_error_response(errors: Vec<String>) -> ErrorResponse {
    Error()
        .render(validation_errors, errors)
        .status(axum::http::StatusCode::BAD_REQUEST)
}

fn validation_errors(errors: Vec<String>) {
    html! {
        <div class="errors">
            <h3>Validation Errors:</h3>
            <ul>
                <li r-for="error in errors">
                    {error}
                </li>
            </ul>
        </div>
    }
}

/// Example 8: Complex nested structure
fn dashboard(user: &User, stats: &Stats) {
    let result = html! {
        <div class="dashboard">
            <header>
                <h1>Welcome, {user.name}</h1>
            </header>
            <main>
                <div class="stats">
                    <div class="stat-card">
                        <h3>Total Users</h3>
                        <p class="number">{stats.total_users}</p>
                    </div>
                    <div class="stat-card">
                        <h3>Active Sessions</h3>
                        <p class="number">{stats.active_sessions}</p>
                    </div>
                </div>
            </main>
        </div>
    };

    println!("Dashboard:\n{}", result.as_str());
}

struct Stats {
    total_users: i32,
    active_sessions: i32,
}

fn main() {
    println!("=== HTML Macro Examples ===\n");

    // Example 1: Simple
    println!("1. Simple Card:");
    simple_card();
    println!();

    // Example 2: With user data
    println!("2. User Card:");
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        is_admin: true,
    };
    user_card(&user);
    println!();

    // Example 3: r-for loop
    println!("3. Users List:");
    let users = vec![
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
    ];
    users_list(users.clone());
    println!();

    // Example 4: r-for with index
    println!("4. Numbered List:");
    numbered_list(vec![
        "First item".to_string(),
        "Second item".to_string(),
        "Third item".to_string(),
    ]);
    println!();

    // Example 5: r-if conditional
    println!("5. Conditional Content:");
    conditional_content(&user);
    println!();

    // Example 8: Complex dashboard
    println!("8. Complex Dashboard:");
    let stats = Stats {
        total_users: 1250,
        active_sessions: 42,
    };
    dashboard(&user, &stats);
}
