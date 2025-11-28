//! String validation functions

use alloc::string::String;
use alloc::format;

/// Validates string length
pub fn validate_min_length(s: &str, min: usize) -> Result<(), String> {
    if s.len() >= min {
        Ok(())
    } else {
        Err(format!("Must be at least {} characters", min))
    }
}

pub fn validate_max_length(s: &str, max: usize) -> Result<(), String> {
    if s.len() <= max {
        Ok(())
    } else {
        Err(format!("Must be at most {} characters", max))
    }
}

pub fn validate_length(s: &str, min: usize, max: usize) -> Result<(), String> {
    if s.len() >= min && s.len() <= max {
        Ok(())
    } else {
        Err(format!("Must be between {} and {} characters", min, max))
    }
}

/// String matching validators
pub fn contains(s: &str, substring: &str) -> bool {
    s.contains(substring)
}

pub fn not_contains(s: &str, substring: &str) -> bool {
    !s.contains(substring)
}

pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// URL validation (basic)
pub fn is_valid_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }

    // Must start with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }

    // Must have content after protocol
    let after_protocol = if let Some(after) = url.strip_prefix("https://") {
        after
    } else if let Some(after) = url.strip_prefix("http://") {
        after
    } else {
        return false;
    };

    if after_protocol.is_empty() {
        return false;
    }

    // Must have at least a domain
    if !after_protocol.contains('.') {
        return false;
    }

    true
}

/// Regex pattern matching
///
/// Note: Requires regex support on the server-side for full validation.
/// This is a placeholder that returns true (real validation happens server-side).
pub fn matches_regex(value: &str, pattern: &str) -> bool {
    // Without regex crate, we can't do pattern matching in no_std
    // Validation will happen on server with full regex support
    let _ = (value, pattern);
    true
}

/// Equality validators
pub fn equals(value: &str, expected: &str) -> bool {
    value == expected
}

pub fn not_equals(value: &str, forbidden: &str) -> bool {
    value != forbidden
}

/// Enum/value restriction
pub fn is_one_of(value: &str, allowed: &[&str]) -> bool {
    allowed.contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_validators() {
        assert!(validate_min_length("hello", 3).is_ok());
        assert!(validate_min_length("hi", 3).is_err());

        assert!(validate_max_length("hello", 10).is_ok());
        assert!(validate_max_length("verylongstring", 5).is_err());

        assert!(validate_length("hello", 3, 10).is_ok());
        assert!(validate_length("hi", 3, 10).is_err());
        assert!(validate_length("verylongstring", 3, 10).is_err());
    }

    #[test]
    fn test_string_matching() {
        assert!(contains("hello world", "world"));
        assert!(!contains("hello world", "foo"));

        assert!(not_contains("hello world", "foo"));
        assert!(!not_contains("hello world", "world"));

        assert!(starts_with("user_john", "user_"));
        assert!(!starts_with("admin_john", "user_"));

        assert!(ends_with("file.txt", ".txt"));
        assert!(!ends_with("file.doc", ".txt"));
    }

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://test.co.uk"));
        assert!(is_valid_url("https://example.com/path"));

        assert!(!is_valid_url(""));
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("https://"));
        assert!(!is_valid_url("http://nodomain"));
    }

    #[test]
    fn test_equality() {
        assert!(equals("test", "test"));
        assert!(!equals("test", "other"));

        assert!(not_equals("test", "other"));
        assert!(!not_equals("test", "test"));
    }

    #[test]
    fn test_enum_variant() {
        let allowed = &["admin", "user", "guest"];
        assert!(is_one_of("admin", allowed));
        assert!(is_one_of("user", allowed));
        assert!(!is_one_of("superuser", allowed));
    }
}
