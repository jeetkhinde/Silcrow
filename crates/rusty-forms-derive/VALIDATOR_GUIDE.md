# RHTMX Validator Complete Guide

## Quick Reference Table

| Category | Validator | Usage | Example Error |
|----------|-----------|-------|---------------|
| **Email** | `#[email]` | `email: String` | "Invalid email address" |
| | `#[no_public_domains]` | `email: String` | "Public email domains not allowed" |
| | `#[blocked_domains("a.com")]` | `email: String` | "Email domain is blocked" |
| **Password** | `#[password("strong")]` | `password: String` | "Password must be at least 8 characters..." |
| | `#[password("medium")]` | `password: String` | "Password must be 8+ chars with upper, lower, digit" |
| | `#[password("basic")]` | `password: String` | "Password must be at least 6 characters" |
| **Numeric** | `#[min(18)]` | `age: i32` | "Must be at least 18" |
| | `#[max(100)]` | `score: i32` | "Must be at most 100" |
| | `#[range(1, 10)]` | `rating: i32` | "Must be between 1 and 10" |
| **String Length** | `#[min_length(3)]` | `name: String` | "Must be at least 3 characters" |
| | `#[max_length(100)]` | `bio: String` | "Must be at most 100 characters" |
| | `#[length(3, 50)]` | `username: String` | "Must be between 3 and 50 characters" |
| **String Pattern** | `#[regex(r"^\d{3}-\d{3}-\d{4}$")]` | `phone: String` | "Invalid format" |
| | `#[url]` | `website: String` | "Invalid URL" |
| **String Matching** | `#[contains("@")]` | `email: String` | "Must contain '@'" |
| | `#[not_contains("admin")]` | `username: String` | "Must not contain 'admin'" |
| | `#[starts_with("user_")]` | `username: String` | "Must start with 'user_'" |
| | `#[ends_with(".com")]` | `domain: String` | "Must end with '.com'" |
| **Equality** | `#[equals("yes")]` | `confirmation: String` | "Must equal 'yes'" |
| | `#[not_equals("password123")]` | `password: String` | "Must not equal 'password123'" |
| | `#[equals_field("password")]` | `confirm: String` | "Must match password" |
| **Conditional** | `#[depends_on("type", "business")]` | `company: Option<String>` | "Required when type is business" |
| **Collections** | `#[min_items(2)]` | `tags: Vec<String>` | "Must have at least 2 items" |
| | `#[max_items(10)]` | `tags: Vec<String>` | "Must have at most 10 items" |
| | `#[unique]` | `emails: Vec<String>` | "All items must be unique" |
| **Enum/Values** | `#[enum_variant("a", "b", "c")]` | `status: String` | "Must be one of: a, b, c" |
| **Custom** | `#[custom("my_validator")]` | `data: String` | Custom message from function |
| | `#[message = "Custom error"]` | Any field | Overrides default message |
| | `#[label("Display Name")]` | Any field | Used in error messages |
| **General** | `#[required]` | `field: Option<String>` | "This field is required" |
| | `#[allow_whitespace]` | `content: String` | (Preserves whitespace) |

---

## 🎯 Common Patterns

### 1. User Registration Form

```rust
use rhtmx::{Validate, ValidateTrait};
use serde::Deserialize;

#[derive(Validate, Deserialize)]
struct RegisterForm {
    // Username: 3-20 chars, starts with letter, alphanumeric + underscore
    #[min_length(3)]
    #[max_length(20)]
    #[regex(r"^[a-zA-Z][a-zA-Z0-9_]*$")]
    #[label("Username")]
    username: String,

    // Email: valid format, no public domains
    #[email]
    #[no_public_domains]
    #[label("Email Address")]
    email: String,

    // Password: strong with confirmation
    #[password("strong")]
    password: String,

    #[equals_field("password")]
    #[message = "Passwords must match"]
    password_confirmation: String,

    // Age: 18-120
    #[range(18, 120)]
    age: i32,

    // Terms: must be accepted
    #[required]
    #[message = "You must agree to the terms"]
    terms_accepted: Option<bool>,
}

// Usage
fn handle_registration(form: RegisterForm) -> Result<(), HashMap<String, String>> {
    form.validate()?;
    // Process registration...
    Ok(())
}
```

### 2. Shipping Form with Conditional Fields

```rust
#[derive(Validate, Deserialize)]
struct ShippingForm {
    // Shipping method selection
    #[enum_variant("pickup", "standard", "express")]
    shipping_method: String,

    // Address required only for delivery
    #[depends_on("shipping_method", "standard")]
    #[min_length(10)]
    standard_address: Option<String>,

    #[depends_on("shipping_method", "express")]
    #[min_length(10)]
    express_address: Option<String>,

    // Notes are optional
    #[max_length(500)]
    notes: Option<String>,
}
```

### 3. Team Management Form

```rust
#[derive(Validate, Deserialize)]
struct TeamForm {
    // Team name: 3-50 chars, no special chars
    #[min_length(3)]
    #[max_length(50)]
    #[regex(r"^[a-zA-Z0-9\s]+$")]
    #[label("Team Name")]
    team_name: String,

    // Members: 2-10 unique emails
    #[min_items(2)]
    #[max_items(10)]
    #[unique]
    #[label("Team Members")]
    member_emails: Vec<String>,

    // Tags: 1-5 items
    #[min_items(1)]
    #[max_items(5)]
    tags: Vec<String>,
}
```

### 4. Configuration Form

```rust
#[derive(Validate, Deserialize)]
struct ConfigForm {
    // Environment: must be one of predefined values
    #[enum_variant("development", "staging", "production")]
    environment: String,

    // API endpoint: must be valid URL, ends with /api
    #[url]
    #[ends_with("/api")]
    api_endpoint: String,

    // API key: must not contain spaces, min length
    #[min_length(32)]
    #[not_contains(" ")]
    api_key: String,

    // Timeout: 1-60 seconds
    #[range(1, 60)]
    timeout_seconds: i32,
}
```

### 5. Custom Validation Example

```rust
// Custom validator function
fn validate_credit_card(number: &String) -> Result<(), String> {
    let digits: String = number.chars().filter(|c| c.is_digit(10)).collect();

    if digits.len() != 16 {
        return Err("Credit card must be 16 digits".to_string());
    }

    // Luhn algorithm check
    let mut sum = 0;
    for (i, digit) in digits.chars().rev().enumerate() {
        let mut d = digit.to_digit(10).unwrap();
        if i % 2 == 1 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }

    if sum % 10 == 0 {
        Ok(())
    } else {
        Err("Invalid credit card number".to_string())
    }
}

#[derive(Validate, Deserialize)]
struct PaymentForm {
    #[custom("validate_credit_card")]
    #[label("Card Number")]
    card_number: String,

    #[regex(r"^\d{3,4}$")]
    cvv: String,
}
```

---

## 🔧 Advanced Techniques

### Combining Multiple Validators

Stack validators for complex rules:

```rust
#[derive(Validate)]
struct AdvancedForm {
    // Multiple constraints on same field
    #[min_length(8)]
    #[max_length(64)]
    #[starts_with("SK_")]
    #[not_contains(" ")]
    #[regex(r"^[A-Z0-9_]+$")]
    #[label("Secret Key")]
    secret_key: String,
}
```

### Custom Error Messages

Override any validator's default message:

```rust
#[derive(Validate)]
struct FriendlyForm {
    #[email]
    #[message = "Oops! That doesn't look like a valid email address. Please check and try again."]
    email: String,

    #[min(18)]
    #[message = "Sorry, you must be 18 or older to use this service."]
    age: i32,
}
```

### Field Labels for Better Errors

Use labels for more readable error messages:

```rust
#[derive(Validate)]
struct UserForm {
    #[required]
    #[label("First Name")]
    first_name: Option<String>,  // Error: "First Name is required"

    #[min_length(10)]
    #[label("Phone Number")]
    phone: String,  // Error: "Phone Number must be at least 10 characters"
}
```

### Complex Conditional Logic

```rust
#[derive(Validate)]
struct ApplicationForm {
    #[enum_variant("student", "professional", "business")]
    applicant_type: String,

    // Student fields
    #[depends_on("applicant_type", "student")]
    school_name: Option<String>,

    #[depends_on("applicant_type", "student")]
    student_id: Option<String>,

    // Professional fields
    #[depends_on("applicant_type", "professional")]
    company_name: Option<String>,

    #[depends_on("applicant_type", "professional")]
    years_experience: Option<i32>,

    // Business fields
    #[depends_on("applicant_type", "business")]
    business_name: Option<String>,

    #[depends_on("applicant_type", "business")]
    tax_id: Option<String>,
}
```

---

## 🎨 Integration with HTMX

### Server-Side Validation

```rust
use rhtmx::{post, Ok, Error, StatusCode};

#[post]
fn create_user(req: RegisterForm) -> Result<OkResponse, ErrorResponse> {
    // Validate the form
    if let Err(errors) = req.validate() {
        return Err(Error()
            .status(StatusCode::BAD_REQUEST)
            .render(validation_errors_component, errors));
    }

    // Process the valid request
    let user = db::create_user(req)?;
    Ok()
        .render(user_card, user)
        .toast("User created successfully!")
}

fn validation_errors_component(errors: HashMap<String, String>) -> Html {
    html! {
        <div class="errors">
            <div r-for="(field, error) in errors">
                <div class="error-item">
                    <strong>{field}": "</strong>
                    <span>{error}</span>
                </div>
            </div>
        </div>
    }
}
```

### Incremental Validation

Validate single field on blur:

```html
<form hx-post="/users" hx-target="#result">
    <input
        name="email"
        hx-post="/validate/email"
        hx-trigger="blur"
        hx-target="next .error"
    />
    <span class="error"></span>

    <input
        name="password"
        hx-post="/validate/password"
        hx-trigger="blur"
        hx-target="next .error"
    />
    <span class="error"></span>

    <button type="submit">Register</button>
</form>
```

```rust
#[derive(Validate, Deserialize)]
struct EmailValidation {
    #[email]
    #[no_public_domains]
    email: String,
}

#[post("/validate/email")]
fn validate_email(req: EmailValidation) -> Result<OkResponse, ErrorResponse> {
    if let Err(errors) = req.validate() {
        let msg = errors.get("email").unwrap();
        return Err(Error().render(error_message, msg));
    }
    Ok().render(success_message, "Email is valid!")
}
```

---

## 📚 Best Practices

### 1. **Always Use Labels for User-Facing Forms**
```rust
#[label("Email Address")]  // ✅ Better UX
email: String,
```

### 2. **Provide Custom Messages for Complex Rules**
```rust
#[regex(r"^[A-Z]{2}\d{6}$")]
#[message = "License must be 2 letters followed by 6 digits (e.g., AB123456)"]
license_number: String,
```

### 3. **Use depends_on for Conditional Logic**
```rust
// ✅ Clear conditional requirement
#[depends_on("account_type", "business")]
company_name: Option<String>,
```

### 4. **Combine Validators for Precision**
```rust
#[min_length(8)]
#[max_length(64)]
#[regex(r"^[a-zA-Z0-9_]+$")]
username: String,
```

### 5. **Use Custom Validators for Domain Logic**
```rust
#[custom("validate_business_hours")]
appointment_time: String,
```

### 6. **Validate Collections Properly**
```rust
#[min_items(1)]      // Ensure not empty
#[max_items(10)]     // Prevent abuse
#[unique]            // No duplicates
tags: Vec<String>,
```

---

## 🐛 Common Mistakes to Avoid

### ❌ DON'T: Forget to import ValidateTrait
```rust
use rhtmx::Validate;  // ❌ Missing ValidateTrait

form.validate();  // Won't compile!
```

### ✅ DO: Import both
```rust
use rhtmx::{Validate, ValidateTrait};  // ✅ Correct

form.validate();  // Works!
```

### ❌ DON'T: Use depends_on on non-Option fields
```rust
#[depends_on("type", "business")]
company_name: String,  // ❌ Should be Option<String>
```

### ✅ DO: Use Option for conditional fields
```rust
#[depends_on("type", "business")]
company_name: Option<String>,  // ✅ Correct
```

### ❌ DON'T: Stack conflicting validators
```rust
#[equals("test")]
#[not_equals("test")]  // ❌ Conflicting!
value: String,
```

---

## 🚀 Performance Tips

1. **Compile-Time Generation** - All validation code is generated at compile time, zero runtime overhead
2. **Early Returns** - Validators short-circuit on first error per field
3. **No Allocations** - Validation checks are in-place, no extra allocations
4. **Type-Safe** - All checks are statically verified by Rust compiler

---

## 🔮 Future: WASM Integration

Coming soon - these same validators will work in the browser:

```javascript
import { validateField } from './pkg/rhtmx_validation.js';

// Real-time validation as user types
input.addEventListener('keyup', debounce(async (e) => {
    const errors = await validateField('email', e.target.value, {
        email: true,
        no_public_domains: true
    });

    if (errors.length > 0) {
        showError(errors[0].message);
    } else {
        clearError();
    }
}, 300));
```

---

## 📖 Reference

All validators are documented in:
- **FEATURES.md** - High-level feature overview
- **VALIDATORS_SUMMARY.md** - Implementation details
- **advanced_validation_demo.rs** - 15 working examples

For questions or issues, see the project README.
