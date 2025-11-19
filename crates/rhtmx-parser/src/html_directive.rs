// src/directives/html_directive.rs

use crate::types::*;
use quote::{quote, TokenStream};
use syn::{parse_str, Expr};

/// Extracts a quoted value from HTML attributes using functional patterns.
///
/// # Functional Approach
/// - Uses method chaining with `and_then` and `map` for composable transformations
/// - Avoids manual index arithmetic and mutation
/// - Returns `Option<String>` for explicit error handling
///
/// # Examples
/// ```
/// let attrs = r#"class="btn" r-html="content" id="main""#;
/// let value = extract_quoted_value(attrs, "r-html");
/// assert_eq!(value, Some("content".to_string()));
/// ```
fn extract_quoted_value(attributes: &str, directive: &str) -> Option<String> {
    attributes
        .find(&format!("{}=\"", directive))
        .and_then(|start| {
            let value_start = start + directive.len() + 2; // Skip directive="
            attributes[value_start..]
                .find('"')
                .map(|end| attributes[value_start..value_start + end].to_string())
        })
}

pub fn process_html_directive(expression: &str, context: &CodegenContext) -> TokenStream {
    match parse_str::<Expr>(expression) {
        Ok(expr) => {
            quote! {
                // WARNING: This does NOT escape HTML
                // Only use with trusted content!
                output.push_str(&format!("{}", #expr));
            }
        }
        Err(e) => {
            let error_msg = format!("Invalid r-html expression: {}", e);
            quote! {
                compile_error!(#error_msg);
            }
        }
    }
}

/// Extracts the r-html directive value from HTML attributes.
///
/// Uses functional pattern matching instead of manual string manipulation.
///
/// # Examples
/// ```
/// let attrs = r#"<div r-html="post.content_html" class="content">"#;
/// assert_eq!(extract_html_directive(attrs), Some("post.content_html".to_string()));
/// ```
pub fn extract_html_directive(attributes: &str) -> Option<String> {
    extract_quoted_value(attributes, "r-html")
}

// Usage
/*
<!-- Example 1: Render markdown as HTML -->

 // I am not sure how this work here. Maybe coding error.
    let post = db::get_post(slug)?;
   html! {
    content_html: markdown::to_html(&post.content_markdown)
   }



  // another example
    <article r-match="props.data">
        <div r-when="Ok(post)">
            <h1>{post.title}</h1>

            <!-- This is escaped (safe) -->
            <p>{post.excerpt}</p>

            <!-- This is NOT escaped (renders HTML) -->
            <div class="content" r-html="{post.content_html}">
            </div>
        </div>
    </article>
}


*/
