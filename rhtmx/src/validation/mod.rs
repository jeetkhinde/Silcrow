// File: src/validation/mod.rs
// Purpose: Validation runtime and validator trait

use std::collections::HashMap;

pub mod validators;

/// Trait for types that can be validated
///
/// Automatically implemented by #[derive(Validate)]
pub trait Validate {
    /// Validates the struct and returns validation errors
    ///
    /// Returns Ok(()) if valid, or Err with a map of field names to error messages
    fn validate(&self) -> Result<(), HashMap<String, String>>;
}

/// Result of validation with errors
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: HashMap<String, String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: HashMap::new(),
        }
    }

    /// Create a failed validation result
    pub fn failure(errors: HashMap<String, String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    /// Convert from Result
    pub fn from_result(result: Result<(), HashMap<String, String>>) -> Self {
        match result {
            Ok(()) => Self::success(),
            Err(errors) => Self::failure(errors),
        }
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get error for a specific field
    pub fn get_error(&self, field: &str) -> Option<&String> {
        self.errors.get(field)
    }
}
