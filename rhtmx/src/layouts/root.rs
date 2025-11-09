// Root Layout
// The main layout used across most pages

use crate::Html;

/// Slots for the root layout
///
/// Use this struct to pass data to the layout.
/// All slots except `title` are optional.
#[derive(Clone)]
pub struct Slots {
    /// Page title (required)
    pub title: String,

    /// Meta description (optional)
    pub description: Option<String>,

    /// Custom header content (optional)
    /// If not provided, uses default navigation
    pub header: Option<Html>,

    /// Custom footer content (optional)
    /// If not provided, uses default footer
    pub footer: Option<Html>,

    /// Additional <head> content (optional)
    /// For custom meta tags, scripts, etc.
    pub head_extra: Option<Html>,
}

impl Slots {
    /// Create slots with just a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            header: None,
            footer: None,
            head_extra: None,
        }
    }

    /// Builder method to set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Builder method to set custom header
    pub fn header(mut self, header: Html) -> Self {
        self.header = Some(header);
        self
    }

    /// Builder method to set custom footer
    pub fn footer(mut self, footer: Html) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Builder method to set extra head content
    pub fn head_extra(mut self, extra: Html) -> Self {
        self.head_extra = Some(extra);
        self
    }
}

/// Root layout function
///
/// Wraps page content in a full HTML document with navigation and footer.
///
/// # Example
///
/// ```ignore
/// use rhtmx::layouts::root::{layout, Slots};
///
/// #[get]
/// fn index() -> OkResponse {
///     let content = html! { <h1>"Home"</h1> };
///
///     Ok().html(layout(content, Slots::new("Home Page")
///         .description("Welcome to my app")))
/// }
/// ```
pub fn layout(content: Html, slots: Slots) -> Html {
    let header = slots.header.unwrap_or_else(default_header);
    let footer = slots.footer.unwrap_or_else(default_footer);

    let meta_desc = if let Some(desc) = slots.description {
        format!(r#"<meta name="description" content="{}" />"#, desc)
    } else {
        String::new()
    };

    let head_extra = slots.head_extra.map(|h| h.0).unwrap_or_default();

    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>{}</title>
        {}
        <script src="https://unpkg.com/htmx.org@1.9.10"></script>
        {}
    </head>
    <body>
        {}
        <main>{}</main>
        {}
    </body>
</html>"#, slots.title, meta_desc, head_extra, header.0, content.0, footer.0))
}

/// Default header/navigation
fn default_header() -> Html {
    Html(r#"<nav class="navbar">
    <div class="container">
        <a href="/">Home</a>
        <a href="/about">About</a>
        <a href="/contact">Contact</a>
    </div>
</nav>"#.into())
}

/// Default footer
fn default_footer() -> Html {
    Html(r#"<footer class="footer">
    <div class="container">
        <p>© 2024 RHTMX App. All rights reserved.</p>
    </div>
</footer>"#.into())
}
