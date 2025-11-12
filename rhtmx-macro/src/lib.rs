// File: rhtml-macro/src/lib.rs
// Purpose: Procedural macros for #[webpage] and #[component] attributes

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ItemFn, Pat};

mod html;
mod http;
mod layout;
mod layout_registry;
mod layout_resolver;
mod slot;
mod validation;

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
#[proc_macro_derive(
    Validate,
    attributes(
        email,
        no_public_domains,
        blocked_domains,
        password,
        min,
        max,
        range,
        min_length,
        max_length,
        length,
        regex,
        url,
        allow_whitespace,
        required,
        query,
        form
    )
)]
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

/// The maud! macro for Maud template syntax
///
/// This is a convenience wrapper around Maud's html! macro that automatically
/// converts the result to RHTMX's Html type. Use this when you prefer Maud's
/// Lisp-like syntax over the JSX-style html! macro.
///
/// # Example
///
/// ```ignore
/// use rhtmx::{Ok, OkResponse, maud};
///
/// get!()
/// fn user_card() -> OkResponse {
///     let user_name = "Alice";
///     let html = maud! {
///         div.card {
///             h3 { (user_name) }
///             p { "User profile card" }
///         }
///     };
///     Ok().html(html)
/// }
/// ```
///
/// # Syntax
///
/// Maud uses a Lisp-like syntax that is more compact than HTML:
/// - `div { content }` - Element with content
/// - `div.class { content }` - Element with class
/// - `div#id { content }` - Element with id
/// - `div[attr=value] { content }` - Element with attribute
/// - `(expr)` - Interpolate Rust expression
/// - `@if condition { ... }` - Conditional
/// - `@for item in items { ... }` - Loop
///
/// # Comparison with html!
///
/// **html! (JSX-style):**
/// ```ignore
/// html! {
///     <div class="card">
///         <h3>{user_name}</h3>
///         <p>"User profile card"</p>
///     </div>
/// }
/// ```
///
/// **maud! (Lisp-style):**
/// ```ignore
/// maud! {
///     div.card {
///         h3 { (user_name) }
///         p { "User profile card" }
///     }
/// }
/// ```
///
/// Choose based on your preference:
/// - **html!**: Familiar HTML-like syntax, good for markup-heavy code
/// - **maud!**: Compact Lisp syntax, good for programmatic HTML generation
#[proc_macro]
pub fn maud(input: TokenStream) -> TokenStream {
    // Pass input to Maud's html! macro and convert result to RHTMX Html
    let input_tokens = proc_macro2::TokenStream::from(input);

    let output = quote! {
        {
            use rhtmx::maud_wrapper::MaudMarkup;
            maud::html! { #input_tokens }.to_html()
        }
    };

    output.into()
}

// ============================================================================
// HTTP Verb Macros - get!, post!, put!, patch!, delete!
// ============================================================================

/// GET request handler macro
///
/// # Example
///
/// ```ignore
/// get!()
/// fn list_users() -> OkResponse {
///     let users = db::get_users()?;
///     Ok().render(users_page, users)
/// }
///
/// // With path parameters
/// get!(":id")
/// fn get_user(id: i32) -> OkResponse {
///     let user = db::get_user(id)?;
///     Ok().render(user_detail, user)
/// }
/// ```
#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    http::http_handler("GET", input)
}

/// POST request handler macro
///
/// # Example
///
/// ```ignore
/// post!()
/// fn create(req: CreateUserRequest) -> OkResponse {
///     let user = db::create_user(req)?;
///     Ok().render(user_card, user).toast("Created!")
/// }
/// ```
#[proc_macro]
pub fn post(input: TokenStream) -> TokenStream {
    http::http_handler("POST", input)
}

/// PUT request handler macro
///
/// # Example
///
/// ```ignore
/// put!(":id")
/// fn replace(id: i32, req: UpdateRequest) -> OkResponse {
///     let user = db::update_user(id, req)?;
///     Ok().render(user_detail, user)
/// }
/// ```
#[proc_macro]
pub fn put(input: TokenStream) -> TokenStream {
    http::http_handler("PUT", input)
}

/// PATCH request handler macro
///
/// # Example
///
/// ```ignore
/// patch!(":id")
/// fn partial_update(id: i32, req: PartialUpdate) -> OkResponse {
///     let user = db::patch_user(id, req)?;
///     Ok().render(user_card, user)
/// }
/// ```
#[proc_macro]
pub fn patch(input: TokenStream) -> TokenStream {
    http::http_handler("PATCH", input)
}

/// DELETE request handler macro
///
/// # Example
///
/// ```ignore
/// delete!(":id")
/// fn delete(id: i32) -> OkResponse {
///     db::delete_user(id)?;
///     Ok().toast("Deleted!")
/// }
/// ```
#[proc_macro]
pub fn delete(input: TokenStream) -> TokenStream {
    http::http_handler("DELETE", input)
}
