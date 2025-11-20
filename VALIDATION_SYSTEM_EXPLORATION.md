# RHTMX-Forms Validation System - Comprehensive Exploration

## Executive Summary

The RHTMX validation system implements a **single source of truth** pattern where Rust struct attributes define validation rules that automatically generate:
1. **Server-side validation code** (procedural macro)
2. **HTML5 validation attributes** (type, required, minlength, etc.)
3. **Client-side data-validate JSON** (for WASM validation)

This eliminates DRY violations and keeps client/server validation synchronized at compile time.

---

## 1. Current Validation Implementation Structure

### 1.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│              Rust Struct with Attributes                    │
│         (Single Source of Truth)                            │
│  #[derive(Validate, FormField)]                             │
│  struct MyForm {                                            │
│      #[email]                                               │
│      #[no_public_domains]                                  │
│      email: String,                                        │
│  }                                                          │
└─────────────────┬───────────────────────────────────────────┘
                  │
        ┌─────────┴──────────┬─────────────────┐
        │                    │                 │
        ▼                    ▼                 ▼
    ┌──────────┐      ┌────────────┐   ┌─────────────┐
    │  Server  │      │   HTML5    │   │    WASM     │
    │Validation│      │ Attributes │   │ data-validate
    │   Code   │      │            │   │     JSON    │
    └──────────┘      └────────────┘   └─────────────┘
```

### 1.2 Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| **Validate Macro** | `/crates/RHTMX-Form/src/lib.rs` | Entry point for `#[derive(Validate)]` |
| **FormField Macro** | `/crates/RHTMX-Form/src/lib.rs` | Entry point for `#[derive(FormField)]` |
| **Validation Engine** | `/crates/RHTMX-Form/src/validation.rs` | Implements both macros (1025 lines) |
| **Validation Trait** | `/crates/rhtmx/src/validation/mod.rs` | Runtime trait definition |
| **Validators** | `/crates/rhtmx/src/validation/validators.rs` | Validator function implementations |
| **Form Field Types** | `/crates/rhtmx/src/form_field.rs` | FieldAttrs struct & FormField trait |
| **Validation Core** | `/crates/rhtmx-validation/core/src/` | Shared no_std validation logic |
| **WASM Bridge** | `/crates/rhtmx-validation/wasm/src/lib.rs` | WebAssembly bindings |

### 1.3 Validation Attribute Lifecycle

```
COMPILE TIME:
1. Parser extracts attributes from struct fields
   - #[email], #[min_length(8)], etc.

2. ValidationAttr enum captures each attribute
   - enum ValidationAttr { Email, MinLength(usize), ... }

3. Code generation produces:
   a) Validation implementation (server-side)
   b) HTML5 attribute mappings
   c) data-validate JSON objects

RUNTIME:
4. Server receives request → calls form.validate()
   - Returns HashMap<String, String> (field → error message)

5. Template code calls form.field_attrs("field_name")
   - Returns FieldAttrs { html5_attrs, data_validate, label }

6. HTML renders with both HTML5 and data-validate attributes
   - type="email" required data-validate='{"email":true,"required":true}'
```

---

## 2. FormField Macro: HTML5 Attribute Generation

### 2.1 How FormField Works

**File:** `/crates/RHTMX-Form/src/validation.rs` lines 940-1024

The `impl_form_field()` function:

```rust
pub fn impl_form_field(input: &DeriveInput) -> TokenStream {
    // Extract all fields from struct
    // For each field:
    //   1. Extract validation attributes
    //   2. Convert to HTML5 attributes (validation_to_html5_attrs)
    //   3. Convert to JSON (validation_to_json)
    //   4. Generate match arm for field_attrs() method
    
    quote! {
        impl rhtmx::FormField for #name {
            fn field_attrs(&self, field_name: &str) -> rhtmx::FieldAttrs {
                match field_name {
                    // Each field has a match arm returning FieldAttrs
                }
            }
            fn field_names(&self) -> Vec<&'static str> { /* ... */ }
        }
    }
}
```

### 2.2 HTML5 Attribute Mapping

**Function:** `validation_to_html5_attrs()` (lines 802-837)

Maps validation attributes to HTML5:

| Validation Attribute | HTML5 Attribute | Generated Code |
|---------------------|-----------------|---|
| `#[email]` | `type="email"` | `attrs.insert("type", "email")` |
| `#[required]` | `required` | `attrs.insert("required", "")` |
| `#[min_length(8)]` | `minlength="8"` | `attrs.insert("minlength", "8")` |
| `#[max_length(100)]` | `maxlength="100"` | `attrs.insert("maxlength", "100")` |
| `#[min(18)]` | `min="18"` | `attrs.insert("min", "18")` |
| `#[max(120)]` | `max="120"` | `attrs.insert("max", "120")` |
| `#[url]` | `type="url"` | `attrs.insert("type", "url")` |
| `#[regex(pattern)]` | `pattern="pattern"` | `attrs.insert("pattern", "pattern")` |

**Note:** Only HTML5-compatible validators generate HTML5 attributes. Custom validators like `#[password("strong")]`, `#[no_public_domains]`, etc. only appear in `data-validate` JSON.

### 2.3 Data-Validate JSON Generation

**Function:** `validation_to_json()` (lines 839-938)

Creates JSON objects for client-side WASM validation:

```rust
// Input: #[email] #[no_public_domains] #[required]
// Output: {"email":true,"noPublicDomains":true,"required":true}

for validation in validations {
    match validation {
        ValidationAttr::Email => json_parts.push(r#""email": true"#),
        ValidationAttr::NoPublicDomains => json_parts.push(r#""noPublicDomains": true"#),
        ValidationAttr::MinLength(n) => json_parts.push(format!(r#""minLength": {}"#, n)),
        // ... etc
    }
}
```

### 2.4 FieldAttrs Structure

**File:** `/crates/rhtmx/src/form_field.rs` lines 6-64

```rust
pub struct FieldAttrs {
    /// HTML5 native attributes (HashMap<String, String>)
    pub html5_attrs: HashMap<String, String>,
    
    /// JSON string for data-validate attribute
    pub data_validate: String,
    
    /// Field label for display
    pub label: String,
}

impl FieldAttrs {
    pub fn render_html5_attrs(&self) -> String;
    pub fn render_data_validate(&self) -> String;
    pub fn render_all(&self) -> String;  // Combined output
}
```

### 2.5 Complete Example

**Input struct:**
```rust
#[derive(Validate, FormField)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    #[label("Email Address")]
    email: String,
}
```

**Generated code (conceptually):**
```rust
impl FormField for RegisterForm {
    fn field_attrs(&self, field_name: &str) -> FieldAttrs {
        match field_name {
            "email" => {
                let mut attrs = HashMap::new();
                attrs.insert("type", "email");
                attrs.insert("required", "");
                
                FieldAttrs {
                    html5_attrs: attrs,
                    data_validate: r#"{"email":true,"noPublicDomains":true,"required":true}"#,
                    label: "Email Address".to_string(),
                }
            }
            _ => FieldAttrs::default(),
        }
    }
    
    fn field_names(&self) -> Vec<&'static str> {
        vec!["email"]
    }
}
```

**Rendered HTML:**
```html
<input
    name="email"
    type="email"
    required
    data-validate='{"email":true,"noPublicDomains":true,"required":true}'
/>
```

---

## 3. WASM Bridge Implementation for Client-Side Validation

### 3.1 Architecture

**File:** `/crates/rhtmx-validation/wasm/src/lib.rs`

Two-tier architecture:

```
JavaScript Code
    ↓
wasm-bindgen Exports (pub fn validate_field)
    ↓
FieldRules Struct (deserialized from JS)
    ↓
Validation Logic (from rhtmx_validation_core)
    ↓
ValidationError Struct (serialized back to JS)
```

### 3.2 WASM Exports

#### `validateField()` - Main Validation Function

**Lines 93-249**

```rust
#[wasm_bindgen(js_name = validateField)]
pub fn validate_field(
    field_name: &str,
    value: &str,
    rules: JsValue,  // JavaScript object with validation rules
) -> Result<JsValue, JsValue> {
    // 1. Deserialize JavaScript rules to FieldRules struct
    let rules: FieldRules = serde_wasm_bindgen::from_value(rules)?;
    
    // 2. Create empty errors vector
    let mut errors = Vec::new();
    
    // 3. Apply validations in sequence
    if rules.required && value.trim().is_empty() {
        errors.push(ValidationError { /* ... */ });
        return Ok(serde_wasm_bindgen::to_value(&errors)?);
    }
    
    if rules.email && !core::is_valid_email(value) {
        errors.push(ValidationError { /* ... */ });
    }
    
    if rules.no_public_domains && core::is_public_domain(value) {
        errors.push(ValidationError { /* ... */ });
    }
    
    // ... more validators ...
    
    // 4. Serialize errors back to JavaScript
    Ok(serde_wasm_bindgen::to_value(&errors)?)
}
```

**JavaScript interface:**
```javascript
// Call from JavaScript
const errors = await validateField('email', 'user@example.com', {
    email: true,
    noPublicDomains: true,
    required: true
});

// Returns:
// []  (if valid)
// [{ field: "email", message: "Invalid email..." }]  (if invalid)
```

#### `isValidEmail()` - Quick Email Check
```rust
#[wasm_bindgen(js_name = isValidEmail)]
pub fn is_valid_email_js(email: &str) -> bool {
    core::is_valid_email(email)
}
```

#### `validatePassword()` - Password Strength Check
```rust
#[wasm_bindgen(js_name = validatePassword)]
pub fn validate_password_js(password: &str, pattern: &str) -> Option<String> {
    core::validate_password(password, pattern).err()
}
```

### 3.3 FieldRules Structure

**Lines 24-73**

Maps JavaScript validation rules to Rust struct using serde:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]  // Converts camelCase to snake_case
pub struct FieldRules {
    pub email: bool,                           // #[email]
    pub no_public_domains: bool,               // #[no_public_domains]
    pub blocked_domains: Option<Vec<String>>,  // #[blocked_domains("a.com")]
    pub password: Option<String>,              // #[password("strong")]
    pub min_length: Option<usize>,             // #[min_length(8)]
    pub max_length: Option<usize>,             // #[max_length(100)]
    pub contains: Option<String>,              // #[contains("text")]
    pub not_contains: Option<String>,          // #[not_contains("text")]
    pub starts_with: Option<String>,           // #[starts_with("prefix")]
    pub ends_with: Option<String>,             // #[ends_with("suffix")]
    pub equals: Option<String>,                // #[equals("value")]
    pub not_equals: Option<String>,            // #[not_equals("value")]
    pub url: bool,                             // #[url]
    pub required: bool,                        // #[required]
    pub message: Option<String>,               // Custom error message
}
```

### 3.4 Shared Validation Core

**Directory:** `/crates/rhtmx-validation/core/src/`

Pure Rust validation functions (no dependencies, no_std compatible):

```
core/
├── lib.rs           # Main library file
├── email.rs         # Email validators
├── password.rs      # Password strength validators
├── string.rs        # String validators
├── numeric.rs       # Numeric validators
└── collection.rs    # Collection validators
```

**Key feature:** Single codebase used by both:
1. **Server:** `rhtmx/src/validation/validators.rs` (re-exports core)
2. **WASM:** `rhtmx-validation/wasm/src/lib.rs` (imports core)

This guarantees identical validation logic on client and server.

### 3.5 Build Process

**WASM compilation:**
```bash
# Compile Rust to WebAssembly
wasm-pack build --target web --release

# Generates:
# - rhtmx_validation_wasm_bg.wasm (binary)
# - rhtmx_validation_wasm.js (JavaScript bindings)
# - rhtmx_validation_wasm.d.ts (TypeScript types)
```

**Size:** ~35KB gzipped (very efficient compared to JavaScript validators)

---

## 4. Custom Validators: Email Domain Logic & Blocked Domains

### 4.1 Email Validation Implementation

**File:** `/crates/rhtmx-validation/core/src/email.rs`

#### Basic Email Format Validation

**Function:** `is_valid_email()` (lines 26-94)

```rust
pub fn is_valid_email(email: &str) -> bool {
    // Check minimum length (3 chars: "a@b")
    if email.len() < 3 { return false; }
    
    // Parse: split('@') to get [local, domain]
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 { return false; }  // Must have exactly 1 '@'
    
    let local = parts[0];   // Part before '@'
    let domain = parts[1];  // Part after '@'
    
    // Local part: 1-64 characters
    if local.is_empty() || local.len() > 64 { return false; }
    
    // Domain part: 1-255 characters
    if domain.is_empty() || domain.len() > 255 { return false; }
    
    // Domain must have at least one dot
    if !domain.contains('.') { return false; }
    
    // Domain can't start/end with dot or hyphen
    if domain.starts_with('.') || domain.ends_with('.')
        || domain.starts_with('-') || domain.ends_with('-') {
        return false;
    }
    
    // No consecutive dots
    if domain.contains("..") { return false; }
    
    // Valid characters in local part: alphanumeric, dot, underscore, hyphen, plus
    let valid_local_chars = |c: char| {
        c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+'
    };
    if !local.chars().all(valid_local_chars) { return false; }
    
    // Valid characters in domain: alphanumeric, dot, hyphen
    let valid_domain_chars = |c: char| {
        c.is_alphanumeric() || c == '.' || c == '-'
    };
    if !domain.chars().all(valid_domain_chars) { return false; }
    
    // TLD (part after last dot) must be 2+ characters
    if let Some(last_dot_pos) = domain.rfind('.') {
        let tld = &domain[last_dot_pos + 1..];
        if tld.len() < 2 { return false; }
    }
    
    true
}
```

**Validation checks:**
- ✅ Format: `local@domain.tld`
- ✅ Local part: 1-64 chars, alphanumeric + `.`, `_`, `-`, `+`
- ✅ Domain: 1-255 chars, alphanumeric + `.`, `-`
- ✅ No consecutive dots
- ✅ TLD minimum 2 characters
- ✅ No leading/trailing dots or hyphens

#### Public Domain Detection

**Function:** `is_public_domain()` (lines 96-103)

```rust
const PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com",
    "yahoo.com",
    "hotmail.com",
    "outlook.com",
    "icloud.com",
    "aol.com",
    "mail.com",
    "protonmail.com",
    "yandex.com",
    "zoho.com",
];

pub fn is_public_domain(email: &str) -> bool {
    // Extract domain from email
    if let Some(domain) = email.split('@').nth(1) {
        // Case-insensitive comparison
        PUBLIC_DOMAINS.iter().any(|&d| d.eq_ignore_ascii_case(domain))
    } else {
        false
    }
}
```

**Checks:** Domain is in the hardcoded list of public providers (case-insensitive)

#### Blocked Domain List

**Function:** `is_blocked_domain()` (lines 105-112)

```rust
pub fn is_blocked_domain(email: &str, blocked: &[String]) -> bool {
    // Extract domain from email
    if let Some(domain) = email.split('@').nth(1) {
        // Exact match against blocked list
        blocked.iter().any(|b| b == domain)
    } else {
        false
    }
}
```

**Usage in validation macro:**
```rust
#[blocked_domains("spam.com", "blocked.net")]
email: String,
```

**Generated code:**
```rust
if rhtmx::validation::validators::is_blocked_domain(
    &self.email,
    &vec!["spam.com".to_string(), "blocked.net".to_string()]
) {
    errors.insert("email".to_string(), "Email domain is blocked");
}
```

### 4.2 Password Strength Validation

**File:** `/crates/rhtmx-validation/core/src/password.rs`

Three-tier password strength system:

```rust
pub enum PasswordPattern {
    Basic,           // 6+ chars
    Medium,          // 8+ chars + uppercase + lowercase + digit
    Strong,          // 8+ chars + upper + lower + digit + special
    Custom(String),  // For future regex support
}
```

#### Basic Pattern (6+ characters)
```rust
fn validate_basic(password: &str) -> Result<(), String> {
    if password.len() >= 6 {
        Ok(())
    } else {
        Err("Password must be at least 6 characters".to_string())
    }
}
```

#### Medium Pattern (8+ with mixed case & digit)
```rust
fn validate_medium(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    
    if !has_uppercase || !has_lowercase || !has_digit {
        return Err("Password must contain uppercase, lowercase, and digit");
    }
    
    Ok(())
}
```

#### Strong Pattern (8+ with special character)
```rust
fn validate_strong(password: &str) -> Result<(), String> {
    if password.len() < 8 { /* error */ }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| {
        matches!(c, '@' | '$' | '!' | '%' | '*' | '?' | '&' 
                   | '#' | '-' | '_' | '+' | '=' | '.' | ',')
    });
    
    if !has_uppercase || !has_lowercase || !has_digit || !has_special {
        return Err("Password must contain all required character types");
    }
    
    Ok(())
}
```

---

## 5. Single Source of Truth Pattern

### 5.1 How It Works

**The Problem (Before):**
```rust
// Server-side validation (Rust)
#[derive(Validate)]
struct Form {
    #[email]
    #[no_public_domains]
    email: String,
}

// Client-side validation (HTML) - DUPLICATED
<input type="email" required data-validate='{"email":true,"noPublicDomains":true}' />
// ^ PROBLEM: Rules defined in TWO places, can get out of sync
```

**The Solution (After):**
```rust
// SINGLE definition - everything else is generated
#[derive(Validate, FormField)]
struct Form {
    #[email]
    #[no_public_domains]
    #[required]
    email: String,
}

// Server validation (generated)
impl Validate for Form {
    fn validate(&self) -> Result<(), HashMap<String, String>> {
        let mut errors = HashMap::new();
        
        if !is_valid_email(&self.email) {
            errors.insert("email".to_string(), "Invalid email");
        }
        if is_public_domain(&self.email) {
            errors.insert("email".to_string(), "Public domains not allowed");
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// Form field generation (generated)
impl FormField for Form {
    fn field_attrs(&self, field_name: &str) -> FieldAttrs {
        match field_name {
            "email" => {
                let mut attrs = HashMap::new();
                attrs.insert("type", "email");
                attrs.insert("required", "");
                
                FieldAttrs {
                    html5_attrs: attrs,
                    data_validate: r#"{"email":true,"noPublicDomains":true,"required":true}"#,
                    label: "Email Address".to_string(),
                }
            }
            _ => FieldAttrs::default(),
        }
    }
}

// HTML (generated from form.field_attrs)
<input name="email" type="email" required data-validate='{"email":true,"noPublicDomains":true,"required":true}' />
```

### 5.2 Validation Flow Diagram

```
┌──────────────────────────────────┐
│ Rust Struct with Attributes      │
│ #[derive(Validate, FormField)]   │
│ struct MyForm { ... }            │
└──────────────────┬───────────────┘
                   │
         ┌─────────┴──────────┐
         │                    │
      COMPILE                RUNTIME
      TIME                   │
         │         ┌─────────┴────────────┐
         │         │                      │
         ▼         ▼                      ▼
    ┌────────┐ ┌──────────┐      ┌──────────────┐
    │Server  │ │HTML5 +   │      │WASM Browser  │
    │Code    │ │data-      │      │Code          │
    │Gen.    │ │validate   │      │(Runtime)     │
    │        │ │Gen.       │      │              │
    └────────┘ └──────────┘      └──────────────┘
         │         │                      │
         │         │        ┌─────────────┘
         │         │        │
         ▼         ▼        ▼
    Validate()  render()  validateField()
    HashMap      HTML      Array<Errors>
```

### 5.3 Key Benefits

| Benefit | Explanation |
|---------|-------------|
| **DRY Principle** | Rules defined once, used everywhere |
| **Type Safety** | Compiler ensures consistency |
| **Zero Runtime Overhead** | All generation at compile time |
| **Easy Maintenance** | Change once, everything updates |
| **Impossible to Desync** | Can't forget to update HTML |
| **Better Developer Experience** | Less boilerplate, clearer intent |

---

## 6. Validation Flow: Server-Side Struct to Client-Side Validation

### 6.1 Complete Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: STRUCT DEFINITION (Developer writes)                    │
└─────────────────────────────────────────────────────────────────┘

#[derive(Validate, FormField, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    #[min_length(3)]
    #[max_length(100)]
    #[label("Email")]
    email: String,
    
    #[password("strong")]
    #[min_length(8)]
    password: String,
}

    ↓

┌─────────────────────────────────────────────────────────────────┐
│ Step 2: COMPILE TIME (Procedural Macros)                        │
└─────────────────────────────────────────────────────────────────┘

extract_validation_attrs() parses attributes:
  email → ValidationAttr::Email
  no_public_domains → ValidationAttr::NoPublicDomains
  required → ValidationAttr::Required
  min_length(3) → ValidationAttr::MinLength(3)
  max_length(100) → ValidationAttr::MaxLength(100)
  label("Email") → ValidationAttr::Label("Email")

impl_validate() generates:
  impl Validate for RegisterForm {
      fn validate(&self) -> Result<(), HashMap<String, String>> {
          let mut errors = HashMap::new();
          if !is_valid_email(&self.email) {
              errors.insert("email".to_string(), "Invalid email");
          }
          if is_public_domain(&self.email) {
              errors.insert("email".to_string(), "Public domains not allowed");
          }
          // ... more checks ...
          if errors.is_empty() { Ok(()) } else { Err(errors) }
      }
  }

impl_form_field() generates:
  impl FormField for RegisterForm {
      fn field_attrs(&self, field_name: &str) -> FieldAttrs { ... }
      fn field_names(&self) -> Vec<&'static str> { ... }
  }

    ↓

┌─────────────────────────────────────────────────────────────────┐
│ Step 3: RUNTIME - SERVER SIDE                                   │
└─────────────────────────────────────────────────────────────────┘

Handler receives form data:
    let form: RegisterForm = form_data.deserialize()?;
    
Validate on server:
    match form.validate() {
        Ok(()) => { /* Process valid form */ }
        Err(errors) => {
            // errors = {"email": "Invalid email", ...}
            // Render error response
        }
    }

    ↓

┌─────────────────────────────────────────────────────────────────┐
│ Step 4: RUNTIME - CLIENT SIDE (HTML Generation)                 │
└─────────────────────────────────────────────────────────────────┘

In template:
    let form = RegisterForm { ... };
    let email_attrs = form.field_attrs("email");
    
email_attrs contains:
    html5_attrs: {
        "type" → "email",
        "required" → "",
        "minlength" → "3",
        "maxlength" → "100"
    }
    data_validate: {"email":true,"noPublicDomains":true,"required":true,"minLength":3,"maxLength":100}
    label: "Email"

Render HTML:
    html! {
        <input
            name="email"
            type="email"
            required
            minlength="3"
            maxlength="100"
            data-validate='{"email":true,"noPublicDomains":true,"required":true,"minLength":3,"maxLength":100}'
        />
    }

    ↓

┌─────────────────────────────────────────────────────────────────┐
│ Step 5: WASM VALIDATION (Browser, Real-time)                    │
└─────────────────────────────────────────────────────────────────┘

JavaScript loads WASM:
    import init, { validateField } from './pkg/rhtmx_validation_wasm.js';
    await init();

On user input:
    const errors = await validateField('email', 'user@example.com', {
        email: true,
        noPublicDomains: true,
        required: true,
        minLength: 3,
        maxLength: 100
    });
    
    if (errors.length > 0) {
        // Show error to user
        // errors = [{ field: "email", message: "Invalid email" }]
    }

Same validation logic runs in browser!
    ↓
    core::is_valid_email("user@example.com")  ← Same function on server & client
```

### 6.2 Data Structure Transformations

```
INPUT (Rust struct):
RegisterForm {
    email: "test@gmail.com",
    password: "SecurePass123!"
}

↓

VALIDATION ATTRIBUTES (Compile time):
[
    ValidationAttr::Email,
    ValidationAttr::NoPublicDomains,
    ValidationAttr::Required,
    ValidationAttr::MinLength(3),
    ValidationAttr::MaxLength(100),
    ValidationAttr::Label("Email"),
    ValidationAttr::Password("strong"),
]

↓

HTML5 ATTRIBUTES (Generated):
HashMap {
    "type" → "email",
    "required" → "",
    "minlength" → "3",
    "maxlength" → "100"
}

↓

DATA-VALIDATE JSON (Generated):
{
    "email": true,
    "noPublicDomains": true,
    "required": true,
    "minLength": 3,
    "maxLength": 100,
    "password": "strong"
}

↓

RENDERED HTML:
<input
    type="email"
    required
    minlength="3"
    maxlength="100"
    data-validate='{"email":true,"noPublicDomains":true,"required":true,"minLength":3,"maxLength":100,"password":"strong"}'
/>

↓

BROWSER VALIDATION (WASM):
validateField('email', 'test@gmail.com', {
    email: true,
    noPublicDomains: true,
    required: true,
    minLength: 3,
    maxLength: 100,
    password: "strong"
})

↓

VALIDATION RESULT:
[
    {
        field: "email",
        message: "Public email domains not allowed"
    }
]
```

### 6.3 Three-Layer Validation Strategy

```
┌────────────────────────────────────────────────────────────────┐
│ Layer 1: HTML5 VALIDATION (Browser Built-in)                   │
│ - Instant user feedback                                        │
│ - No JavaScript required                                       │
│ - Examples: type="email", minlength="8", required              │
│ - Limitations: Only basic rules                                │
└────────────────────────────────────────────────────────────────┘
        ↓ (User tries to submit invalid form)
        
┌────────────────────────────────────────────────────────────────┐
│ Layer 2: WASM VALIDATION (Client-side, JavaScript)             │
│ - Real-time validation as user types                           │
│ - Advanced rules (email domain, password strength)             │
│ - No network call required                                     │
│ - Same logic as server                                         │
│ - Examples: noPublicDomains, password strength                 │
└────────────────────────────────────────────────────────────────┘
        ↓ (User tries to submit)
        
┌────────────────────────────────────────────────────────────────┐
│ Layer 3: SERVER VALIDATION (Security)                          │
│ - Final validation before storing data                         │
│ - Prevents bypassing client-side validation                    │
│ - Can access database (check email uniqueness, etc.)           │
│ - Returns errors if invalid                                    │
└────────────────────────────────────────────────────────────────┘
```

---

## 7. Validation Attributes Overview (30 Total)

### 7.1 Email Validators (3)
- `#[email]` - Valid email format
- `#[no_public_domains]` - Reject gmail, yahoo, etc.
- `#[blocked_domains("a.com", "b.com")]` - Block specific domains

### 7.2 Password Validators (1)
- `#[password("strong"|"medium"|"basic")]` - Password strength

### 7.3 Numeric Validators (3)
- `#[min(n)]` - Minimum value
- `#[max(n)]` - Maximum value
- `#[range(min, max)]` - Numeric range

### 7.4 String Length Validators (3)
- `#[min_length(n)]` - Minimum length
- `#[max_length(n)]` - Maximum length
- `#[length(min, max)]` - Length range

### 7.5 String Pattern Validators (2)
- `#[regex(r"pattern")]` - Custom regex
- `#[url]` - Valid URL format

### 7.6 String Matching Validators (4)
- `#[contains("text")]` - Must contain substring
- `#[not_contains("text")]` - Must not contain substring
- `#[starts_with("prefix")]` - Must start with prefix
- `#[ends_with("suffix")]` - Must end with suffix

### 7.7 Equality Validators (3)
- `#[equals("value")]` - Must equal exact value
- `#[not_equals("value")]` - Must not equal value
- `#[equals_field("other_field")]` - Must match another field

### 7.8 Conditional Validators (1)
- `#[depends_on("field", "value")]` - Required when another field has value

### 7.9 Collection Validators (3)
- `#[min_items(n)]` - Minimum collection size
- `#[max_items(n)]` - Maximum collection size
- `#[unique]` - All items must be unique

### 7.10 Enum/Values Validators (1)
- `#[enum_variant("val1", "val2")]` - Must be one of allowed values

### 7.11 Custom & Message Validators (4)
- `#[custom("func_name")]` - Call custom validation function
- `#[message = "text"]` - Override error message
- `#[label("Name")]` - Display name for errors
- `#[message_key("key")]` - i18n message key

### 7.12 General Validators (2)
- `#[required]` - Required for Option<T> fields
- `#[allow_whitespace]` - Don't trim whitespace

---

## 8. Code Organization & Files

### Core Macro Implementation
```
/crates/RHTMX-Form/
├── src/
│   ├── lib.rs                    # Macro entry points (Validate, FormField)
│   └── validation.rs             # Core macro implementation (1025 lines)
│       ├── extract_validation_attrs()    # Parse attributes
│       ├── ValidationAttr enum           # Attribute representation
│       ├── impl_validate()               # Generate Validate impl
│       ├── impl_form_field()             # Generate FormField impl
│       ├── validation_to_html5_attrs()   # Convert to HTML5
│       └── validation_to_json()          # Convert to JSON
└── Cargo.toml                    # Dependencies: syn, quote, proc-macro2
```

### Runtime Components
```
/crates/rhtmx/src/
├── validation/
│   ├── mod.rs                    # Validate trait definition
│   └── validators.rs             # Validator functions (82 lines)
│       └── Re-exports from core
└── form_field.rs                 # FieldAttrs struct (104 lines)
                                  # FormField trait definition
```

### Validation Core Library
```
/crates/rhtmx-validation/
├── core/
│   ├── src/
│   │   ├── lib.rs               # no_std compatible
│   │   ├── email.rs             # Email validation (156 lines)
│   │   ├── password.rs          # Password strength (146 lines)
│   │   ├── string.rs            # String validators (155 lines)
│   │   ├── numeric.rs           # Numeric validators
│   │   └── collection.rs        # Collection validators
│   └── Cargo.toml               # No dependencies!
└── wasm/
    ├── src/
    │   └── lib.rs               # WASM bindings (292 lines)
    └── Cargo.toml               # wasm-bindgen, serde-wasm-bindgen
```

---

## 9. Example: Complete Registration Form

```rust
use rhtmx::{Validate, FormField};
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct RegistrationForm {
    // Username: 3-20 chars, alphanumeric + underscore
    #[min_length(3)]
    #[max_length(20)]
    #[regex(r"^[a-zA-Z0-9_]+$")]
    #[label("Username")]
    username: String,

    // Email: corporate only
    #[email]
    #[no_public_domains]
    #[required]
    #[label("Corporate Email")]
    email: String,

    // Password: strong
    #[password("strong")]
    #[min_length(8)]
    password: String,

    // Confirm password
    #[equals_field("password")]
    #[message = "Passwords must match"]
    password_confirm: String,

    // Age: 18-120
    #[range(18, 120)]
    age: i32,

    // Terms: must accept
    #[required]
    #[message = "You must agree to terms"]
    terms_accepted: Option<bool>,
}

// Server Handler
#[post("/register")]
async fn register(form: RegistrationForm) -> Result<Html, String> {
    // Validate server-side
    form.validate()?;
    
    // Process registration...
    Ok(Html::from("<p>Welcome!</p>"))
}

// Template with auto-generated validation attributes
html! {
    form hx-post="/register" {
        div {
            label { "Username" }
            input[{
                let attrs = form.field_attrs("username");
                attrs.render_all()
            }]
        }
        
        div {
            label { "Email" }
            input[{
                let attrs = form.field_attrs("email");
                attrs.render_all()
            }]
        }
        
        div {
            label { "Password" }
            input[{
                let attrs = form.field_attrs("password");
                attrs.render_all()
            }]
        }
        
        button { "Register" }
    }
}
```

---

## 10. Key Insights

### 10.1 Why Compile-Time Generation Matters

1. **Zero Runtime Cost**: All generation happens at compile time
2. **Type Safety**: Rust compiler catches attribute errors
3. **Performance**: No reflection or dynamic dispatch
4. **Syncing**: Changes to attributes automatically propagate
5. **Code Generation**: Less boilerplate for developers

### 10.2 Shared Validation Logic Benefits

```
Before:
  Server validators (Rust) ≠ Client validators (JavaScript)
  → Possible to have bugs that only appear on one side

After:
  Server validators (Rust) = Client validators (WASM)
  → Same Rust code compiles to both native and WebAssembly
  → Guaranteed consistency
```

### 10.3 Three-Level Validation Strategy

| Level | Technology | Purpose |
|-------|-----------|---------|
| HTML5 | Browser built-in | Instant feedback, no JS required |
| WASM | WebAssembly | Complex rules, real-time, no server |
| Server | Rust validation | Security, database checks |

This provides:
- **Best UX**: Instant feedback before network
- **Best Security**: Can't bypass server validation
- **Best Performance**: No network roundtrip for basic validation

---

## Conclusion

The RHTMX validation system achieves a true "single source of truth" through compile-time procedural macros that:

1. **Parse** Rust attributes at compile time
2. **Generate** server-side validation code
3. **Generate** HTML5 + data-validate attributes
4. **Share** the same validation logic via no_std core
5. **Bridge** to WASM for identical client-side validation

This eliminates DRY violations, improves maintainability, and provides a superior developer experience compared to managing validation rules in multiple places.
