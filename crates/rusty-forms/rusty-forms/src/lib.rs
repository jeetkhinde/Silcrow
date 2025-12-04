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

// Re-export derive macros (always available)
pub use rusty_forms_derive::{FormField, Validate};

// Re-export core traits and types from validation crate
pub use rusty_forms_validation::{FieldAttrs, FormField as FormFieldTrait, Validate as ValidateTrait};

// Re-export validation module for access to validation functions
pub use rusty_forms_validation as validation;

// Re-export types module (if feature enabled)
#[cfg(feature = "nutype")]
pub use rusty_forms_types as types;
