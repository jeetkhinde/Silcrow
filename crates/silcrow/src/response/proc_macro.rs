// silcrow/crates/silcrow/src/response/proc_macro.rs — Silcrow server-side procedural macro for auto-generating Silcrow-compatible response handlers from regular Rust functions
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn silcrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let fn_name = &sig.ident;

    // Extract original return type
    let output = &sig.output;

    // Extract generics if any
    let generics = &sig.generics;

    let expanded = quote! {
        #vis async fn #fn_name(
            req: silcrow::SilcrowRequest
        ) -> silcrow::Response #generics {

            // Execute original function body
            let __page = (async #output #block).await;

            if req.is_silcrow {
                if req.wants_html {
                    silcrow::HtmlOk(__page.view).ok()
                } else {
                    silcrow::JsonOk()
                        .set_value("data", __page.data)
                        .ok()
                }
            } else {
                __page.render().ok()
            }
        }
    };

    TokenStream::from(expanded)
}
