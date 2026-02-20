// silcrow/crates/silcrow/src/layout.rs — Silcrow server-side HTML layout utilities
// Not sure if it is needed when we can use simple rust function with Axum to build the layout, but it is a nice convenience to have a simple function to render a full HTML page with the Silcrow script included by default. This is especially useful for simple applications that don't need a complex layout system.
use maud::{html, Markup, DOCTYPE};

/// Renders a full HTML page with the Silcrow script auto-included.
///
/// For custom layouts, use `silcrow::assets::script_tag()` directly instead.
///
/// ```rust
/// use silcrow::layout::page;
/// use maud::html;
///
/// let markup = page("My App", html! {
///     h1 { "Hello" }
/// });
/// ```
pub fn page(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                (crate::assets::script_tag())
            }
            body {
                (body)
            }
        }
    }
}
