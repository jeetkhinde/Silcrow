// Example: Unified Validation - Single Source of Truth
//
// This example demonstrates how to eliminate DRY violations by defining
// validation rules once in Rust and automatically generating both server-side
// validation and client-side HTML5/data-validate attributes.

use rhtmx::{Validate, FormField};
use serde::Deserialize;

// ============================================================================
// THE PROBLEM (Before)
// ============================================================================
//
// Previously, you had to define validation rules twice:
//
// 1. In Rust (server-side):
// ```rust
// #[derive(Validate, Deserialize)]
// struct RegisterForm {
//     #[email]
//     #[no_public_domains]
//     #[required]
//     email: String,
// }
// ```
//
// 2. In HTML (client-side):
// ```html
// <input name="email"
//     type="email"
//     required
//     data-validate='{"email": true, "noPublicDomains": true, "required": true}'
// />
// ```
//
// Issues:
// ❌ Same rules written twice
// ❌ Easy to get out of sync (change Rust, forget HTML)
// ❌ Manual JSON writing is error-prone
// ❌ No type safety between struct and form
// ❌ Extra work for developers
//
// ============================================================================
// THE SOLUTION (After)
// ============================================================================
//
// Now you define validation rules ONCE in Rust, and they automatically generate
// both server-side validation AND client-side HTML attributes:

#[derive(Validate, FormField, Deserialize, Clone, Debug)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    #[label("Email Address")]
    email: String,

    #[min_length(8)]
    #[max_length(100)]
    #[password("strong")]
    #[required]
    #[label("Password")]
    password: String,

    #[min_length(3)]
    #[max_length(50)]
    #[required]
    #[label("Full Name")]
    name: String,

    #[min(18)]
    #[max(120)]
    age: i32,

    #[url]
    #[label("Website")]
    website: Option<String>,
}

// ============================================================================
// USAGE: Automatic Attribute Generation
// ============================================================================

fn main() {
    println!("========================================");
    println!("  Unified Validation Demo");
    println!("  Single Source of Truth for Validation");
    println!("========================================\n");

    // Create a form instance
    let form = RegisterForm {
        email: String::new(),
        password: String::new(),
        name: String::new(),
        age: 18,
        website: None,
    };

    println!("✅ Define validation rules ONCE in Rust:\n");
    println!("#[derive(Validate, FormField, Deserialize)]");
    println!("struct RegisterForm {{");
    println!("    #[email]");
    println!("    #[no_public_domains]");
    println!("    #[required]");
    println!("    email: String,");
    println!();
    println!("    #[min_length(8)]");
    println!("    #[password(\"strong\")]");
    println!("    password: String,");
    println!("    ...");
    println!("}}\n");

    println!("========================================");
    println!("Generated Form Field Attributes:");
    println!("========================================\n");

    // Get attributes for each field
    for field_name in form.field_names() {
        let attrs = form.field_attrs(field_name);

        println!("📌 Field: {}", field_name);
        println!("   Label: {}", attrs.label);

        if !attrs.html5_attrs.is_empty() {
            println!("   HTML5 Attributes:");
            for (key, value) in &attrs.html5_attrs {
                if value.is_empty() {
                    println!("     - {}", key);
                } else {
                    println!("     - {}=\"{}\"", key, value);
                }
            }
        }

        println!("   Client-side Validation:");
        println!("     data-validate='{}'", attrs.data_validate);
        println!();
    }

    println!("========================================");
    println!("Benefits:");
    println!("========================================");
    println!("✅ Single Source of Truth");
    println!("✅ Type Safety");
    println!("✅ Automatic Sync");
    println!("✅ Less Code");
    println!("✅ Fewer Errors");
    println!("✅ Better Developer Experience\n");

    println!("========================================");
    println!("How to use in templates:");
    println!("========================================\n");

    let email_attrs = form.field_attrs("email");
    println!("let email_attrs = form.field_attrs(\"email\");");
    println!();
    println!("// Then in your HTML template:");
    println!("<input");
    println!("    name=\"email\"");
    for (key, value) in &email_attrs.html5_attrs {
        if value.is_empty() {
            println!("    {}", key);
        } else {
            println!("    {}=\"{}\"", key, value);
        }
    }
    println!("    data-validate='{}'", email_attrs.data_validate);
    println!("/>");
    println!();

    println!("========================================");
    println!("Validation Mapping:");
    println!("========================================\n");
    println!("Rust Attribute          → HTML5 Attribute    → data-validate JSON");
    println!("────────────────────────────────────────────────────────────────────");
    println!("#[email]                → type=\"email\"       → \"email\": true");
    println!("#[required]             → required           → \"required\": true");
    println!("#[min_length(n)]        → minlength=\"n\"      → \"minLength\": n");
    println!("#[max_length(n)]        → maxlength=\"n\"      → \"maxLength\": n");
    println!("#[min(n)]               → min=\"n\"            → \"min\": n");
    println!("#[max(n)]               → max=\"n\"            → \"max\": n");
    println!("#[url]                  → type=\"url\"         → \"url\": true");
    println!("#[regex(pattern)]       → pattern=\"...\"      → \"pattern\": \"...\"");
    println!("#[password(\"strong\")]   → (none)             → \"password\": \"strong\"");
    println!("#[no_public_domains]    → (none)             → \"noPublicDomains\": true");
    println!();
}

// ============================================================================
// BENEFITS
// ============================================================================
//
// ✅ Single Source of Truth: Define rules once in Rust
// ✅ Type Safety: Impossible to have mismatched validation
// ✅ Automatic Sync: Changes in Rust automatically update HTML
// ✅ Less Code: No manual data-validate JSON writing
// ✅ Less Errors: Can't forget to update client-side validation
// ✅ Better DX: Cleaner, more maintainable code
