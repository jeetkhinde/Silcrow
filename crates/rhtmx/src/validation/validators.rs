// File: src/validation/validators.rs
// Purpose: Re-export validators from rusty-forms + std-only validators
// This maintains backward compatibility while using the shared validation logic

use regex::Regex;
use once_cell::sync::Lazy;

// Re-export from rusty-forms validation module (no_std compatible validators)
pub use rusty_forms::validation::email::{
    is_valid_email,
    is_public_domain,
    is_blocked_domain,
};

pub use rusty_forms::validation::password::validate_password;

pub use rusty_forms::validation::string::is_valid_url;

// Regex matching (requires std, so kept here)
#[allow(dead_code)]
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

/// Check if string matches regex pattern (std-only function)
pub fn matches_regex(value: &str, pattern: &str) -> bool {
    if let Ok(regex) = Regex::new(pattern) {
        regex.is_match(value)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@example.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
    }

    #[test]
    fn test_public_domain() {
        assert!(is_public_domain("user@gmail.com"));
        assert!(is_public_domain("test@YAHOO.COM"));
        assert!(!is_public_domain("admin@company.com"));
    }

    #[test]
    fn test_password_validation() {
        // Strong password
        assert!(validate_password("Pass123!@#", "strong").is_ok());
        assert!(validate_password("weak", "strong").is_err());

        // Medium password
        assert!(validate_password("Pass1234", "medium").is_ok());
        assert!(validate_password("password", "medium").is_err());

        // Basic password
        assert!(validate_password("simple", "basic").is_ok());
        assert!(validate_password("bad", "basic").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://sub.example.com/path?query=1"));
        assert!(!is_valid_url("not a url"));
        assert!(!is_valid_url("ftp://example.com"));
    }

    #[test]
    fn test_regex_matching() {
        assert!(matches_regex("123-456-7890", r"^\d{3}-\d{3}-\d{4}$"));
        assert!(!matches_regex("123456789", r"^\d{3}-\d{3}-\d{4}$"));
    }
}
