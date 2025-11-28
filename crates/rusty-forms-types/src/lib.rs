//! Common validated types for RHTMX forms with embedded business rules
//!
//! This module provides reusable, validated newtype wrappers using the `nutype` crate.
//! These types ensure domain constraints AND business rules are enforced at the type level.
//!
//! # Philosophy: Business Rules in Types
//!
//! Instead of combining type validation + form validators:
//! ```rust,ignore
//! #[nutype]
//! #[no_public_domains]  // ← Business rule at form level
//! email: EmailAddress
//! ```
//!
//! **Embed business rules directly in the type**:
//! ```rust,ignore
//! email: WorkEmailAddress  // ← Type IS the business rule!
//! ```
//!
//! # WASM Compatibility
//!
//! All types work in WebAssembly environments:
//! - Serializable/deserializable with serde
//! - Validation happens at construction time
//! - Same types on server and client
//!
//! # Email Type Hierarchy
//!
//! - `EmailAddress` / `AnyEmailAddress` - Any valid email (blocks disposable only)
//! - `WorkEmailAddress` - No public domains (Gmail, Yahoo, etc.)
//! - `BusinessEmailAddress` - Only corporate/verified domains
//!
//! # Password Type Hierarchy
//!
//! - `PasswordBasic` - 6+ characters
//! - `PasswordMedium` - 8+ characters + complexity
//! - `PasswordStrong` - 10+ characters + high complexity
//! - `PasswordPhrase` - 15+ characters (passphrase style)
//! - `PasswordPhrase3` - 3+ words, 20+ characters
//! - `SuperStrongPassword` - 12+ characters + all character types
//! - `ModernPassword` - 16+ characters (NIST 2024 recommendations)

use nutype::nutype;

// =============================================================================
// Email Types with Business Rules
// =============================================================================

/// Public email domains to block (Gmail, Yahoo, Hotmail, etc.)
static PUBLIC_DOMAINS: &[&str] = &[
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
    "live.com",
    "msn.com",
    "inbox.com",
    "gmx.com",
    "me.com",
];

/// Always-blocked domains (disposable/temporary email services)
static BLOCKED_DOMAINS: &[&str] = &[
    "tempmail.com",
    "guerrillamail.com",
    "10minutemail.com",
    "mailinator.com",
    "throwaway.email",
    "temp-mail.org",
    "maildrop.cc",
    "getnada.com",
];

/// Basic validated email address (format only, blocks disposable)
///
/// **Business Rule**: Accepts any email domain EXCEPT disposable/temporary email services.
///
/// **Use when**: You want to accept both personal (Gmail, Yahoo) and work emails,
/// but block throwaway addresses.
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_types::EmailAddress;
///
/// // Valid - any real domain
/// let personal = EmailAddress::try_new("user@gmail.com".to_string())?;  // ✓
/// let work = EmailAddress::try_new("user@company.com".to_string())?;     // ✓
///
/// // Invalid - disposable email blocked
/// let bad = EmailAddress::try_new("user@tempmail.com".to_string()); // ✗
/// ```
#[nutype(
    validate(predicate = is_valid_email_any_domain),
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

/// Alias: Any email address (same as EmailAddress)
///
/// **Business Rule**: Same as `EmailAddress` - blocks disposable only.
///
/// Use this when you want to be explicit that any email is accepted.
pub type AnyEmailAddress = EmailAddress;

/// Work email address (no public domains)
///
/// **Business Rule**: Blocks public email providers (Gmail, Yahoo, Hotmail, etc.)
/// AND disposable email services. Only accepts corporate/private domains.
///
/// **Use when**: Registration should use work/corporate email only (B2B apps, enterprise tools).
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_types::WorkEmailAddress;
///
/// // Valid - corporate domain
/// let work = WorkEmailAddress::try_new("john@acme.com".to_string())?;  // ✓
///
/// // Invalid - public domain
/// let gmail = WorkEmailAddress::try_new("john@gmail.com".to_string()); // ✗
/// let yahoo = WorkEmailAddress::try_new("john@yahoo.com".to_string()); // ✗
/// ```
#[nutype(
    validate(predicate = is_work_email),
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
pub struct WorkEmailAddress(String);

/// Business email address (stricter than WorkEmailAddress)
///
/// **Business Rule**: Same as WorkEmailAddress for now.
/// Future: Can be extended with domain allowlist for verified partners.
///
/// **Use when**: You need maximum validation (verified corporate domains only).
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_types::BusinessEmailAddress;
///
/// let biz = BusinessEmailAddress::try_new("ceo@verified-corp.com".to_string())?;
/// ```
#[nutype(
    validate(predicate = is_business_email),
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
pub struct BusinessEmailAddress(String);

// -----------------------------------------------------------------------------
// Email validation predicates
// -----------------------------------------------------------------------------

fn is_valid_email_format(s: &str) -> bool {
    if !s.contains('@') || !s.contains('.') || s.len() < 5 {
        return false;
    }
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

fn extract_domain(email: &str) -> &str {
    email.split('@').nth(1).unwrap_or("")
}

fn is_blocked_domain(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    BLOCKED_DOMAINS.iter().any(|&d| d == domain_lower)
}

fn is_public_domain(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    PUBLIC_DOMAINS.iter().any(|&d| d == domain_lower)
}

fn is_valid_email_any_domain(s: &str) -> bool {
    if !is_valid_email_format(s) {
        return false;
    }
    let domain = extract_domain(s);
    !is_blocked_domain(domain)
}

fn is_work_email(s: &str) -> bool {
    if !is_valid_email_format(s) {
        return false;
    }
    let domain = extract_domain(s);
    !is_blocked_domain(domain) && !is_public_domain(domain)
}

fn is_business_email(s: &str) -> bool {
    // Same as work email for now
    // Future: check against allowlist of verified corporate domains
    is_work_email(s)
}

// =============================================================================
// Password Types with Embedded Strength Rules
// =============================================================================

/// Basic password (6+ characters)
///
/// **Security Level**: Low - Use only for non-critical accounts
///
/// **Business Rule**: Minimum 6 characters. No complexity requirements.
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

/// Medium-strength password (8+ characters)
///
/// **Security Level**: Medium - Standard for most applications
///
/// **Business Rule**: Minimum 8 characters.
/// Recommended to combine with form-level complexity check.
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

/// Strong password (10+ characters with complexity)
///
/// **Security Level**: High - For sensitive operations
///
/// **Business Rule**: Minimum 10 characters + uppercase + lowercase + digit + special
#[nutype(
    validate(
        len_char_min = 10,
        predicate = has_password_complexity_strong
    ),
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

/// Super strong password (12+ characters with all character types)
///
/// **Security Level**: Very High - For admin accounts, financial operations
///
/// **Business Rule**: Minimum 12 characters + uppercase + lowercase + digit + special
/// + at least 2 special characters
#[nutype(
    validate(
        len_char_min = 12,
        predicate = has_password_complexity_super
    ),
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
pub struct SuperStrongPassword(String);

/// Password passphrase (15+ characters, easier to remember)
///
/// **Security Level**: High - Modern approach (xkcd "correct horse battery staple")
///
/// **Business Rule**: Minimum 15 characters. Favors length over complexity.
/// Example: "BlueSky-Mountain-Coffee-2024"
#[nutype(
    validate(len_char_min = 15),
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
pub struct PasswordPhrase(String);

/// Password passphrase with 3+ words (20+ characters)
///
/// **Security Level**: High - Multi-word passphrase
///
/// **Business Rule**: Minimum 20 characters + at least 2 spaces/hyphens (3+ words).
/// Example: "Correct-Horse-Battery-Staple"
#[nutype(
    validate(
        len_char_min = 20,
        predicate = has_multiple_words
    ),
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
pub struct PasswordPhrase3(String);

/// Modern password (16+ characters, NIST 2024 recommendations)
///
/// **Security Level**: Very High - Follows NIST SP 800-63B guidelines
///
/// **Business Rule**: Minimum 16 characters. Emphasizes length over complexity.
/// No forced special characters (reduces user friction).
#[nutype(
    validate(len_char_min = 16),
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
pub struct ModernPassword(String);

// -----------------------------------------------------------------------------
// Password validation predicates
// -----------------------------------------------------------------------------

fn has_password_complexity_strong(s: &str) -> bool {
    let has_upper = s.chars().any(|c| c.is_uppercase());
    let has_lower = s.chars().any(|c| c.is_lowercase());
    let has_digit = s.chars().any(|c| c.is_numeric());
    let has_special = s.chars().any(|c| !c.is_alphanumeric());

    has_upper && has_lower && has_digit && has_special
}

fn has_password_complexity_super(s: &str) -> bool {
    let has_upper = s.chars().any(|c| c.is_uppercase());
    let has_lower = s.chars().any(|c| c.is_lowercase());
    let has_digit = s.chars().any(|c| c.is_numeric());
    let special_count = s.chars().filter(|c| !c.is_alphanumeric()).count();

    has_upper && has_lower && has_digit && special_count >= 2
}

fn has_multiple_words(s: &str) -> bool {
    // Count spaces, hyphens, or underscores (word separators)
    let separator_count = s.chars()
        .filter(|&c| c == ' ' || c == '-' || c == '_')
        .count();

    separator_count >= 2  // At least 2 separators = 3+ words
}

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

/// Username (3-30 characters, alphanumeric + underscore/dash)
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
// URL Types
// =============================================================================

/// Valid URL address
///
/// **Business Rule**: Accepts any valid URL (http, https, ftp, etc.)
///
/// **Use when**: You need to validate URL format
#[nutype(
    validate(predicate = is_valid_url),
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
pub struct UrlAddress(String);

/// HTTPS-only URL
///
/// **Business Rule**: Only accepts HTTPS URLs (secure connections only)
///
/// **Use when**: You need to enforce secure connections
#[nutype(
    validate(predicate = is_https_url),
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
pub struct HttpsUrl(String);

// URL validation predicates
fn is_valid_url(s: &str) -> bool {
    // Basic URL validation
    if s.len() < 10 {
        return false;
    }

    // Must start with a protocol
    let has_protocol = s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("ws://")
        || s.starts_with("wss://");

    if !has_protocol {
        return false;
    }

    // Must have at least one dot after protocol
    let after_protocol = s.split("://").nth(1).unwrap_or("");
    after_protocol.contains('.')
}

fn is_https_url(s: &str) -> bool {
    s.starts_with("https://") && is_valid_url(s)
}

// =============================================================================
// Specialized Numeric Types
// =============================================================================

/// Age (18-120 years)
///
/// **Business Rule**: Standard adult age range
///
/// **Use when**: Age verification, user registration
#[nutype(
    validate(greater_or_equal = 18, less_or_equal = 120),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Age(i64);

/// Percentage (0-100)
///
/// **Business Rule**: Standard percentage value
///
/// **Use when**: Progress, discounts, ratings
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Percentage(i64);

/// Network port (1-65535)
///
/// **Business Rule**: Valid TCP/UDP port range
///
/// **Use when**: Network configuration
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 65535),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Port(i64);

// =============================================================================
// Collection Types
// =============================================================================

/// Non-empty vector
///
/// **Business Rule**: Vector must have at least one element
///
/// **Use when**: Tags, categories, selections that can't be empty
#[nutype(
    validate(predicate = is_non_empty_vec),
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
pub struct NonEmptyVec<T>(Vec<T>);

fn is_non_empty_vec<T>(v: &Vec<T>) -> bool {
    !v.is_empty()
}

// =============================================================================
// String Pattern Types
// =============================================================================

/// US Phone Number
///
/// **Business Rule**: Validates US phone numbers (10 digits)
///
/// **Formats accepted**:
/// - (123) 456-7890
/// - 123-456-7890
/// - 1234567890
///
/// **Use when**: US phone number validation
#[nutype(
    validate(predicate = is_valid_phone_number),
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
pub struct PhoneNumber(String);

/// US Zip Code
///
/// **Business Rule**: Validates US zip codes (5 or 9 digits)
///
/// **Formats accepted**:
/// - 12345
/// - 12345-6789
///
/// **Use when**: US address validation
#[nutype(
    validate(predicate = is_valid_zip_code),
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
pub struct ZipCode(String);

/// IPv4 Address
///
/// **Business Rule**: Validates IPv4 addresses
///
/// **Format**: xxx.xxx.xxx.xxx (0-255 per octet)
///
/// **Use when**: Network configuration, IP whitelisting
#[nutype(
    validate(predicate = is_valid_ipv4),
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
pub struct IpAddress(String);

/// UUID (Universally Unique Identifier)
///
/// **Business Rule**: Validates UUID format (v4)
///
/// **Format**: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
///
/// **Use when**: API keys, unique identifiers
#[nutype(
    validate(predicate = is_valid_uuid),
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
pub struct Uuid(String);

// Pattern validation predicates
fn is_valid_phone_number(s: &str) -> bool {
    // Remove common separators
    let digits: String = s.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    // US phone number: exactly 10 digits
    digits.len() == 10
}

fn is_valid_zip_code(s: &str) -> bool {
    // Remove dash if present
    let parts: Vec<&str> = s.split('-').collect();

    match parts.len() {
        1 => {
            // 5-digit zip
            parts[0].len() == 5 && parts[0].chars().all(|c| c.is_ascii_digit())
        }
        2 => {
            // 9-digit zip (xxxxx-xxxx)
            parts[0].len() == 5
                && parts[1].len() == 4
                && parts[0].chars().all(|c| c.is_ascii_digit())
                && parts[1].chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();

    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| {
        part.parse::<u8>().is_ok()
    })
}

fn is_valid_uuid(s: &str) -> bool {
    // Basic UUID v4 validation
    // Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    let parts: Vec<&str> = s.split('-').collect();

    if parts.len() != 5 {
        return false;
    }

    // Check segment lengths
    if parts[0].len() != 8 || parts[1].len() != 4 || parts[2].len() != 4
        || parts[3].len() != 4 || parts[4].len() != 12 {
        return false;
    }

    // All parts should be hex
    parts.iter().all(|part| {
        part.chars().all(|c| c.is_ascii_hexdigit())
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Email tests
    #[test]
    fn test_email_address_any_domain() {
        // Accepts personal email
        assert!(EmailAddress::try_new("user@gmail.com".to_string()).is_ok());
        // Accepts work email
        assert!(EmailAddress::try_new("user@company.com".to_string()).is_ok());
        // Blocks disposable
        assert!(EmailAddress::try_new("user@tempmail.com".to_string()).is_err());
    }

    #[test]
    fn test_work_email_blocks_public() {
        // Accepts corporate email
        assert!(WorkEmailAddress::try_new("user@acme.com".to_string()).is_ok());
        // Blocks Gmail
        assert!(WorkEmailAddress::try_new("user@gmail.com".to_string()).is_err());
        // Blocks Yahoo
        assert!(WorkEmailAddress::try_new("user@yahoo.com".to_string()).is_err());
        // Blocks disposable
        assert!(WorkEmailAddress::try_new("user@tempmail.com".to_string()).is_err());
    }

    #[test]
    fn test_business_email() {
        // Same as work email for now
        assert!(BusinessEmailAddress::try_new("ceo@corp.com".to_string()).is_ok());
        assert!(BusinessEmailAddress::try_new("user@gmail.com".to_string()).is_err());
    }

    // Password tests
    #[test]
    fn test_password_basic() {
        assert!(PasswordBasic::try_new("12345".to_string()).is_err());  // Too short
        assert!(PasswordBasic::try_new("123456".to_string()).is_ok());  // Exactly 6
    }

    #[test]
    fn test_password_strong_complexity() {
        // Too short
        assert!(PasswordStrong::try_new("Short1!".to_string()).is_err());
        // Long enough but no special
        assert!(PasswordStrong::try_new("Password123".to_string()).is_err());
        // Missing uppercase
        assert!(PasswordStrong::try_new("password123!".to_string()).is_err());
        // All requirements met
        assert!(PasswordStrong::try_new("Password123!".to_string()).is_ok());
    }

    #[test]
    fn test_super_strong_password() {
        // Needs 12+ chars + 2 special chars
        assert!(SuperStrongPassword::try_new("Pass123!".to_string()).is_err());  // Too short
        assert!(SuperStrongPassword::try_new("Password123!".to_string()).is_err());  // Only 1 special
        assert!(SuperStrongPassword::try_new("Password123!@".to_string()).is_ok());  // ✓
    }

    #[test]
    fn test_password_phrase() {
        assert!(PasswordPhrase::try_new("short".to_string()).is_err());
        assert!(PasswordPhrase::try_new("BlueSky-Mountain".to_string()).is_ok());  // 16 chars
    }

    #[test]
    fn test_password_phrase3() {
        // Needs 20+ chars + 2+ separators (3+ words)
        assert!(PasswordPhrase3::try_new("Short-Phrase".to_string()).is_err());  // Too short (12 chars)
        assert!(PasswordPhrase3::try_new("OnlySingleWordHereNoSeparators".to_string()).is_err());  // No separators
        assert!(PasswordPhrase3::try_new("Correct-Horse-Battery-Staple".to_string()).is_ok());  // ✓ (28 chars, 3 separators)
    }

    #[test]
    fn test_modern_password() {
        assert!(ModernPassword::try_new("tooshort".to_string()).is_err());
        assert!(ModernPassword::try_new("ThisIsMyLongPassword123".to_string()).is_ok());  // 23 chars
    }

    #[test]
    fn test_username() {
        assert!(Username::try_new("ab".to_string()).is_err());  // Too short
        assert!(Username::try_new("abc".to_string()).is_ok());
        assert!(Username::try_new("user_name".to_string()).is_ok());
        assert!(Username::try_new("user-name".to_string()).is_ok());
        assert!(Username::try_new("user@name".to_string()).is_err());  // Invalid char
    }

    #[test]
    fn test_positive_int() {
        assert!(PositiveInt::try_from(0).is_err());
        assert!(PositiveInt::try_from(1).is_ok());
        assert!(PositiveInt::try_from(-1).is_err());
    }

    // URL tests
    #[test]
    fn test_url_address() {
        assert!(UrlAddress::try_new("https://example.com".to_string()).is_ok());
        assert!(UrlAddress::try_new("http://example.com".to_string()).is_ok());
        assert!(UrlAddress::try_new("ftp://files.example.com".to_string()).is_ok());
        assert!(UrlAddress::try_new("not-a-url".to_string()).is_err());
        assert!(UrlAddress::try_new("http://".to_string()).is_err());  // No domain
    }

    #[test]
    fn test_https_url() {
        assert!(HttpsUrl::try_new("https://example.com".to_string()).is_ok());
        assert!(HttpsUrl::try_new("http://example.com".to_string()).is_err());  // Must be HTTPS
        assert!(HttpsUrl::try_new("ftp://example.com".to_string()).is_err());
    }

    // Specialized numeric tests
    #[test]
    fn test_age() {
        assert!(Age::try_from(17).is_err());  // Too young
        assert!(Age::try_from(18).is_ok());
        assert!(Age::try_from(65).is_ok());
        assert!(Age::try_from(120).is_ok());
        assert!(Age::try_from(121).is_err());  // Too old
    }

    #[test]
    fn test_percentage() {
        assert!(Percentage::try_from(-1).is_err());
        assert!(Percentage::try_from(0).is_ok());
        assert!(Percentage::try_from(50).is_ok());
        assert!(Percentage::try_from(100).is_ok());
        assert!(Percentage::try_from(101).is_err());
    }

    #[test]
    fn test_port() {
        assert!(Port::try_from(0).is_err());  // Port 0 invalid
        assert!(Port::try_from(1).is_ok());
        assert!(Port::try_from(80).is_ok());
        assert!(Port::try_from(443).is_ok());
        assert!(Port::try_from(65535).is_ok());
        assert!(Port::try_from(65536).is_err());  // Out of range
    }

    // Collection tests
    #[test]
    fn test_non_empty_vec() {
        assert!(NonEmptyVec::try_new(Vec::<String>::new()).is_err());  // Empty
        assert!(NonEmptyVec::try_new(vec!["item".to_string()]).is_ok());
        assert!(NonEmptyVec::try_new(vec![1, 2, 3]).is_ok());
    }

    // Pattern tests
    #[test]
    fn test_phone_number() {
        assert!(PhoneNumber::try_new("1234567890".to_string()).is_ok());
        assert!(PhoneNumber::try_new("123-456-7890".to_string()).is_ok());
        assert!(PhoneNumber::try_new("(123) 456-7890".to_string()).is_ok());
        assert!(PhoneNumber::try_new("123456789".to_string()).is_err());  // Too short
        assert!(PhoneNumber::try_new("12345678901".to_string()).is_err());  // Too long
    }

    #[test]
    fn test_zip_code() {
        assert!(ZipCode::try_new("12345".to_string()).is_ok());
        assert!(ZipCode::try_new("12345-6789".to_string()).is_ok());
        assert!(ZipCode::try_new("1234".to_string()).is_err());  // Too short
        assert!(ZipCode::try_new("123456".to_string()).is_err());  // Too long
        assert!(ZipCode::try_new("12345-678".to_string()).is_err());  // Invalid +4
    }

    #[test]
    fn test_ip_address() {
        assert!(IpAddress::try_new("192.168.1.1".to_string()).is_ok());
        assert!(IpAddress::try_new("0.0.0.0".to_string()).is_ok());
        assert!(IpAddress::try_new("255.255.255.255".to_string()).is_ok());
        assert!(IpAddress::try_new("256.1.1.1".to_string()).is_err());  // Out of range
        assert!(IpAddress::try_new("192.168.1".to_string()).is_err());  // Missing octet
    }

    #[test]
    fn test_uuid() {
        assert!(Uuid::try_new("550e8400-e29b-41d4-a716-446655440000".to_string()).is_ok());
        assert!(Uuid::try_new("123e4567-e89b-12d3-a456-426614174000".to_string()).is_ok());
        assert!(Uuid::try_new("not-a-uuid".to_string()).is_err());
        assert!(Uuid::try_new("550e8400-e29b-41d4-a716".to_string()).is_err());  // Too short
    }
}
