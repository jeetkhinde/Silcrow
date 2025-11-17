// Example: HTTP Verb Macros with Query and Route Parameters
//
// This example demonstrates how to use the HTTP verb macros (get!, post!, put!, patch!, delete!)
// with query parameters and route parameters for building RESTful APIs with RHTMX.
//
// To run this example:
// 1. Add these handlers to your application
// 2. Register them with ActionHandlerRegistry
// 3. Access them at the specified routes

use rhtmx::{get, post, put, patch, delete, OkResponse, ErrorResponse};
use rhtmx::action_executor::ActionResult;
use rhtmx::RequestContext;
use std::pin::Pin;
use std::future::Future;

// ============================================================================
// Basic Handlers (No Parameters)
// ============================================================================

/// GET /users - List all users
/// Basic handler that responds to the base route
get! {
    fn index() -> OkResponse {
        let users = vec!["Alice", "Bob", "Charlie"];
        let html = format!(
            "<ul>{}</ul>",
            users.iter()
                .map(|u| format!("<li>{}</li>", u))
                .collect::<String>()
        );
        Ok().content(html)
    }
}

/// POST /users - Create a new user
/// Basic POST handler for creating resources
post! {
    fn create() -> OkResponse {
        Ok().content("<p>User created!</p>")
            .toast("User created successfully!")
    }
}

// ============================================================================
// Query Parameter Handlers
// ============================================================================

/// GET /users?partial=stats - Get user statistics as a partial
/// This handler responds only when the specific query parameter is present
get!("partial=stats") {
    fn user_stats() -> OkResponse {
        Ok().content(r#"
            <div id="stats">
                <h3>User Statistics</h3>
                <p>Total Users: 42</p>
                <p>Active: 38</p>
                <p>Inactive: 4</p>
            </div>
        "#)
    }
}

/// GET /users?partial=list - Get user list as a partial
/// Another query parameter handler for a different partial view
get!("partial=list") {
    fn user_list_partial() -> OkResponse {
        Ok().content(r#"
            <div id="user-list">
                <div class="user-card">Alice</div>
                <div class="user-card">Bob</div>
                <div class="user-card">Charlie</div>
            </div>
        "#)
    }
}

/// GET /users?partial=form - Get user creation form
get!("partial=form") {
    fn user_form() -> OkResponse {
        Ok().content(r#"
            <form hx-post="/users" hx-target="#user-list">
                <input type="text" name="name" placeholder="Name" />
                <input type="email" name="email" placeholder="Email" />
                <button type="submit">Create User</button>
            </form>
        "#)
    }
}

// ============================================================================
// Route Parameter Handlers
// ============================================================================

/// GET /users/:id - Get a specific user
/// Route parameter handler that captures the :id segment
get!(":id") {
    fn show(id: i32) -> OkResponse {
        Ok().content(format!(r#"
            <div class="user-detail">
                <h2>User #{}</h2>
                <p>Name: User {}</p>
                <p>Email: user{}@example.com</p>
            </div>
        "#, id, id, id))
    }
}

/// PUT /users/:id - Update a specific user
/// Route parameter with PUT method
put!(":id") {
    fn update(id: i32) -> OkResponse {
        Ok().content(format!("<p>User {} updated!</p>", id))
            .toast(format!("User {} updated successfully!", id))
    }
}

/// DELETE /users/:id - Delete a specific user
/// Route parameter with DELETE method
delete!(":id") {
    fn delete(id: i32) -> OkResponse {
        Ok().content("")
            .toast(format!("User {} deleted!", id))
            .oob("user-count", "41") // Update user count out-of-band
    }
}

// ============================================================================
// Sub-Route Handlers
// ============================================================================

/// GET /users/:id/edit - Get edit form for a user
/// Sub-route with route parameter
get!(":id/edit") {
    fn edit(id: i32) -> OkResponse {
        Ok().content(format!(r#"
            <form hx-put="/users/{}" hx-target="#user-detail">
                <input type="text" name="name" value="User {}" />
                <input type="email" name="email" value="user{}@example.com" />
                <button type="submit">Update User</button>
            </form>
        "#, id, id, id))
    }
}

/// PATCH /users/:id/activate - Activate a user
/// Sub-route with PATCH method
patch!(":id/activate") {
    fn activate(id: i32) -> OkResponse {
        Ok().content(format!(r#"
            <span class="badge badge-success">Active</span>
        "#))
            .toast(format!("User {} activated!", id))
    }
}

/// PATCH /users/:id/deactivate - Deactivate a user
patch!(":id/deactivate") {
    fn deactivate(id: i32) -> OkResponse {
        Ok().content(r#"
            <span class="badge badge-danger">Inactive</span>
        "#)
            .toast("User deactivated!")
    }
}

// ============================================================================
// Combined: Query Parameters + Route Parameters
// ============================================================================

/// GET /users/:id?partial=avatar - Get user avatar as partial
/// This would match both the route parameter AND the query parameter
get!(":id") {
    fn show_with_query(id: i32) -> OkResponse {
        // In a real application, you'd check ctx.query for "partial"
        // and return different content accordingly
        Ok().content(format!(r#"
            <img src="/avatars/{}.jpg" alt="User {} Avatar" />
        "#, id, id))
    }
}

// ============================================================================
// Registration Example
// ============================================================================

/// Example of how to register these handlers with the ActionHandlerRegistry
///
/// This is pseudo-code showing the registration pattern.
/// In the future, this will be automatic via proc macros.
#[allow(dead_code)]
fn register_handlers(registry: &mut rhtmx_server::action_handlers::ActionHandlerRegistry) {
    use rhtmx_server::action_handlers::HandlerMetadata;

    // Helper to convert our function to ActionHandler type
    fn to_handler<F>(f: F) -> rhtmx_server::action_handlers::ActionHandler
    where
        F: Fn(RequestContext) -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + 'static,
    {
        f
    }

    // Register base handlers
    registry.register_with_metadata(
        HandlerMetadata::new(
            "/users".to_string(),
            "GET".to_string(),
            |ctx| Box::pin(async move {
                // Call index() and convert to ActionResult
                ActionResult::Html {
                    content: "<ul><li>Alice</li><li>Bob</li></ul>".to_string(),
                    headers: Default::default(),
                }
            })
        )
    );

    // Register query parameter handlers
    registry.register_with_metadata(
        HandlerMetadata::new(
            "/users".to_string(),
            "GET".to_string(),
            |ctx| Box::pin(async move {
                ActionResult::Html {
                    content: "<div>Stats...</div>".to_string(),
                    headers: Default::default(),
                }
            })
        )
        .with_query_params("partial=stats".to_string())
    );

    // Register route parameter handlers
    registry.register_with_metadata(
        HandlerMetadata::new(
            "/users".to_string(),
            "GET".to_string(),
            |ctx| Box::pin(async move {
                // Extract :id from path and call show()
                ActionResult::Html {
                    content: "<div>User detail...</div>".to_string(),
                    headers: Default::default(),
                }
            })
        )
        .with_route_pattern(":id".to_string())
    );

    // Register sub-route handlers
    registry.register_with_metadata(
        HandlerMetadata::new(
            "/users".to_string(),
            "GET".to_string(),
            |ctx| Box::pin(async move {
                ActionResult::Html {
                    content: "<form>...</form>".to_string(),
                    headers: Default::default(),
                }
            })
        )
        .with_route_pattern(":id/edit".to_string())
    );
}

// ============================================================================
// Matching Priority
// ============================================================================

/// The ActionHandlerRegistry matches handlers with the following priority (most specific first):
///
/// 1. Route + Method + Query params + Route pattern (e.g., GET /users/:id?partial=detail)
/// 2. Route + Method + Query params (e.g., GET /users?partial=stats)
/// 3. Route + Method + Route pattern (e.g., GET /users/:id)
/// 4. Route + Method (e.g., GET /users)
///
/// Examples:
///
/// Request: GET /users
/// Matches: index() - base handler
///
/// Request: GET /users?partial=stats
/// Matches: user_stats() - query param handler takes precedence
///
/// Request: GET /users?partial=list
/// Matches: user_list_partial() - specific query param handler
///
/// Request: GET /users?other=value
/// Matches: index() - falls back to base handler
///
/// Request: GET /users/123
/// Matches: show(123) - route param handler
///
/// Request: GET /users/123/edit
/// Matches: edit(123) - sub-route handler
///
/// Request: PUT /users/123
/// Matches: update(123) - PUT with route param
///
/// Request: DELETE /users/123
/// Matches: delete(123) - DELETE with route param

fn main() {
    println!("This is an example file demonstrating HTTP verb macro usage.");
    println!("See the code comments for detailed explanations.");
}
