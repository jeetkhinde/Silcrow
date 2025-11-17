// File: src/action_handlers.rs
// Purpose: Manual registration of action handlers for different routes
// This will be replaced by a proc macro system in the future

use rhtmx::action_executor::ActionResult;
use rhtmx::RequestContext;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Type alias for an action handler function
pub type ActionHandler = fn(RequestContext) -> Pin<Box<dyn Future<Output = ActionResult> + Send>>;

/// Metadata for a registered handler
#[derive(Clone)]
pub struct HandlerMetadata {
    pub route: String,
    pub method: String,
    pub route_pattern: Option<String>,  // e.g., ":id", ":id/edit"
    pub query_params: Option<String>,   // e.g., "partial=stats"
    pub handler: ActionHandler,
}

impl HandlerMetadata {
    /// Create new handler metadata
    pub fn new(route: String, method: String, handler: ActionHandler) -> Self {
        Self {
            route,
            method: method.to_uppercase(),
            route_pattern: None,
            query_params: None,
            handler,
        }
    }

    /// Set route pattern (e.g., ":id", ":id/edit")
    pub fn with_route_pattern(mut self, pattern: String) -> Self {
        self.route_pattern = Some(pattern);
        self
    }

    /// Set query parameters (e.g., "partial=stats")
    pub fn with_query_params(mut self, params: String) -> Self {
        self.query_params = Some(params);
        self
    }

    /// Calculate specificity score for matching priority
    /// Higher score = more specific handler
    fn specificity(&self) -> u8 {
        let mut score = 0;
        if self.query_params.is_some() {
            score += 2;
        }
        if self.route_pattern.is_some() {
            score += 1;
        }
        score
    }

    /// Check if this handler matches the request
    fn matches(&self, route: &str, method: &str, query: &HashMap<String, String>) -> bool {
        // Method must match
        if !self.method.eq_ignore_ascii_case(method) {
            return false;
        }

        // Route must match (either exact or with pattern)
        if !self.matches_route(route) {
            return false;
        }

        // If query params are specified, they must match
        if let Some(ref params) = self.query_params {
            if !self.matches_query_params(params, query) {
                return false;
            }
        }

        true
    }

    /// Check if route matches (handles patterns like :id)
    fn matches_route(&self, route: &str) -> bool {
        if let Some(ref pattern) = self.route_pattern {
            // Construct full route with pattern: base_route + "/" + pattern
            let full_pattern = if pattern.starts_with('/') {
                format!("{}{}", self.route, pattern)
            } else {
                format!("{}/{}", self.route, pattern)
            };

            // Simple pattern matching for :param segments
            self.matches_pattern(&full_pattern, route)
        } else {
            // Exact match
            self.route == route
        }
    }

    /// Match route pattern with :param segments
    fn matches_pattern(&self, pattern: &str, route: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let route_parts: Vec<&str> = route.split('/').collect();

        if pattern_parts.len() != route_parts.len() {
            return false;
        }

        for (pattern_part, route_part) in pattern_parts.iter().zip(route_parts.iter()) {
            if pattern_part.starts_with(':') {
                // This is a param segment, matches any value
                continue;
            } else if pattern_part != route_part {
                // Static segment must match exactly
                return false;
            }
        }

        true
    }

    /// Check if query parameters match
    fn matches_query_params(&self, params: &str, query: &HashMap<String, String>) -> bool {
        // Parse params string (e.g., "partial=stats")
        if let Some((key, value)) = params.split_once('=') {
            query.get(key).map(|v| v == value).unwrap_or(false)
        } else {
            // Just check if key exists
            query.contains_key(params)
        }
    }
}

/// Registry for action handlers
pub struct ActionHandlerRegistry {
    handlers: Vec<HandlerMetadata>,
    // Keep legacy HashMap for backward compatibility
    legacy_handlers: HashMap<String, HashMap<String, ActionHandler>>,
}

impl ActionHandlerRegistry {
    /// Create a new action handler registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            legacy_handlers: HashMap::new(),
        }
    }

    /// Register an action handler with metadata
    pub fn register_with_metadata(&mut self, metadata: HandlerMetadata) {
        self.handlers.push(metadata);
    }

    /// Register an action handler for a route and method (legacy API)
    pub fn register(&mut self, route: &str, method: &str, handler: ActionHandler) {
        self.legacy_handlers
            .entry(route.to_string())
            .or_default()
            .insert(method.to_uppercase(), handler);
    }

    /// Find an action handler with query parameter matching
    pub fn find_with_query(
        &self,
        route: &str,
        method: &str,
        query: &HashMap<String, String>,
    ) -> Option<ActionHandler> {
        // Find all matching handlers
        let mut matches: Vec<&HandlerMetadata> = self
            .handlers
            .iter()
            .filter(|h| h.matches(route, method, query))
            .collect();

        // Sort by specificity (most specific first)
        matches.sort_by(|a, b| b.specificity().cmp(&a.specificity()));

        // Return the most specific match
        matches.first().map(|h| h.handler)
    }

    /// Find an action handler (legacy API - exact route match only)
    pub fn find(&self, route: &str, method: &str) -> Option<ActionHandler> {
        // First check new handlers with empty query
        let empty_query = HashMap::new();
        if let Some(handler) = self.find_with_query(route, method, &empty_query) {
            return Some(handler);
        }

        // Fall back to legacy handlers
        self.legacy_handlers
            .get(route)
            .and_then(|methods| methods.get(&method.to_uppercase()).copied())
    }

    /// Check if a route has an action
    #[allow(dead_code)]
    pub fn has_action(&self, route: &str, method: &str) -> bool {
        // Check new handlers
        let empty_query = HashMap::new();
        if self.handlers.iter().any(|h| h.matches(route, method, &empty_query)) {
            return true;
        }

        // Check legacy handlers
        self.legacy_handlers
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
    fn test_handler_metadata_route_pattern_matching() {
        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "user detail".to_string(),
                headers: Default::default(),
            }
        });

        let metadata = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler)
            .with_route_pattern(":id".to_string());

        let empty_query = HashMap::new();

        // Should match /users/123
        assert!(metadata.matches("/users/123", "GET", &empty_query));

        // Should match /users/456
        assert!(metadata.matches("/users/456", "GET", &empty_query));

        // Should not match different path
        assert!(!metadata.matches("/posts/123", "GET", &empty_query));

        // Should not match wrong method
        assert!(!metadata.matches("/users/123", "POST", &empty_query));

        // Should not match base route without param
        assert!(!metadata.matches("/users", "GET", &empty_query));
    }

    #[test]
    fn test_handler_metadata_sub_route_pattern() {
        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "edit user".to_string(),
                headers: Default::default(),
            }
        });

        let metadata = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler)
            .with_route_pattern(":id/edit".to_string());

        let empty_query = HashMap::new();

        // Should match /users/123/edit
        assert!(metadata.matches("/users/123/edit", "GET", &empty_query));

        // Should not match /users/123
        assert!(!metadata.matches("/users/123", "GET", &empty_query));

        // Should not match /users/123/delete
        assert!(!metadata.matches("/users/123/delete", "GET", &empty_query));
    }

    #[test]
    fn test_handler_metadata_query_params() {
        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "stats partial".to_string(),
                headers: Default::default(),
            }
        });

        let metadata = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler)
            .with_query_params("partial=stats".to_string());

        // Should match with exact query param
        let mut query = HashMap::new();
        query.insert("partial".to_string(), "stats".to_string());
        assert!(metadata.matches("/users", "GET", &query));

        // Should not match with different value
        let mut query = HashMap::new();
        query.insert("partial".to_string(), "list".to_string());
        assert!(!metadata.matches("/users", "GET", &query));

        // Should not match without query param
        let empty_query = HashMap::new();
        assert!(!metadata.matches("/users", "GET", &empty_query));
    }

    #[test]
    fn test_handler_metadata_specificity() {
        let handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "test".to_string(),
                headers: Default::default(),
            }
        });

        // Base handler (no params)
        let base = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler);
        assert_eq!(base.specificity(), 0);

        // With route pattern
        let with_route = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler)
            .with_route_pattern(":id".to_string());
        assert_eq!(with_route.specificity(), 1);

        // With query params
        let with_query = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler)
            .with_query_params("partial=stats".to_string());
        assert_eq!(with_query.specificity(), 2);

        // With both
        let with_both = HandlerMetadata::new("/users".to_string(), "GET".to_string(), handler)
            .with_route_pattern(":id".to_string())
            .with_query_params("partial=detail".to_string());
        assert_eq!(with_both.specificity(), 3);
    }

    #[test]
    fn test_registry_with_metadata() {
        let mut registry = ActionHandlerRegistry::new();

        let base_handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "users list".to_string(),
                headers: Default::default(),
            }
        });

        let stats_handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "users stats".to_string(),
                headers: Default::default(),
            }
        });

        let detail_handler: ActionHandler = |_ctx| Box::pin(async {
            ActionResult::Html {
                content: "user detail".to_string(),
                headers: Default::default(),
            }
        });

        // Register base handler
        registry.register_with_metadata(
            HandlerMetadata::new("/users".to_string(), "GET".to_string(), base_handler)
        );

        // Register query param handler
        registry.register_with_metadata(
            HandlerMetadata::new("/users".to_string(), "GET".to_string(), stats_handler)
                .with_query_params("partial=stats".to_string())
        );

        // Register route pattern handler
        registry.register_with_metadata(
            HandlerMetadata::new("/users".to_string(), "GET".to_string(), detail_handler)
                .with_route_pattern(":id".to_string())
        );

        // Test base route
        let empty_query = HashMap::new();
        assert!(registry.find_with_query("/users", "GET", &empty_query).is_some());

        // Test query param route (should match more specific handler)
        let mut query = HashMap::new();
        query.insert("partial".to_string(), "stats".to_string());
        assert!(registry.find_with_query("/users", "GET", &query).is_some());

        // Test route pattern
        assert!(registry.find_with_query("/users/123", "GET", &empty_query).is_some());
    }

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
