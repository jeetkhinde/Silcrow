// HTTP verb macro implementation
// Handles #[get], #[post], #[put], #[patch], #[delete]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Generate HTTP handler code
pub fn http_handler(method: &str, _args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

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
