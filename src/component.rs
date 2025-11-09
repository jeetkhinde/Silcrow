// File: src/component.rs
// Purpose: Component trait and registry for #[component] macro

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for components that can be rendered
pub trait Component: Send + Sync {
    /// Get the component name
    fn name(&self) -> &'static str;

    /// Render the component with the given props JSON
    fn render(&self, props: serde_json::Value) -> anyhow::Result<String>;

    /// Check if this is a public component (accessible via HTTP)
    fn is_public(&self) -> bool;
}

/// Global component registry
pub struct ComponentRegistry {
    components: HashMap<String, Arc<dyn Component>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component
    pub fn register(&mut self, component: Arc<dyn Component>) {
        self.components.insert(component.name().to_string(), component);
    }

    /// Get a component by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Component>> {
        self.components.get(name).cloned()
    }

    /// Get all public components
    pub fn public_components(&self) -> Vec<String> {
        self.components
            .iter()
            .filter(|(_, comp)| comp.is_public())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// List all component names
    pub fn list_all(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    /// Global component registry
    pub static ref COMPONENT_REGISTRY: Mutex<ComponentRegistry> = Mutex::new(ComponentRegistry::new());
}

/// Helper function to register a component
pub fn register_component(component: Arc<dyn Component>) {
    if let Ok(mut registry) = COMPONENT_REGISTRY.lock() {
        registry.register(component);
    }
}

/// Helper function to get a component
pub fn get_component(name: &str) -> Option<Arc<dyn Component>> {
    if let Ok(registry) = COMPONENT_REGISTRY.lock() {
        registry.get(name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent;

    impl Component for TestComponent {
        fn name(&self) -> &'static str {
            "test"
        }

        fn render(&self, _props: serde_json::Value) -> anyhow::Result<String> {
            Ok("<div>Test</div>".to_string())
        }

        fn is_public(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = ComponentRegistry::new();
        let comp = Arc::new(TestComponent);
        registry.register(comp);

        assert!(registry.get("test").is_some());
        assert_eq!(registry.list_all().len(), 1);
    }
}
