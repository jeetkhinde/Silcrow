// File: src/validation/validators.rs
// Purpose: Individual validator functions

use regex::Regex;
use once_cell::sync::Lazy;

// Common public email domains
static PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com", "yahoo.com", "hotmail.com", "outlook.com",
    "aol.com", "icloud.com", "mail.com", "protonmail.com",
    "yandex.com", "zoho.com", "gmx.com", "mail.ru"
];

// Password patterns (note: regex crate doesn't support lookaheads, so we validate manually)
static PASSWORD_BASIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^.{6,}$").unwrap()
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Check if email is from a public domain
pub fn is_public_domain(email: &str) -> bool {
    if let Some(domain) = email.split('@').nth(1) {
        PUBLIC_DOMAINS.iter().any(|&d| d.eq_ignore_ascii_case(domain))
    } else {
        false
    }
}

/// Check if email is from a blocked domain
pub fn is_blocked_domain(email: &str, blocked: &[String]) -> bool {
    if let Some(domain) = email.split('@').nth(1) {
        blocked.iter().any(|d| d.eq_ignore_ascii_case(domain))
    } else {
        false
    }
}

/// Validate password with pattern
pub fn validate_password(password: &str, pattern: &str) -> Result<(), String> {
    match pattern {
        "strong" => {
            // Validate password manually: at least 8 chars, uppercase, lowercase, digit, special char
            if password.len() < 8 {
                return Err("Password must be at least 8 characters".to_string());
            }
            if !password.chars().any(|c| c.is_uppercase()) {
                return Err("Password must contain at least one uppercase letter".to_string());
            }
            if !password.chars().any(|c| c.is_lowercase()) {
                return Err("Password must contain at least one lowercase letter".to_string());
            }
            if !password.chars().any(|c| c.is_numeric()) {
                return Err("Password must contain at least one digit".to_string());
            }
            if !password.chars().any(|c| "@$!%*?&".contains(c)) {
                return Err("Password must contain at least one special character (@$!%*?&)".to_string());
            }
            Ok(())
        }
        "medium" => {
            // Validate password manually: at least 8 chars, uppercase, lowercase, digit
            if password.len() < 8 {
                return Err("Password must be at least 8 characters".to_string());
            }
            if !password.chars().any(|c| c.is_uppercase()) {
                return Err("Password must contain at least one uppercase letter".to_string());
            }
            if !password.chars().any(|c| c.is_lowercase()) {
                return Err("Password must contain at least one lowercase letter".to_string());
            }
            if !password.chars().any(|c| c.is_numeric()) {
                return Err("Password must contain at least one digit".to_string());
            }
            Ok(())
        }
        "basic" => {
            if PASSWORD_BASIC_REGEX.is_match(password) {
                Ok(())
            } else {
                Err("Password must be at least 6 characters".to_string())
            }
        }
        // Custom regex pattern
        _ => {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(password) {
                    Ok(())
                } else {
                    Err("Password does not meet requirements".to_string())
                }
            } else {
                Err("Invalid password pattern".to_string())
            }
        }
    }
}

/// Check if string matches regex pattern
pub fn matches_regex(value: &str, pattern: &str) -> bool {
    if let Ok(regex) = Regex::new(pattern) {
        regex.is_match(value)
    } else {
        false
    }
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
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
