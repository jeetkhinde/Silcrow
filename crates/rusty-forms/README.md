# rusty-forms

> **A complete Rust form validation library with derive macros, type-safe validation, and automatic client-side/server-side synchronization.**

[![Crates.io](https://img.shields.io/crates/v/rusty-forms.svg)](https://crates.io/crates/rusty-forms)
[![Documentation](https://docs.rs/rusty-forms/badge.svg)](https://docs.rs/rusty-forms)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## ✨ Features

- 🎯 **Single Source of Truth** - Define validation rules once, use everywhere
- 🔄 **Client & Server Sync** - Automatic HTML5 + WASM + server-side validation
- 🛡️ **Type-Safe** - Leverage Rust's type system with nutype integration
- 📦 **40+ Built-in Validators** - Email, password, numeric, string, collections, etc.
- 🎨 **Custom Validators** - Easy to extend with your own validation logic
- 🌐 **i18n Ready** - Built-in internationalization support
- 🚀 **Zero Runtime** - Validation code generated at compile-time
- 📝 **Great DX** - Clear error messages, excellent documentation

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rusty-forms = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

Define your form with validation rules:

```rust
use rusty_forms::{Validate, FormField};
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    email: String,

    #[min_length(8)]
    #[password("strong")]
    password: String,

    #[equals_field("password")]
    confirm_password: String,

    #[min(18)]
    #[max(120)]
    age: i32,
}
```

Validate on the server:

```rust
fn handle_registration(form: RegisterForm) -> Result<(), HashMap<String, Vec<String>>> {
    // Validate returns field-level errors
    form.validate()?;

    // Process valid form...
    Ok(())
}
```

Render with validation attributes in HTML:

```rust
let form = RegisterForm::default();
let email_attrs = form.field_attrs("email");

// Renders: <input type="email" required data-validate='{"email":true,"noPublicDomains":true}' />
```

## 📦 Optional Features

Enable additional features in your `Cargo.toml`:

```toml
# Include pre-built validated types (EmailAddress, Password, etc.)
rusty-forms = { version = "0.1", features = ["nutype"] }

# Include validation functions for custom validators
rusty-forms = { version = "0.1", features = ["validation"] }

# Enable all features
rusty-forms = { version = "0.1", features = ["full"] }
```

## 📚 Available Validators

### Email Validators
- `#[email]` - Valid email format
- `#[no_public_domains]` - Reject gmail, yahoo, etc.
- `#[blocked_domains("a.com", "b.com")]` - Block specific domains

### Password Validators
- `#[password("strong")]` - 8+ chars, upper, lower, digit, special
- `#[password("medium")]` - 8+ chars, upper, lower, digit
- `#[password("basic")]` - 6+ chars

### Numeric Validators
- `#[min(n)]` - Minimum value
- `#[max(n)]` - Maximum value
- `#[range(min, max)]` - Value range

### String Validators
- `#[min_length(n)]` - Minimum length
- `#[max_length(n)]` - Maximum length
- `#[regex(r"pattern")]` - Custom regex
- `#[url]` - Valid URL format

### Field Comparison
- `#[equals_field("field")]` - Must match another field
- `#[not_equals("value")]` - Must not equal value

### Collections
- `#[min_items(n)]` - Minimum items in Vec/HashSet
- `#[max_items(n)]` - Maximum items
- `#[unique]` - All items must be unique

### Custom
- `#[custom("function")]` - Call custom validation function
- `#[message = "text"]` - Override default error message
- `#[label("Name")]` - Use friendly name in errors

[See full list of 40+ validators in the documentation →](https://docs.rs/rusty-forms)

## 🎨 Using Pre-built Types (nutype feature)

Enable the `nutype` feature to access type-safe validated types:

```rust
use rusty_forms::types::{EmailAddress, PasswordStrong, Age};

#[derive(Validate, FormField)]
struct UserForm {
    #[nutype]  // Skip base validation, type already enforces it
    email: EmailAddress,

    #[nutype]
    password: PasswordStrong,

    #[nutype]
    age: Age,
}
```

Available types: `EmailAddress`, `WorkEmailAddress`, `PasswordStrong`, `PhoneNumber`, `ZipCode`, `Age`, `Percentage`, and 20+ more.

## 🏗️ Architecture

`rusty-forms` is a parent crate that re-exports three component crates:

```
rusty-forms                  (you add this)
├── rusty-forms-derive       (proc macros)
├── rusty-forms-validation   (core validators)
└── rusty-forms-types        (nutype types)
```

**99% of users** should just use `rusty-forms`.

**Advanced users** can depend on individual components:
- Just macros: `rusty-forms-derive`
- Just validators: `rusty-forms-validation`
- Just types: `rusty-forms-types`

## 🔧 Custom Validators

Create custom validation functions:

```rust
fn validate_username(value: &str) -> bool {
    value.len() >= 3 && value.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[derive(Validate)]
struct Form {
    #[custom("validate_username")]
    #[message = "Username must be 3+ chars and alphanumeric"]
    username: String,
}
```

## 🌐 Internationalization

Use message keys for i18n:

```rust
#[derive(Validate)]
struct Form {
    #[email]
    #[message_key("errors.email.invalid")]
    email: String,
}
```

## 📖 Documentation

- [Full API Documentation](https://docs.rs/rusty-forms)
- [Validator Guide](https://github.com/jeetkhinde/RHTMX/blob/main/crates/rusty-forms-derive/VALIDATOR_GUIDE.md)
- [Examples](https://github.com/jeetkhinde/RHTMX/tree/main/crates/rusty-forms/examples)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built as part of the [RHTMX](https://github.com/jeetkhinde/RHTMX) framework, but fully standalone and reusable.

---

**Made with ❤️ for the Rust community**
