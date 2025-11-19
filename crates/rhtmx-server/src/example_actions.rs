// File: src/example_actions.rs
// Purpose: Example action implementations for /examples/actions-validation
// This demonstrates how actions work with validation and form helpers

use rhtmx::action_executor::ActionResult;
use rhtmx::RequestContext;
#[allow(unused_imports)]
use rhtmx::ValidateTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Example User struct (used for demonstration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub bio: Option<String>,
    pub username: String,
}

/// Create user request (with validation attributes processed by macro)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub age: i32,
    pub bio: Option<String>,
    pub username: String,
    pub website: Option<String>,
}

/// Update user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i32>,
    pub bio: Option<String>,
}

/// Search request with query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUsersRequest {
    pub filter: Option<String>,
    pub page: Option<i32>,
}

// Implement Validate for CreateUserRequest
// TODO: Use the Validate derive macro from rhtmx instead of manual implementation
impl rhtmx::ValidateTrait for CreateUserRequest {
    fn validate(&self) -> Result<(), HashMap<String, Vec<String>>> {
        let mut errors: HashMap<String, Vec<String>> = HashMap::new();

        // Validate name
        if self.name.trim().is_empty() {
            errors.insert("name".to_string(), vec!["Name is required".to_string()]);
        }

        // Validate email
        if !self.email.contains('@') {
            errors.insert("email".to_string(), vec!["Invalid email format".to_string()]);
        }

        // Validate password (at least 8 characters)
        if self.password.len() < 8 {
            errors.insert(
                "password".to_string(),
                vec!["Password must be at least 8 characters".to_string()],
            );
        }

        // Validate age
        if self.age < 18 {
            errors.insert(
                "age".to_string(),
                vec!["Must be at least 18 years old".to_string()],
            );
        } else if self.age > 120 {
            errors.insert("age".to_string(), vec!["Please enter a valid age".to_string()]);
        }

        // Validate username
        if self.username.len() < 3 {
            errors.insert(
                "username".to_string(),
                vec!["Username must be at least 3 characters".to_string()],
            );
        } else if self.username.len() > 50 {
            errors.insert(
                "username".to_string(),
                vec!["Username must be at most 50 characters".to_string()],
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Implement Validate for UpdateUserRequest
impl rhtmx::ValidateTrait for UpdateUserRequest {
    fn validate(&self) -> Result<(), HashMap<String, Vec<String>>> {
        let mut errors = HashMap::new();

        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.insert("name".to_string(), vec!["Name cannot be empty".to_string()]);
            }
        }

        if let Some(email) = &self.email {
            if !email.contains('@') {
                errors.insert("email".to_string(), vec!["Invalid email format".to_string()]);
            }
        }

        if let Some(age) = &self.age {
            if *age < 18 {
                errors.insert(
                    "age".to_string(),
                    vec!["Must be at least 18 years old".to_string()],
                );
            } else if *age > 120 {
                errors.insert("age".to_string(), vec!["Please enter a valid age".to_string()]);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Implement Validate for SearchUsersRequest (no validation needed)
impl rhtmx::ValidateTrait for SearchUsersRequest {
    fn validate(&self) -> Result<(), HashMap<String, Vec<String>>> {
        Ok(())
    }
}

/// GET /examples/actions-validation
pub async fn get_actions_validation(_ctx: RequestContext) -> ActionResult {
    // For now, just return HTML indicating we're rendering the page
    // In a real implementation, this would use query params for filtering
    ActionResult::Html {
        content: "<p>GET /examples/actions-validation - Users page loaded</p>".to_string(),
        headers: Default::default(),
    }
}

/// POST /examples/actions-validation - Create a user
pub async fn post_actions_validation(_ctx: RequestContext) -> ActionResult {
    // TODO: Implement validation_pipeline module for form validation
    ActionResult::Html {
        content:
            "<p>POST /examples/actions-validation - Validation pipeline not yet implemented</p>"
                .to_string(),
        headers: Default::default(),
    }
}

/// Helper function to format validation errors as HTML
// TODO: Implement form context for validation errors
/*
fn format_validation_errors(context: &crate::form_context::FormContext) -> String {
    let mut html = String::from(r#"<div class="validation-errors"><h3>Please fix the following errors:</h3><ul>"#);

    for (field, error) in context.get_errors() {
        html.push_str(&format!(r#"<li><strong>{}</strong>: {}</li>"#, field, error));
    }

    html.push_str("</ul></div>");
    html
}
*/
/// PATCH /examples/actions-validation/:id - Update a user
pub async fn patch_actions_validation(_ctx: RequestContext) -> ActionResult {
    ActionResult::Html {
        content: "<p>PATCH /examples/actions-validation - User updated</p>".to_string(),
        headers: Default::default(),
    }
}

/// DELETE /examples/actions-validation/:id - Delete a user
pub async fn delete_actions_validation(ctx: RequestContext) -> ActionResult {
    use rhtmx::database;

    // Get the updated user count after deletion
    let count = if let Some(pool) = ctx.db.as_ref() {
        match database::count_users(pool).await {
            Ok(c) => c.saturating_sub(1), // Assume one was deleted
            Err(_) => 0,                  // Default to 0 if count fails
        }
    } else {
        0 // Default to 0 if database is not configured
    };

    // Return only OOB update
    let oob_html = format!(r#"<div id="user-count" hx-swap-oob="true">{}</div>"#, count);

    let mut headers = axum::http::HeaderMap::new();
    let trigger = serde_json::json!({
        "showToast": {
            "message": "User deleted!"
        }
    });
    if let Ok(value) = trigger.to_string().parse() {
        headers.insert("HX-Trigger", value);
    }

    ActionResult::Html {
        content: oob_html,
        headers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhtmx::ValidateTrait;

    #[test]
    fn test_create_user_validation_valid() {
        let req = CreateUserRequest {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            age: 28,
            bio: Some("Developer".to_string()),
            username: "charlie".to_string(),
            website: None,
        };

        let result = req.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_user_validation_invalid_email() {
        let req = CreateUserRequest {
            name: "Charlie".to_string(),
            email: "invalid-email".to_string(),
            password: "SecurePass123!".to_string(),
            age: 28,
            bio: None,
            username: "charlie".to_string(),
            website: None,
        };

        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains_key("email"));
    }

    #[test]
    fn test_create_user_validation_short_password() {
        let req = CreateUserRequest {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            password: "short".to_string(),
            age: 28,
            bio: None,
            username: "charlie".to_string(),
            website: None,
        };

        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains_key("password"));
    }
}
