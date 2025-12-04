//! Password validation functions

use alloc::string::{String, ToString};

/// Password strength patterns
pub enum PasswordPattern {
    /// 6+ characters minimum
    Basic,
    /// 8+ chars with uppercase, lowercase, and digit
    Medium,
    /// 8+ chars with uppercase, lowercase, digit, and special character
    Strong,
    /// Custom regex pattern
    Custom(String),
}

impl PasswordPattern {
    /// Parse a password pattern from a string
    ///
    /// # Examples
    /// ```
    /// use rusty_forms_validation::password::PasswordPattern;
    /// assert!(matches!(PasswordPattern::parse("basic"), PasswordPattern::Basic));
    /// assert!(matches!(PasswordPattern::parse("medium"), PasswordPattern::Medium));
    /// assert!(matches!(PasswordPattern::parse("strong"), PasswordPattern::Strong));
    /// ```
    pub fn parse(pattern: &str) -> Self {
        match pattern {
            "basic" => PasswordPattern::Basic,
            "medium" => PasswordPattern::Medium,
            "strong" => PasswordPattern::Strong,
            custom => PasswordPattern::Custom(custom.to_string()),
        }
    }
}

/// Validates password against a pattern
///
/// Uses garde custom validator when available.
///
/// # Patterns
/// - "basic": 6+ characters
/// - "medium": 8+ chars with uppercase, lowercase, digit
/// - "strong": 8+ chars with uppercase, lowercase, digit, special char
/// - Any other string: treated as custom pattern name (not implemented in core)
pub fn validate_password(password: &str, pattern: &str) -> Result<(), String> {
    let pattern_enum = PasswordPattern::parse(pattern);

    match pattern_enum {
        PasswordPattern::Basic => validate_basic(password),
        PasswordPattern::Medium => validate_medium(password),
        PasswordPattern::Strong => validate_strong(password),
        PasswordPattern::Custom(_) => {
            // For custom patterns, we'd need regex support
            // For now, default to strong validation
            validate_strong(password)
        }
    }
}

/// Basic password validation: 6+ characters
/// Pure function: no side effects, deterministic
fn validate_basic(password: &str) -> Result<(), String> {
    (password.len() >= 6)
        .then_some(())
        .ok_or_else(|| "Password must be at least 6 characters".to_string())
}

/// Medium password validation: 8+ chars with uppercase, lowercase, and digit
/// Functional composition: chains validation checks
fn validate_medium(password: &str) -> Result<(), String> {
    let checks = [
        (password.len() >= 8, "Password must be at least 8 characters"),
        (password.chars().any(|c| c.is_uppercase()), "Password must contain uppercase letter"),
        (password.chars().any(|c| c.is_lowercase()), "Password must contain lowercase letter"),
        (password.chars().any(|c| c.is_numeric()), "Password must contain digit"),
    ];

    checks
        .iter()
        .find(|(valid, _)| !valid)
        .map(|(_, msg)| Err(msg.to_string()))
        .unwrap_or(Ok(()))
}

/// Strong password validation: 8+ chars with uppercase, lowercase, digit, and special char
/// Functional composition: uses declarative validation rules
fn validate_strong(password: &str) -> Result<(), String> {
    let checks = [
        (password.len() >= 8, "Password must be at least 8 characters"),
        (password.chars().any(|c| c.is_uppercase()), "Password must contain at least one uppercase letter"),
        (password.chars().any(|c| c.is_lowercase()), "Password must contain at least one lowercase letter"),
        (password.chars().any(|c| c.is_numeric()), "Password must contain at least one digit"),
        (
            password.chars().any(|c| {
                matches!(c, '@' | '$' | '!' | '%' | '*' | '?' | '&' | '#' | '-' | '_' | '+' | '=' | '.' | ',')
            }),
            "Password must contain at least one special character (@$!%*?&#-_+=.,)"
        ),
    ];

    checks
        .iter()
        .find(|(valid, _)| !valid)
        .map(|(_, msg)| Err(msg.to_string()))
        .unwrap_or(Ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_password() {
        assert!(validate_password("simple", "basic").is_ok());
        assert!(validate_password("123456", "basic").is_ok());
        assert!(validate_password("short", "basic").is_err());
    }

    #[test]
    fn test_medium_password() {
        assert!(validate_password("Password123", "medium").is_ok());
        assert!(validate_password("Test1234", "medium").is_ok());
        assert!(validate_password("lowercase1", "medium").is_err());
        assert!(validate_password("UPPERCASE1", "medium").is_err());
        assert!(validate_password("NoDigits", "medium").is_err());
        assert!(validate_password("Short1A", "medium").is_err());
    }

    #[test]
    fn test_strong_password() {
        assert!(validate_password("Password123!", "strong").is_ok());
        assert!(validate_password("Secure@Pass1", "strong").is_ok());
        assert!(validate_password("NoSpecial123", "strong").is_err());
        assert!(validate_password("nouppercas!1", "strong").is_err());
        assert!(validate_password("NOLOWERCASE!1", "strong").is_err());
        assert!(validate_password("NoDigits!Aa", "strong").is_err());
    }
}
