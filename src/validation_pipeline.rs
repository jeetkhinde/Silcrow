// File: src/validation_pipeline.rs
// Purpose: Pipeline for deserializing, validating, and handling form submissions

use crate::action_executor::deserialize_form;
use crate::form_context::FormContext;
use crate::request_context::FormData;
use crate::validation::Validate;
use std::collections::HashMap;

/// Result of the validation pipeline
pub enum ValidationPipelineResult<T> {
    /// Validation passed, data is ready for processing
    Valid(T),
    /// Validation failed, contains errors and original form values
    Invalid(FormContext),
}

impl<T> ValidationPipelineResult<T> {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationPipelineResult::Valid(_))
    }

    /// Check if validation failed
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// Extract the valid value if validation passed
    pub fn ok(self) -> Option<T> {
        match self {
            ValidationPipelineResult::Valid(data) => Some(data),
            ValidationPipelineResult::Invalid(_) => None,
        }
    }

    /// Extract the form context if validation failed
    pub fn err(self) -> Option<FormContext> {
        match self {
            ValidationPipelineResult::Valid(_) => None,
            ValidationPipelineResult::Invalid(context) => Some(context),
        }
    }
}

/// Execute the validation pipeline
///
/// This function:
/// 1. Deserializes form data into the request type
/// 2. Validates the request using the Validate trait
/// 3. Returns either the valid request or validation errors with original values
pub fn validate_request<T: serde::de::DeserializeOwned + Validate>(
    form_data: &FormData,
) -> ValidationPipelineResult<T> {
    // Deserialize form data
    let request = match deserialize_form::<T>(form_data) {
        Ok(req) => req,
        Err(e) => {
            // Deserialization error - return as validation error
            let mut errors = HashMap::new();
            errors.insert(
                "_form".to_string(),
                format!("Failed to parse form data: {}", e),
            );
            return ValidationPipelineResult::Invalid(FormContext::new(
                errors,
                form_data.as_map().clone(),
            ));
        }
    };

    // Validate the request
    match request.validate() {
        Ok(()) => ValidationPipelineResult::Valid(request),
        Err(errors) => ValidationPipelineResult::Invalid(FormContext::new(
            errors,
            form_data.as_map().clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct TestForm {
        name: String,
        email: String,
    }

    impl Validate for TestForm {
        fn validate(&self) -> Result<(), HashMap<String, String>> {
            let mut errors = HashMap::new();

            if self.name.trim().is_empty() {
                errors.insert("name".to_string(), "Name is required".to_string());
            }

            if !self.email.contains('@') {
                errors.insert("email".to_string(), "Invalid email format".to_string());
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    #[test]
    fn test_valid_request() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("email".to_string(), "john@example.com".to_string());

        let form = FormData::from_fields(fields);
        let result = validate_request::<TestForm>(&form);

        assert!(result.is_valid());
    }

    #[test]
    fn test_invalid_email() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("email".to_string(), "invalid-email".to_string());

        let form = FormData::from_fields(fields);
        let result = validate_request::<TestForm>(&form);

        assert!(result.is_invalid());
        let context = result.err().expect("Should have errors");
        assert!(context.has_error("email"));
        assert_eq!(context.get_value("name"), Some("John"));
    }

    #[test]
    fn test_validator_directly() {
        let form = TestForm {
            name: "".to_string(),
            email: "bad".to_string(),
        };

        let result = form.validate();
        assert!(result.is_err());
        let errors = result.err().unwrap();
        assert!(errors.contains_key("name"));
        assert!(errors.contains_key("email"));
    }

    #[test]
    fn test_multiple_errors() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "   ".to_string()); // Whitespace, will be trimmed to ""
        fields.insert("email".to_string(), "bad-email".to_string());

        let form = FormData::from_fields(fields);
        let result = validate_request::<TestForm>(&form);

        assert!(result.is_invalid());
        let context = result.err().expect("Should have errors");
        assert!(context.has_error("name"), "Expected name error, got errors: {:?}", context.errors);
        assert!(context.has_error("email"));
        assert!(context.has_errors());
    }

    #[test]
    fn test_deserialization_error_returns_invalid() {
        // Create form data with missing required field
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        // Missing "email" field which is required

        let form = FormData::from_fields(fields);
        let result = validate_request::<TestForm>(&form);

        // Should be invalid due to deserialization error (missing field)
        assert!(result.is_invalid());
        let context = result.err().expect("Should have deserialization error");
        assert!(context.has_error("_form"), "Expected _form error for deserialization");
    }

    #[test]
    fn test_validation_result_methods() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("email".to_string(), "john@example.com".to_string());

        let form = FormData::from_fields(fields);
        let result = validate_request::<TestForm>(&form);

        // Test methods on valid result
        assert!(result.is_valid());
        assert!(!result.is_invalid());

        // Test ok() separately
        let result2 = validate_request::<TestForm>(&form);
        assert!(result2.ok().is_some());

        // Test err() separately
        let result3 = validate_request::<TestForm>(&form);
        assert!(result3.err().is_none());
    }

    #[test]
    fn test_form_values_preserved_on_error() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "Alice".to_string());
        fields.insert("email".to_string(), "invalid".to_string());

        let form = FormData::from_fields(fields);
        let result = validate_request::<TestForm>(&form);

        // Original values should be preserved in form context
        let context = result.err().expect("Should have errors");
        assert_eq!(context.get_value("name"), Some("Alice"), "Name should be preserved");
        assert_eq!(context.get_value("email"), Some("invalid"), "Email should be preserved");
    }

    #[test]
    fn test_empty_form_validation() {
        let form = FormData::new();
        let result = validate_request::<TestForm>(&form);

        // Empty form should fail validation
        assert!(result.is_invalid());
        let context = result.err().expect("Should have errors");
        // Should have deserialization error for missing required fields
        assert!(context.has_error("_form") || context.has_error("name"));
    }
}
