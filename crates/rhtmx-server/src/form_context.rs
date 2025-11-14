// File: src/form_context.rs
// Purpose: Form context for templates to display validation errors and preserve values

use std::collections::HashMap;

/// Context for forms that includes validation errors and original values
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FormContext {
    /// Field names to error messages
    pub errors: HashMap<String, String>,
    /// Original field values submitted
    pub values: HashMap<String, String>,
}

impl FormContext {
    #[allow(dead_code)]
    /// Create a new form context with errors and values
    pub fn new(errors: HashMap<String, String>, values: HashMap<String, String>) -> Self {
        Self { errors, values }
    }

    #[allow(dead_code)]
    /// Create empty form context
    pub fn empty() -> Self {
        Self {
            errors: HashMap::new(),
            values: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    /// Check if field has an error
    pub fn has_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    #[allow(dead_code)]
    /// Get error message for a field
    pub fn get_error(&self, field: &str) -> Option<&str> {
        self.errors.get(field).map(|s| s.as_str())
    }

    #[allow(dead_code)]
    /// Get all errors
    pub fn get_errors(&self) -> &HashMap<String, String> {
        &self.errors
    }

    #[allow(dead_code)]
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[allow(dead_code)]
    /// Get original value for a field
    pub fn get_value(&self, field: &str) -> Option<&str> {
        self.values.get(field).map(|s| s.as_str())
    }

    #[allow(dead_code)]
    /// Get all original values
    pub fn get_values(&self) -> &HashMap<String, String> {
        &self.values
    }
}

impl Default for FormContext {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_context_errors() {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), "Invalid email format".to_string());

        let context = FormContext::new(errors, HashMap::new());

        assert!(context.has_error("email"));
        assert_eq!(context.get_error("email"), Some("Invalid email format"));
        assert!(context.has_errors());
    }

    #[test]
    fn test_form_context_values() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "John".to_string());

        let context = FormContext::new(HashMap::new(), values);

        assert_eq!(context.get_value("name"), Some("John"));
    }

    #[test]
    fn test_empty_form_context() {
        let context = FormContext::empty();
        assert!(!context.has_errors());
        assert!(context.get_error("any").is_none());
        assert!(context.get_value("any").is_none());
    }
}
