// Advanced Validation Demo - Showcasing All New Validators
// This file demonstrates the complete RHTMX validation system including all new features

use rhtmx::{Validate, ValidateTrait};
use serde::Deserialize;
use std::collections::HashMap;

// ===== String Matching Validators =====

#[derive(Validate, Deserialize, Debug)]
struct UsernameForm {
    #[starts_with("user_")]
    #[not_contains("admin")]
    #[ends_with("_dev")]
    username: String,

    #[contains("@github.com")]
    github_handle: String,
}

// ===== Equality Validators =====

#[derive(Validate, Deserialize, Debug)]
struct PasswordChangeForm {
    #[password("strong")]
    new_password: String,

    #[equals_field("new_password")]
    #[label("Password Confirmation")]
    confirm_password: String,

    #[not_equals("password123")]
    #[message = "Please don't use a common password"]
    must_not_be_common: String,
}

// ===== Conditional Validators (depends_on) =====

#[derive(Validate, Deserialize, Debug)]
struct ShippingForm {
    #[enum_variant("pickup", "delivery")]
    shipping_method: String,

    // Only required when shipping_method is "delivery"
    #[depends_on("shipping_method", "delivery")]
    delivery_address: Option<String>,

    #[depends_on("shipping_method", "delivery")]
    postal_code: Option<String>,
}

// ===== Collection Validators =====

#[derive(Validate, Deserialize, Debug)]
struct TeamForm {
    #[min_length(3)]
    team_name: String,

    #[min_items(2)]
    #[max_items(10)]
    #[unique]
    team_members: Vec<String>,

    #[min_items(1)]
    #[max_items(5)]
    tags: Vec<String>,
}

// ===== Enum Variant Validator =====

#[derive(Validate, Deserialize, Debug)]
struct ConfigForm {
    #[enum_variant("production", "staging", "development")]
    environment: String,

    #[enum_variant("debug", "info", "warn", "error")]
    log_level: String,

    #[enum_variant("us-east-1", "us-west-2", "eu-west-1")]
    region: String,
}

// ===== Custom Message & Label =====

#[derive(Validate, Deserialize, Debug)]
struct FriendlyForm {
    #[min_length(3)]
    #[label("Full Name")]
    #[message = "Please enter your complete name (at least 3 characters)"]
    name: String,

    #[email]
    #[label("Email Address")]
    #[message = "We need a valid email to contact you"]
    email: String,

    #[min(18)]
    #[label("Age")]
    #[message = "You must be at least 18 years old to register"]
    age: i32,
}

// ===== Custom Validator Function =====

// Custom validation function must return Result<(), String>
fn validate_even_number(value: &i32) -> Result<(), String> {
    if value % 2 == 0 {
        Ok(())
    } else {
        Err("Number must be even".to_string())
    }
}

fn validate_no_profanity(value: &String) -> Result<(), String> {
    let profanity = ["badword1", "badword2"];
    let lower = value.to_lowercase();

    for word in &profanity {
        if lower.contains(word) {
            return Err("Content contains inappropriate language".to_string());
        }
    }
    Ok(())
}

#[derive(Validate, Deserialize, Debug)]
struct CustomValidationForm {
    #[custom("validate_even_number")]
    lucky_number: i32,

    #[custom("validate_no_profanity")]
    #[min_length(10)]
    bio: String,
}

// ===== Complex Real-World Example =====

#[derive(Validate, Deserialize, Debug)]
struct CompleteRegistrationForm {
    // Basic info with custom labels
    #[min_length(3)]
    #[max_length(50)]
    #[starts_with("user_")]
    #[label("Username")]
    username: String,

    #[email]
    #[no_public_domains]
    #[label("Work Email")]
    email: String,

    // Password with confirmation
    #[password("strong")]
    password: String,

    #[equals_field("password")]
    #[message = "Passwords do not match"]
    password_confirmation: String,

    // Optional fields with conditional requirements
    #[enum_variant("individual", "business")]
    account_type: String,

    #[depends_on("account_type", "business")]
    #[label("Company Name")]
    company_name: Option<String>,

    #[depends_on("account_type", "business")]
    #[label("Tax ID")]
    tax_id: Option<String>,

    // Collection validations
    #[min_items(1)]
    #[max_items(5)]
    #[unique]
    #[label("Areas of Interest")]
    interests: Vec<String>,

    // Custom validation
    #[custom("validate_no_profanity")]
    #[max_length(500)]
    #[label("About Me")]
    about: String,

    // Required option with custom message
    #[required]
    #[message = "You must agree to the terms of service"]
    terms_accepted: Option<bool>,
}

// ===== Test Runner =====

fn main() {
    println!("=== Advanced RHTMX Validation Demo ===\n");

    // Test 1: String Matching
    println!("Test 1: String Matching Validators");
    let username_form = UsernameForm {
        username: "user_alice_dev".to_string(),
        github_handle: "alice@github.com".to_string(),
    };

    match username_form.validate() {
        Ok(()) => println!("✓ String matching validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 2: Invalid String Matching
    println!("Test 2: Invalid String Matching");
    let bad_username = UsernameForm {
        username: "admin_user".to_string(), // Contains "admin"
        github_handle: "alice@gitlab.com".to_string(), // Doesn't contain @github.com
    };

    match bad_username.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Expected validation failures:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 3: Password Confirmation (equals_field)
    println!("Test 3: Password Confirmation");
    let password_form = PasswordChangeForm {
        new_password: "SecurePass123!".to_string(),
        confirm_password: "SecurePass123!".to_string(),
        must_not_be_common: "UniquePass456!".to_string(),
    };

    match password_form.validate() {
        Ok(()) => println!("✓ Password confirmation passed!\n"),
        Err(errors) => {
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Test 4: Mismatched Passwords
    println!("Test 4: Mismatched Passwords");
    let bad_password = PasswordChangeForm {
        new_password: "SecurePass123!".to_string(),
        confirm_password: "DifferentPass456!".to_string(),
        must_not_be_common: "password123".to_string(),
    };

    match bad_password.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Expected validation failures:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 5: Conditional Validation (depends_on)
    println!("Test 5: Conditional Validation - Pickup (no address needed)");
    let pickup_form = ShippingForm {
        shipping_method: "pickup".to_string(),
        delivery_address: None,
        postal_code: None,
    };

    match pickup_form.validate() {
        Ok(()) => println!("✓ Pickup form valid (no address required)!\n"),
        Err(errors) => {
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Test 6: Conditional Validation - Delivery (address required)
    println!("Test 6: Conditional Validation - Delivery (missing address)");
    let delivery_form = ShippingForm {
        shipping_method: "delivery".to_string(),
        delivery_address: None, // Should fail - required for delivery
        postal_code: None,
    };

    match delivery_form.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Expected validation failures:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 7: Collection Validators
    println!("Test 7: Collection Validators - Valid Team");
    let team_form = TeamForm {
        team_name: "DevTeam".to_string(),
        team_members: vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
        tags: vec!["rust".to_string(), "web".to_string()],
    };

    match team_form.validate() {
        Ok(()) => println!("✓ Team form valid!\n"),
        Err(errors) => {
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Test 8: Collection Validators - Duplicates
    println!("Test 8: Collection Validators - Duplicate Members");
    let bad_team = TeamForm {
        team_name: "DevTeam".to_string(),
        team_members: vec!["Alice".to_string(), "Bob".to_string(), "Alice".to_string()], // Duplicate!
        tags: vec!["rust".to_string()],
    };

    match bad_team.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Expected validation failure:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 9: Enum Variant Validation
    println!("Test 9: Enum Variant - Valid Config");
    let config_form = ConfigForm {
        environment: "production".to_string(),
        log_level: "info".to_string(),
        region: "us-east-1".to_string(),
    };

    match config_form.validate() {
        Ok(()) => println!("✓ Config valid!\n"),
        Err(errors) => {
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Test 10: Enum Variant - Invalid Values
    println!("Test 10: Enum Variant - Invalid Values");
    let bad_config = ConfigForm {
        environment: "test".to_string(), // Not in allowed list
        log_level: "verbose".to_string(), // Not in allowed list
        region: "ap-south-1".to_string(), // Not in allowed list
    };

    match bad_config.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Expected validation failures:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 11: Custom Messages and Labels
    println!("Test 11: Custom Messages - Invalid Form");
    let friendly_form = FriendlyForm {
        name: "Al".to_string(), // Too short
        email: "not-an-email".to_string(),
        age: 16, // Under 18
    };

    match friendly_form.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Custom error messages:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 12: Custom Validation Functions
    println!("Test 12: Custom Validators - Valid");
    let custom_form = CustomValidationForm {
        lucky_number: 42, // Even number
        bio: "I love programming in Rust and building web apps".to_string(),
    };

    match custom_form.validate() {
        Ok(()) => println!("✓ Custom validation passed!\n"),
        Err(errors) => {
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Test 13: Custom Validation - Failed
    println!("Test 13: Custom Validators - Failed");
    let bad_custom = CustomValidationForm {
        lucky_number: 13, // Odd number
        bio: "Short".to_string(), // Too short
    };

    match bad_custom.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Custom validation failures:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 14: Complete Real-World Form - Valid
    println!("Test 14: Complete Registration Form - Valid");
    let registration = CompleteRegistrationForm {
        username: "user_johndoe".to_string(),
        email: "john@company.com".to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        account_type: "individual".to_string(),
        company_name: None,
        tax_id: None,
        interests: vec!["rust".to_string(), "web".to_string(), "ai".to_string()],
        about: "Passionate software developer with 5 years of experience".to_string(),
        terms_accepted: Some(true),
    };

    match registration.validate() {
        Ok(()) => println!("✓ Complete registration form valid!\n"),
        Err(errors) => {
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Test 15: Complete Form - Business Type Missing Required Fields
    println!("Test 15: Complete Form - Business Type (Missing Fields)");
    let business_registration = CompleteRegistrationForm {
        username: "user_acmecorp".to_string(),
        email: "admin@acmecorp.com".to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        account_type: "business".to_string(), // Business type
        company_name: None, // Should be required!
        tax_id: None, // Should be required!
        interests: vec!["saas".to_string()],
        about: "Enterprise software solutions provider".to_string(),
        terms_accepted: Some(true),
    };

    match business_registration.validate() {
        Ok(()) => println!("✓ Validation passed\n"),
        Err(errors) => {
            println!("✗ Expected conditional validation failures:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    println!("=== Advanced Validation Demo Complete ===");
    println!("\n📚 Summary of New Validators Demonstrated:");
    println!("  ✓ String matching: contains, not_contains, starts_with, ends_with");
    println!("  ✓ Equality: equals, not_equals, equals_field");
    println!("  ✓ Conditional: depends_on");
    println!("  ✓ Collections: min_items, max_items, unique");
    println!("  ✓ Enum variants: enum_variant");
    println!("  ✓ Custom messages: message, label");
    println!("  ✓ Custom functions: custom");
}
