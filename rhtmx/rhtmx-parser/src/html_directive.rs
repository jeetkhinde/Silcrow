// src/directives/html_directive.rs

use crate::types::*;
use quote::{quote, TokenStream};
use syn::{parse_str, Expr};

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

pub fn extract_html_directive(attributes: &str) -> Option<String> {
    if let Some(start) = attributes.find(r#"r-html=""#) {
        let start = start + 8; // length of 'r-html="'
        if let Some(end) = attributes[start..].find('"') {
            return Some(attributes[start..start + end].to_string());
        }
    }
    None
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
