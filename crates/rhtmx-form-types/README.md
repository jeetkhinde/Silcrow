# rhtmx-form-types

**Business rules embedded in types** - validated newtype wrappers for RHTMX forms using `nutype`.

## 🎯 Philosophy: Types ARE Business Rules

**Instead of this** (type + form validators):
```rust
#[nutype]
#[no_public_domains]  // ← Business rule at form level
email: EmailAddress
```

**Do this** (business rule IN the type):
```rust
email: WorkEmailAddress  // ← Type IS the business rule!
```

## ✅ WASM Compatible

All types work in WebAssembly - same validation on server and client!

## 📧 Email Type Hierarchy

Choose the type that matches your business requirement:

| Type | Blocks Disposable | Blocks Public (Gmail/Yahoo) | Use Case |
|------|------------------|----------------------------|-----------|
| `EmailAddress` / `AnyEmailAddress` | ✅ | ❌ | Consumer apps |
| `WorkEmailAddress` | ✅ | ✅ | B2B SaaS, enterprise tools |
| `BusinessEmailAddress` | ✅ | ✅ | Verified partners only |

### Examples

```rust
// Consumer app - accepts Gmail, Yahoo, etc.
let personal = EmailAddress::try_new("user@gmail.com".to_string())?;  // ✓

// B2B app - corporate emails only
let work = WorkEmailAddress::try_new("user@acme.com".to_string())?;  // ✓
let gmail = WorkEmailAddress::try_new("user@gmail.com".to_string()); // ✗

// Enterprise - verified domains only
let biz = BusinessEmailAddress::try_new("ceo@verified.com".to_string())?;
```

## 🔐 Password Type Hierarchy

Choose the security level that matches your requirements:

| Type | Min Length | Complexity | Use Case |
|------|-----------|------------|-----------|
| `PasswordBasic` | 6 | None | Low-security accounts |
| `PasswordMedium` | 8 | None | Standard apps |
| `PasswordStrong` | 10 | Upper+lower+digit+special | Sensitive data |
| `SuperStrongPassword` | 12 | Upper+lower+digit+2 special | Admin/financial |
| `PasswordPhrase` | 15 | None (favors length) | User-friendly security |
| `PasswordPhrase3` | 20 | 3+ words | xkcd "correct horse..." |
| `ModernPassword` | 16 | None | NIST 2024 guidelines |

### Examples

```rust
// Basic: 6+ characters
let basic = PasswordBasic::try_new("pass12".to_string())?;  // ✓

// Strong: 10+ chars + complexity
let strong = PasswordStrong::try_new("Password123!".to_string())?;  // ✓
let weak = PasswordStrong::try_new("password".to_string());  // ✗

// Super strong: 12+ chars + 2 special
let super_pwd = SuperStrongPassword::try_new("MyPass123!@".to_string())?;  // ✓

// Modern passphrase (user-friendly)
let phrase = PasswordPhrase3::try_new("Correct-Horse-Battery-Staple".to_string())?;  // ✓
```

## 💡 Usage with RHTMX Forms

### Simple: Type IS the Rule

```rust
use rhtmx::{Validate, FormField};
use rhtmx_form_types::{WorkEmailAddress, PasswordStrong};

#[derive(Validate, FormField, Deserialize)]
struct B2BSignupForm {
    // No #[nutype] or #[no_public_domains] needed!
    // Type enforces: no Gmail, no Yahoo, no disposable
    email: WorkEmailAddress,

    // Type enforces: 10+ chars + uppercase + lowercase + digit + special
    password: PasswordStrong,
}
```

**That's it!** No form-level validators needed. The type does everything.

### Route-Specific Security Levels

```rust
mod consumer {
    use rhtmx_form_types::*;

    struct LoginForm {
        email: EmailAddress,        // Accepts Gmail
        password: PasswordBasic,    // 6+ chars
    }
}

mod business {
    use rhtmx_form_types::*;

    struct LoginForm {
        email: WorkEmailAddress,    // No Gmail!
        password: PasswordStrong,   // 10+ chars + complexity
    }
}

mod admin {
    use rhtmx_form_types::*;

    struct LoginForm {
        email: BusinessEmailAddress,     // Verified only
        password: SuperStrongPassword,   // 12+ chars + 2 special
    }
}
```

### Type Safety Prevents Mistakes

```rust
fn send_business_welcome(email: WorkEmailAddress) {
    // Guaranteed: no Gmail, no Yahoo, no disposable
}

fn validate_admin_password(pwd: SuperStrongPassword) {
    // Guaranteed: 12+ chars + 2 special characters
}

// Compile-time enforcement!
let gmail = EmailAddress::try_new("user@gmail.com".to_string())?;
send_business_welcome(gmail);  // ❌ Compile error! Type mismatch.

let work = WorkEmailAddress::try_new("user@acme.com".to_string())?;
send_business_welcome(work);  // ✅ Compiles!
```

## 🌐 WASM Client-Side Validation

Same types work in the browser!

### Server Code
```rust
use rhtmx_form_types::WorkEmailAddress;

#[derive(Validate, FormField)]
struct SignupForm {
    email: WorkEmailAddress,  // Blocks public domains
}
```

### WASM Code (Same Types!)
```rust
use rhtmx_form_types::WorkEmailAddress;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate_work_email(input: String) -> bool {
    WorkEmailAddress::try_new(input).is_ok()
}
```

### JavaScript Usage
```javascript
import { validate_work_email } from './pkg';

validate_work_email("user@acme.com");  // true
validate_work_email("user@gmail.com");  // false
```

## 📚 All Available Types

### Email Types
- `EmailAddress` / `AnyEmailAddress` - Any valid email (blocks disposable only)
- `WorkEmailAddress` - No public domains (blocks Gmail, Yahoo, etc.)
- `BusinessEmailAddress` - Only verified corporate domains

### Password Types
- `PasswordBasic` - 6+ characters
- `PasswordMedium` - 8+ characters
- `PasswordStrong` - 10+ characters + upper + lower + digit + special
- `SuperStrongPassword` - 12+ characters + 2 special chars
- `PasswordPhrase` - 15+ characters (passphrase style)
- `PasswordPhrase3` - 20+ characters, 3+ words
- `ModernPassword` - 16+ characters (NIST 2024)

### String Types
- `NonEmptyString` - Cannot be empty
- `Username` - 3-30 characters, alphanumeric + underscore/dash

### Numeric Types
- `PositiveInt` - Integer > 0
- `NonNegativeInt` - Integer >= 0
- `Age` - Integer 18-120 (adult age range)
- `Percentage` - Integer 0-100
- `Port` - Integer 1-65535 (valid network port)

### URL Types
- `UrlAddress` - Valid URL (http, https, ftp, ws, wss)
- `HttpsUrl` - HTTPS-only URL (secure connections only)

### Pattern Types
- `PhoneNumber` - US phone number (10 digits, accepts formatting)
- `ZipCode` - US zip code (12345 or 12345-6789)
- `IpAddress` - IPv4 address (xxx.xxx.xxx.xxx)
- `Uuid` - UUID format (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)

### Collection Types
- `NonEmptyVec<T>` - Vector with at least one element

## 🎨 Custom Types for Your Domain

Extend with your own business rules:

```rust
use nutype::nutype;
use serde::{Serialize, Deserialize};

// Only emails from Fortune 500 companies
#[nutype(
    validate(predicate = is_fortune_500_domain),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct Fortune500Email(String);

fn is_fortune_500_domain(email: &str) -> bool {
    // Check against Fortune 500 domain list
    let domain = email.split('@').nth(1).unwrap_or("");
    FORTUNE_500_DOMAINS.contains(&domain)
}
```

## ✨ Benefits

1. **Self-Documenting Code**
   ```rust
   // Before
   fn register(email: String, password: String) // What rules?

   // After
   fn register(email: WorkEmailAddress, password: PasswordStrong) // Crystal clear!
   ```

2. **Compile-Time Safety**
   - Can't pass `EmailAddress` where `WorkEmailAddress` is expected
   - Can't pass `PasswordBasic` where `SuperStrongPassword` is required

3. **Single Source of Truth**
   - Business rules defined once in the type
   - Same rules on server and client (WASM)

4. **Zero Runtime Cost**
   - Nutype compiles to plain Rust types
   - No performance overhead

5. **Gradual Adoption**
   - Start with `EmailAddress` (permissive)
   - Upgrade to `WorkEmailAddress` when business needs change
   - Type system enforces migration

## 🧪 Testing

```bash
# Test all types
cargo test -p rhtmx-form-types

# Test WASM compatibility
cargo build -p rhtmx-form-types --target wasm32-unknown-unknown

# Run examples
cargo run --example business_rules_in_types
cargo run --example route_specific_types
```

All 25 tests passing ✅
WASM compatible ✅

## 📦 Installation

```toml
[dependencies]
rhtmx-form-types = { path = "crates/rhtmx-form-types" }
```

## 🚀 Quick Start

```rust
use rhtmx_form_types::*;

// Consumer app
let email = EmailAddress::try_new("user@gmail.com".to_string())?;

// B2B app
let work_email = WorkEmailAddress::try_new("user@acme.com".to_string())?;

// Passwords
let basic = PasswordBasic::try_new("pass12".to_string())?;
let strong = PasswordStrong::try_new("Password123!".to_string())?;
let phrase = PasswordPhrase3::try_new("Correct-Horse-Battery-Staple".to_string())?;

// Specialized types
let age = Age::try_from(25)?;
let discount = Percentage::try_from(15)?;
let server_port = Port::try_from(8080)?;

// URLs
let website = UrlAddress::try_new("https://example.com".to_string())?;
let secure_api = HttpsUrl::try_new("https://api.example.com".to_string())?;

// Patterns
let phone = PhoneNumber::try_new("(555) 123-4567".to_string())?;
let zip = ZipCode::try_new("12345-6789".to_string())?;
let ip = IpAddress::try_new("192.168.1.1".to_string())?;
let id = Uuid::try_new("550e8400-e29b-41d4-a716-446655440000".to_string())?;

// Collections
let tags = NonEmptyVec::try_new(vec!["rust".to_string(), "htmx".to_string()])?;
```

## 📖 See Also

- [Business Rules in Types Example](examples/business_rules_in_types.rs)
- [Route-Specific Types Example](examples/route_specific_types.rs)
- [NUTYPE_INTEGRATION.md](../../NUTYPE_INTEGRATION.md) - Phase 1.5 documentation
- [NUTYPE_TYPES_ANSWER.md](../../NUTYPE_TYPES_ANSWER.md) - Comprehensive Q&A

## 📝 License

MIT
