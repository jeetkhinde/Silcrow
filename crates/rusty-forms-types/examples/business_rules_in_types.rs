//! Example: Business Rules Embedded in Types
//!
//! This example demonstrates the power of embedding business rules directly
//! in types using nutype. No need for `#[no_public_domains]` or `#[password(...)]`
//! attributes - the type IS the business rule!
//!
//! Run: cargo run --example business_rules_in_types

use rusty_forms_types::*;
use serde::{Serialize, Deserialize};

// =============================================================================
// Example 1: Consumer App (accepts any email)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct ConsumerSignupForm {
    // Accepts Gmail, Yahoo, corporate emails - anything except disposable
    email: EmailAddress,  // or: AnyEmailAddress
    password: PasswordMedium,  // 8+ chars minimum
}

// =============================================================================
// Example 2: B2B SaaS (work emails only)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct B2BSignupForm {
    // NO public domains (Gmail, Yahoo, etc.)
    // Type enforces the business rule!
    work_email: WorkEmailAddress,

    // Strong password for enterprise security
    password: PasswordStrong,  // 10+ chars + complexity
}

// =============================================================================
// Example 3: Enterprise/Financial (maximum security)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct EnterpriseAdminForm {
    // Verified business domains only
    business_email: BusinessEmailAddress,

    // Super strong password required
    password: SuperStrongPassword,  // 12+ chars + 2 special chars
}

// =============================================================================
// Example 4: Modern Auth (passphrase approach)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct ModernAuthForm {
    email: EmailAddress,

    // Longer but easier to remember: "Correct-Horse-Battery-Staple"
    passphrase: PasswordPhrase3,  // 3+ words, 20+ chars
}

// =============================================================================
// Example 5: Different Security Levels (route-specific)
// =============================================================================

mod routes {
    use super::*;

    pub mod consumer {
        use super::*;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginForm {
            email: EmailAddress,
            password: PasswordBasic,  // Low security: 6+ chars
        }
    }

    pub mod business {
        use super::*;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginForm {
            email: WorkEmailAddress,  // No Gmail/Yahoo
            password: PasswordMedium,  // Medium security: 8+ chars
        }
    }

    pub mod admin {
        use super::*;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginForm {
            email: BusinessEmailAddress,  // Verified domains only
            password: SuperStrongPassword,  // High security: 12+ chars
        }
    }
}

// =============================================================================
// Demonstration
// =============================================================================

fn main() {
    println!("=== Business Rules Embedded in Types ===\n");

    // -------------------------------------------------------------------------
    // Example 1: Consumer App
    // -------------------------------------------------------------------------
    println!("1. Consumer App (Any Email):");

    match EmailAddress::try_new("user@gmail.com".to_string()) {
        Ok(email) => println!("   ✅ Gmail accepted: {}", email),
        Err(_) => println!("   ❌ Gmail rejected"),
    }

    match EmailAddress::try_new("user@tempmail.com".to_string()) {
        Ok(_) => println!("   ❌ Disposable accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Disposable email rejected"),
    }

    println!();

    // -------------------------------------------------------------------------
    // Example 2: B2B SaaS (Work Emails Only)
    // -------------------------------------------------------------------------
    println!("2. B2B SaaS (Work Emails Only):");

    match WorkEmailAddress::try_new("employee@acme.com".to_string()) {
        Ok(email) => println!("   ✅ Corporate email accepted: {}", email),
        Err(_) => println!("   ❌ Corporate email rejected"),
    }

    match WorkEmailAddress::try_new("user@gmail.com".to_string()) {
        Ok(_) => println!("   ❌ Gmail accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Gmail rejected (no public domains)"),
    }

    println!();

    // -------------------------------------------------------------------------
    // Example 3: Password Strength Levels
    // -------------------------------------------------------------------------
    println!("3. Password Strength Levels:");

    // Basic: 6+ chars
    match PasswordBasic::try_new("pass12".to_string()) {
        Ok(_) => println!("   ✅ Basic password (6 chars) accepted"),
        Err(_) => println!("   ❌ Basic password rejected"),
    }

    // Strong: 10+ chars + complexity
    match PasswordStrong::try_new("Pass123!ab".to_string()) {
        Ok(_) => println!("   ✅ Strong password accepted"),
        Err(_) => println!("   ❌ Strong password rejected"),
    }

    match PasswordStrong::try_new("password123".to_string()) {
        Ok(_) => println!("   ❌ Weak password accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Weak password rejected (no special char)"),
    }

    // Super Strong: 12+ chars + 2 special
    match SuperStrongPassword::try_new("MyPass123!@#".to_string()) {
        Ok(_) => println!("   ✅ Super strong password accepted"),
        Err(_) => println!("   ❌ Super strong password rejected"),
    }

    println!();

    // -------------------------------------------------------------------------
    // Example 4: Passphrase (Modern Approach)
    // -------------------------------------------------------------------------
    println!("4. Password Passphrase (Modern, User-Friendly):");

    match PasswordPhrase3::try_new("Correct-Horse-Battery-Staple".to_string()) {
        Ok(_) => println!("   ✅ 3-word passphrase accepted"),
        Err(_) => println!("   ❌ 3-word passphrase rejected"),
    }

    match PasswordPhrase3::try_new("Short".to_string()) {
        Ok(_) => println!("   ❌ Short passphrase accepted (shouldn't happen)"),
        Err(_) => println!("   ✅ Short passphrase rejected"),
    }

    println!();

    // -------------------------------------------------------------------------
    // Example 5: Type Safety Prevents Mistakes
    // -------------------------------------------------------------------------
    println!("5. Type Safety (Compile-Time Enforcement):");

    fn send_consumer_welcome(email: EmailAddress) {
        println!("   → Sending welcome email to: {}", email);
    }

    fn send_business_welcome(email: WorkEmailAddress) {
        println!("   → Sending business welcome to: {}", email);
    }

    let personal_email = EmailAddress::try_new("user@gmail.com".to_string()).unwrap();
    let work_email = WorkEmailAddress::try_new("user@acme.com".to_string()).unwrap();

    // This works
    send_consumer_welcome(personal_email.clone());

    // This would NOT compile (type safety!)
    // send_business_welcome(personal_email);  // ❌ Compile error!

    // This works
    send_business_welcome(work_email);

    println!();

    // -------------------------------------------------------------------------
    // Example 6: Self-Documenting APIs
    // -------------------------------------------------------------------------
    println!("6. Self-Documenting Code:");
    println!("   Before: fn register(email: String, password: String)");
    println!("   After:  fn register(email: WorkEmailAddress, password: PasswordStrong)");
    println!();
    println!("   ✨ Function signature tells you EXACTLY what's required!");
    println!("   ✨ No need to read documentation!");
    println!("   ✨ Compiler prevents mistakes!");

    println!("\n=== Benefits ===");
    println!("✅ Business rules embedded in types");
    println!("✅ No form-level validators needed for type rules");
    println!("✅ Compile-time type safety");
    println!("✅ Self-documenting code");
    println!("✅ Same types work in WASM (client-side)");
    println!("✅ Can't accidentally mix different security levels");
}
