//! Email validation functions

use alloc::string::String;

/// List of common public email domains
const PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com",
    "yahoo.com",
    "hotmail.com",
    "outlook.com",
    "icloud.com",
    "aol.com",
    "mail.com",
    "protonmail.com",
    "yandex.com",
    "zoho.com",
];

/// Validates basic email format
///
/// Checks for:
/// - Contains exactly one '@' symbol
/// - Has content before and after '@'
/// - Has at least one '.' in domain part
/// - Minimum length requirements
pub fn is_valid_email(email: &str) -> bool {
    // Basic email validation (works reliably)
    if email.len() < 3 {
        return false;
    }

    let parts: alloc::vec::Vec<&str> = email.split('@').collect();

    // Must have exactly one @ symbol
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part validation
    if local.is_empty() || local.len() > 64 {
        return false;
    }

    // Domain part validation
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Domain must have at least one dot
    if !domain.contains('.') {
        return false;
    }

    // Domain can't start or end with dot or hyphen
    if domain.starts_with('.') || domain.ends_with('.')
        || domain.starts_with('-') || domain.ends_with('-') {
        return false;
    }

    // Check for consecutive dots
    if domain.contains("..") {
        return false;
    }

    // Basic character validation for local part
    let valid_local_chars = |c: char| {
        c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+'
    };

    if !local.chars().all(valid_local_chars) {
        return false;
    }

    // Basic character validation for domain
    let valid_domain_chars = |c: char| {
        c.is_alphanumeric() || c == '.' || c == '-'
    };

    if !domain.chars().all(valid_domain_chars) {
        return false;
    }

    // TLD must be at least 2 characters
    if let Some(last_dot_pos) = domain.rfind('.') {
        let tld = &domain[last_dot_pos + 1..];
        if tld.len() < 2 {
            return false;
        }
    }

    true
}

/// Checks if email domain is a public domain (gmail, yahoo, etc.)
pub fn is_public_domain(email: &str) -> bool {
    #[cfg(feature = "garde")]
    {
        // Use garde custom validator
        use crate::garde_validators::no_public_email;
        no_public_email(email, &()).is_err()
    }

    #[cfg(not(feature = "garde"))]
    {
        // Fallback implementation
        if let Some(domain) = email.split('@').nth(1) {
            PUBLIC_DOMAINS.iter().any(|&d| d.eq_ignore_ascii_case(domain))
        } else {
            false
        }
    }
}

/// Checks if email domain is in the blocked list
pub fn is_blocked_domain(email: &str, blocked: &[String]) -> bool {
    #[cfg(feature = "garde")]
    {
        // Use garde custom validator
        use crate::garde_validators::blocked_domain_validator;
        blocked_domain_validator(email, blocked).is_err()
    }

    #[cfg(not(feature = "garde"))]
    {
        // Fallback implementation
        if let Some(domain) = email.split('@').nth(1) {
            blocked.iter().any(|b| b == domain)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@example.co.uk"));
        assert!(is_valid_email("user+tag@example.com"));
        assert!(is_valid_email("user_name@example-domain.com"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("@"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@@example.com"));
        assert!(!is_valid_email("user@example"));
        assert!(!is_valid_email("user@.com"));
        assert!(!is_valid_email("user@example..com"));
    }

    #[test]
    fn test_public_domains() {
        assert!(is_public_domain("user@gmail.com"));
        assert!(is_public_domain("user@yahoo.com"));
        assert!(is_public_domain("user@GMAIL.COM"));  // Case-insensitive
        assert!(is_public_domain("user@Yahoo.Com"));  // Case-insensitive
        assert!(!is_public_domain("user@company.com"));
    }

    #[test]
    fn test_blocked_domains() {
        let blocked = vec!["spam.com".to_string(), "blocked.net".to_string()];
        assert!(is_blocked_domain("user@spam.com", &blocked));
        assert!(is_blocked_domain("user@blocked.net", &blocked));
        assert!(!is_blocked_domain("user@allowed.com", &blocked));
    }
}
