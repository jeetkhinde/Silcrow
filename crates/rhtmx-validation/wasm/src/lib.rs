//! RHTMX Validation WASM
//!
//! WebAssembly bindings for RHTMX validation system.
//! Provides real-time client-side validation using the same logic as server-side.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use rusty_forms_validation as core;

/// Set panic hook for better error messages in the browser
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Validation error returned to JavaScript
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Validation rules for a single field
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FieldRules {
    // Email validators
    #[serde(default)]
    pub email: bool,
    #[serde(default)]
    pub no_public_domains: bool,
    #[serde(default)]
    pub blocked_domains: Option<Vec<String>>,

    // Password validators
    #[serde(default)]
    pub password: Option<String>, // "basic", "medium", "strong"

    // String length
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,

    // String matching
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    pub not_contains: Option<String>,
    #[serde(default)]
    pub starts_with: Option<String>,
    #[serde(default)]
    pub ends_with: Option<String>,

    // Equality
    #[serde(default)]
    pub equals: Option<String>,
    #[serde(default)]
    pub not_equals: Option<String>,

    // URL
    #[serde(default)]
    pub url: bool,

    // Required
    #[serde(default)]
    pub required: bool,

    // Custom message
    #[serde(default)]
    pub message: Option<String>,
}

/// Validate a single field value
///
/// # Arguments
/// * `field_name` - Name of the field being validated
/// * `value` - The value to validate
/// * `rules` - JavaScript object with validation rules
///
/// # Returns
/// Array of validation errors (empty if valid)
///
/// # Example (JavaScript)
/// ```javascript
/// const errors = validateField('email', 'user@example.com', {
///     email: true,
///     noPublicDomains: true,
///     required: true
/// });
/// ```
#[wasm_bindgen(js_name = validateField)]
pub fn validate_field(
    field_name: &str,
    value: &str,
    rules: JsValue,
) -> Result<JsValue, JsValue> {
    let rules: FieldRules = serde_wasm_bindgen::from_value(rules)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse rules: {}", e)))?;

    let mut errors = Vec::new();

    // Required validation
    if rules.required && value.trim().is_empty() {
        let msg = rules
            .message
            .clone()
            .unwrap_or_else(|| format!("{} is required", field_name));
        errors.push(ValidationError {
            field: field_name.to_string(),
            message: msg,
        });
        // If required and empty, skip other validations
        return Ok(serde_wasm_bindgen::to_value(&errors)?);
    }

    // Skip validation if value is empty (unless required)
    if value.is_empty() {
        return Ok(serde_wasm_bindgen::to_value(&errors)?);
    }

    // Email validation
    if rules.email && !core::is_valid_email(value) {
        errors.push(ValidationError {
            field: field_name.to_string(),
            message: rules
                .message
                .clone()
                .unwrap_or_else(|| "Invalid email address".to_string()),
        });
    }

    // No public domains
    if rules.no_public_domains && core::is_public_domain(value) {
        errors.push(ValidationError {
            field: field_name.to_string(),
            message: "Public email domains not allowed".to_string(),
        });
    }

    // Blocked domains
    if let Some(ref blocked) = rules.blocked_domains {
        if core::is_blocked_domain(value, blocked) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: "Email domain is blocked".to_string(),
            });
        }
    }

    // Password validation
    if let Some(ref pattern) = rules.password {
        if let Err(msg) = core::validate_password(value, pattern) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: rules.message.clone().unwrap_or(msg),
            });
        }
    }

    // String length validations
    if let Some(min) = rules.min_length {
        if let Err(msg) = core::validate_min_length(value, min) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: rules.message.clone().unwrap_or(msg),
            });
        }
    }

    if let Some(max) = rules.max_length {
        if let Err(msg) = core::validate_max_length(value, max) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: rules.message.clone().unwrap_or(msg),
            });
        }
    }

    // String matching validations
    if let Some(ref substring) = rules.contains {
        if !core::contains(value, substring) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: format!("Must contain '{}'", substring),
            });
        }
    }

    if let Some(ref substring) = rules.not_contains {
        if !core::not_contains(value, substring) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: format!("Must not contain '{}'", substring),
            });
        }
    }

    if let Some(ref prefix) = rules.starts_with {
        if !core::starts_with(value, prefix) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: format!("Must start with '{}'", prefix),
            });
        }
    }

    if let Some(ref suffix) = rules.ends_with {
        if !core::ends_with(value, suffix) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: format!("Must end with '{}'", suffix),
            });
        }
    }

    // Equality validations
    if let Some(ref expected) = rules.equals {
        if !core::equals(value, expected) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: format!("Must equal '{}'", expected),
            });
        }
    }

    if let Some(ref forbidden) = rules.not_equals {
        if !core::not_equals(value, forbidden) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: format!("Must not equal '{}'", forbidden),
            });
        }
    }

    // URL validation
    if rules.url && !core::is_valid_url(value) {
        errors.push(ValidationError {
            field: field_name.to_string(),
            message: rules
                .message
                .clone()
                .unwrap_or_else(|| "Invalid URL".to_string()),
        });
    }

    Ok(serde_wasm_bindgen::to_value(&errors)?)
}

/// Quick email validation
#[wasm_bindgen(js_name = isValidEmail)]
pub fn is_valid_email_js(email: &str) -> bool {
    core::is_valid_email(email)
}

/// Quick password validation
#[wasm_bindgen(js_name = validatePassword)]
pub fn validate_password_js(password: &str, pattern: &str) -> Option<String> {
    core::validate_password(password, pattern).err()
}

/// Quick URL validation
#[wasm_bindgen(js_name = isValidUrl)]
pub fn is_valid_url_js(url: &str) -> bool {
    core::is_valid_url(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_email_validation() {
        assert!(is_valid_email_js("user@example.com"));
        assert!(!is_valid_email_js("invalid-email"));
    }

    #[wasm_bindgen_test]
    fn test_password_validation() {
        assert!(validate_password_js("simple", "basic").is_none());
        assert!(validate_password_js("weak", "strong").is_some());
    }

    #[wasm_bindgen_test]
    fn test_url_validation() {
        assert!(is_valid_url_js("https://example.com"));
        assert!(!is_valid_url_js("not-a-url"));
    }
}
