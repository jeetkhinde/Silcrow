//! Custom garde validators for RHTMX-specific validation features
//!
//! This module provides custom validators that extend garde's built-in validators
//! with RHTMX-specific functionality like public email domain blocking,
//! custom password strength tiers, and domain blocklists.

/// Static list of public email domains to block
///
/// These are common free email providers that some business applications
/// may want to reject in favor of company/organization email addresses.
pub static PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com",
    "yahoo.com",
    "hotmail.com",
    "outlook.com",
    "aol.com",
    "icloud.com",
    "mail.com",
    "protonmail.com",
    "zoho.com",
    "yandex.com",
];

/// Validator: Block public email domains
///
/// This validator checks if an email address uses a public email provider
/// (like Gmail, Yahoo, etc.) and rejects it if found.
///
/// # Example
///
/// ```ignore
/// use garde::Validate;
///
/// #[derive(Validate)]
/// struct BusinessForm {
///     #[garde(email, custom(no_public_email))]
///     email: String,
/// }
/// ```
pub fn no_public_email(value: &str, _ctx: &()) -> Result<(), garde::Error> {
    let domain = extract_domain(value);
    let domain_lower = domain.to_lowercase();

    if PUBLIC_DOMAINS.iter().any(|&d| d == domain_lower) {
        return Err(garde::Error::new("public email domains are not allowed"));
    }

    Ok(())
}

/// Validator: Block specific email domains
///
/// This validator checks against a custom list of blocked domains.
/// Useful for blocking specific competitors, spam domains, etc.
///
/// # Example
///
/// ```ignore
/// let blocked = vec!["competitor.com".to_string(), "spam.net".to_string()];
/// blocked_domain_validator("user@competitor.com", &blocked); // Error
/// ```
pub fn blocked_domain_validator(value: &str, blocked: &[String]) -> Result<(), garde::Error> {
    let domain = extract_domain(value);
    let domain_lower = domain.to_lowercase();

    if blocked.iter().any(|d| d.to_lowercase() == domain_lower) {
        return Err(garde::Error::new("this email domain is blocked"));
    }

    Ok(())
}

/// Validator: Password strength validation
///
/// Implements three tiers of password strength:
/// - `basic`: 6+ characters
/// - `medium`: 8+ characters with uppercase, lowercase, and digit
/// - `strong`: 8+ characters with uppercase, lowercase, digit, and special character
///
/// # Example
///
/// ```ignore
/// password_strength("Abcd123!", "strong"); // Ok
/// password_strength("password", "medium"); // Error
/// ```
pub fn password_strength(value: &str, tier: &str) -> Result<(), garde::Error> {
    let has_upper = value.chars().any(|c| c.is_uppercase());
    let has_lower = value.chars().any(|c| c.is_lowercase());
    let has_digit = value.chars().any(|c| c.is_numeric());
    let has_special = value
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?/~`".contains(c));

    match tier {
        "basic" => {
            if value.len() < 6 {
                return Err(garde::Error::new(
                    "password must be at least 6 characters",
                ));
            }
        }
        "medium" => {
            if value.len() < 8 {
                return Err(garde::Error::new(
                    "password must be at least 8 characters",
                ));
            }
            if !(has_upper && has_lower && has_digit) {
                return Err(garde::Error::new(
                    "password must contain uppercase, lowercase, and digit",
                ));
            }
        }
        "strong" => {
            if value.len() < 8 {
                return Err(garde::Error::new(
                    "password must be at least 8 characters",
                ));
            }
            if !(has_upper && has_lower && has_digit && has_special) {
                return Err(garde::Error::new(
                    "password must contain uppercase, lowercase, digit, and special character",
                ));
            }
        }
        _ => {
            return Err(garde::Error::new("invalid password strength tier"));
        }
    }

    Ok(())
}

/// Extract domain from email address
///
/// Returns the domain portion of an email address (everything after @).
/// If the email is malformed, returns an empty string.
fn extract_domain(email: &str) -> &str {
    email.split('@').nth(1).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("user@example.com"), "example.com");
        assert_eq!(extract_domain("test@gmail.com"), "gmail.com");
        assert_eq!(extract_domain("invalid-email"), "");
        assert_eq!(extract_domain(""), "");
    }

    #[test]
    fn test_no_public_email() {
        // Public domains should fail
        assert!(no_public_email("user@gmail.com", &()).is_err());
        assert!(no_public_email("test@yahoo.com", &()).is_err());
        assert!(no_public_email("someone@GMAIL.COM", &()).is_err()); // Case insensitive

        // Business domains should pass
        assert!(no_public_email("user@company.com", &()).is_ok());
        assert!(no_public_email("test@business.org", &()).is_ok());
    }

    #[test]
    fn test_blocked_domain_validator() {
        let blocked = vec!["competitor.com".to_string(), "spam.net".to_string()];

        // Blocked domains should fail
        assert!(blocked_domain_validator("user@competitor.com", &blocked).is_err());
        assert!(blocked_domain_validator("test@SPAM.NET", &blocked).is_err()); // Case insensitive

        // Non-blocked domains should pass
        assert!(blocked_domain_validator("user@allowed.com", &blocked).is_ok());
        assert!(blocked_domain_validator("test@business.org", &blocked).is_ok());
    }

    #[test]
    fn test_password_strength_basic() {
        // Basic tier: 6+ characters
        assert!(password_strength("abc123", "basic").is_ok());
        assert!(password_strength("123456", "basic").is_ok());

        // Too short
        assert!(password_strength("abc12", "basic").is_err());
    }

    #[test]
    fn test_password_strength_medium() {
        // Medium tier: 8+ with upper, lower, digit
        assert!(password_strength("Abcd1234", "medium").is_ok());
        assert!(password_strength("Password1", "medium").is_ok());

        // Too short
        assert!(password_strength("Abcd123", "medium").is_err());

        // Missing uppercase
        assert!(password_strength("abcd1234", "medium").is_err());

        // Missing lowercase
        assert!(password_strength("ABCD1234", "medium").is_err());

        // Missing digit
        assert!(password_strength("Abcdabcd", "medium").is_err());
    }

    #[test]
    fn test_password_strength_strong() {
        // Strong tier: 8+ with upper, lower, digit, special
        assert!(password_strength("Abcd123!", "strong").is_ok());
        assert!(password_strength("P@ssw0rd", "strong").is_ok());

        // Missing special character
        assert!(password_strength("Abcd1234", "strong").is_err());

        // Too short
        assert!(password_strength("Abc12!@", "strong").is_err());

        // Missing digit
        assert!(password_strength("Abcdefg!", "strong").is_err());
    }

    #[test]
    fn test_password_strength_invalid_tier() {
        assert!(password_strength("anything", "invalid").is_err());
    }
}
