// Example: RHTMX with Maud! Macro
//
// This example demonstrates the maud! macro integrated with RHTMX's
// response builders and layouts. It shows various patterns and features.

use rhtmx::{get, post, Ok, OkResponse, FormContext, maud};

// ============================================================================
// Basic Maud! Example
// ============================================================================

/// Simple page using maud! macro
get!()
fn simple_page() -> OkResponse {
    let title = "Welcome to RHTMX + Maud";
    let message = "This page uses the maud! macro for HTML generation.";

    let html = maud! {
        div.container {
            h1 { (title) }
            p { (message) }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Classes and IDs Example
// ============================================================================

/// Demonstrates class and ID syntax
get!()
fn styled_components() -> OkResponse {
    let html = maud! {
        div.hero.dark#main-hero {
            h1.title { "Featured Section" }
            p.subtitle { "This uses Maud's compact syntax" }
            button.btn.btn-primary[type="button"] { "Click Me" }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Attributes Example
// ============================================================================

/// Shows various attribute patterns
get!()
fn attributes_demo() -> OkResponse {
    let username = "alice";
    let user_id = 42;

    let html = maud! {
        form.login-form[method="post"] {
            input[
                type="text"
                name="username"
                placeholder="Username"
                value=(username)
                data-field="username"
            ]

            input[
                type="password"
                name="password"
                placeholder="Password"
                required=""
            ]

            button[
                type="submit"
                class="btn btn-primary"
                data-user-id=(user_id)
            ] { "Login" }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Interpolation Example
// ============================================================================

/// Shows expression interpolation
get!()
fn interpolation_demo() -> OkResponse {
    let user_name = "Alice";
    let user_email = "alice@example.com";
    let post_count = 42;
    let is_verified = true;

    let html = maud! {
        div.user-profile {
            h2 { (user_name) }
            p { "Email: " (user_email) }
            p { "Posts: " (post_count) }

            @if is_verified {
                span.badge { "✓ Verified" }
            }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Conditionals Example
// ============================================================================

/// Shows @if and @else patterns
get!()
fn conditionals_demo() -> OkResponse {
    let user_role = "admin";  // Could be "admin", "user", or "guest"
    let is_logged_in = true;

    let html = maud! {
        nav.navbar {
            ul {
                li { a[href="/"] { "Home" } }
                li { a[href="/about"] { "About" } }

                @if is_logged_in {
                    li { a[href="/dashboard"] { "Dashboard" } }

                    @if user_role == "admin" {
                        li { a[href="/admin"] { "Admin Panel" } }
                    } @else if user_role == "moderator" {
                        li { a[href="/moderation"] { "Moderation" } }
                    }

                    li { a[href="/logout"] { "Logout" } }
                } @else {
                    li { a[href="/login"] { "Login" } }
                    li { a[href="/signup"] { "Sign Up" } }
                }
            }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Loops Example
// ============================================================================

/// Shows @for loop patterns
get!()
fn loops_demo() -> OkResponse {
    let items = vec!["Apples", "Bananas", "Cherries"];
    let products = vec![
        ("Laptop", 999),
        ("Mouse", 25),
        ("Keyboard", 75),
    ];

    let html = maud! {
        div {
            h2 { "Simple List" }
            ul {
                @for item in &items {
                    li { (item) }
                }
            }

            h2 { "Product List" }
            table.products-table {
                thead {
                    tr {
                        th { "Product" }
                        th { "Price" }
                    }
                }
                tbody {
                    @for (name, price) in &products {
                        tr {
                            td { (name) }
                            td { "$" (price) }
                        }
                    }
                }
            }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Nested Loops Example
// ============================================================================

/// Shows nested loop patterns
get!()
fn nested_loops_demo() -> OkResponse {
    let teams = vec![
        ("Team A", vec!["Alice", "Bob"]),
        ("Team B", vec!["Carol", "Dave"]),
        ("Team C", vec!["Eve"]),
    ];

    let html = maud! {
        div.teams {
            @for (team_name, members) in &teams {
                section.team {
                    h2 { (team_name) }
                    ul.members {
                        @for member in members {
                            li { (member) }
                        }
                    }
                }
            }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Form Validation Example
// ============================================================================

/// Shows form with validation error handling
get!()
fn form_validation_demo() -> OkResponse {
    let errors = vec!["Email is required", "Password must be at least 8 characters"];
    let email_value = "user@example.com";

    let html = maud! {
        div.form-container {
            @if !errors.is_empty() {
                div.alert.alert-error {
                    h3 { "Errors" }
                    ul {
                        @for error in &errors {
                            li { (error) }
                        }
                    }
                }
            }

            form.signup-form[method="post"] {
                div.form-group {
                    label[for="email"] { "Email" }
                    input[
                        type="email"
                        id="email"
                        name="email"
                        value=(email_value)
                        class=(if errors.contains(&"Email is required") { "input-error" } else { "" })
                    ]
                }

                div.form-group {
                    label[for="password"] { "Password" }
                    input[
                        type="password"
                        id="password"
                        name="password"
                        class=(if errors.iter().any(|e| e.contains("Password")) { "input-error" } else { "" })
                    ]
                }

                button[type="submit"].btn { "Sign Up" }
            }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Response Handler Example
// ============================================================================

/// Shows maud! used in a form handler
post!()
fn create_post(form: FormContext) -> OkResponse {
    let title = form.get("title").unwrap_or("Untitled");
    let content = form.get("content").unwrap_or("");

    let html = maud! {
        div.post-created {
            h2 { "Post Created" }
            div.post {
                h3 { (title) }
                p { (content) }
            }
            a[href="/posts"] { "Back to Posts" }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Layout Integration Example
// ============================================================================

/// Shows maud! integrated with RHTMX layouts
get!()
fn dashboard() -> OkResponse {
    let user_name = "Alice";
    let post_count = 12;
    let follower_count = 345;

    let content = maud! {
        div.dashboard-content {
            h1 { "Dashboard" }

            div.stats {
                div.stat-card {
                    h3 { "Posts" }
                    p.stat-value { (post_count) }
                }
                div.stat-card {
                    h3 { "Followers" }
                    p.stat-value { (follower_count) }
                }
            }

            section.recent-posts {
                h2 { "Recent Posts" }
                ul {
                    li { "Post 1" }
                    li { "Post 2" }
                    li { "Post 3" }
                }
            }
        }
    };

    // In a real app, you'd wrap this with your custom layout function
    Ok().html(content)
}

// ============================================================================
// Component-like Pattern Example
// ============================================================================

/// Helper function that returns HTML
fn user_card(name: &str, email: &str, role: &str) -> String {
    maud! {
        div.card.user-card {
            h3.name { (name) }
            p.email { (email) }
            span.role[class=(
                match role {
                    "admin" => "badge-danger",
                    "mod" => "badge-warning",
                    _ => "badge-info"
                }
            )] { (role) }
        }
    }.into_string()
}

/// Uses the helper component
get!()
fn users_list() -> OkResponse {
    let users = vec![
        ("Alice", "alice@example.com", "admin"),
        ("Bob", "bob@example.com", "user"),
        ("Carol", "carol@example.com", "mod"),
    ];

    let html = maud! {
        div.users-container {
            h1 { "Users" }
            div.users-grid {
                @for (name, email, role) in &users {
                    (user_card(name, email, role))
                }
            }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Complex Real-World Example
// ============================================================================

#[derive(Clone)]
struct BlogPost {
    id: i32,
    title: String,
    excerpt: String,
    content: String,
    author: String,
    date: String,
    tags: Vec<String>,
    is_published: bool,
}

/// Complex example with mixed patterns
get!()
fn blog_posts() -> OkResponse {
    let posts = vec![
        BlogPost {
            id: 1,
            title: "Getting Started with RHTMX".to_string(),
            excerpt: "Learn the basics of RHTMX...".to_string(),
            content: "Full content here...".to_string(),
            author: "Alice".to_string(),
            date: "2024-01-15".to_string(),
            tags: vec!["rust".to_string(), "web".to_string()],
            is_published: true,
        },
        BlogPost {
            id: 2,
            title: "Advanced Maud Patterns".to_string(),
            excerpt: "Deep dive into maud...".to_string(),
            content: "Full content here...".to_string(),
            author: "Bob".to_string(),
            date: "2024-01-14".to_string(),
            tags: vec!["maud".to_string(), "templating".to_string()],
            is_published: true,
        },
    ];

    let content = maud! {
        div.blog-container {
            h1 { "Blog Posts" }

            @for post in &posts {
                article.post-card[data-post-id=(post.id)] {
                    header {
                        h2 { (post.title) }
                        div.meta {
                            span.author { "By " (post.author) }
                            " • "
                            time[datetime=(post.date)] { (post.date) }
                            @if !post.is_published {
                                span.badge.draft { "Draft" }
                            }
                        }
                    }

                    p.excerpt { (post.excerpt) }

                    footer.tags {
                        @for tag in &post.tags {
                            span.tag { (tag) }
                        }
                    }

                    a.read-more[href=("/posts/" (post.id))] { "Read More →" }
                }
            }
        }
    };

    // In a real app, you'd wrap this with your custom layout function
    Ok().html(content)
}

// ============================================================================
// Error Response Example
// ============================================================================

/// Shows error handling with maud!
fn example_error_handler() -> OkResponse {
    let error_message = "Resource not found";
    let error_code = 404;

    let html = maud! {
        div.error-page {
            h1.error-code { (error_code) }
            p.error-message { (error_message) }
            a[href="/"] { "Back to Home" }
        }
    };

    Ok().html(html)
}

// ============================================================================
// Mixed html! and maud! Example
// ============================================================================

use rhtmx::html;

/// Shows using both macros together
get!()
fn mixed_macros() -> OkResponse {
    let header = html! {
        <header class="site-header">
            <h1>"My Site"</h1>
        </header>
    };

    let footer = maud! {
        footer.site-footer {
            p { "© 2024 My Site" }
        }
    };

    let page = maud! {
        html {
            head {
                title { "Page Title" }
            }
            body {
                (header)
                main { "Content here" }
                (footer)
            }
        }
    };

    Ok().html(page)
}
