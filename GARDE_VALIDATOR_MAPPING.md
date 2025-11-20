# Garde Validator Mapping & Simplified API Design

## What Garde Can Replace (14/30 validators)

### ✅ Direct Garde Replacements (14 validators)

| Current RHTMX Validator | Garde Equivalent | Notes |
|------------------------|------------------|-------|
| `#[email]` | `garde::email` | ✅ Direct replacement |
| `#[url]` | `garde::url` | ✅ Direct replacement |
| `#[min(n)]` | `garde::range(min=n)` | ✅ Direct replacement |
| `#[max(n)]` | `garde::range(max=n)` | ✅ Direct replacement |
| `#[range(min, max)]` | `garde::range(min=n, max=n)` | ✅ Direct replacement |
| `#[min_length(n)]` | `garde::length(min=n)` | ✅ Direct replacement |
| `#[max_length(n)]` | `garde::length(max=n)` | ✅ Direct replacement |
| `#[length(min, max)]` | `garde::length(min=n, max=n)` | ✅ Direct replacement |
| `#[regex(r"pattern")]` | `garde::pattern(regex)` | ✅ Direct replacement |
| `#[required]` | `garde::required` | ✅ Direct replacement |
| `#[contains("text")]` | `garde::contains(text)` | ✅ Direct replacement |
| `#[starts_with("prefix")]` | `garde::prefix(str)` | ✅ Direct replacement |
| `#[ends_with("suffix")]` | `garde::suffix(str)` | ✅ Direct replacement |
| `#[equals_field("field")]` | `garde::matches(field)` | ✅ Direct replacement |

**Total replaced by garde**: **14 validators**

### ❌ Must Keep Custom (16 validators)

| Current RHTMX Validator | Why Custom Needed |
|------------------------|-------------------|
| `#[no_public_domains]` | Unique RHTMX feature - domain blocklist |
| `#[blocked_domains(...)]` | Unique RHTMX feature - custom blocklist |
| `#[password("pattern")]` | Unique 3-tier strength logic |
| `#[allow_whitespace]` | Unique trimming behavior |
| `#[not_contains("text")]` | Not in garde (negation) |
| `#[equals("value")]` | Not in garde (literal comparison) |
| `#[not_equals("value")]` | Not in garde (literal negation) |
| `#[depends_on("field", "val")]` | Complex conditional logic |
| `#[min_items(n)]` | Collection size validation |
| `#[max_items(n)]` | Collection size validation |
| `#[unique]` | Collection uniqueness check |
| `#[enum_variant(...)]` | Value restriction list |
| `#[custom("func")]` | By definition custom |
| `#[message = "text"]` | Error message override |
| `#[label("name")]` | Display name for errors |
| `#[message_key("key")]` | i18n localization key |

**Total custom validators**: **16 validators**

---

## 🎨 Proposed Simplified API

### Problem: Current garde API is ugly

```rust
// ❌ UGLY: Too verbose, exposes garde complexity
#[derive(garde::Validate, FormField)]
#[garde(context(EmailValidationContext))]
struct LoginForm {
    #[garde(email, custom(no_public_email))]
    email: String,

    #[garde(length(min=8), custom(password_strength))]
    password: String,
}

// User has to call: form.validate(&ctx)?;
```

**Issues**:
1. Context boilerplate at struct level
2. Mixing `garde` and `custom` is verbose
3. User sees garde internals
4. Need to pass context explicitly

---

## ✨ Solution: Hide Garde, Super Simple API

### Option 1: Pure RHTMX API (Recommended)

```rust
// ✅ CLEAN: User never sees garde at all
#[derive(FormField)]
struct LoginForm {
    #[email]
    #[no_public_domains]
    email: String,

    #[password(min = 8, strength = "strong")]
    password: String,
}

// Validation: Simple, no context needed
form.validate()?;
```

**How it works**:
- `FormField` macro handles EVERYTHING
- Uses garde internally for `#[email]`, `#[min]`, etc.
- Uses custom validators for `#[no_public_domains]`, `#[password]`
- Context injected automatically (no user boilerplate)
- Single derive, clean attributes

**Implementation**:
```rust
// In FormField macro:
#[proc_macro_derive(FormField, attributes(email, password, url, ...))]
pub fn derive_form_field(input: TokenStream) -> TokenStream {
    // Parse attributes
    // Map to garde where possible
    // Map to custom where needed
    // Auto-inject context
    // Generate Validate trait impl
}
```

---

### Option 2: Consolidated Validate Attribute

```rust
// ✅ CLEAN: Single validate attribute, comma-separated
#[derive(FormField)]
struct LoginForm {
    #[validate(email, no_public_domains)]
    email: String,

    #[validate(min_length = 8, password = "strong")]
    password: String,
}
```

**Benefits**:
- Familiar syntax (like current system)
- All validators in one attribute
- Still hides garde complexity

---

### Option 3: Semantic Field Types

```rust
// ✅ ULTRA CLEAN: Let the type system do the work
#[derive(FormField)]
struct LoginForm {
    email: BusinessEmail,  // Auto-validates, blocks public domains
    password: StrongPassword,  // Auto-validates strength
}

// Field types carry validation rules
#[derive(FieldType)]
#[email]
#[no_public_domains]
struct BusinessEmail(String);

#[derive(FieldType)]
#[password(strength = "strong")]
#[min_length(8)]
struct StrongPassword(String);
```

**Benefits**:
- Maximum reusability
- Type-safe
- Zero attributes on form struct
- Validators defined once, used everywhere

**Trade-off**: More types to define

---

## 🏆 Recommended: Option 1 (Pure RHTMX API)

### Why This is Best

1. **Simplest user experience**: Just stack attributes
2. **Backward compatible**: Same attribute names as current
3. **Hides implementation**: User doesn't care about garde
4. **No boilerplate**: No context, no garde imports
5. **Single derive**: Just `#[derive(FormField)]`

### Example: Complex Form

```rust
#[derive(FormField)]
struct SignupForm {
    // Email with domain blocking
    #[email]
    #[no_public_domains]
    #[message = "Please use your company email"]
    email: String,

    // Password with strength
    #[password(strength = "strong")]
    #[min_length(12)]
    #[label("Your Password")]
    password: String,

    // Password confirmation
    #[password]
    #[equals_field("password")]
    #[label("Confirm Password")]
    password_confirm: String,

    // Username with constraints
    #[min_length(3)]
    #[max_length(20)]
    #[regex(r"^[a-zA-Z0-9_]+$")]
    #[not_contains("admin")]
    username: String,

    // Optional bio
    #[max_length(500)]
    bio: Option<String>,

    // Age range
    #[range(min = 18, max = 120)]
    age: u8,

    // Terms acceptance
    #[required]
    #[equals("true")]
    accept_terms: Option<bool>,
}

// Usage: Super simple!
let form = SignupForm { /* ... */ };
form.validate()?;  // That's it!
```

---

## 🔧 Implementation Strategy

### FormField Macro Responsibilities

```rust
#[proc_macro_derive(FormField, attributes(
    // Validators
    email, password, url,
    min, max, range,
    min_length, max_length, length,
    regex, contains, starts_with, ends_with,
    equals, equals_field, not_equals,
    required, unique,
    no_public_domains, blocked_domains,
    // Meta
    message, label, message_key,
    allow_whitespace,
))]
pub fn derive_form_field(input: TokenStream) -> TokenStream {
    let parsed = parse_all_validators(&input);

    // Generate code
    generate_impl(&parsed) {
        // 1. Implement Validate trait
        impl Validate for #struct_name {
            fn validate(&self) -> Result<(), ValidationErrors> {
                // Use garde for built-in validators
                // Use custom functions for RHTMX-specific ones
                // Merge all errors
            }
        }

        // 2. Implement FormField trait (HTML5 gen)
        impl FormField for #struct_name {
            fn field_attrs(&self, name: &str) -> FieldAttrs {
                // Map validators → HTML5 attrs
                // Map validators → data-validate JSON
            }
        }
    }
}
```

### Validator Mapping Logic

```rust
fn map_validator(attr: &ValidatorAttr) -> ValidatorImpl {
    match attr {
        // Use garde
        ValidatorAttr::Email => ValidatorImpl::Garde(garde::email),
        ValidatorAttr::MinLength(n) => ValidatorImpl::Garde(garde::length(min=n)),
        ValidatorAttr::Range { min, max } => ValidatorImpl::Garde(garde::range(min, max)),

        // Use custom
        ValidatorAttr::NoPublicDomains => ValidatorImpl::Custom(no_public_email),
        ValidatorAttr::Password { strength } => ValidatorImpl::Custom(password_strength),
        ValidatorAttr::BlockedDomains(list) => ValidatorImpl::Custom(blocked_domains),
    }
}
```

### Context Auto-Injection

```rust
// User never sees this!
impl Validate for LoginForm {
    fn validate(&self) -> Result<(), ValidationErrors> {
        // Auto-create context
        let ctx = EmailValidationContext {
            public_domains: BLOCKED_PUBLIC_DOMAINS,
            blocked_domains: vec![],
        };

        // Validate with garde (for built-in validators)
        garde::Validate::validate(&self, &())?;

        // Validate with custom (for RHTMX validators)
        if let Err(e) = no_public_email(&self.email, &ctx) {
            errors.insert("email", e);
        }

        // Merge and return
        Ok(())
    }
}
```

---

## 📊 Comparison Table

| Aspect | Current (garde exposed) | Proposed (hidden) |
|--------|------------------------|-------------------|
| **Derives** | `garde::Validate, FormField` | `FormField` only |
| **Context** | `#[garde(context(...))]` | Auto-injected |
| **Attributes** | `#[garde(email, custom(...))]` | `#[email]` `#[no_public_domains]` |
| **Imports** | `use garde::Validate;` | None needed |
| **Validation call** | `form.validate(&ctx)?` | `form.validate()?` |
| **User complexity** | High (sees garde internals) | Low (just attributes) |
| **LOC (form definition)** | ~10 lines | ~5 lines |

**Reduction in boilerplate**: ~50%

---

## 🚀 Migration Path

### For Existing Users

Current code still works (backward compatible):
```rust
// Old syntax: Keep working
#[derive(Validate, FormField)]
struct Form {
    #[validate(email)]
    email: String,
}
```

New syntax (opt-in):
```rust
// New syntax: Simpler
#[derive(FormField)]  // No Validate needed
struct Form {
    #[email]  // Direct attribute
    email: String,
}
```

Both generate the same code under the hood!

---

## 💡 Key Insight

> **Garde is an implementation detail, not part of the API.**

Users should think:
- "I want email validation" → `#[email]`
- "I want strong password" → `#[password(strength = "strong")]`
- "I want to block Gmail" → `#[no_public_domains]`

They should NOT think:
- "I need to derive garde::Validate"
- "I need to set up a context"
- "I need to use garde::custom for everything"

**FormField macro should hide ALL complexity.**

---

## 🎯 Recommended Action Plan

1. **Implement Option 1**: Pure RHTMX API
2. **Use garde internally**: For 14 built-in validators
3. **Use custom internally**: For 16 RHTMX-specific validators
4. **Auto-inject context**: No user boilerplate
5. **Single derive**: Just `FormField`

**Result**:
- Simple user API
- Leverage garde's maturity
- Preserve RHTMX uniqueness
- Reduce user code by ~50%

---

## Example: Before & After

### Before (garde exposed)
```rust
use garde::Validate;

#[derive(garde::Validate, FormField)]
#[garde(context(EmailValidationContext))]
struct LoginForm {
    #[garde(email, custom(no_public_email))]
    email: String,

    #[garde(length(min=8), custom(password_strength))]
    password: String,
}

let ctx = EmailValidationContext {
    public_domains: &["gmail.com", "yahoo.com"],
    blocked_domains: vec![],
};

form.validate(&ctx)?;
```

**Lines**: 17

### After (garde hidden)
```rust
#[derive(FormField)]
struct LoginForm {
    #[email]
    #[no_public_domains]
    email: String,

    #[password(strength = "strong")]
    #[min_length(8)]
    password: String,
}

form.validate()?;
```

**Lines**: 11

**Reduction**: 35% less code, infinitely cleaner!
