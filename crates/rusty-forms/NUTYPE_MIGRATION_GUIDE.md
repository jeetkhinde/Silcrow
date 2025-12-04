# Nutype Migration Guide
**From Proc Macros to Type-Level Validation**

This guide shows you how to replace proc macro validation attributes with Nutype validated types for better type safety, compile-time guarantees, and reusability.

---

## Table of Contents

1. [Why Migrate to Nutype?](#1-why-migrate-to-nutype)
2. [Quick Start Examples](#2-quick-start-examples)
3. [Complete Migration Patterns](#3-complete-migration-patterns)
4. [Custom Nutype Validators](#4-custom-nutype-validators)
5. [What Cannot Be Migrated](#5-what-cannot-be-migrated)
6. [Migration Checklist](#6-migration-checklist)

---

## 1. Why Migrate to Nutype?

### Benefits of Nutype

| Proc Macro Validation | Nutype Validation |
|----------------------|------------------|
| ⏱️ **Runtime validation** - Checked when `.validate()` is called | ✅ **Compile-time validation** - Checked at construction |
| 🔄 **Per-form validation** - Rules repeated across forms | ✅ **Reusable types** - Define once, use everywhere |
| 💥 **Late error discovery** - Errors found during validation | ✅ **Early error discovery** - Errors at `.try_new()` |
| 📦 **Type is String** - No semantic meaning | ✅ **Type is constraint** - Type system enforces rules |
| 🐌 **Slower compile times** - Proc macros are expensive | ✅ **Faster compilation** - Less proc macro expansion |

### Example: Type Safety

**Proc Macro Approach:**
```rust
#[derive(Validate)]
struct User {
    #[email]
    email: String,  // Can be assigned any String!
}

// This compiles but is wrong:
let user = User {
    email: "not-an-email".to_string()  // ❌ No compile error
};

// Error only found at validation time:
user.validate()?;  // ❌ Runtime error
```

**Nutype Approach:**
```rust
use rusty_forms_types::EmailAddress;

struct User {
    email: EmailAddress,  // Can ONLY be a valid email!
}

// This won't compile:
let user = User {
    email: "not-an-email".to_string()  // ❌ Compile error: type mismatch
};

// Must construct safely:
let email = EmailAddress::try_new("user@example.com".to_string())?;  // ✅ Validated here
let user = User { email };  // ✅ Guaranteed valid
```

**Result:** Invalid data can never exist in your system!

---

## 2. Quick Start Examples

### Example 1: Email Validation

**Before (Proc Macro):**
```rust
use rusty_forms::Validate;

#[derive(Validate)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    email: String,
}
```

**After (Nutype):**
```rust
use rusty_forms_types::WorkEmailAddress;

struct RegisterForm {
    email: WorkEmailAddress,  // Type = Business rule!
}

// Construction:
let form = RegisterForm {
    email: WorkEmailAddress::try_new("user@company.com".to_string())
        .map_err(|e| format!("Invalid work email: {:?}", e))?
};
```

**Explanation:**
- `WorkEmailAddress` already validates format + blocks public domains
- No need for proc macro attributes
- Type system prevents invalid emails

---

### Example 2: Password Strength

**Before (Proc Macro):**
```rust
#[derive(Validate)]
struct LoginForm {
    #[password("strong")]
    password: String,
}
```

**After (Nutype):**
```rust
use rusty_forms_types::PasswordStrong;

struct LoginForm {
    password: PasswordStrong,
}

// Construction:
let form = LoginForm {
    password: PasswordStrong::try_new("MyP@ssw0rd123!".to_string())?
};
```

**Available password types:**
- `PasswordBasic` - 6+ characters
- `PasswordMedium` - 8+ characters
- `PasswordStrong` - 10+ chars + complexity
- `SuperStrongPassword` - 12+ chars + 2 special
- `PasswordPhrase` - 15+ characters
- `ModernPassword` - 16+ characters (NIST 2024)

---

### Example 3: Numeric Ranges

**Before (Proc Macro):**
```rust
#[derive(Validate)]
struct ProfileForm {
    #[min(18)]
    #[max(120)]
    age: i32,
}
```

**After (Nutype):**
```rust
use rusty_forms_types::Age;

struct ProfileForm {
    age: Age,  // Already constrained to 18-120
}

// Construction:
let form = ProfileForm {
    age: Age::try_from(25)?
};
```

---

### Example 4: String Length

**Before (Proc Macro):**
```rust
#[derive(Validate)]
struct AccountForm {
    #[min_length(3)]
    #[max_length(30)]
    username: String,
}
```

**After (Nutype):**
```rust
use rusty_forms_types::Username;

struct AccountForm {
    username: Username,  // 3-30 chars, alphanumeric + _ -
}

// Construction:
let form = AccountForm {
    username: Username::try_new("john_doe".to_string())?
};
```

---

### Example 5: URL Validation

**Before (Proc Macro):**
```rust
#[derive(Validate)]
struct WebsiteForm {
    #[url]
    homepage: String,
}
```

**After (Nutype):**
```rust
use rusty_forms_types::UrlAddress;

struct WebsiteForm {
    homepage: UrlAddress,
}

// Or for HTTPS-only:
use rusty_forms_types::HttpsUrl;

struct SecureWebsiteForm {
    homepage: HttpsUrl,  // Only accepts https:// URLs
}
```

---

## 3. Complete Migration Patterns

### Pattern 1: Multiple Validations → Single Type

**Before:**
```rust
#[derive(Validate)]
struct BusinessContact {
    #[email]
    #[no_public_domains]
    #[blocked_domains("competitor.com", "spam.com")]
    #[required]
    email: Option<String>,
}
```

**After:**
```rust
use rusty_forms_types::WorkEmailAddress;

struct BusinessContact {
    email: WorkEmailAddress,  // All rules in one type!
}

// Note: No Option needed - type itself guarantees validity
// If email is optional, use Option<WorkEmailAddress>
```

**For custom blocked domains, create a custom type:**
```rust
#[nutype(
    validate(predicate = is_valid_business_email),
    derive(Debug, Clone, Serialize, Deserialize, TryFrom, Into, Deref)
)]
pub struct BusinessEmail(String);

fn is_valid_business_email(s: &str) -> bool {
    is_work_email(s) && !is_competitor_domain(s)
}

fn is_competitor_domain(email: &str) -> bool {
    let domain = email.split('@').nth(1).unwrap_or("");
    matches!(domain, "competitor.com" | "spam.com")
}
```

---

### Pattern 2: Form with Mixed Validations

**Before:**
```rust
#[derive(Validate)]
struct CompleteProfile {
    #[email]
    email: String,

    #[password("strong")]
    password: String,

    #[min_length(3)]
    #[max_length(30)]
    username: String,

    #[min(18)]
    #[max(120)]
    age: i32,

    #[url]
    website: String,
}
```

**After:**
```rust
use rusty_forms_types::{
    EmailAddress,
    PasswordStrong,
    Username,
    Age,
    UrlAddress,
};

struct CompleteProfile {
    email: EmailAddress,
    password: PasswordStrong,
    username: Username,
    age: Age,
    website: UrlAddress,
}

// Construction from raw data:
impl CompleteProfile {
    pub fn from_raw(
        email: String,
        password: String,
        username: String,
        age: i32,
        website: String,
    ) -> Result<Self, ProfileError> {
        Ok(Self {
            email: EmailAddress::try_new(email)
                .map_err(|_| ProfileError::InvalidEmail)?,
            password: PasswordStrong::try_new(password)
                .map_err(|_| ProfileError::WeakPassword)?,
            username: Username::try_new(username)
                .map_err(|_| ProfileError::InvalidUsername)?,
            age: Age::try_from(age)
                .map_err(|_| ProfileError::InvalidAge)?,
            website: UrlAddress::try_new(website)
                .map_err(|_| ProfileError::InvalidUrl)?,
        })
    }
}

#[derive(Debug)]
enum ProfileError {
    InvalidEmail,
    WeakPassword,
    InvalidUsername,
    InvalidAge,
    InvalidUrl,
}
```

---

### Pattern 3: Optional Fields

**Before:**
```rust
#[derive(Validate)]
struct UserProfile {
    #[email]
    #[required]
    email: Option<String>,

    #[url]
    website: Option<String>,  // Not required
}
```

**After:**
```rust
use rusty_forms_types::{EmailAddress, UrlAddress};

struct UserProfile {
    email: EmailAddress,  // Required - no Option
    website: Option<UrlAddress>,  // Optional
}
```

**Explanation:**
- If field is required, don't use `Option<T>` at all - just use the validated type
- If field is optional, use `Option<ValidatedType>`
- The type itself guarantees validity when present

---

## 4. Custom Nutype Validators

### Pattern 1: String Pattern Matching

**Goal:** Create a type for SKU codes (format: ABC-1234)

```rust
use nutype::nutype;

#[nutype(
    validate(predicate = is_valid_sku),
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
pub struct SkuCode(String);

fn is_valid_sku(s: &str) -> bool {
    if s.len() != 8 {
        return false;
    }
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let (prefix, number) = (parts[0], parts[1]);

    prefix.len() == 3
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && number.len() == 4
        && number.chars().all(|c| c.is_ascii_digit())
}

// Usage:
let sku = SkuCode::try_new("ABC-1234".to_string())?;  // ✅
let bad = SkuCode::try_new("invalid".to_string());     // ❌ Error
```

---

### Pattern 2: Combining Multiple Constraints

**Goal:** Create a product price type (0.01 to 999999.99)

```rust
use nutype::nutype;

#[nutype(
    validate(
        greater_or_equal = 0.01,
        less_or_equal = 999999.99,
        predicate = has_max_two_decimals
    ),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct ProductPrice(f64);

fn has_max_two_decimals(f: &f64) -> bool {
    let rounded = (f * 100.0).round() / 100.0;
    (f - rounded).abs() < 0.0001
}

// Usage:
let price = ProductPrice::try_from(19.99)?;  // ✅
let bad1 = ProductPrice::try_from(0.0)?;     // ❌ Too low
let bad2 = ProductPrice::try_from(19.999)?;  // ❌ Too many decimals
```

---

### Pattern 3: Regex-Based Validation

**Goal:** Create a type for US Social Security Numbers (XXX-XX-XXXX)

```rust
use nutype::nutype;
use regex::Regex;
use once_cell::sync::Lazy;

static SSN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{3}-\d{2}-\d{4}$").unwrap()
});

#[nutype(
    validate(predicate = is_valid_ssn),
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
pub struct SocialSecurityNumber(String);

fn is_valid_ssn(s: &str) -> bool {
    SSN_REGEX.is_match(s)
}

// Usage:
let ssn = SocialSecurityNumber::try_new("123-45-6789".to_string())?;  // ✅
let bad = SocialSecurityNumber::try_new("12345678".to_string());       // ❌
```

---

### Pattern 4: Business Rule Validation

**Goal:** Promo codes (6-12 uppercase letters/numbers, must start with letter)

```rust
#[nutype(
    validate(
        len_char_min = 6,
        len_char_max = 12,
        predicate = is_valid_promo_code
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
pub struct PromoCode(String);

fn is_valid_promo_code(s: &str) -> bool {
    // Must start with letter
    let first = s.chars().next();
    if !first.map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        return false;
    }

    // All chars must be uppercase alphanumeric
    s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

// Usage:
let promo = PromoCode::try_new("SAVE20".to_string())?;   // ✅
let bad1 = PromoCode::try_new("save20".to_string())?;    // ❌ Lowercase
let bad2 = PromoCode::try_new("123ABC".to_string())?;    // ❌ Starts with digit
```

---

## 5. What Cannot Be Migrated

### Structural Validations (Keep Proc Macros)

#### 1. Field Comparisons

**Cannot migrate:**
```rust
#[derive(Validate)]
struct PasswordChangeForm {
    new_password: String,

    #[equals_field("new_password")]  // ← Requires runtime field access
    confirm_password: String,
}
```

**Why:** Nutype validates a single value in isolation. It cannot access other fields.

**Solution:** Keep using proc macro for this validation.

---

#### 2. Conditional Validation

**Cannot migrate:**
```rust
#[derive(Validate)]
struct AddressForm {
    country: String,

    #[depends_on("country", "US")]  // ← Conditional requirement
    state: Option<String>,
}
```

**Why:** Validation depends on another field's value at runtime.

**Solution:** Keep using proc macro or implement as custom validator.

---

#### 3. Collection-Level Validation

**Cannot migrate (fully):**
```rust
#[derive(Validate)]
struct TagForm {
    #[min_items(1)]
    #[max_items(10)]
    #[unique]  // ← Collection-level constraint
    tags: Vec<String>,
}
```

**Partial solution:**
```rust
use rusty_forms_types::NonEmptyVec;

struct TagForm {
    tags: NonEmptyVec<String>,  // ✅ Handles min_items(1)
    // ❌ Still need proc macro for max_items and unique
}
```

---

### Hybrid Approach (Best Practice)

Combine Nutype for type-level validation + proc macros for structural validation:

```rust
use rusty_forms::{Validate, FormField};
use rusty_forms_types::PasswordStrong;

#[derive(Validate, FormField)]
struct CompleteForm {
    // ✅ Type-level validation (Nutype)
    password: PasswordStrong,
    confirm: PasswordStrong,

    // ✅ Structural validation (Proc macro)
    #[equals_field("password")]
    confirm: PasswordStrong,
}
```

**Note:** Mark Nutype fields with `#[nutype]` to skip duplicate validation:

```rust
#[derive(Validate)]
struct Form {
    #[nutype]  // ← Skip proc macro validation for this field
    #[equals_field("password")]  // ← But keep structural validation
    confirm: PasswordStrong,
}
```

---

## 6. Migration Checklist

### Step 1: Identify Candidates

Review your forms and identify fields using these attributes:

**Easy migrations (do first):**
- [ ] `#[email]` → `EmailAddress` or `WorkEmailAddress`
- [ ] `#[password("...")]` → `PasswordStrong`, `PasswordMedium`, etc.
- [ ] `#[url]` → `UrlAddress` or `HttpsUrl`
- [ ] `#[min(n)] #[max(m)]` → Custom numeric type or `Age`, `Percentage`, etc.
- [ ] `#[min_length(n)] #[max_length(m)]` → `Username` or custom string type

**Medium migrations (do second):**
- [ ] `#[contains("...")]` → Custom type with predicate
- [ ] `#[starts_with("...")]` → Custom type with predicate
- [ ] `#[regex(r"...")]` → Custom type with regex predicate
- [ ] `#[blocked_domains([...])]` → Custom email type

**Keep as proc macros:**
- [ ] `#[equals_field("...")]` - Field comparison
- [ ] `#[depends_on("...", "...")]` - Conditional validation
- [ ] `#[required]` on `Option<T>` - Runtime requirement
- [ ] `#[unique]` on collections - Collection validation
- [ ] `#[label("...")]`, `#[message("...")]` - Metadata

---

### Step 2: Create Custom Types

For each validation pattern, create a Nutype:

```rust
// Add to rusty-forms-types/src/lib.rs or your own types module

#[nutype(
    validate(/* your constraints */),
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
pub struct YourType(String);  // or i64, f64, etc.
```

---

### Step 3: Update Forms

Replace `String`/primitive types with Nutype:

**Before:**
```rust
struct Form {
    #[email]
    email: String,
}
```

**After:**
```rust
struct Form {
    email: EmailAddress,
}
```

---

### Step 4: Update Construction Code

Add `.try_new()` or `try_from()` calls:

**Before:**
```rust
let form = Form {
    email: "user@example.com".to_string(),
};
form.validate()?;
```

**After:**
```rust
let form = Form {
    email: EmailAddress::try_new("user@example.com".to_string())?,
};
// No need for .validate() - already validated!
```

---

### Step 5: Handle Errors

Update error handling:

**Before:**
```rust
match form.validate() {
    Ok(_) => println!("Valid"),
    Err(errors) => {
        for (field, msgs) in errors {
            println!("{}: {:?}", field, msgs);
        }
    }
}
```

**After:**
```rust
match EmailAddress::try_new(input) {
    Ok(email) => {
        let form = Form { email };
        println!("Valid");
    }
    Err(e) => {
        println!("Invalid email: {:?}", e);
    }
}
```

---

### Step 6: Add `#[nutype]` Marker (Optional)

If you still use `#[derive(Validate)]` for structural validation, mark Nutype fields:

```rust
#[derive(Validate)]
struct Form {
    #[nutype]  // ← Skip duplicate validation
    #[equals_field("other")]  // ← Keep structural validation
    field: MyNutypeType,
}
```

---

## 7. Real-World Example: Complete Migration

### Before: Proc Macro Heavy

```rust
use rusty_forms::{Validate, FormField};

#[derive(Validate, FormField, Deserialize)]
struct UserRegistration {
    #[email]
    #[no_public_domains]
    #[label("Work Email")]
    email: String,

    #[password("strong")]
    #[label("Password")]
    password: String,

    #[equals_field("password")]
    #[label("Confirm Password")]
    password_confirm: String,

    #[min_length(3)]
    #[max_length(30)]
    #[regex(r"^[a-zA-Z0-9_-]+$")]
    #[label("Username")]
    username: String,

    #[min(18)]
    #[max(120)]
    #[label("Age")]
    age: i32,

    #[url]
    #[label("Website")]
    website: Option<String>,
}

// Handler
async fn register(Form(data): Form<UserRegistration>) -> Result<(), HashMap<String, Vec<String>>> {
    data.validate()?;  // Runtime validation
    // ... save to database
    Ok(())
}
```

---

### After: Nutype First

```rust
use rusty_forms::{Validate, FormField};
use rusty_forms_types::{
    WorkEmailAddress,
    PasswordStrong,
    Username,
    Age,
    UrlAddress,
};

#[derive(Validate, FormField, Deserialize)]
#[serde(try_from = "RawRegistration")]
struct UserRegistration {
    #[nutype]
    #[label("Work Email")]
    email: WorkEmailAddress,

    #[nutype]
    #[label("Password")]
    password: PasswordStrong,

    #[nutype]
    #[equals_field("password")]  // ← Still need this for comparison
    #[label("Confirm Password")]
    password_confirm: PasswordStrong,

    #[nutype]
    #[label("Username")]
    username: Username,

    #[nutype]
    #[label("Age")]
    age: Age,

    #[nutype]
    #[label("Website")]
    website: Option<UrlAddress>,
}

// Deserialize from raw strings
#[derive(Deserialize)]
struct RawRegistration {
    email: String,
    password: String,
    password_confirm: String,
    username: String,
    age: i32,
    website: Option<String>,
}

impl TryFrom<RawRegistration> for UserRegistration {
    type Error = RegistrationError;

    fn try_from(raw: RawRegistration) -> Result<Self, Self::Error> {
        Ok(Self {
            email: WorkEmailAddress::try_new(raw.email)
                .map_err(|_| RegistrationError::InvalidEmail)?,
            password: PasswordStrong::try_new(raw.password)
                .map_err(|_| RegistrationError::WeakPassword)?,
            password_confirm: PasswordStrong::try_new(raw.password_confirm)
                .map_err(|_| RegistrationError::WeakPassword)?,
            username: Username::try_new(raw.username)
                .map_err(|_| RegistrationError::InvalidUsername)?,
            age: Age::try_from(raw.age)
                .map_err(|_| RegistrationError::InvalidAge)?,
            website: raw.website
                .map(|w| UrlAddress::try_new(w))
                .transpose()
                .map_err(|_| RegistrationError::InvalidWebsite)?,
        })
    }
}

#[derive(Debug)]
enum RegistrationError {
    InvalidEmail,
    WeakPassword,
    InvalidUsername,
    InvalidAge,
    InvalidWebsite,
}

// Handler
async fn register(Form(data): Form<UserRegistration>) -> Result<(), HashMap<String, Vec<String>>> {
    data.validate()?;  // Only validates structural rules (equals_field)
    // All type-level validation already done during deserialization!
    // ... save to database
    Ok(())
}
```

**Benefits:**
- ✅ Most validation happens at construction (deserialization)
- ✅ Only structural validation (field comparisons) happens at `.validate()`
- ✅ Type system prevents invalid data from existing
- ✅ Better error messages (per-field errors)
- ✅ Reusable types across multiple forms

---

## 8. Tips & Best Practices

### Tip 1: Start Small

Don't migrate everything at once. Start with:
1. Most common types (Email, Password, Username)
2. Forms with simple validation
3. New code (avoid breaking changes in existing code)

---

### Tip 2: Create a Types Module

Organize your Nutype definitions:

```rust
// src/types/mod.rs
mod email;
mod password;
mod username;
mod business;

pub use email::{EmailAddress, WorkEmailAddress, BusinessEmailAddress};
pub use password::{PasswordStrong, PasswordMedium};
pub use username::Username;
pub use business::{SkuCode, PromoCode, ProductPrice};
```

---

### Tip 3: Document Business Rules

Use doc comments to explain types:

```rust
/// Work email address for B2B registration.
///
/// **Business Rules:**
/// - Valid email format (RFC 5322)
/// - No public domains (Gmail, Yahoo, etc.)
/// - No disposable email services
///
/// **Use when:** Registering for business/enterprise accounts
#[nutype(
    validate(predicate = is_work_email),
    derive(/* ... */)
)]
pub struct WorkEmailAddress(String);
```

---

### Tip 4: Provide Helpers

Add convenience methods:

```rust
impl WorkEmailAddress {
    /// Extract domain part of email
    pub fn domain(&self) -> &str {
        self.as_ref()
            .split('@')
            .nth(1)
            .unwrap_or("")
    }

    /// Check if email is from a specific domain
    pub fn is_from_domain(&self, domain: &str) -> bool {
        self.domain().eq_ignore_ascii_case(domain)
    }
}

// Usage:
let email = WorkEmailAddress::try_new("john@acme.com".to_string())?;
assert_eq!(email.domain(), "acme.com");
assert!(email.is_from_domain("ACME.COM"));
```

---

### Tip 5: Test Your Types

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_sku_codes() {
        assert!(SkuCode::try_new("ABC-1234".to_string()).is_ok());
        assert!(SkuCode::try_new("XYZ-9999".to_string()).is_ok());
    }

    #[test]
    fn invalid_sku_codes() {
        assert!(SkuCode::try_new("abc-1234".to_string()).is_err());  // Lowercase
        assert!(SkuCode::try_new("AB-1234".to_string()).is_err());   // Too short
        assert!(SkuCode::try_new("ABC-12345".to_string()).is_err()); // Too long
        assert!(SkuCode::try_new("ABC1234".to_string()).is_err());   // No dash
    }
}
```

---

## 9. Summary

**Migration Priority:**

1. ✅ **High Value**: Email, Password, Username, URL (use built-in types)
2. ✅ **Medium Value**: Numeric ranges, custom string patterns (create custom types)
3. ❌ **Not Worth It**: Field comparisons, conditional validation (keep proc macros)

**Expected Results:**
- 60% fewer proc macro invocations
- Faster compile times
- Better type safety
- More maintainable code
- Reusable validation types

**Next Steps:**
1. Review your forms
2. Identify migration candidates
3. Create custom Nutype definitions
4. Update forms incrementally
5. Test thoroughly

---

Happy migrating! 🚀
