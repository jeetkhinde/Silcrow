// RHTMX Procedural Macros
// Provides compile-time HTML generation and HTTP routing macros

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod html;
mod http;

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
/// # Syntax
///
/// - `get! { fn name() { ... } }` - Basic handler
/// - `get!("partial=name") { fn name() { ... } }` - Query parameter handler
/// - `get!(":id") { fn name() { ... } }` - Route parameter handler
/// - `get!(":id/edit") { fn name() { ... } }` - Sub-route handler
///
/// # Examples
///
/// ```ignore
/// // File: pages/users.rs
///
/// // Basic GET handler - responds to /users
/// get! {
///     fn index() -> OkResponse {
///         let users = db::get_users()?;
///         Ok().render(users_list, users)
///     }
/// }
///
/// // Query param handler - responds to /users?partial=stats
/// get!("partial=stats") {
///     fn stats() -> OkResponse {
///         let stats = db::get_stats()?;
///         Ok().render(stats_component, stats)
///     }
/// }
///
/// // Query param handler - responds to /users?partial=list
/// get!("partial=list") {
///     fn user_list() -> OkResponse {
///         let users = db::get_users()?;
///         Ok().render(user_list_component, users)
///     }
/// }
///
/// // Route param handler - responds to /users/:id
/// get!(":id") {
///     fn show(id: i32) -> OkResponse {
///         let user = db::get_user(id)?;
///         Ok().render(user_detail, user)
///     }
/// }
/// ```
#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    http::http_handler("GET", input)
}

/// HTTP POST handler macro
///
/// # Examples
///
/// ```ignore
/// // Basic POST handler
/// post! {
///     fn create(req: CreateUserRequest) -> OkResponse {
///         let user = db::create_user(req)?;
///         Ok().render(user_card, user)
///             .toast("User created!")
///     }
/// }
///
/// // Query param POST handler - responds to POST /users?action=bulk
/// post!("action=bulk") {
///     fn bulk_create(req: BulkCreateRequest) -> OkResponse {
///         let users = db::bulk_create_users(req)?;
///         Ok().render(user_list, users)
///     }
/// }
/// ```
#[proc_macro]
pub fn post(input: TokenStream) -> TokenStream {
    http::http_handler("POST", input)
}

/// HTTP PUT handler macro
///
/// # Examples
///
/// ```ignore
/// // Route param handler - responds to PUT /users/:id
/// put!(":id") {
///     fn update(id: i32, req: UpdateUserRequest) -> OkResponse {
///         let user = db::update_user(id, req)?;
///         Ok().render(user_card, user)
///             .toast("User updated!")
///     }
/// }
///
/// // Sub-route handler - responds to PUT /users/:id/activate
/// put!(":id/activate") {
///     fn activate(id: i32) -> OkResponse {
///         let user = db::activate_user(id)?;
///         Ok().render(user_card, user)
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


/// Derive macro for Syncable trait
///
/// # Example
///
/// ```ignore
/// #[derive(Syncable)]
/// #[sync(table = "users")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(Syncable, attributes(sync))]
pub fn derive_syncable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract the table name from #[sync(table = "...")] or use struct name
    let table_name = extract_table_name(&input).unwrap_or_else(|| {
        // Convert CamelCase to snake_case
        let struct_name = name.to_string();
        to_snake_case(&struct_name)
    });

    // Find the id field (must be present)
    let id_field = find_id_field(&input).expect("Syncable structs must have an 'id' field");

    // Check if the struct has version and modified_at fields
    let has_version = has_field(&input, "version") || has_field(&input, "_version");
    let has_modified_at = has_field(&input, "modified_at") || has_field(&input, "_modified_at");

    // Generate the trait implementation
    let version_impl = if has_version {
        quote! {
            fn version(&self) -> Option<i64> {
                self.version.or(self._version)
            }

            fn set_version(&mut self, version: i64) {
                if let Some(v) = self.version.as_mut() {
                    *v = version;
                } else if let Some(v) = self._version.as_mut() {
                    *v = version;
                }
            }
        }
    } else {
        quote! {}
    };

    let modified_at_impl = if has_modified_at {
        quote! {
            fn modified_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                self.modified_at.or(self._modified_at)
            }

            fn set_modified_at(&mut self, timestamp: chrono::DateTime<chrono::Utc>) {
                if let Some(ts) = self.modified_at.as_mut() {
                    *ts = timestamp;
                } else if let Some(ts) = self._modified_at.as_mut() {
                    *ts = timestamp;
                }
            }
        }
    } else {
        quote! {}
    };

    let has_metadata = has_version || has_modified_at;

    let expanded = quote! {
        impl #impl_generics rhtmx_sync::Syncable for #name #ty_generics #where_clause {
            fn entity_name() -> &'static str {
                #table_name
            }

            fn id(&self) -> String {
                self.#id_field.to_string()
            }

            #version_impl

            #modified_at_impl

            fn has_sync_metadata() -> bool {
                #has_metadata
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract table name from #[sync(table = "...")] attribute
fn extract_table_name(input: &DeriveInput) -> Option<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("sync") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args::<syn::Meta>() {
                if nv.path.is_ident("table") {
                    if let syn::Expr::Lit(lit) = nv.value {
                        if let syn::Lit::Str(s) = lit.lit {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Find the id field in the struct
fn find_id_field(input: &DeriveInput) -> Option<syn::Ident> {
    if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    if ident == "id" {
                        return Some(ident.clone());
                    }
                }
            }
        }
    }
    None
}

/// Check if a struct has a specific field
fn has_field(input: &DeriveInput, field_name: &str) -> bool {
    if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            return fields
                .named
                .iter()
                .any(|f| f.ident.as_ref().map(|i| i == field_name).unwrap_or(false));
        }
    }
    false
}

/// Convert CamelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}
