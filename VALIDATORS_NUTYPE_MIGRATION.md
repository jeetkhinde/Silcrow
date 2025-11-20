# RHTMX Validators Analysis: Nutype vs Macros

## Current State: All 30+ Validators

### ✅ CAN Replace with Nutype Types (Type-Level Validation)

These validators enforce **domain-level constraints** that belong in the type:

#### Email Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[email]` | `EmailAddress` | ✅ Replaced |
| `#[no_public_domains]` | `WorkEmailAddress` | ✅ Replaced |
| `#[blocked_domains(list)]` | Custom nutype | ⚠️ Can create per-app |

**Example:**
```rust
// OLD: Macro approach
#[email]
#[no_public_domains]
email: String

// NEW: Type approach
email: WorkEmailAddress  // Done! ✅
```

#### Password Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[password("basic")]` | `PasswordBasic` | ✅ Replaced |
| `#[password("medium")]` | `PasswordMedium` | ✅ Replaced |
| `#[password("strong")]` | `PasswordStrong` | ✅ Replaced |
| N/A | `SuperStrongPassword` | ✅ New type |
| N/A | `PasswordPhrase3` | ✅ New type |
| N/A | `ModernPassword` | ✅ New type |

#### String Length Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[min_length(n)]` | Custom nutype | ⚠️ Create as needed |
| `#[max_length(n)]` | Custom nutype | ⚠️ Create as needed |
| `#[length(min, max)]` | Custom nutype | ⚠️ Create as needed |

**Example:**
```rust
// Create your own length types
#[nutype(
    validate(len_char_min = 5, len_char_max = 100),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct ProductName(String);

// Use in form
product_name: ProductName  // ✅
```

#### URL Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[url]` | `UrlAddress` | ✅ Replaced |
| N/A | `HttpsUrl` | ✅ New type |

**Created:**
```rust
#[nutype(
    validate(predicate = is_valid_url),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct UrlAddress(String);

#[nutype(
    validate(predicate = is_https_url),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct HttpsUrl(String);
```

#### Numeric Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[min(n)]` | `PositiveInt`, `NonNegativeInt` | ✅ Replaced |
| `#[max(n)]` | Custom nutype | ⚠️ Create as needed |
| `#[range(min, max)]` | `Age`, `Percentage`, `Port` | ✅ Replaced |

**Created:**
```rust
#[nutype(
    validate(greater_or_equal = 18, less_or_equal = 120),
    derive(Debug, Clone, Copy, Serialize, Deserialize)
)]
pub struct Age(i64);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    derive(Debug, Clone, Copy, Serialize, Deserialize)
)]
pub struct Percentage(i64);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 65535),
    derive(Debug, Clone, Copy, Serialize, Deserialize)
)]
pub struct Port(i64);

age: Age  // ✅ 18-120 enforced by type
```

#### String Pattern Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[regex(pattern)]` | `PhoneNumber`, `ZipCode`, `IpAddress`, `Uuid` | ✅ Replaced |
| `#[contains(str)]` | Custom nutype | ⚠️ Create as needed |
| `#[starts_with(str)]` | `HttpsUrl` | ✅ Replaced |
| `#[ends_with(str)]` | Custom nutype | ⚠️ Create as needed |

**Created:**
```rust
#[nutype(
    validate(predicate = is_valid_phone_number),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct PhoneNumber(String);

#[nutype(
    validate(predicate = is_valid_zip_code),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct ZipCode(String);

#[nutype(
    validate(predicate = is_valid_ipv4),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct IpAddress(String);

#[nutype(
    validate(predicate = is_valid_uuid),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct Uuid(String);
```

#### Collection Validators
| Macro | Nutype Type | Status |
|-------|-------------|--------|
| `#[min_items(n)]` | `NonEmptyVec<T>` | ✅ Replaced |
| `#[max_items(n)]` | Custom nutype | ⚠️ Create as needed |
| `#[unique]` | Custom nutype | ⚠️ Create as needed |

**Created:**
```rust
#[nutype(
    validate(predicate = is_non_empty_vec),
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct NonEmptyVec<T>(Vec<T>);

fn is_non_empty_vec<T>(v: &Vec<T>) -> bool {
    !v.is_empty()
}
```

---

### ❌ CANNOT Replace with Nutype (Form-Level Validation)

These validators require **form context** - they need access to other fields or external state:

#### Cross-Field Validators
| Macro | Why Can't Use Nutype? | Keep Macro? |
|-------|----------------------|-------------|
| `#[equals_field = "other"]` | Needs access to another field | ✅ YES |
| `#[depends_on("field", "value")]` | Needs access to another field | ✅ YES |

**Example - MUST keep macros:**
```rust
#[derive(Validate, FormField)]
struct SignupForm {
    password: PasswordStrong,

    #[equals_field = "password"]  // ← MUST keep! Cross-field validation
    confirm_password: PasswordStrong,
}
```

#### External Validators
| Macro | Why Can't Use Nutype? | Keep Macro? |
|-------|----------------------|-------------|
| `#[custom = "fn_name"]` | Calls external function (DB, API) | ✅ YES |

**Example - MUST keep macros:**
```rust
#[derive(Validate, FormField)]
struct UserForm {
    username: Username,  // Type validates format

    #[custom = "check_username_available"]  // ← MUST keep! Database check
    _marker: PhantomData<()>,
}
```

#### Metadata (Not Validation)
| Macro | Purpose | Keep? |
|-------|---------|-------|
| `#[message("...")]` | Custom error message | ✅ YES |
| `#[label("...")]` | Field label | ✅ YES |
| `#[required]` | Field presence | ⚠️ Maybe (see below) |
| `#[allow_whitespace]` | Skip trim check | ✅ YES |

#### Source Markers (Not Validation)
| Macro | Purpose | Keep? |
|-------|---------|-------|
| `#[query]` | Extract from query params | ✅ YES |
| `#[form]` | Extract from form data | ✅ YES |
| `#[path]` | Extract from URL path | ✅ YES |

---

## Migration Strategy

### Phase 2 (Current): Types Available

```rust
// Available NOW:
// Email types
EmailAddress, AnyEmailAddress
WorkEmailAddress
BusinessEmailAddress

// Password types
PasswordBasic, PasswordMedium, PasswordStrong
SuperStrongPassword, PasswordPhrase, PasswordPhrase3, ModernPassword

// String types
Username, NonEmptyString

// Numeric types
PositiveInt, NonNegativeInt
Age, Percentage, Port

// URL types
UrlAddress, HttpsUrl

// Pattern types
PhoneNumber, ZipCode, IpAddress, Uuid

// Collection types
NonEmptyVec<T>
```

### Phase 3 (Future): More Domain Types

**Could add (as needed):**
```rust
// Collections
UniqueVec<T>
BoundedVec<T, MIN, MAX>

// International patterns
InternationalPhoneNumber
PostalCode  // International zip codes

// More URL variants
WebSocketUrl (ws://, wss://)
FtpUrl

// IPv6
Ipv6Address

// Custom domains
SocialSecurityNumber
CreditCardNumber
Iban
```

---

## Answer to Your Questions

### Q1: Which validators are still in the code?

**30+ validators currently in `ValidationAttr` enum:**
- Email: `Email`, `NoPublicDomains`, `BlockedDomains`
- Password: `Password`
- String: `MinLength`, `MaxLength`, `Length`, `Regex`, `Url`, `Contains`, etc.
- Numeric: `Min`, `Max`, `Range`
- Equality: `Equals`, `NotEquals`, `EqualsField`
- Conditional: `DependsOn`
- Collections: `MinItems`, `MaxItems`, `Unique`
- Other: `Custom`, `Required`, `Message`, `Label`, etc.

### Q2: Can we replace them with nutype?

**Yes for ~60%**, **No for ~40%**:

#### ✅ CAN Replace (Type-Level Rules):
- `Email` → `EmailAddress`
- `NoPublicDomains` → `WorkEmailAddress`
- `Password` → `PasswordStrong`, etc.
- `MinLength`, `MaxLength` → Custom nutype
- `Url` → `UrlAddress` (TODO)
- `Min`, `Max`, `Range` → Custom nutype
- `Regex`, `Contains`, `StartsWith` → Custom nutype

#### ❌ CANNOT Replace (Form-Level Logic):
- `EqualsField` → Needs other field access
- `DependsOn` → Needs other field access
- `Custom` → External validation (DB, API)
- `Message`, `Label` → Metadata
- `Query`, `Form`, `Path` → Source markers

### Q3: Will they work for both client and server?

**YES! ✅** Nutype types work everywhere:

**Server:**
```rust
let email = WorkEmailAddress::try_new("user@acme.com".to_string())?;
```

**WASM Client:**
```rust
#[wasm_bindgen]
pub fn validate_work_email(input: String) -> bool {
    WorkEmailAddress::try_new(input).is_ok()
}
```

**Deserialization (both):**
```rust
#[derive(Deserialize)]
struct Form {
    email: WorkEmailAddress,  // ✅ Validates during deserialization
}

let form: Form = serde_json::from_str(json)?;  // Fails if Gmail
```

---

## The Future of Garde

### Current State

**Garde is currently used for:**
1. Custom validators in `/crates/rhtmx-validation/core/src/garde_validators.rs`
2. Returns `garde::Error` types
3. Integration with garde crate (v0.22)

**Files using garde:**
```
crates/rhtmx-validation/core/src/garde_validators.rs  (300 lines)
crates/rhtmx-validation/core/Cargo.toml               (garde dependency)
crates/rhtmx-validation/wasm/Cargo.toml               (garde dependency)
crates/RHTMX-Form/Cargo.toml                          (garde dependency)
```

### Future Path: 3 Options

#### Option 1: Keep Garde for Custom Validators ⚠️

**Pro:**
- Already integrated
- Good error types
- Works with WASM

**Con:**
- Nutype does same thing better
- Extra dependency
- Overlapping functionality

#### Option 2: Phase Out Garde, Use Nutype Only ✅ RECOMMENDED

**Replace:**
```rust
// OLD: garde custom validator
pub fn no_public_email(value: &str, _ctx: &()) -> garde::Result {
    // ...
    Err(garde::Error::new("public email domains not allowed"))
}

// NEW: nutype type
#[nutype(
    validate(predicate = is_work_email),
    derive(...)
)]
pub struct WorkEmailAddress(String);
```

**Benefits:**
- Single approach (nutype for everything)
- Simpler mental model
- Less dependencies
- More type-safe

#### Option 3: Hybrid (Garde for Complex Validation) 🤔

Keep garde for **very complex validation** that nutype can't express:

```rust
// Very complex rule that needs garde's full power
#[garde(inner(custom = complex_validation))]
pub struct ComplexType(Vec<CustomData>);
```

**When needed:**
- Nested validation
- Very complex predicates
- Need garde's derive macros

---

## Recommendation: Migration Plan

### Phase 2 (Done ✅)
- Created nutype types for common cases
- `WorkEmailAddress`, `PasswordStrong`, etc.

### Phase 3 (Next - 2-4 weeks)
1. **Add more nutype types:**
   - `UrlAddress`, `HttpsUrl`
   - `Age`, `Percentage`, `Port`
   - `PhoneNumber`, `ZipCode`, `IpAddress`

2. **Update documentation:**
   - Show nutype types as primary approach
   - Macros as fallback for form-level validation

3. **Mark macros as deprecated:**
   ```rust
   #[deprecated(note = "Use WorkEmailAddress type instead")]
   NoPublicDomains,

   #[deprecated(note = "Use PasswordStrong type instead")]
   Password(String),
   ```

### Phase 4 (Future - 3-6 months)
1. **Remove garde dependency:**
   - Delete `garde_validators.rs`
   - Remove from Cargo.toml
   - Simplify validation core

2. **Keep only form-level macros:**
   - `equals_field`
   - `depends_on`
   - `custom`
   - Metadata (message, label)

---

## Summary Table

| Category | Current Approach | Future Approach | Timeline |
|----------|-----------------|-----------------|----------|
| **Email validation** | Macros + garde | Nutype types | ✅ Done |
| **Password validation** | Macros + garde | Nutype types | ✅ Done |
| **String constraints** | Macros | Nutype types | Phase 3 |
| **Numeric constraints** | Macros | Nutype types | Phase 3 |
| **Cross-field validation** | Macros | Keep macros | Permanent |
| **External validation** | Macros | Keep macros | Permanent |
| **Garde crate** | Used | Remove | Phase 4 |

---

## Final Answer

### Q: Can we replace validators with nutype?
**A: YES for ~60% (type-level rules), NO for ~40% (form-level logic)**

### Q: Will they work for client and server?
**A: YES! ✅ Nutype types are WASM-compatible and work everywhere**

### Q: What's the future of garde?
**A: Phase it out. Nutype does what we need. Garde becomes unnecessary overhead.**

**Recommendation: Go all-in on nutype, deprecate garde over next 3-6 months.**
