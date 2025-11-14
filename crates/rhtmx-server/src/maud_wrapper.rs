/// Maud integration for RHTMX
///
/// This module provides seamless integration between Maud templating engine
/// and RHTMX response builders (Ok, Error, Redirect).
///
/// Maud is a compile-time HTML templating library for Rust with a Lisp-like syntax.
/// This wrapper allows you to use Maud's powerful templating alongside RHTMX's
/// response builders and file-based routing.
use rhtmx::html::Html;

/// Convert Maud's Markup to RHTMX's Html type
#[allow(dead_code)]
pub fn maud_to_html(markup: maud::Markup) -> Html {
    Html(markup.into_string())
}

/// Trait to enable seamless conversion from Maud markup to Html
#[allow(dead_code)]
pub trait MaudMarkup {
    fn to_html(self) -> Html;
}

impl MaudMarkup for maud::Markup {
    fn to_html(self) -> Html {
        maud_to_html(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maud_to_html_basic() {
        let markup = maud::html! {
            div { "Hello, World!" }
        };

        let html = maud_to_html(markup);
        assert!(html.as_str().contains("Hello, World!"));
    }

    #[test]
    fn test_maud_markup_trait() {
        let markup = maud::html! {
            div { "Test" }
        };

        let html = markup.to_html();
        assert!(html.as_str().contains("Test"));
    }

    #[test]
    fn test_maud_with_nested_elements() {
        let markup = maud::html! {
            section {
                h1 { "Header" }
                p { "Content" }
                footer { "Footer" }
            }
        };

        let html = maud_to_html(markup);
        let html_str = html.as_str();

        assert!(html_str.contains("<section>"));
        assert!(html_str.contains("<h1>Header</h1>"));
        assert!(html_str.contains("<p>Content</p>"));
        assert!(html_str.contains("</section>"));
    }

    #[test]
    fn test_maud_with_complex_content() {
        let markup = maud::html! {
            div {
                h2 { "Title" }
                p { "Paragraph 1" }
                p { "Paragraph 2" }
            }
        };

        let html = maud_to_html(markup);
        let html_str = html.as_str();

        assert!(html_str.contains("<h2>Title</h2>"));
        assert!(html_str.contains("<p>Paragraph 1</p>"));
        assert!(html_str.contains("<p>Paragraph 2</p>"));
    }

    #[test]
    fn test_maud_with_interpolation() {
        let name = "Alice";
        let count = 42;

        let markup = maud::html! {
            div {
                p { (name) }
                p { (count) }
            }
        };

        let html = maud_to_html(markup);
        let html_str = html.as_str();

        assert!(html_str.contains("Alice"));
        assert!(html_str.contains("42"));
    }

    #[test]
    fn test_maud_with_loop() {
        let items = vec!["A", "B", "C"];

        let markup = maud::html! {
            ul {
                @for item in &items {
                    li { (item) }
                }
            }
        };

        let html = maud_to_html(markup);
        let html_str = html.as_str();

        assert!(html_str.contains("<li>A</li>"));
        assert!(html_str.contains("<li>B</li>"));
        assert!(html_str.contains("<li>C</li>"));
    }

    #[test]
    fn test_maud_with_conditional() {
        let show = true;

        let markup = maud::html! {
            div {
                @if show {
                    p { "Visible" }
                }
            }
        };

        let html = maud_to_html(markup);
        assert!(html.as_str().contains("Visible"));
    }

    #[test]
    fn test_maud_escaping() {
        let user_input = "<script>alert('xss')</script>";

        let markup = maud::html! {
            div { (user_input) }
        };

        let html = maud_to_html(markup);
        let html_str = html.as_str();

        // Should be escaped
        assert!(html_str.contains("&lt;script&gt;"));
        assert!(!html_str.contains("<script>"));
    }
}
