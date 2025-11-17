// HTTP verb macro implementation
// Handles get!{}, post!{}, put!(), patch!(), delete!()
// Supports optional path/query parameters: get!("param=value") { ... }

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

/// Parse function macro input to extract optional arguments and function
// Pure function: same input → same output
fn parse_function_macro_input(input_str: &str) -> (Option<String>, &str) {
    let trimmed = input_str.trim();
    
    // Pattern matching (FP principle)
    trimmed
        .strip_prefix('"')
        .and_then(|rest| {
            rest.find('"').map(|end_idx| {
                let arg = rest[..end_idx].to_string();
                let remaining = rest[end_idx + 1..].trim_start();
                (Some(arg), remaining)
            })
        })
        .unwrap_or((None, trimmed))  // Default case
}

/// Generate HTTP handler code from function macro
pub fn http_handler(method: &str, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // Parse optional arguments and function
    let (route_args, fn_input) = parse_function_macro_input(&input_str);

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

    // Parse route/query parameters from route_args
    let (route_pattern, query_params) = if let Some(args) = route_args {
        parse_route_args(&args)
    } else {
        (None, None)
    };

    // Generate metadata with optional route pattern and query params
    let route_pattern_const = if let Some(pattern) = route_pattern {
        quote! { pub const ROUTE_PATTERN: Option<&str> = Some(#pattern); }
    } else {
        quote! { pub const ROUTE_PATTERN: Option<&str> = None; }
    };

    let query_params_const = if let Some(params) = query_params {
        quote! { pub const QUERY_PARAMS: Option<&str> = Some(#params); }
    } else {
        quote! { pub const QUERY_PARAMS: Option<&str> = None; }
    };

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
            #route_pattern_const
            #query_params_const
        }
    };

    output.into()
}

/// Parse route arguments into route pattern and query parameters
/// Examples:
///   ":id" -> (Some(":id"), None)
///   "partial=stats" -> (None, Some("partial=stats"))
///   ":id/edit" -> (Some(":id/edit"), None)
fn parse_route_args(args: &str) -> (Option<String>, Option<String>) {
    let args = args.trim();
    
    // Using a match expression can be more idiomatic for this kind of logic.
    match args {
        a if a.contains('=') => (None, Some(a.to_string())),
        a if a.starts_with(':') || a.contains('/') => (Some(a.to_string()), None),
        // Default to route pattern if not empty
        a if !a.is_empty() => (Some(a.to_string()), None),
        _ => (None, None),
    }
}
