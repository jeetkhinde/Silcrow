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
    /// use rhtmx_validation_core::password::PasswordPattern;
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
fn validate_basic(password: &str) -> Result<(), String> {
    if password.len() >= 6 {
        Ok(())
    } else {
        Err("Password must be at least 6 characters".to_string())
    }
}

/// Medium password validation: 8+ chars with uppercase, lowercase, and digit
fn validate_medium(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(
            "Password must contain uppercase, lowercase, and digit".to_string()
        );
    }

    Ok(())
}

/// Strong password validation: 8+ chars with uppercase, lowercase, digit, and special char
fn validate_strong(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| {
        matches!(c, '@' | '$' | '!' | '%' | '*' | '?' | '&' | '#' | '-' | '_' | '+' | '=' | '.' | ',')
    });

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character (@$!%*?&#-_+=.,)".to_string());
    }

    Ok(())
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
