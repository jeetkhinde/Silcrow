// File: src/action_handlers.rs
// Purpose: Manual registration of action handlers for different routes
// This will be replaced by a proc macro system in the future

use crate::action_executor::ActionResult;
use crate::request_context::RequestContext;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Type alias for an action handler function
pub type ActionHandler = fn(RequestContext) -> Pin<Box<dyn Future<Output = ActionResult> + Send>>;

/// Registry for action handlers
pub struct ActionHandlerRegistry {
    handlers: HashMap<String, HashMap<String, ActionHandler>>,
}

impl ActionHandlerRegistry {
    /// Create a new action handler registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an action handler for a route and method
    pub fn register(&mut self, route: &str, method: &str, handler: ActionHandler) {
        self.handlers
            .entry(route.to_string())
            .or_insert_with(HashMap::new)
            .insert(method.to_uppercase(), handler);
    }

    /// Find an action handler
    pub fn find(&self, route: &str, method: &str) -> Option<ActionHandler> {
        self.handlers
            .get(route)
            .and_then(|methods| methods.get(&method.to_uppercase()).copied())
    }

    /// Check if a route has an action
    pub fn has_action(&self, route: &str, method: &str) -> bool {
        self.handlers
            .get(route)
            .map(|methods| methods.contains_key(&method.to_uppercase()))
            .unwrap_or(false)
    }
}

impl Default for ActionHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all built-in action handlers
pub fn register_built_in_handlers(registry: &mut ActionHandlerRegistry) {
    use crate::example_actions;

    // Register example actions
    registry.register(
        "/examples/actions-validation",
        "GET",
        |ctx| Box::pin(example_actions::get_actions_validation(ctx)),
    );

    registry.register(
        "/examples/actions-validation",
        "POST",
        |ctx| Box::pin(example_actions::post_actions_validation(ctx)),
    );

    registry.register(
        "/examples/actions-validation",
        "PATCH",
        |ctx| Box::pin(example_actions::patch_actions_validation(ctx)),
    );

    registry.register(
        "/examples/actions-validation",
        "DELETE",
        |ctx| Box::pin(example_actions::delete_actions_validation(ctx)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_handler_registry() {
        let mut registry = ActionHandlerRegistry::new();

        // Create a dummy handler
        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "test".to_string(),
                headers: Default::default(),
            }
        });

        registry.register("/test", "GET", handler);

        assert!(registry.has_action("/test", "GET"));
        assert!(!registry.has_action("/test", "POST"));
        assert!(registry.find("/test", "get").is_some());
    }

    #[test]
    fn test_built_in_handlers_registered() {
        let mut registry = ActionHandlerRegistry::new();
        register_built_in_handlers(&mut registry);

        // Verify all example action handlers are registered
        assert!(registry.has_action("/examples/actions-validation", "GET"));
        assert!(registry.has_action("/examples/actions-validation", "POST"));
        assert!(registry.has_action("/examples/actions-validation", "PATCH"));
        assert!(registry.has_action("/examples/actions-validation", "DELETE"));
    }

    #[test]
    fn test_handler_case_insensitive_method() {
        let mut registry = ActionHandlerRegistry::new();

        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "success".to_string(),
                headers: Default::default(),
            }
        });

        registry.register("/test", "POST", handler);

        // Methods should be case-insensitive
        assert!(registry.find("/test", "post").is_some());
        assert!(registry.find("/test", "POST").is_some());
        assert!(registry.find("/test", "Post").is_some());
    }

    #[test]
    fn test_handler_route_matching() {
        let mut registry = ActionHandlerRegistry::new();

        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "matched".to_string(),
                headers: Default::default(),
            }
        });

        registry.register("/api/users", "GET", handler);

        // Exact path match
        assert!(registry.has_action("/api/users", "GET"));

        // Different path should not match
        assert!(!registry.has_action("/api/users/123", "GET"));
        assert!(!registry.has_action("/api/user", "GET"));
    }

    #[test]
    fn test_multiple_methods_same_route() {
        let mut registry = ActionHandlerRegistry::new();

        let get_handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "GET".to_string(),
                headers: Default::default(),
            }
        });

        let post_handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "POST".to_string(),
                headers: Default::default(),
            }
        });

        registry.register("/test", "GET", get_handler);
        registry.register("/test", "POST", post_handler);

        assert!(registry.has_action("/test", "GET"));
        assert!(registry.has_action("/test", "POST"));
        assert!(!registry.has_action("/test", "PUT"));
    }

    #[test]
    fn test_handler_execution_returns_html() {
        let mut registry = ActionHandlerRegistry::new();

        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "<p>Hello World</p>".to_string(),
                headers: Default::default(),
            }
        });

        registry.register("/test", "GET", handler);

        // Verify the handler was successfully registered and can be found
        let found = registry.find("/test", "GET");
        assert!(found.is_some(), "Handler should be found after registration");
        assert!(registry.has_action("/test", "GET"), "Registry should report action exists");
    }

    #[test]
    fn test_empty_registry() {
        let registry = ActionHandlerRegistry::new();

        assert!(!registry.has_action("/any/path", "GET"));
        assert!(registry.find("/any/path", "GET").is_none());
    }

    #[test]
    fn test_handler_not_found_returns_none() {
        let mut registry = ActionHandlerRegistry::new();

        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "test".to_string(),
                headers: Default::default(),
            }
        });

        registry.register("/test", "GET", handler);

        // Requesting wrong method should return None
        assert!(registry.find("/test", "DELETE").is_none());

        // Requesting wrong path should return None
        assert!(registry.find("/wrong", "GET").is_none());
    }
}
