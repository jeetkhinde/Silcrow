//! RHTMX Validation Core
//!
//! Pure Rust validation functions compatible with both std and no_std environments.
//! Used by both server-side validation and WASM client-side validation.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod email;
pub mod password;
pub mod string;
pub mod numeric;
pub mod collection;

// Re-export all validators
pub use email::*;
pub use password::*;
pub use string::*;
pub use numeric::*;
pub use collection::*;
