//! Example: Route-Specific Type Overrides
//!
//! This example shows how to:
//! 1. Use common types from rhtmx-form-types
//! 2. Create route-specific types with stricter validation
//! 3. Organize types by domain/route
//!
//! Run: cargo run --example route_specific_types

use rusty_forms_types::*;
use nutype::nutype;
use serde::{Serialize, Deserialize};

// =============================================================================
// Application Types Module
// =============================================================================

pub mod types {
    use super::*;

    // Re-export common types
    pub use rusty_forms_types::{
        EmailAddress,
        PasswordBasic,
        PasswordMedium,
        PasswordStrong,
        Username,
        NonEmptyString,
        PositiveInt,
        NonNegativeInt,
    };

    // -----------------------------------------------------------------------------
    // Admin-specific types (stricter validation)
    // -----------------------------------------------------------------------------
    pub mod admin {
        use super::*;

        /// Admin password requires 12+ characters (stricter than regular users)
        #[nutype(
            validate(len_char_min = 12),
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
        pub struct AdminPassword(String);

        /// Admin username must start with "admin_"
        #[nutype(
            validate(
                len_char_min = 9,  // "admin_" + at least 3 chars
                predicate = starts_with_admin_prefix
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
                Display,
                Serialize,
                Deserialize,
            )
        )]
        pub struct AdminUsername(String);

        fn starts_with_admin_prefix(s: &str) -> bool {
            s.starts_with("admin_")
        }
    }

    // -----------------------------------------------------------------------------
    // User-specific types (standard validation)
    // -----------------------------------------------------------------------------
    pub mod users {
        use super::*;

        /// Regular user password (8+ chars minimum)
        /// Use PasswordMedium from common types
        pub type UserPassword = PasswordMedium;

        /// Regular username (3-30 chars)
        /// Use Username from common types
        pub type UserUsername = Username;

        /// User display name (1-50 characters)
        #[nutype(
            validate(
                len_char_min = 1,
                len_char_max = 50
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
                Display,
                Serialize,
                Deserialize,
            )
        )]
        pub struct DisplayName(String);
    }

    // -----------------------------------------------------------------------------
    // API-specific types (tokens, keys, etc.)
    // -----------------------------------------------------------------------------
    pub mod api {
        use super::*;

        /// API key (32-64 hexadecimal characters)
        #[nutype(
            validate(
                len_char_min = 32,
                len_char_max = 64,
                predicate = is_hex_string
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
                Serialize,
                Deserialize,
            )
        )]
        pub struct ApiKey(String);

        fn is_hex_string(s: &str) -> bool {
            s.chars().all(|c| c.is_ascii_hexdigit())
        }

        /// Rate limit (1-10000 requests per hour)
        #[nutype(
            validate(
                greater_or_equal = 1,
                less_or_equal = 10000
            ),
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
        pub struct RateLimit(i64);
    }
}

// =============================================================================
// Example Forms
// =============================================================================

// Note: These would normally use #[derive(Validate, FormField)]
// but we're keeping them simple for this example

/// Regular user registration form
#[derive(Debug, Serialize, Deserialize)]
struct UserRegistrationForm {
    email: types::EmailAddress,
    password: types::users::UserPassword,
    username: types::users::UserUsername,
    display_name: types::users::DisplayName,
}

/// Admin registration form (stricter requirements)
#[derive(Debug, Serialize, Deserialize)]
struct AdminRegistrationForm {
    email: types::EmailAddress,
    password: types::admin::AdminPassword,  // Requires 12+ chars!
    username: types::admin::AdminUsername,  // Must start with "admin_"!
}

/// API key creation form
#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyCreateForm {
    key: types::api::ApiKey,
    rate_limit: types::api::RateLimit,
}

// =============================================================================
// Example Usage
// =============================================================================

fn main() {
    println!("=== Route-Specific Type Overrides Example ===\n");

    // -----------------------------------------------------------------------------
    // 1. Regular user registration (standard validation)
    // -----------------------------------------------------------------------------
    println!("1. Regular User Registration:");

    let user_email = types::EmailAddress::try_new("user@example.com".to_string())
        .expect("Valid email");
    println!("   Email: {}", user_email);

    let user_password = types::users::UserPassword::try_new("MyPass123".to_string())
        .expect("8+ chars required");
    println!("   Password: {} chars", user_password.len());

    let username = types::users::UserUsername::try_new("john_doe".to_string())
        .expect("Valid username");
    println!("   Username: {}", username);

    let display_name = types::users::DisplayName::try_new("John Doe".to_string())
        .expect("Valid display name");
    println!("   Display Name: {}\n", display_name);

    // -----------------------------------------------------------------------------
    // 2. Admin registration (stricter validation)
    // -----------------------------------------------------------------------------
    println!("2. Admin Registration (Stricter Requirements):");

    // Admin password must be 12+ chars
    match types::admin::AdminPassword::try_new("short".to_string()) {
        Ok(_) => println!("   ❌ Short password accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Short password rejected (< 12 chars)"),
    }

    let admin_password = types::admin::AdminPassword::try_new("SuperSecureAdmin123!".to_string())
        .expect("12+ chars required");
    println!("   Admin Password: {} chars", admin_password.len());

    // Admin username must start with "admin_"
    match types::admin::AdminUsername::try_new("johndoe".to_string()) {
        Ok(_) => println!("   ❌ Username without prefix accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Username without 'admin_' prefix rejected"),
    }

    let admin_username = types::admin::AdminUsername::try_new("admin_superuser".to_string())
        .expect("Must start with admin_");
    println!("   Admin Username: {}\n", admin_username);

    // -----------------------------------------------------------------------------
    // 3. API key validation
    // -----------------------------------------------------------------------------
    println!("3. API Key Validation:");

    // API key must be 32-64 hex characters
    match types::api::ApiKey::try_new("not-hex!".to_string()) {
        Ok(_) => println!("   ❌ Non-hex key accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Non-hex characters rejected"),
    }

    match types::api::ApiKey::try_new("abc123".to_string()) {
        Ok(_) => println!("   ❌ Short key accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Too short key rejected (< 32 chars)"),
    }

    let api_key = types::api::ApiKey::try_new(
        "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2".to_string()
    ).expect("Valid 64-char hex key");
    println!("   API Key: {} chars\n", api_key.len());

    // Rate limit: 1-10000 requests per hour
    match types::api::RateLimit::try_from(0) {
        Ok(_) => println!("   ❌ Zero rate limit accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Zero rate limit rejected"),
    }

    match types::api::RateLimit::try_from(20000) {
        Ok(_) => println!("   ❌ Excessive rate limit accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Excessive rate limit rejected (> 10000)"),
    }

    let rate_limit = types::api::RateLimit::try_from(1000)
        .expect("Valid rate limit");
    println!("   Rate Limit: {} req/hour\n", rate_limit);

    // -----------------------------------------------------------------------------
    // 4. Type safety demonstration
    // -----------------------------------------------------------------------------
    println!("4. Type Safety:");

    fn process_user_password(pwd: types::users::UserPassword) {
        println!("   Processing user password: {} chars", pwd.len());
    }

    fn process_admin_password(pwd: types::admin::AdminPassword) {
        println!("   Processing admin password: {} chars", pwd.len());
    }

    // This works - same type
    process_user_password(user_password);

    // This would NOT compile - different types!
    // process_user_password(admin_password);  // Compile error!
    // process_admin_password(user_password);  // Compile error!

    // Types are distinct at compile time
    process_admin_password(admin_password);

    println!("\n=== Type Safety Enforced at Compile Time! ===");
    println!("✅ Admin passwords cannot be used where user passwords are expected");
    println!("✅ Prevents mixing up validation requirements");
    println!("✅ Self-documenting code: function signature tells you the requirements");
}
