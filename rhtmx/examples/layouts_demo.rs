// Example: Layout System Demo
// Shows how to use layouts with RHTMX

use rhtmx::{html, layouts};

fn main() {
    println!("=== RHTMX Layout System Demo ===\n");

    // Demo 1: Basic root layout
    println!("1. Basic Root Layout:");
    let page1 = basic_page();
    println!("Generated {} characters of HTML\n", page1.0.len());

    // Demo 2: Admin layout
    println!("2. Admin Layout:");
    let page2 = admin_page();
    println!("Generated {} characters of HTML\n", page2.0.len());

    // Demo 3: Page without layout (HTMX partial)
    println!("3. HTMX Partial (No Layout):");
    let page3 = htmx_partial();
    println!("{}\n", page3.0);

    println!("=== Layout Demo Complete ===");
    println!("\nLayout system features:");
    println!("✓ Root layout with customizable slots");
    println!("✓ Admin layout with sidebar");
    println!("✓ Builder pattern for slots");
    println!("✓ Default components (header/footer)");
    println!("✓ Support for HTMX partials (no layout)");
}

// ===== Example 1: Basic Root Layout =====

fn basic_page() {
    // In a real app, this would be returned from a #[get] handler
    let content = Html(
        r#"<div class="container">
    <h1>Welcome to RHTMX</h1>
    <p>This page uses the default root layout.</p>
</div>"#
            .into(),
    );

    layouts::root::layout(content, layouts::root::Slots::new("Home Page"))
}

// ===== Example 2: Root Layout with Custom Slots =====

#[allow(dead_code)]
fn custom_slots_page() -> Html {
    let content = Html(
        r#"<div class="container">
    <h1>About Us</h1>
    <p>Learn more about our company.</p>
</div>"#
            .into(),
    );

    // Custom header
    let custom_header = Html(
        r#"<nav class="custom-nav">
    <div class="container">
        <a href="/">Home</a>
        <a href="/about" class="active">About</a>
        <a href="/products">Products</a>
        <a href="/contact">Contact</a>
    </div>
</nav>"#
            .into(),
    );

    // Custom footer
    let custom_footer = Html(
        r#"<footer class="custom-footer">
    <div class="container">
        <p>© 2024 RHTMX Inc.</p>
    </div>
</footer>"#
            .into(),
    );

    layouts::root::layout(
        content,
        layouts::root::Slots::new("About Us")
            .description("Learn more about our company and mission")
            .header(custom_header)
            .footer(custom_footer),
    )
}

// ===== Example 3: Admin Layout =====

fn admin_page() -> Html {
    let content = Html(
        r#"<div class="dashboard">
    <h1>Admin Dashboard</h1>
    <div class="stats">
        <div class="stat-card">
            <h3>Users</h3>
            <p class="stat-number">1,234</p>
        </div>
        <div class="stat-card">
            <h3>Revenue</h3>
            <p class="stat-number">$45,678</p>
        </div>
    </div>
</div>"#
            .into(),
    );

    layouts::admin::layout(content, layouts::admin::Slots::new("Dashboard"))
}

// ===== Example 4: HTMX Partial (No Layout) =====

fn htmx_partial() -> Html {
    // For HTMX partials, return just the HTML fragment
    // No layout needed - HTMX will swap this into the page
    Html(
        r#"<div id="user-list">
    <div class="user-item">
        <span>Alice</span>
        <button hx-delete="/api/users/1">Delete</button>
    </div>
    <div class="user-item">
        <span>Bob</span>
        <button hx-delete="/api/users/2">Delete</button>
    </div>
</div>"#
            .into(),
    )
}

// ===== Example Showing Real Handler Pattern =====

/// Example of how layouts would be used in actual route handlers
#[allow(dead_code)]
mod handlers {
    use rhtmx::{layouts, Html};

    pub fn index_handler() -> Html {
        let content = Html(
            r#"<div class="hero">
    <h1>Welcome to RHTMX</h1>
    <p>Build fast, reactive web apps with Rust + HTMX</p>
    <a href="/docs" class="btn">Get Started</a>
</div>"#
                .into(),
        );

        layouts::root::layout(
            content,
            layouts::root::Slots::new("RHTML - Rust + HTMX Framework"),
        )
    }

    pub fn about_handler() -> Html {
        let content = Html(
            r#"<article>
    <h1>About RHTMX</h1>
    <p>RHTMX combines the power of Rust with the simplicity of HTMX.</p>
</article>"#
                .into(),
        );

        layouts::root::layout(
            content,
            layouts::root::Slots::new("About - RHTMX").description("Learn about RHTMX framework"),
        )
    }

    pub fn admin_dashboard() -> Html {
        let content = Html(
            r#"<div>
    <h1>Dashboard</h1>
    <p>Admin analytics and stats</p>
</div>"#
                .into(),
        );

        let breadcrumbs = Html(
            r#"<nav>
    <a href="/admin">Admin</a> / <span>Dashboard</span>
</nav>"#
                .into(),
        );

        layouts::admin::layout(
            content,
            layouts::admin::Slots::new("Dashboard").breadcrumbs(breadcrumbs),
        )
    }
}
