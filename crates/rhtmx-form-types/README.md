# rhtmx-form-types

Common validated types for RHTMX forms using `nutype`.

## ✅ WASM Compatible

All types in this crate work in WebAssembly environments and can be used for client-side validation!

## Features

- **Type-safe domain modeling** with nutype
- **WASM compatible** - use the same types on server and client
- **Serde support** - serialize/deserialize for forms
- **Zero-cost abstractions** - compiled away to plain Rust types
- **Integrates with RHTMX validation** - combine type validation with form validators

## Usage

### Basic Example

```rust
use rhtmx::{Validate, FormField};
use rhtmx_form_types::EmailAddress;
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct LoginForm {
    // Type validates email format (at construction time)
    // Form adds business rule (no public domains)
    #[nutype]
    #[no_public_domains]
    email: EmailAddress,

    // Type validates minimum length (6+ chars)
    // Form adds strength requirements
    #[nutype]
    #[password("medium")]  // Uppercase + lowercase + digit
    password: PasswordMedium,
}
```

### Available Types

#### Email Types
- `EmailAddress` - Basic email format validation

#### Password Types
- `PasswordBasic` - 6+ characters
- `PasswordMedium` - 8+ characters (combine with `#[password("medium")]` for full validation)
- `PasswordStrong` - 10+ characters (combine with `#[password("strong")]` for full validation)

#### String Types
- `NonEmptyString` - Cannot be empty
- `Username` - 3-30 characters, alphanumeric + underscore/dash

#### Numeric Types
- `PositiveInt` - Integer > 0
- `NonNegativeInt` - Integer >= 0

### Construction

Since these types have validation, use `try_new()` or `TryFrom`:

```rust
use rhtmx_form_types::EmailAddress;

// Valid email
let email = EmailAddress::try_new("user@example.com".to_string())?;

// Invalid email - returns error
let invalid = EmailAddress::try_new("not-an-email".to_string()); // Err

// From integers
let positive = PositiveInt::try_from(42)?;
```

### Deserialization (Forms)

When deserializing from HTTP form data, validation happens automatically:

```rust
#[derive(Deserialize)]
struct RegistrationData {
    email: EmailAddress,  // Validates during deserialization
    password: PasswordStrong,
}

// If form data is invalid, deserialization fails with clear error
```

### WASM Usage

The same types work in WebAssembly for client-side validation:

```rust
// In your WASM validation code
use rhtmx_form_types::EmailAddress;

#[wasm_bindgen]
pub fn validate_email_client(input: String) -> bool {
    EmailAddress::try_new(input).is_ok()
}
```

## Route-Specific Types

You can extend or override types for specific routes:

```rust
// In your application
pub mod types {
    pub use rhtmx_form_types::*;

    // Route-specific types
    pub mod admin {
        use nutype::nutype;

        /// Admin requires extra-strong passwords
        #[nutype(
            validate(len_char_min = 12),
            derive(Debug, Clone, Serialize, Deserialize)
        )]
        pub struct AdminPassword(String);
    }

    pub mod users {
        use super::*;

        /// Regular user password (reuse from common types)
        pub use PasswordMedium as UserPassword;
    }
}

// Use in forms
use types::admin::AdminPassword;

#[derive(Validate, FormField)]
struct AdminLoginForm {
    #[nutype]
    password: AdminPassword,  // Requires 12+ chars
}
```

## Integration with RHTMX Validation

### Type Validation vs Form Validation

**Type Validation** (enforced by nutype):
- Email format
- String length minimums/maximums
- Numeric ranges
- Pattern matching

**Form Validation** (enforced by RHTMX):
- Business rules (`#[no_public_domains]`)
- Password strength tiers (`#[password("strong")]`)
- Cross-field validation (`#[equals_field = "password"]`)
- Conditional validation (`#[depends_on(...)]`)

### Hybrid Approach (Best Practice)

```rust
#[derive(Validate, FormField, Deserialize)]
struct SignupForm {
    // Type: email format ✓
    // Form: no Gmail/Yahoo/etc. ✓
    #[nutype]
    #[no_public_domains]
    #[required]
    email: EmailAddress,

    // Type: 10+ chars ✓
    // Form: uppercase + lowercase + digit + special ✓
    #[nutype]
    #[password("strong")]
    password: PasswordStrong,

    // Type: 10+ chars ✓
    // Form: must match password field ✓
    #[nutype]
    #[equals_field = "password"]
    confirm_password: PasswordStrong,

    // Type: 3-30 chars, alphanumeric ✓
    // Form: unique in database (custom validator) ✓
    #[nutype]
    #[custom = "check_username_available"]
    username: Username,
}
```

## Benefits

1. **Type Safety**: Can't accidentally pass `String` where `EmailAddress` is expected
2. **Early Validation**: Catches invalid data at deserialization time
3. **Reusable**: Define once, use everywhere
4. **WASM Ready**: Same types work in browser
5. **Self-Documenting**: Function signature tells you what's expected
6. **Zero Runtime Cost**: Compiles to plain Rust types

## Example: Before vs After

### Before (plain strings)

```rust
fn send_email(to: String) {  // What format? Is it validated?
    // Hope it's a valid email...
}

let email = user_input;  // No validation
send_email(email);  // Might fail at runtime
```

### After (validated types)

```rust
fn send_email(to: EmailAddress) {  // Guaranteed valid!
    // Can safely use email
}

let email = EmailAddress::try_new(user_input)?;  // Validates
send_email(email);  // Type-safe!
```

## Dependencies

- `nutype` v0.5+ (validation types)
- `serde` v1.0+ (serialization)

## WASM Build

```bash
cargo build -p rhtmx-form-types --target wasm32-unknown-unknown
```

✅ All types compile successfully to WASM!

## Testing

```bash
cargo test -p rhtmx-form-types
```

All 9+ tests passing ✅

## License

MIT
