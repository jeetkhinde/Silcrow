// src/directives/show_directive.rs

use crate::types::*;
use quote::{quote, TokenStream};
use syn::{parse_str, Expr};

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

pub fn extract_show_directive(attributes: &str) -> Option<String> {
    if let Some(start) = attributes.find(r#"r-show=""#) {
        let start = start + 8; // length of 'r-show="'
        if let Some(end) = attributes[start..].find('"') {
            return Some(attributes[start..start + end].to_string());
        }
    }
    None
}
