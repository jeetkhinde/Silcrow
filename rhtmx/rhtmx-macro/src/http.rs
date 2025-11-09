// HTTP verb macro implementation
// Handles get!{}, post!{}, put!(), patch!(), delete!()
// Supports optional path/query parameters: get!("param=value") { ... }

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

/// Parse function macro input to extract optional arguments and function
fn parse_function_macro_input(input_str: &str) -> (Option<String>, String) {
    let trimmed = input_str.trim();

    // Check if it starts with a string literal (arguments)
    if trimmed.starts_with('"') {
        // Find the matching closing quote
        let mut chars = trimmed.chars().peekable();
        chars.next(); // skip opening quote

        let mut arg_string = String::new();
        let mut escaped = false;

        while let Some(ch) = chars.next() {
            if escaped {
                arg_string.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                // Found closing quote
                let remaining = chars.collect::<String>().trim_start().to_string();
                return (Some(arg_string), remaining);
            } else {
                arg_string.push(ch);
            }
        }
    }

    // No arguments, entire input is the function block
    (None, input_str.to_string())
}

/// Generate HTTP handler code from function macro
pub fn http_handler(method: &str, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // Parse optional arguments and function
    let (_route_args, fn_input) = parse_function_macro_input(&input_str);

    // Parse the function
    let input_fn = match syn::parse_str::<ItemFn>(&fn_input) {
        Ok(f) => f,
        Err(e) => {
            return syn::Error::new_spanned(&fn_input, format!("Expected function definition: {}", e))
                .to_compile_error()
                .into();
        }
    };

    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    let fn_name = &fn_sig.ident;

    // Create unique module name for each handler
    let meta_mod_name = quote::format_ident!("__rhtmx_route_meta_{}", fn_name);

    // For now, just preserve the function and add metadata
    // File-based routing will discover these functions at compile time
    let output = quote! {
        #[doc = concat!("HTTP ", #method, " handler")]
        #[allow(non_snake_case)]
        #fn_vis #fn_sig {
            #fn_block
        }

        // Register route metadata (will be used by file-based routing)
        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub mod #meta_mod_name {
            pub const METHOD: &str = #method;
            pub const HANDLER_NAME: &str = stringify!(#fn_name);
        }
    };

    output.into()
}
