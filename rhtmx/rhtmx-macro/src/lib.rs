// RHTMX Procedural Macros
// Provides compile-time HTML generation and HTTP routing macros

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod html;
mod http;
mod validation;

/// The html! macro for compile-time HTML generation
///
/// Parses JSX-like syntax and generates efficient Rust code with r-directives support.
///
/// # Example
///
/// ```ignore
/// fn user_card(user: &User) -> Html {
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
/// - `r-for="item in items"` - Loop over collections
/// - `r-for="(i, item) in items"` - Loop with index
/// - `r-if="condition"` - Conditional rendering
///
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    let mut parser = html::HtmlParser::new(input_str);
    let nodes = match parser.parse() {
        Ok(nodes) => nodes,
        Err(e) => return e.to_compile_error().into(),
    };

    let output = html::CodeGenerator::generate(nodes);
    output.into()
}

/// The css! macro for scoped CSS generation
///
/// Generates scoped CSS with automatic class prefixing using data attributes.
///
/// # Example
///
/// ```ignore
/// fn user_card(user: &User) -> Html {
///     css! {
///         scope: "user-card",
///         .card {
///             border: 1px solid #ccc;
///             padding: 1rem;
///         }
///         .card:hover {
///             box-shadow: 0 2px 4px rgba(0,0,0,0.1);
///         }
///     }
///
///     html! {
///         <div class="card" data-scope="user-card">
///             <h3>{user.name}</h3>
///         </div>
///     }
/// }
/// ```
///
/// The macro generates:
/// - A unique scope identifier (e.g., "user-card")
/// - Scoped CSS rules with `[data-scope="user-card"]` selector
/// - HTML elements with matching `data-scope` attribute
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // Parse scope name if provided (e.g., "scope: \"user-card\", .card { ... }")
    let (scope_name, css_content) = if input_str.contains("scope:") {
        // Extract scope name
        let parts: Vec<&str> = input_str.splitn(2, ',').collect();
        if parts.len() == 2 {
            let scope_part = parts[0].replace("scope:", "").trim().to_string();
            let scope = scope_part.trim_matches(|c| c == '"' || c == ' ');
            (scope.to_string(), parts[1].trim().to_string())
        } else {
            // Generate hash from content
            let hash = format!("css_{:x}", input_str.len());
            (hash, input_str)
        }
    } else {
        // Generate hash from content
        let hash = format!("css_{:x}", input_str.len());
        (hash, input_str)
    };

    // Scope the CSS by adding data-scope attribute selector
    let scoped_css = scope_css_rules(&scope_name, &css_content);

    quote! {
        {
            // Return scoped CSS as a string that can be injected into <style> tags
            let __scoped_css = #scoped_css;
            // In production, this would be collected and added to <head>
            // For now, it's just documentation
            #scope_name
        }
    }.into()
}

/// Scope CSS rules by prepending [data-scope="name"] to selectors
fn scope_css_rules(scope_name: &str, css: &str) -> String {
    let scope_attr = format!("[data-scope=\"{}\"]", scope_name);
    let mut result = String::new();

    // Simple CSS rule parser
    for line in css.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }

        // Check if this is a selector line (ends with { or contains {)
        if trimmed.contains('{') {
            let parts: Vec<&str> = trimmed.splitn(2, '{').collect();
            let selector = parts[0].trim();
            let rest = if parts.len() > 1 { parts[1] } else { "" };

            // Scope the selector
            let scoped_selector = if selector.starts_with(':') {
                // Pseudo-class on root: [data-scope="name"]:hover
                format!("{}{}", scope_attr, selector)
            } else if selector.contains('&') {
                // & placeholder: replace with scope
                selector.replace('&', &scope_attr)
            } else {
                // Normal selector: [data-scope="name"] .selector
                format!("{} {}", scope_attr, selector)
            };

            result.push_str(&format!("{} {{{}\n", scoped_selector, rest));
        } else {
            result.push_str(trimmed);
            result.push('\n');
        }
    }

    result
}

/// HTTP GET handler macro
///
/// Marks a function as a GET request handler. When used with file-based routing,
/// the route is determined by the file location.
///
/// # Example
///
/// ```ignore
/// // File: pages/users.rs
/// get! {
///     fn index() -> OkResponse {
///         let users = db::get_users()?;
///         Ok().render(users_list, users)
///     }
/// }
///
/// get!("partial=stats") {
///     fn stats() -> OkResponse {
///         Ok().render(stats_component, get_stats())
///     }
/// }
/// ```
#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    http::http_handler("GET", input)
}

/// HTTP POST handler macro
///
/// # Example
///
/// ```ignore
/// post! {
///     fn create(req: CreateUserRequest) -> OkResponse {
///         let user = db::create_user(req)?;
///         Ok().render(user_card, user)
///             .toast("User created!")
///     }
/// }
/// ```
#[proc_macro]
pub fn post(input: TokenStream) -> TokenStream {
    http::http_handler("POST", input)
}

/// HTTP PUT handler macro
///
/// # Example
///
/// ```ignore
/// put!(":id") {
///     fn update(id: i32, req: UpdateUserRequest) -> OkResponse {
///         let user = db::update_user(id, req)?;
///         Ok().render(user_card, user)
///             .toast("User updated!")
///     }
/// }
/// ```
#[proc_macro]
pub fn put(input: TokenStream) -> TokenStream {
    http::http_handler("PUT", input)
}

/// HTTP PATCH handler macro
///
/// # Example
///
/// ```ignore
/// patch!(":id") {
///     fn partial_update(id: i32, req: PatchUserRequest) -> OkResponse {
///         let user = db::patch_user(id, req)?;
///         Ok().render(user_card, user)
///     }
/// }
/// ```
#[proc_macro]
pub fn patch(input: TokenStream) -> TokenStream {
    http::http_handler("PATCH", input)
}

/// HTTP DELETE handler macro
///
/// # Example
///
/// ```ignore
/// delete!(":id") {
///     fn delete(id: i32) -> OkResponse {
///         db::delete_user(id)?;
///         Ok().toast("User deleted!")
///     }
/// }
/// ```
#[proc_macro]
pub fn delete(input: TokenStream) -> TokenStream {
    http::http_handler("DELETE", input)
}

/// Derive macro for automatic validation
///
/// Generates a `Validate` trait implementation that validates struct fields
/// based on attributes like #[email], #[min], #[max], etc.
///
/// # Example
///
/// ```ignore
/// use rhtmx::Validate;
/// use serde::Deserialize;
///
/// #[derive(Validate, Deserialize)]
/// struct CreateUserRequest {
///     #[min_length(3)]
///     #[max_length(50)]
///     name: String,
///
///     #[email]
///     #[no_public_domains]
///     email: String,
///
///     #[password("strong")]
///     password: String,
///
///     #[min(18)]
///     #[max(120)]
///     age: i32,
///
///     bio: Option<String>,  // Optional fields
/// }
/// ```
///
/// # Available Validators
///
/// **Email Validators:**
/// - `#[email]` - Valid email format
/// - `#[no_public_domains]` - Reject gmail, yahoo, etc.
/// - `#[blocked_domains("a.com", "b.com")]` - Block specific domains
///
/// **Password Validators:**
/// - `#[password("strong")]` - 8+ chars, upper, lower, digit, special
/// - `#[password("medium")]` - 8+ chars, upper, lower, digit
/// - `#[password("basic")]` - 6+ chars
/// - `#[password(r"regex")]` - Custom regex pattern
///
/// **Numeric Validators:**
/// - `#[min(n)]` - Minimum value
/// - `#[max(n)]` - Maximum value
/// - `#[range(min, max)]` - Value range
///
/// **String Validators:**
/// - `#[min_length(n)]` - Minimum length
/// - `#[max_length(n)]` - Maximum length
/// - `#[length(min, max)]` - Length range
/// - `#[regex(r"pattern")]` - Custom regex
/// - `#[url]` - Valid URL format
///
/// **General:**
/// - `#[required]` - Required for Option<T> fields
/// - `#[allow_whitespace]` - Don't trim whitespace
///
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
    validation::impl_validate(&input).into()
}
