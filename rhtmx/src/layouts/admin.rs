// Admin Layout
// Layout for admin pages with sidebar navigation

use crate::Html;

/// Slots for the admin layout
#[derive(Clone)]
pub struct Slots {
    /// Page title (required)
    pub title: String,

    /// Sidebar content (optional)
    /// If not provided, uses default admin nav
    pub sidebar: Option<Html>,

    /// Breadcrumbs (optional)
    pub breadcrumbs: Option<Html>,
}

impl Slots {
    /// Create slots with just a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sidebar: None,
            breadcrumbs: None,
        }
    }

    /// Builder method to set custom sidebar
    pub fn sidebar(mut self, sidebar: Html) -> Self {
        self.sidebar = Some(sidebar);
        self
    }

    /// Builder method to set breadcrumbs
    pub fn breadcrumbs(mut self, breadcrumbs: Html) -> Self {
        self.breadcrumbs = Some(breadcrumbs);
        self
    }
}

/// Admin layout function
///
/// Provides a two-column layout with sidebar navigation.
///
/// # Example
///
/// ```ignore
/// use rhtmx::layouts::admin::{layout, Slots};
///
/// #[get]
/// fn dashboard() -> OkResponse {
///     let content = html! { <h1>"Admin Dashboard"</h1> };
///
///     Ok().html(layout(content, Slots::new("Dashboard")))
/// }
/// ```
pub fn layout(content: Html, slots: Slots) -> Html {
    let sidebar = slots.sidebar.unwrap_or_else(default_sidebar);

    let breadcrumbs_html = if let Some(bc) = slots.breadcrumbs {
        format!(r#"<div class="breadcrumbs">{}</div>"#, bc.0)
    } else {
        String::new()
    };

    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>{} - Admin</title>
        <script src="https://unpkg.com/htmx.org@1.9.10"></script>
        <style>
            .admin-layout {{ display: flex; min-height: 100vh; }}
            .sidebar {{ width: 250px; background: #2c3e50; color: white; padding: 1rem; }}
            .main-content {{ flex: 1; padding: 2rem; }}
            .breadcrumbs {{ margin-bottom: 1rem; color: #666; }}
        </style>
    </head>
    <body>
        <div class="admin-layout">
            {}
            <div class="main-content">
                {}
                <main>{}</main>
            </div>
        </div>
    </body>
</html>"#, slots.title, sidebar.0, breadcrumbs_html, content.0))
}

/// Default admin sidebar navigation
fn default_sidebar() -> Html {
    Html(r#"<nav class="sidebar">
    <h2>Admin Panel</h2>
    <ul>
        <li><a href="/admin">Dashboard</a></li>
        <li><a href="/admin/users">Users</a></li>
        <li><a href="/admin/settings">Settings</a></li>
        <li><a href="/admin/reports">Reports</a></li>
    </ul>
</nav>"#.into())
}
