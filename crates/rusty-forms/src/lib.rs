//! # rusty-forms
//!
//! A complete Rust form validation library with derive macros, type-safe validation,
//! and automatic client-side/server-side synchronization.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rusty_forms::{Validate, FormField};
//! use serde::Deserialize;
//!
//! #[derive(Validate, FormField, Deserialize)]
//! struct RegisterForm {
//!     #[email]
//!     #[no_public_domains]
//!     #[required]
//!     email: String,
//!
//!     #[min_length(8)]
//!     #[password("strong")]
//!     password: String,
//!
//!     #[min(18)]
//!     #[max(120)]
//!     age: i32,
//! }
//!
//! // Server-side validation
//! fn handle_form(form: RegisterForm) -> Result<(), HashMap<String, Vec<String>>> {
//!     form.validate()?;
//!     // Process valid form...
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **`default`** - Just the derive macros (minimal)
//! - **`validation`** - Include validation functions for custom use
//! - **`nutype`** - Include pre-built validated types (EmailAddress, Password, etc.)
//! - **`full`** - All features enabled
//!
//! ## Architecture
//!
//! This crate is a convenience wrapper that re-exports three component crates:
//!
//! - **`rusty-forms-derive`** - Procedural macros (`#[derive(Validate)]`, `#[derive(FormField)]`)
//! - **`rusty-forms-validation`** - Core validation functions (no_std compatible)
//! - **`rusty-forms-types`** - Pre-built validated types using nutype (optional)
//!
//! Most users should use this parent crate. Advanced users can depend on individual
//! components for fine-grained control.

#![doc(html_root_url = "https://docs.rs/rusty-forms/0.1.0")]

use std::collections::HashMap;

// Re-export derive macros (always available)
pub use rusty_forms_derive::{FormField, Validate};

// Re-export validation module (if feature enabled)
#[cfg(feature = "validation")]
pub use rusty_forms_validation as validation;

// Re-export types module (if feature enabled)
#[cfg(feature = "nutype")]
pub use rusty_forms_types as types;

/// Core validation trait that all forms implement
///
/// This trait is automatically implemented when you use `#[derive(Validate)]`
pub trait ValidateTrait {
    /// Validate the form and return errors by field name
    fn validate(&self) -> Result<(), HashMap<String, Vec<String>>>;
}

/// Form field attributes for HTML5 and client-side validation
///
/// This trait is automatically implemented when you use `#[derive(FormField)]`
pub trait FormFieldTrait {
    /// Get validation attributes for a specific field
    fn field_attrs(&self, field_name: &str) -> FieldAttrs;

    /// Get list of all field names
    fn field_names(&self) -> Vec<&'static str>;
}

/// Attributes for a form field (HTML5 + data-validate JSON)
#[derive(Debug, Clone, Default)]
pub struct FieldAttrs {
    /// HTML5 validation attributes (type, required, min, max, etc.)
    pub html5_attrs: HashMap<String, String>,

    /// JSON for data-validate attribute (for WASM validation)
    pub data_validate: String,
}

impl FieldAttrs {
    /// Render all attributes as HTML string
    pub fn render_all(&self) -> String {
        let mut attrs = vec![];

        for (key, value) in &self.html5_attrs {
            if value.is_empty() {
                attrs.push(key.clone());
            } else {
                attrs.push(format!(r#"{}="{}""#, key, value));
            }
        }

        if !self.data_validate.is_empty() {
            attrs.push(format!(r#"data-validate='{}'"#, self.data_validate));
        }

        attrs.join(" ")
    }
}
