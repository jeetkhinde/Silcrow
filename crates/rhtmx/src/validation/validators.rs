// File: src/validation/validators.rs
// Purpose: Basic validators (no external dependencies)

use regex::Regex;
use once_cell::sync::Lazy;

// Email validation regex
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

// URL validation regex
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Check if email is from a public domain (basic implementation)
pub fn is_public_domain(email: &str) -> bool {
    let public_domains = vec!["gmail.com", "yahoo.com", "hotmail.com", "outlook.com"];
    email.split('@').nth(1).map(|domain| {
        public_domains.iter().any(|&pd| domain == pd)
    }).unwrap_or(false)
}

/// Check if email domain is blocked (placeholder)
pub fn is_blocked_domain(_email: &str) -> bool {
    false // No blocked domains by default
}

/// Validate password strength
pub fn validate_password(password: &str, strength: &str) -> Result<(), String> {
    match strength {
        "strong" => {
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
                return Err("Password must contain at least one number".to_string());
            }
            if !password.chars().any(|c| !c.is_alphanumeric()) {
                return Err("Password must contain at least one special character".to_string());
            }
            Ok(())
        }
        "medium" => {
            if password.len() < 6 {
                return Err("Password must be at least 6 characters".to_string());
            }
            if !password.chars().any(|c| c.is_numeric()) {
                return Err("Password must contain at least one number".to_string());
            }
            Ok(())
        }
        "weak" => {
            if password.len() < 4 {
                return Err("Password must be at least 4 characters".to_string());
            }
            Ok(())
        }
        _ => Err("Invalid strength level. Use 'weak', 'medium', or 'strong'".to_string()),
    }
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
}

/// Check if string matches regex pattern
pub fn matches_regex(value: &str, pattern: &str) -> bool {
    if let Ok(regex) = Regex::new(pattern) {
        regex.is_match(value)
    } else {
        false
    }
}
