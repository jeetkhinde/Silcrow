// File: rhtml-macro/src/layout_registry.rs
// Purpose: Store layout metadata at compile-time for slot validation

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Metadata about a layout's slot contract
/// Note: Part of future layout system infrastructure
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LayoutMetadata {
    /// Layout file path
    pub file_path: String,
    /// Slot field definitions
    pub slots: Vec<SlotField>,
}

/// A single slot field definition from LayoutSlots struct
/// Note: Part of future layout system infrastructure
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SlotField {
    /// Field name (e.g., "title", "description")
    pub name: String,
    /// Field type (e.g., "&str", "impl Render")
    pub type_str: String,
    /// Whether this field is optional (wrapped in Option<T>)
    pub is_optional: bool,
    /// Whether this is the content slot (auto-filled)
    pub is_content: bool,
}

/// Global registry of layouts
/// Maps directory path to layout metadata
static LAYOUT_REGISTRY: Lazy<Mutex<HashMap<String, LayoutMetadata>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a layout with its metadata
#[allow(dead_code)]
pub fn register_layout(dir_path: String, metadata: LayoutMetadata) {
    LAYOUT_REGISTRY
        .lock()
        .unwrap()
        .insert(dir_path, metadata);
}

/// Get layout metadata for a directory
#[allow(dead_code)]
pub fn get_layout(dir_path: &str) -> Option<LayoutMetadata> {
    LAYOUT_REGISTRY.lock().unwrap().get(dir_path).cloned()
}

/// Check if a layout exists for a directory
#[allow(dead_code)]
pub fn has_layout(dir_path: &str) -> bool {
    LAYOUT_REGISTRY.lock().unwrap().contains_key(dir_path)
}

/// Clear all registered layouts (useful for testing)
#[allow(dead_code)]
pub fn clear_registry() {
    LAYOUT_REGISTRY.lock().unwrap().clear()
}
