// File: src/actions.rs
// Purpose: Action-based routing and form helpers

use axum::http::{HeaderMap, HeaderValue};
use std::collections::HashMap;

/// Empty response for actions that don't return content
pub struct Empty {
    headers: HeaderMap,
    toast_message: Option<String>,
    oob_updates: Vec<(String, String)>,
}

impl Empty {
    /// Create a new empty response
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            toast_message: None,
            oob_updates: Vec::new(),
        }
    }

    /// Add a toast notification
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.toast_message = Some(message.into());
        self
    }

    /// Add an out-of-band update
    pub fn oob<T: ToString>(mut self, target: impl Into<String>, content: T) -> Self {
        self.oob_updates.push((target.into(), content.to_string()));
        self
    }

    /// Build the response
    pub fn build(self) -> (HeaderMap, String) {
        // Build headers functionally by combining base headers with optional toast trigger
        let headers = self.toast_message
            .as_ref()
            .and_then(|message| {
                serde_json::json!({
                    "showToast": {
                        "message": message
                    }
                })
                .to_string()
                .parse::<HeaderValue>()
                .ok()
            })
            .map(|trigger_value| {
                let mut headers = self.headers.clone();
                headers.insert("HX-Trigger", trigger_value);
                headers
            })
            .unwrap_or(self.headers);

        // Build OOB content functionally using iterator chain
        let content = self.oob_updates
            .iter()
            .map(|(target, update)| {
                format!(
                    r#"<div id="{}" hx-swap-oob="true">{}</div>"#,
                    target, update
                )
            })
            .collect::<Vec<_>>()
            .join("");

        (headers, content)
    }
}

impl Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

/// Response wrapper that adds helper methods
pub struct ActionResponse<T> {
    inner: T,
    toast_message: Option<String>,
    oob_updates: Vec<(String, String)>,
}

impl<T> ActionResponse<T> {
    /// Create a new action response
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            toast_message: None,
            oob_updates: Vec::new(),
        }
    }

    /// Add a toast notification
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.toast_message = Some(message.into());
        self
    }

    /// Add an out-of-band update
    pub fn oob<U: ToString>(mut self, target: impl Into<String>, content: U) -> Self {
        self.oob_updates.push((target.into(), content.to_string()));
        self
    }

    /// Get the inner value
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get the toast message if any
    pub fn get_toast(&self) -> Option<&str> {
        self.toast_message.as_deref()
    }

    /// Get OOB updates
    pub fn get_oob_updates(&self) -> &[(String, String)] {
        &self.oob_updates
    }
}

/// Extension trait to add helper methods to Result
pub trait ResultExt<T, E> {
    /// Convert to ActionResponse
    fn action(self) -> Result<ActionResponse<T>, E>;

    /// Add toast message to Ok variant
    fn toast(self, message: impl Into<String>) -> Result<ActionResponse<T>, E>;

    /// Add OOB update to Ok variant
    fn oob<U: ToString>(self, target: impl Into<String>, content: U) -> Result<ActionResponse<T>, E>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn action(self) -> Result<ActionResponse<T>, E> {
        self.map(ActionResponse::new)
    }

    fn toast(self, message: impl Into<String>) -> Result<ActionResponse<T>, E> {
        self.map(|v| ActionResponse::new(v).toast(message))
    }

    fn oob<U: ToString>(self, target: impl Into<String>, content: U) -> Result<ActionResponse<T>, E> {
        self.map(|v| ActionResponse::new(v).oob(target, content))
    }
}

/// Action metadata extracted from function name
#[derive(Debug, Clone, PartialEq)]
pub struct ActionInfo {
    pub method: ActionMethod,
    pub function_name: String,
    pub route_name: String,
}

/// HTTP methods for actions
#[derive(Debug, Clone, PartialEq)]
pub enum ActionMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl ActionInfo {
    /// Parse action info from function name
    ///
    /// Examples:
    /// - `get_users` -> GET, function: get_users, route: users
    /// - `post_user` -> POST, function: post_user, route: user
    /// - `delete_user` -> DELETE, function: delete_user, route: user
    pub fn from_function_name(name: &str) -> Option<Self> {
        let parts: Vec<&str> = name.splitn(2, '_').collect();
        if parts.len() != 2 {
            return None;
        }

        let method = match parts[0] {
            "get" => ActionMethod::Get,
            "post" => ActionMethod::Post,
            "put" => ActionMethod::Put,
            "patch" => ActionMethod::Patch,
            "delete" => ActionMethod::Delete,
            _ => return None,
        };

        Some(Self {
            method,
            function_name: name.to_string(),
            route_name: parts[1].to_string(),
        })
    }

    /// Check if method matches
    pub fn matches_method(&self, http_method: &str) -> bool {
        match self.method {
            ActionMethod::Get => http_method.eq_ignore_ascii_case("GET"),
            ActionMethod::Post => http_method.eq_ignore_ascii_case("POST"),
            ActionMethod::Put => http_method.eq_ignore_ascii_case("PUT"),
            ActionMethod::Patch => http_method.eq_ignore_ascii_case("PATCH"),
            ActionMethod::Delete => http_method.eq_ignore_ascii_case("DELETE"),
        }
    }
}

/// Action registry for managing actions in templates
#[derive(Clone)]
pub struct ActionRegistry {
    actions: HashMap<String, Vec<ActionInfo>>,
}

impl ActionRegistry {
    /// Create a new action registry
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Register an action (functional builder pattern)
    pub fn with_action(mut self, template_path: impl Into<String>, action: ActionInfo) -> Self {
        self.actions
            .entry(template_path.into())
            .or_default()
            .push(action);
        self
    }

    /// Register multiple actions (functional builder pattern)
    pub fn with_actions(mut self, template_path: impl Into<String>, actions: Vec<ActionInfo>) -> Self {
        self.actions
            .entry(template_path.into())
            .or_default()
            .extend(actions);
        self
    }

    /// Register an action (deprecated - use with_action for FP style)
    #[deprecated(since = "0.1.0", note = "Use with_action() for functional programming style")]
    pub fn register(&mut self, template_path: &str, action: ActionInfo) {
        *self = std::mem::replace(self, Self::new())
            .with_action(template_path.to_string(), action);
    }

    /// Find action for a template and HTTP method
    pub fn find_action(&self, template_path: &str, http_method: &str) -> Option<&ActionInfo> {
        self.actions
            .get(template_path)
            .and_then(|actions| actions.iter().find(|action| action.matches_method(http_method)))
    }

    /// Get all actions for a template
    pub fn get_actions(&self, template_path: &str) -> Option<&[ActionInfo]> {
        self.actions.get(template_path).map(|v| v.as_slice())
    }

    /// Get all template paths
    pub fn template_paths(&self) -> impl Iterator<Item = &String> {
        self.actions.keys()
    }

    /// Get all actions across all templates
    pub fn all_actions(&self) -> impl Iterator<Item = (&String, &Vec<ActionInfo>)> {
        self.actions.iter()
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_info_parsing() {
        let info = ActionInfo::from_function_name("get_users").unwrap();
        assert_eq!(info.method, ActionMethod::Get);
        assert_eq!(info.function_name, "get_users");
        assert_eq!(info.route_name, "users");

        let info = ActionInfo::from_function_name("post_user").unwrap();
        assert_eq!(info.method, ActionMethod::Post);
        assert_eq!(info.route_name, "user");

        let info = ActionInfo::from_function_name("delete_users").unwrap();
        assert_eq!(info.method, ActionMethod::Delete);

        assert!(ActionInfo::from_function_name("invalid").is_none());
        assert!(ActionInfo::from_function_name("notaverb_test").is_none());
    }

    #[test]
    fn test_method_matching() {
        let get_action = ActionInfo::from_function_name("get_users").unwrap();
        assert!(get_action.matches_method("GET"));
        assert!(!get_action.matches_method("POST"));

        let post_action = ActionInfo::from_function_name("post_user").unwrap();
        assert!(post_action.matches_method("POST"));
        assert!(!post_action.matches_method("GET"));
    }

    #[test]
    fn test_empty_response() {
        let empty = Empty::new()
            .toast("User created!")
            .oob("user-count", "42");

        let (headers, content) = empty.build();

        // Check HX-Trigger header for toast
        assert!(headers.contains_key("HX-Trigger"));

        // Check OOB content
        assert!(content.contains(r#"id="user-count""#));
        assert!(content.contains("42"));
    }

    #[test]
    fn test_action_registry_functional_builder() {
        let registry = ActionRegistry::new()
            .with_action(
                "users/index",
                ActionInfo::from_function_name("get_users").unwrap(),
            )
            .with_action(
                "users/create",
                ActionInfo::from_function_name("post_user").unwrap(),
            );

        assert!(registry.find_action("users/index", "GET").is_some());
        assert!(registry.find_action("users/create", "POST").is_some());
        assert!(registry.find_action("users/index", "POST").is_none());
    }

    #[test]
    fn test_action_registry_with_actions() {
        let actions = vec![
            ActionInfo::from_function_name("get_users").unwrap(),
            ActionInfo::from_function_name("post_user").unwrap(),
        ];

        let registry = ActionRegistry::new()
            .with_actions("users", actions);

        assert!(registry.find_action("users", "GET").is_some());
        assert!(registry.find_action("users", "POST").is_some());
        assert_eq!(registry.get_actions("users").unwrap().len(), 2);
    }
}
