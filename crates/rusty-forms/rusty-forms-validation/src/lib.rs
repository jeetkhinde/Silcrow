//! Rusty-Forms-Validation Core
//!
//! Pure Rust validation functions compatible with both std and no_std environments.
//! Used by both server-side validation and WASM client-side validation.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub mod collection;
pub mod email;
pub mod numeric;
pub mod password;
pub mod string;

// Re-export all validators
pub use collection::*;
pub use email::*;
pub use numeric::*;
pub use password::*;
pub use string::*;

/// Core validation trait that all forms implement
///
/// This trait is automatically implemented when you use `#[derive(Validate)]`
pub trait Validate {
    /// Validate the form and return errors by field name
    fn validate(&self) -> Result<(), BTreeMap<String, Vec<String>>>;
}

/// Form field attributes for HTML5 and client-side validation
///
/// This trait is automatically implemented when you use `#[derive(FormField)]`
pub trait FormField {
    /// Get validation attributes for a specific field
    fn field_attrs(&self, field_name: &str) -> FieldAttrs;

    /// Get list of all field names
    fn field_names(&self) -> Vec<&'static str>;
}

/// Attributes for a form field (HTML5 + data-validate JSON)
#[derive(Debug, Clone, Default)]
pub struct FieldAttrs {
    /// HTML5 validation attributes (type, required, min, max, etc.)
    pub html5_attrs: BTreeMap<String, String>,

    /// JSON for data-validate attribute (for WASM validation)
    pub data_validate: String,
}
