// File: rhtmx-macro/src/http.rs
// Purpose: HTTP verb function macros (get!, post!, put!, patch!, delete!)

use proc_macro::TokenStream;
use quote::quote;

/// HTTP handler function macro
/// Parses combined input (optional path args + function) and generates handler metadata
pub fn http_handler(method: &str, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let (path_args, function_body) = parse_function_macro_input(&input_str);

    // TODO: Use method and path_args to generate route metadata at compile time
    // For now, just return the function as-is
    let _method = method;  // Will be used for route metadata generation
    let _path_args = path_args;  // Will be used for path parameter parsing

    // In a real implementation, this would generate route metadata
    function_body.parse().unwrap_or_else(|_| {
        quote! {
            compile_error!("Failed to parse HTTP handler function");
        }
        .into()
    })
}

/// Parses function macro input format (attribute macro style):
/// - `get!() fn handler() { ... }`
/// - `get!(":id") fn handler(id: i32) { ... }`
///
/// Returns: (Option<path_string>, function_definition)
fn parse_function_macro_input(input_str: &str) -> (Option<String>, String) {
    let trimmed = input_str.trim();

    // Check if it starts with parentheses (attribute macro function style)
    if trimmed.starts_with('(') {
        // Find the matching closing parenthesis
        let mut paren_count = 0;
        let mut end_paren_pos = 0;

        for (i, ch) in trimmed.chars().enumerate() {
            if ch == '(' {
                paren_count += 1;
            } else if ch == ')' {
                paren_count -= 1;
                if paren_count == 0 {
                    end_paren_pos = i;
                    break;
                }
            }
        }

        if end_paren_pos > 0 {
            // Extract content inside parentheses (the path arguments)
            let paren_content = trimmed[1..end_paren_pos].trim();

            // Check if there's content (path arguments)
            let path_args = if paren_content.is_empty() {
                None
            } else {
                // Remove surrounding quotes if present
                let content = paren_content.trim_matches('"');
                if content.is_empty() {
                    None
                } else {
                    Some(content.to_string())
                }
            };

            // The rest is the function definition (e.g., "fn handler() { ... }")
            let function_body = trimmed[end_paren_pos + 1..].trim().to_string();
            return (path_args, function_body);
        }
    }

    // Fallback: treat entire input as function
    (None, trimmed.to_string())
}
