// Example: Complete validation demo with all validators
// Shows how to use #[derive(Validate)] with RHTMX

use rhtmx::{html, Validate, ValidateTrait};
use serde::Deserialize;
use std::collections::HashMap;

// ===== User Registration Example =====

#[derive(Validate, Deserialize, Debug)]
struct RegisterUserRequest {
    // String validators
    #[min_length(3)]
    #[max_length(50)]
    name: String,

    // Email validators
    #[email]
    #[no_public_domains]
    email: String,

    // Password validators
    #[password("strong")]
    password: String,

    // Numeric validators
    #[min(18)]
    #[max(120)]
    age: i32,

    // Optional fields
    bio: Option<String>,

    // URL validator
    #[url]
    website: String,
}

// ===== Profile Update Example =====

#[derive(Validate, Deserialize, Debug)]
struct UpdateProfileRequest {
    #[min_length(3)]
    username: String,

    #[max_length(200)]
    bio: String,

    #[regex(r"^\d{3}-\d{3}-\d{4}$")]
    phone: String,

    #[url]
    website: String,
}

// ===== Product Example =====

#[derive(Validate, Deserialize, Debug)]
struct CreateProductRequest {
    #[length(5, 100)]
    name: String,

    #[min_length(20)]
    description: String,

    #[range(1, 10000)]
    price: i32,

    #[range(0, 1000)]
    stock: i32,
}

// ===== Admin User Example =====

#[derive(Validate, Deserialize, Debug)]
struct CreateAdminRequest {
    name: String,

    #[email]
    #[blocked_domains("gmail.com", "yahoo.com", "hotmail.com")]
    email: String,

    #[password("strong")]
    password: String,
}

// ===== Optional Fields Example =====

#[derive(Validate, Deserialize, Debug)]
struct UpdateSettingsRequest {
    #[required]
    username: Option<String>,

    email: Option<String>,

    bio: Option<String>,
}

// ===== Helper Functions =====

fn render_validation_errors(errors: &HashMap<String, String>) {
    let mut items = String::new();
    for (field, error) in errors {
        items.push_str(&format!("<li><strong>{}:</strong> {}</li>", field, error));
    }

    html! {
        <div class="errors">
            <h3>"Validation Errors:"</h3>
            <ul>{items}</ul>
        </div>
    }
}

fn main() {
    println!("=== RHTMX Validation Demo ===\n");

    // Test 1: Valid user registration
    println!("Test 1: Valid User Registration");
    let valid_user = RegisterUserRequest {
        name: "Alice Smith".to_string(),
        email: "alice@company.com".to_string(),
        password: "SecurePass123!".to_string(),
        age: 25,
        bio: Some("Software engineer".to_string()),
        website: "https://alice.dev".to_string(),
    };

    match valid_user.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 2: Invalid user registration (multiple errors)
    println!("Test 2: Invalid User Registration");
    let invalid_user = RegisterUserRequest {
        name: "Al".to_string(),             // Too short
        email: "bob@gmail.com".to_string(), // Public domain
        password: "weak".to_string(),       // Weak password
        age: 15,                            // Too young
        bio: None,
        website: "not-a-url".to_string(), // Invalid URL
    };

    match invalid_user.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed ({} errors):", errors.len());
            for (field, error) in &errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 3: Profile update with regex validation
    println!("Test 3: Profile Update (Invalid Phone)");
    let profile = UpdateProfileRequest {
        username: "alice_dev".to_string(),
        bio: "Full-stack developer".to_string(),
        phone: "123456789".to_string(), // Invalid format
        website: "https://example.com".to_string(),
    };

    match profile.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 4: Valid profile update
    println!("Test 4: Valid Profile Update");
    let valid_profile = UpdateProfileRequest {
        username: "alice_dev".to_string(),
        bio: "Full-stack developer".to_string(),
        phone: "555-123-4567".to_string(), // Valid format
        website: "https://example.com".to_string(),
    };

    match valid_profile.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 5: Product with range validation
    println!("Test 5: Product (Price out of range)");
    let product = CreateProductRequest {
        name: "Laptop".to_string(),
        description: "High-performance gaming laptop".to_string(),
        price: 15000, // Out of range
        stock: 50,
    };

    match product.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 6: Admin user with blocked domains
    println!("Test 6: Admin User (Blocked Domain)");
    let admin = CreateAdminRequest {
        name: "Admin".to_string(),
        email: "admin@gmail.com".to_string(), // Blocked domain
        password: "AdminPass123!".to_string(),
    };

    match admin.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 7: Valid admin user
    println!("Test 7: Valid Admin User");
    let valid_admin = CreateAdminRequest {
        name: "Admin".to_string(),
        email: "admin@company.com".to_string(),
        password: "AdminPass123!".to_string(),
    };

    match valid_admin.validate() {
        Ok(()) => println!("✓ Validation passed!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    // Test 8: Password strength levels
    println!("Test 8: Password Strength Levels");

    #[derive(Validate, Deserialize)]
    struct PasswordTest {
        #[password("basic")]
        basic: String,
        #[password("medium")]
        medium: String,
        #[password("strong")]
        strong: String,
    }

    let passwords = PasswordTest {
        basic: "simple".to_string(),          // Valid for basic (6+ chars)
        medium: "Password123".to_string(),    // Valid for medium (8+ with upper, lower, digit)
        strong: "SecurePass123!".to_string(), // Valid for strong (8+ with all)
    };

    match passwords.validate() {
        Ok(()) => println!("✓ All password levels validated!\n"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for (field, error) in errors {
                println!("  - {}: {}", field, error);
            }
            println!();
        }
    }

    println!("=== Validation Demo Complete ===");
}
