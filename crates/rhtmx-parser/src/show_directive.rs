// src/directives/show_directive.rs

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
/// let attrs = r#"class="btn" r-show="isVisible" id="main""#;
/// let value = extract_quoted_value(attrs, "r-show");
/// assert_eq!(value, Some("isVisible".to_string()));
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

pub fn process_show_directive(
    condition: &str,
    element: &HtmlNode,
    context: &CodegenContext,
) -> TokenStream {
    // Parse condition
    match parse_str::<Expr>(condition) {
        Ok(expr) => {
            let element_code = super::generate_html_node(element, context);

            quote! {
                // Generate element with conditional display style
                let show_condition = #expr;
                if !show_condition {
                    output.push_str(" style=\"display: none;\"");
                }
                #element_code
            }
        }
        Err(e) => {
            let error_msg = format!("Invalid r-show condition: {}", e);
            quote! {
                compile_error!(#error_msg);
            }
        }
    }
}

/// Extracts the r-show directive value from HTML attributes.
///
/// Uses functional pattern matching instead of manual string manipulation.
///
/// # Examples
/// ```
/// let attrs = r#"<div r-show="user.active" class="panel">"#;
/// assert_eq!(extract_show_directive(attrs), Some("user.active".to_string()));
/// ```
pub fn extract_show_directive(attributes: &str) -> Option<String> {
    extract_quoted_value(attributes, "r-show")
}
