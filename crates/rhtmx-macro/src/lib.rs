// RHTMX Procedural Macros
// Provides HTTP routing macros

use proc_macro::TokenStream;

mod http;

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
