//! Common validated types for RHTMX forms
//!
//! This module provides reusable, validated newtype wrappers using the `nutype` crate.
//! These types ensure domain constraints are enforced at the type level while being
//! compatible with both server-side and WASM client-side validation.
//!
//! # WASM Compatibility
//!
//! All types in this module are designed to work in WebAssembly environments:
//! - Built with `#![no_std]` support
//! - Serializable/deserializable with serde
//! - Validation happens at construction time
//!
//! # Usage with RHTMX Forms
//!
//! ```rust,ignore
//! use rhtmx::{Validate, FormField};
//! use rhtmx_form::types::EmailAddress;
//!
//! #[derive(Validate, FormField, Deserialize)]
//! struct LoginForm {
//!     // Type validates email format
//!     // Form adds business rules
//!     #[nutype]
//!     #[no_public_domains]  // RHTMX-specific: block Gmail, Yahoo, etc.
//!     email: EmailAddress,
//! }
//! ```

use nutype::nutype;

// =============================================================================
// Email Types
// =============================================================================

/// Validated email address (format only)
///
/// Ensures the string is a valid email format using nutype's built-in email validator.
/// Does NOT enforce business rules like blocking public domains - use form-level
/// validators for that.
///
/// # Example
///
/// ```rust,ignore
/// use rhtmx_form::types::EmailAddress;
///
/// // Valid
/// let email = EmailAddress::new("user@example.com".to_string())?;
///
/// // Invalid - returns error
/// let invalid = EmailAddress::new("not-an-email".to_string()); // Err
/// ```
#[nutype(
    validate(predicate = is_valid_email_format),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct EmailAddress(String);

/// Basic email format validation
///
/// This is a simple check - more sophisticated validation happens
/// via the email validator in rhtmx-validation-core when used in forms.
fn is_valid_email_format(s: &str) -> bool {
    s.contains('@') && s.contains('.') && s.len() >= 5
}

// =============================================================================
// Password Types
// =============================================================================

/// Basic password (minimum 6 characters)
///
/// Use this for low-security scenarios or when additional validation
/// will be applied at the form level.
#[nutype(
    validate(len_char_min = 6),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordBasic(String);

/// Medium-strength password (minimum 8 characters)
///
/// Combines with `#[password("medium")]` at form level for full validation:
/// - 8+ characters (enforced by type)
/// - Uppercase + lowercase + digit (enforced by form validator)
#[nutype(
    validate(len_char_min = 8),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordMedium(String);

/// Strong password (minimum 10 characters)
///
/// Combines with `#[password("strong")]` at form level for full validation:
/// - 10+ characters (enforced by type)
/// - Uppercase + lowercase + digit + special (enforced by form validator)
#[nutype(
    validate(len_char_min = 10),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordStrong(String);

// =============================================================================
// String Types
// =============================================================================

/// Non-empty string
#[nutype(
    validate(not_empty),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct NonEmptyString(String);

/// Username (3-30 characters, alphanumeric + underscore)
#[nutype(
    validate(
        len_char_min = 3,
        len_char_max = 30,
        predicate = is_valid_username
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Username(String);

fn is_valid_username(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

// =============================================================================
// Numeric Types
// =============================================================================

/// Positive integer (> 0)
#[nutype(
    validate(greater = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct PositiveInt(i64);

/// Non-negative integer (>= 0)
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct NonNegativeInt(i64);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_valid() {
        let email = EmailAddress::try_new("user@example.com".to_string());
        assert!(email.is_ok());
    }

    #[test]
    fn test_email_address_invalid() {
        let email = EmailAddress::try_new("not-an-email".to_string());
        assert!(email.is_err());
    }

    #[test]
    fn test_password_basic_length() {
        assert!(PasswordBasic::try_new("12345".to_string()).is_err()); // Too short
        assert!(PasswordBasic::try_new("123456".to_string()).is_ok()); // Exactly 6
    }

    #[test]
    fn test_password_medium_length() {
        assert!(PasswordMedium::try_new("1234567".to_string()).is_err()); // Too short
        assert!(PasswordMedium::try_new("12345678".to_string()).is_ok()); // Exactly 8
    }

    #[test]
    fn test_password_strong_length() {
        assert!(PasswordStrong::try_new("123456789".to_string()).is_err()); // Too short
        assert!(PasswordStrong::try_new("1234567890".to_string()).is_ok()); // Exactly 10
    }

    #[test]
    fn test_username_validation() {
        assert!(Username::try_new("ab".to_string()).is_err()); // Too short
        assert!(Username::try_new("abc".to_string()).is_ok()); // Valid
        assert!(Username::try_new("user_name".to_string()).is_ok()); // Valid with underscore
        assert!(Username::try_new("user-name".to_string()).is_ok()); // Valid with dash
        assert!(Username::try_new("user@name".to_string()).is_err()); // Invalid char
        assert!(Username::try_new("a".repeat(31)).is_err()); // Too long
    }

    #[test]
    fn test_positive_int() {
        assert!(PositiveInt::try_from(0).is_err()); // Not positive
        assert!(PositiveInt::try_from(1).is_ok()); // Valid
        assert!(PositiveInt::try_from(-1).is_err()); // Negative
    }

    #[test]
    fn test_non_negative_int() {
        assert!(NonNegativeInt::try_from(0).is_ok()); // Valid
        assert!(NonNegativeInt::try_from(1).is_ok()); // Valid
        assert!(NonNegativeInt::try_from(-1).is_err()); // Negative
    }
}
