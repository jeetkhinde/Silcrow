# Unified Validation: Single Source of Truth

## Problem Statement

### The DRY Violation

Previously, validation rules had to be defined **twice** - once in Rust for server-side validation, and once in HTML for client-side validation:

**Server-side (Rust):**
```rust
#[derive(Validate, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    email: String,
}
```

**Client-side (HTML):**
```html
<input name="email"
    type="email"
    required
    data-validate='{"email": true, "noPublicDomains": true, "required": true}'
/>
```

### Issues with Duplication

❌ **Same rules written twice** - Violates DRY principle
❌ **Easy to get out of sync** - Change Rust, forget HTML
❌ **Manual JSON writing is error-prone** - Easy to make typos
❌ **No type safety** - HTML and struct can diverge
❌ **Extra work for developers** - Double maintenance burden

## Solution: Unified Validation

The Rust struct is now the **single source of truth**. Validation rules defined in Rust attributes automatically generate:

1. ✅ Server-side validation code
2. ✅ HTML5 validation attributes
3. ✅ Client-side `data-validate` JSON

## How It Works

### Step 1: Define Validation Rules Once

```rust
use rhtmx::{Validate, FormField};
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    #[label("Email Address")]
    email: String,

    #[min_length(8)]
    #[max_length(100)]
    #[password("strong")]
    password: String,

    #[min(18)]
    #[max(120)]
    age: i32,

    #[url]
    website: Option<String>,
}
```

### Step 2: Get Field Attributes

```rust
let form = RegisterForm { /* ... */ };
let email_attrs = form.field_attrs("email");

// email_attrs now contains:
// - html5_attrs: {"type": "email", "required": ""}
// - data_validate: {"email": true, "noPublicDomains": true, "required": true}
// - label: "Email Address"
```

### Step 3: Use in Templates

```rust
html! {
    <input
        name="email"
        type={email_attrs.html5_attrs["type"]}
        required={if email_attrs.html5_attrs.contains_key("required") { "required" } else { "" }}
        data-validate={email_attrs.data_validate}
    />
}
```

Or get all attributes as a string:
```rust
let attrs_string = email_attrs.render_all();
// Returns: type="email" required data-validate='{"email":true,"noPublicDomains":true,"required":true}'
```

## Validation Attribute Mapping

| Rust Attribute | HTML5 Attribute | data-validate JSON |
|----------------|-----------------|-------------------|
| `#[email]` | `type="email"` | `"email": true` |
| `#[required]` | `required` | `"required": true` |
| `#[min_length(n)]` | `minlength="n"` | `"minLength": n` |
| `#[max_length(n)]` | `maxlength="n"` | `"maxLength": n` |
| `#[min(n)]` | `min="n"` | `"min": n` |
| `#[max(n)]` | `max="n"` | `"max": n` |
| `#[url]` | `type="url"` | `"url": true` |
| `#[regex(pattern)]` | `pattern="..."` | `"pattern": "..."` |
| `#[password("strong")]` | *(none)* | `"password": "strong"` |
| `#[no_public_domains]` | *(none)* | `"noPublicDomains": true` |
| `#[blocked_domains(...)]` | *(none)* | `"blockedDomains": [...]` |
| `#[contains(s)]` | *(none)* | `"contains": "s"` |
| `#[starts_with(s)]` | *(none)* | `"startsWith": "s"` |
| `#[ends_with(s)]` | *(none)* | `"endsWith": "s"` |
| `#[min_items(n)]` | *(none)* | `"minItems": n` |
| `#[max_items(n)]` | *(none)* | `"maxItems": n` |
| `#[unique]` | *(none)* | `"unique": true` |

## API Reference

### `FormField` Trait

Automatically implemented when you derive `FormField`.

```rust
pub trait FormField {
    /// Get field attributes for the specified field name
    fn field_attrs(&self, field_name: &str) -> FieldAttrs;

    /// Get all field names
    fn field_names(&self) -> Vec<&'static str>;
}
```

### `FieldAttrs` Struct

Contains all validation metadata for a field.

```rust
pub struct FieldAttrs {
    /// HTML5 native attributes (e.g., "required", "minlength", "type")
    pub html5_attrs: HashMap<String, String>,

    /// JSON string for data-validate attribute
    pub data_validate: String,

    /// Field label for display
    pub label: String,
}

impl FieldAttrs {
    /// Render HTML5 attributes as a string
    pub fn render_html5_attrs(&self) -> String;

    /// Render data-validate attribute
    pub fn render_data_validate(&self) -> String;

    /// Render all attributes (HTML5 + data-validate)
    pub fn render_all(&self) -> String;
}
```

## Benefits

### ✅ Single Source of Truth
Define validation rules **once** in Rust. No duplication.

### ✅ Type Safety
Impossible to have mismatched validation between server and client. The compiler ensures consistency.

### ✅ Automatic Synchronization
Change validation rules in Rust, and HTML automatically updates. No manual sync required.

### ✅ Less Code to Write
No manual `data-validate` JSON writing. Everything is generated.

### ✅ Fewer Errors
Can't forget to update client-side validation when server-side rules change.

### ✅ Better Developer Experience
Cleaner, more maintainable code. Focus on business logic, not boilerplate.

## Example: Before vs After

### ❌ Before (Duplicated Validation)

**Rust:**
```rust
#[derive(Validate, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    email: String,
}
```

**HTML (Manual):**
```html
<input name="email"
    type="email"
    required
    data-validate='{"email": true, "noPublicDomains": true, "required": true}'
/>
```

**Problems:**
- Rules defined in two places
- Easy to forget updating HTML when Rust changes
- Manual JSON is error-prone

### ✅ After (Unified Validation)

**Rust (Single Source of Truth):**
```rust
#[derive(Validate, FormField, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    email: String,
}

// Get attributes
let email_attrs = form.field_attrs("email");
```

**HTML (Generated):**
```rust
// Attributes are automatically generated from Rust struct
<input name="email" {email_attrs.render_all()} />
```

**Benefits:**
- Rules defined once in Rust
- HTML automatically syncs with Rust
- Type-safe and compiler-verified

## Running the Demo

```bash
cargo run --example unified_validation
```

This will show you:
- How validation rules are defined once in Rust
- What HTML5 attributes are generated
- What data-validate JSON is generated
- A complete mapping reference

## Files Added/Modified

### New Files
- `/crates/rhtmx/src/form_field.rs` - FormField trait and FieldAttrs struct
- `/crates/rhtmx/examples/unified_validation.rs` - Complete working example
- `/UNIFIED_VALIDATION.md` - This documentation

### Modified Files
- `/crates/RHTMX-Form/src/validation.rs` - Added FormField implementation generation
- `/crates/RHTMX-Form/src/lib.rs` - Added FormField derive macro
- `/crates/rhtmx/src/lib.rs` - Export FormField trait and types

## Implementation Details

### How It Works Under the Hood

1. **Parse Stage**: The `#[derive(FormField)]` macro parses validation attributes from your struct at compile time.

2. **Generate Stage**: For each field, it generates:
   - HTML5 attributes mapping (e.g., `#[min_length(8)]` → `minlength="8"`)
   - JSON data-validate object (e.g., `#[email]` → `"email": true`)
   - Field labels (from `#[label(...)]` or auto-generated)

3. **Runtime Stage**: The generated `FormField` implementation provides methods to access these attributes at runtime.

### Key Functions

**Attribute Conversion:**
- `validation_to_html5_attrs()` - Converts validation attributes to HTML5 attributes
- `validation_to_json()` - Converts validation attributes to data-validate JSON

**Code Generation:**
- `impl_form_field()` - Generates FormField trait implementation
- `extract_validation_attrs()` - Parses validation attributes from field

## Conclusion

This solution eliminates DRY violations by making the Rust struct the **single source of truth** for validation rules. Server-side and client-side validation are now automatically synchronized, reducing errors and maintenance burden.

**No more duplicate validation rules!** 🎉
