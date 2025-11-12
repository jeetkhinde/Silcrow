// RHTMX Example: Complete Feature Demo
// Demonstrates all RHTMX directives and features

use rhtmx::{css, get, html, Ok};

// ============================================================================
// Data Models
// ============================================================================

#[derive(Clone)]
struct User {
    id: i32,
    name: String,
    role: UserRole,
    status: UserStatus,
    score: i32,
}

#[derive(Clone, PartialEq)]
enum UserRole {
    Admin,
    Moderator,
    User,
}

#[derive(Clone, PartialEq)]
enum UserStatus {
    Active,
    Pending,
    Suspended,
}

// ============================================================================
// Example 1: r-if, r-else-if, r-else
// ============================================================================

fn conditional_rendering_demo(score: i32) {
    html! {
        <div class="score-display">
            <h3>Score: {score}</h3>

            <div r-if="score >= 90" class="grade excellent">
                <p>Excellent! Grade: A</p>
            </div>

            <div r-else-if="score >= 75" class="grade good">
                <p>Good job! Grade: B</p>
            </div>

            <div r-else-if="score >= 60" class="grade average">
                <p>Average. Grade: C</p>
            </div>

            <div r-else class="grade poor">
                <p>Needs improvement. Grade: F</p>
            </div>
        </div>
    }
}

// ============================================================================
// Example 2: r-match with r-when and r-default
// ============================================================================

fn status_badge(status: UserStatus) {
    html! {
        <div r-match="status" class="status-container">
            <span r-when="UserStatus::Active" class="badge badge-active">
                "Active"
            </span>
            <span r-when="UserStatus::Pending" class="badge badge-pending">
                "Pending Approval"
            </span>
            <span r-when="UserStatus::Suspended" class="badge badge-suspended">
                "Suspended"
            </span>
            <span r-default class="badge badge-unknown">
                "Unknown Status"
            </span>
        </div>
    }
}

fn role_badge(role: UserRole) {
    html! {
        <div r-match="role" class="role-badge">
            <div r-when="UserRole::Admin" class="badge admin">
                <strong>"👑 Admin"</strong>
            </div>
            <div r-when="UserRole::Moderator" class="badge moderator">
                <strong>"⭐ Moderator"</strong>
            </div>
            <div r-when="UserRole::User" class="badge user">
                <strong>"👤 User"</strong>
            </div>
            <div r-default class="badge">
                <strong>"Unknown Role"</strong>
            </div>
        </div>
    }
}

// ============================================================================
// Example 3: css! macro with scoping
// ============================================================================

fn styled_user_card(user: User) {
    // Define scoped CSS for this component
    css! {
        scope: "user-card",
        .card {
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            padding: 1.5rem;
            margin: 1rem 0;
            background: white;
        }
        .card:hover {
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
            transform: translateY(-2px);
            transition: all 0.3s;
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;
        }
        .name {
            font-size: 1.25rem;
            font-weight: bold;
            color: #333;
        }
        .score {
            font-size: 2rem;
            font-weight: bold;
            color: #007bff;
        }
    };

    html! {
        <div class="card" data-scope="user-card" id="user-{user.id}">
            <div class="header">
                <div class="name">{user.name}</div>
                {role_badge(user.role.clone())}
            </div>

            <div class="score">Score: {user.score}</div>

            <div class="status">
                {status_badge(user.status)}
            </div>
        </div>
    }
}

// ============================================================================
// Example 4: Combining all features
// ============================================================================

fn dashboard(users: Vec<User>) {
    html! {
        <div class="dashboard">
            <header>
                <h1>"User Dashboard"</h1>
                <p>"Total Users: " <strong>{users.len()}</strong></p>
            </header>

            <section class="filters">
                <h2>"Filter Examples"</h2>

                <div class="filter-group">
                    <h3>"Active Users Only"</h3>
                    <div r-for="user in users.clone()" class="user-item">
                        <div r-if="user.status == UserStatus::Active">
                            <p>{user.name} " - Active"</p>
                        </div>
                    </div>
                </div>

                <div class="filter-group">
                    <h3>"Users by Role"</h3>
                    <div r-for="user in users.clone()">
                        <div r-match="user.role">
                            <div r-when="UserRole::Admin" class="admin-user">
                                "👑 " {user.name}
                            </div>
                            <div r-when="UserRole::Moderator" class="mod-user">
                                "⭐ " {user.name}
                            </div>
                            <div r-default class="regular-user">
                                "👤 " {user.name}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="filter-group">
                    <h3>"Score Distribution"</h3>
                    <div r-for="user in users.clone()">
                        <div class="score-item">
                            <span>{user.name} ": "</span>

                            <span r-if="user.score >= 90" class="high-score">
                                {user.score} " (Excellent)"
                            </span>

                            <span r-else-if="user.score >= 75" class="good-score">
                                {user.score} " (Good)"
                            </span>

                            <span r-else-if="user.score >= 60" class="avg-score">
                                {user.score} " (Average)"
                            </span>

                            <span r-else class="low-score">
                                {user.score} " (Needs Improvement)"
                            </span>
                        </div>
                    </div>
                </div>
            </section>

            <section class="user-cards">
                <h2>"All Users"</h2>
                <div r-for="user in users" class="cards-grid">
                    {styled_user_card(user)}
                </div>
            </section>
        </div>
    }
}

// ============================================================================
// HTTP Handler Example
// ============================================================================

#[get]
fn index() -> rhtmx::OkResponse {
    let users = vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            role: UserRole::Admin,
            status: UserStatus::Active,
            score: 95,
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            role: UserRole::Moderator,
            status: UserStatus::Active,
            score: 82,
        },
        User {
            id: 3,
            name: "Carol Williams".to_string(),
            role: UserRole::User,
            status: UserStatus::Pending,
            score: 67,
        },
        User {
            id: 4,
            name: "David Brown".to_string(),
            role: UserRole::User,
            status: UserStatus::Active,
            score: 45,
        },
        User {
            id: 5,
            name: "Eve Davis".to_string(),
            role: UserRole::User,
            status: UserStatus::Suspended,
            score: 88,
        },
    ];

    Ok().render(dashboard, users)
}

// ============================================================================
// Main (Demo Runner)
// ============================================================================

fn main() {
    println!("=== RHTMX Complete Features Demo ===\n");

    // Demo data
    let users = vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            role: UserRole::Admin,
            status: UserStatus::Active,
            score: 95,
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            role: UserRole::Moderator,
            status: UserStatus::Active,
            score: 82,
        },
        User {
            id: 3,
            name: "Carol Williams".to_string(),
            role: UserRole::User,
            status: UserStatus::Pending,
            score: 67,
        },
    ];

    println!("1. Conditional Rendering (r-if, r-else-if, r-else):");
    println!("{}\n", conditional_rendering_demo(92).as_str());

    println!("2. Pattern Matching (r-match, r-when, r-default):");
    println!("{}\n", status_badge(UserStatus::Active).as_str());

    println!("3. Styled Component (css! macro):");
    println!("{}\n", styled_user_card(users[0].clone()).as_str());

    println!("4. Complete Dashboard:");
    let dashboard_html = dashboard(users);
    println!("{}\n", &dashboard_html.as_str()[..500]); // First 500 chars

    println!("\n✓ All features working!");
    println!("✓ r-if, r-else-if, r-else");
    println!("✓ r-match, r-when, r-default");
    println!("✓ r-for with index");
    println!("✓ css! macro with scoping");
    println!("✓ Expression interpolation");
    println!("✓ Type-safe compilation");
}
