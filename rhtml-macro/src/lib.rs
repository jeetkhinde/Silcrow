// File: rhtml-macro/src/lib.rs
// Purpose: Procedural macros for #[webpage] and #[component] attributes

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, FnArg, Pat, DeriveInput};

mod html;
mod layout;
mod layout_registry;
mod layout_resolver;
mod slot;
mod validation;

/// The #[webpage] attribute macro for defining pages
///
/// # Example
///
/// ```ignore
/// #[webpage]
/// pub fn users(props: UsersProps) {
///     <div class="container">
///         <h1>Users</h1>
///         <div r-for="user in props.data">
///             <user_card user={user} />
///         </div>
///     </div>
/// }
/// ```
///
/// This gets transformed into:
///
/// ```ignore
/// WebPage {
///     <div class="container">
///         <h1>Users</h1>
///         <div r-for="user in props.data">
///             <user_card user={user} />
///         </div>
///     </div>
/// }
/// ```
#[proc_macro_attribute]
pub fn webpage(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract the function body
    let body = &input_fn.block;

    // Transform to WebPage syntax
    // The body already contains the HTML/RHTML tokens
    let output = quote! {
        WebPage #body
    };

    output.into()
}

/// The #[component] attribute macro for defining reusable components
///
/// Marks a function as a renderable component. Public components are accessible
/// via HTTP ?partial=name query parameter. Private components are internal only.
///
/// # Public Component Example
///
/// ```ignore
/// #[component]
/// pub fn analytics(props: AnalyticsProps) {
///     <div id="analytics" class="stats">
///         <span class="label">Total Users:</span>
///         <span class="value">{props.total}</span>
///     </div>
/// }
/// ```
///
/// # Private Component Example
///
/// ```ignore
/// #[component]
/// fn user_card(props: UserCardProps) {
///     <div class="card" id="user-{props.user.id}">
///         <h3>{props.user.name}</h3>
///         <p>{props.user.email}</p>
///     </div>
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Check if the function is public
    let is_public = matches!(input_fn.vis, syn::Visibility::Public(_));

    // Extract function metadata
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let body = &input_fn.block;
    let visibility = &input_fn.vis;
    let sig = &input_fn.sig;

    // Get the first parameter name (should be props)
    let _props_param = sig.inputs.iter().find_map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(ident) = &*pat_type.pat {
                return Some(ident.ident.clone());
            }
        }
        None
    });

    // For now, keep the original function as-is
    // The component macro just marks the function and can be used for future enhancements
    // like automatic route registration, documentation generation, etc.
    let output = quote! {
        #[doc = "Component: "]
        #visibility #sig {
            #body
        }

        // Auto-register public components at compile time
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const _: () = {
            #[doc(hidden)]
            pub mod __component_registry {
                use super::*;

                #[doc(hidden)]
                pub fn component_info() -> (&'static str, bool) {
                    (#fn_name_str, #is_public)
                }
            }
        };
    };

    output.into()
}

/// Derive macro for automatic validation
///
/// # Example
///
/// ```ignore
/// #[derive(Validate)]
/// struct CreateUserRequest {
///     #[email]
///     #[no_public_domains]
///     email: String,
///
///     #[password("strong")]
///     password: String,
///
///     #[min(18)] #[max(120)]
///     age: i32,
/// }
/// ```
#[proc_macro_derive(Validate, attributes(
    email, no_public_domains, blocked_domains,
    password, min, max, range,
    min_length, max_length, length,
    regex, url, allow_whitespace,
    required, query, form
))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = validation::impl_validate(&input);
    expanded.into()
}

/// The html! macro for compile-time HTML generation
///
/// This macro parses JSX-like syntax and generates efficient Rust code.
/// It supports r-directives like r-for and r-if for control flow.
///
/// # Example
///
/// ```ignore
/// fn user_card(user: &User) {
///     html! {
///         <div class="card">
///             <h3>{user.name}</h3>
///             <p>{user.email}</p>
///         </div>
///     }
/// }
/// ```
///
/// # R-Directives
///
/// ## r-for
/// ```ignore
/// html! {
///     <div r-for="user in users">
///         <p>{user.name}</p>
///     </div>
/// }
/// ```
///
/// ## r-if
/// ```ignore
/// html! {
///     <div r-if="user.is_admin">
///         Admin Panel
///     </div>
/// }
/// ```
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    // Convert input to string
    let input_str = input.to_string();

    // Parse HTML
    let mut parser = html::HtmlParser::new(input_str);
    let nodes = match parser.parse() {
        Ok(nodes) => nodes,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate Rust code
    let output = html::CodeGenerator::generate(nodes);

    output.into()
}
