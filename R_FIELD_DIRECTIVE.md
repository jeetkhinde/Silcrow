# r-field Directive: Zero Boilerplate Form Generation

## What is r-field?

The `r-field` directive is a compile-time macro that automatically generates form input attributes from your Rust struct validation annotations. **Zero boilerplate, complete type safety.**

## The Problem It Solves

### Before r-field (Manual Approach)

```rust
let email_attrs = form.field_attrs("email");
let password_attrs = form.field_attrs("password");

html! {
    <div>
        <input name="email" {email_attrs.render_all()} />
        <input name="password" {password_attrs.render_all()} />
    </div>
}
```

❌ Manual `field_attrs()` calls for every field
❌ Boilerplate for each input
❌ Easy to forget fields

### After r-field (Directive Approach)

```rust
html! {
    <div>
        <input r-field={form.email} />
        <input r-field={form.password} />
    </div>
}
```

✅ Zero boilerplate - just `r-field`!
✅ Automatic attribute generation
✅ Impossible to forget validation

## How It Works

### 1. Define Validation Once

```rust
#[derive(Validate, FormField, Deserialize)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    email: String,

    #[min_length(8)]
    #[password("strong")]
    password: String,

    #[min(18)]
    #[max(120)]
    age: i32,
}
```

### 2. Use r-field in Templates

```rust
fn registration_form(form: &RegisterForm) -> Html {
    html! {
        <form>
            // ✨ Magic! All validation attributes automatically generated
            <input r-field={form.email} />

            <input type="password" r-field={form.password} />

            <input type="number" r-field={form.age} />
        </form>
    }
}
```

### 3. Generated HTML

```html
<input
    name="email"
    type="email"
    required
    data-validate='{"email":true,"noPublicDomains":true,"required":true}'
/>

<input
    name="password"
    type="password"
    required
    data-validate='{"required":true}'
/>

<input
    name="age"
    type="number"
/>
```

## What Gets Generated

### Automatic `name` Attribute

The field name is **automatically extracted** from the expression:

```rust
<input r-field={form.email} />          // → name="email"
<input r-field={user.profile.email} />  // → name="email" (last part)
```

### HTML5 Validation Attributes

Rust validation annotations automatically generate HTML5 attributes:

| Rust Annotation | HTML5 Attribute |
|-----------------|-----------------|
| `#[email]` | `type="email"` |
| `#[required]` | `required` |
| `#[min_length(n)]` | `minlength="n"` |
| `#[max_length(n)]` | `maxlength="n"` |
| `#[min(n)]` | `min="n"` |
| `#[max(n)]` | `max="n"` |
| `#[url]` | `type="url"` |
| `#[regex(pattern)]` | `pattern="..."` |

### Client-Side Validation JSON

The `data-validate` attribute is automatically generated for WASM/JavaScript validation:

| Rust Annotation | data-validate JSON |
|-----------------|-------------------|
| `#[email]` | `"email": true` |
| `#[no_public_domains]` | `"noPublicDomains": true` |
| `#[min_length(n)]` | `"minLength": n` |
| `#[password("strong")]` | `"password": "strong"` |

## Advanced Features

### Composable with Other Attributes

You can still add your own attributes:

```rust
html! {
    <input
        r-field={form.email}
        class="form-control"
        placeholder="Enter your email"
        autocomplete="email"
    />
}
```

**Generates:**
```html
<input
    name="email"
    class="form-control"
    placeholder="Enter your email"
    autocomplete="email"
    type="email"
    required
    data-validate='{"email":true,"required":true}'
/>
```

### Works with Nested Fields

```rust
struct User {
    profile: Profile,
}

struct Profile {
    #[email]
    email: String,
}

html! {
    <input r-field={user.profile.email} />
    // Generates: name="email"
}
```

### Custom Validators (Server-Side Only)

Custom validators run **only** on the server and don't leak to the client:

```rust
#[derive(Validate, FormField)]
struct UserForm {
    #[custom("validate_username")]
    #[min_length(3)]
    #[regex(r"^[a-zA-Z0-9_]+$")]
    username: String,
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.contains("admin") {
        Err("Username cannot contain 'admin'".to_string())
    } else {
        Ok(())
    }
}
```

```html
<input r-field={form.username} />
```

**Generates:**
```html
<!-- Custom validator NOT exposed to client (security!) -->
<!-- Only HTML5/regex validation included -->
<input name="username" minlength="3" pattern="^[a-zA-Z0-9_]+$" />
```

✅ Server-side business logic stays private
✅ Client gets basic format validation
✅ No security leaks!

## Complete Example

```rust
use rhtmx::{html, Html, Validate, FormField};
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct ContactForm {
    #[min_length(2)]
    #[max_length(100)]
    #[required]
    name: String,

    #[email]
    #[no_public_domains]
    #[required]
    email: String,

    #[url]
    website: Option<String>,

    #[min_length(10)]
    #[max_length(1000)]
    #[required]
    message: String,
}

fn contact_page(form: &ContactForm) -> Html {
    html! {
        <div class="container">
            <h1>Contact Us</h1>

            <form action="/contact" method="post">
                <div class="form-group">
                    <label for="name">Name</label>
                    <input id="name" r-field={form.name} class="form-control" />
                </div>

                <div class="form-group">
                    <label for="email">Email</label>
                    <input id="email" r-field={form.email} class="form-control" />
                </div>

                <div class="form-group">
                    <label for="website">Website (optional)</label>
                    <input id="website" r-field={form.website} class="form-control" />
                </div>

                <div class="form-group">
                    <label for="message">Message</label>
                    <textarea id="message" r-field={form.message} class="form-control"></textarea>
                </div>

                <button type="submit" class="btn-primary">Send Message</button>
            </form>
        </div>
    }
}
```

## Benefits

### ✅ Zero Boilerplate
Just `r-field={field}` - that's it!

### ✅ Single Source of Truth
Define validation once in Rust, auto-generate HTML.

### ✅ Type-Safe
Compile-time verification. Typos in field names = compiler error.

### ✅ Impossible to Forget
Can't forget to add validation attributes - they're automatic.

### ✅ Security First
Custom validators stay server-side, never exposed to client.

### ✅ Fully Composable
Mix with your own attributes, classes, IDs, etc.

### ✅ Zero Runtime Cost
All code generated at compile time. No reflection, no runtime overhead.

## Implementation Details

### Compile-Time Code Generation

The `r-field` directive is implemented as part of the `html!` proc macro. When you write:

```rust
<input r-field={form.email} />
```

The macro generates (simplified):

```rust
{
    // Extract field name
    let __field_name = "email";

    // Get field attributes from FormField trait
    let __field_attrs = form.field_attrs(__field_name);

    // Render opening tag
    __html.push_str("<input name=\"");
    __html.push_str(__field_name);
    __html.push_str("\"");

    // Add HTML5 attributes
    for (name, value) in &__field_attrs.html5_attrs {
        __html.push_str(" ");
        __html.push_str(name);
        if !value.is_empty() {
            __html.push_str("=\"");
            __html.push_str(value);
            __html.push_str("\"");
        }
    }

    // Add data-validate
    if !__field_attrs.data_validate.is_empty() {
        __html.push_str(" data-validate='");
        __html.push_str(&__field_attrs.data_validate);
        __html.push_str("'");
    }

    __html.push_str(" />");
}
```

### Field Name Extraction

The macro parses expressions like:
- `form.email` → `"email"`
- `user.profile.email` → `"email"`
- `self.data.field` → `"field"`

The last identifier becomes the `name` attribute.

## Comparison with Other Frameworks

### React (Manual)
```jsx
<input
  name="email"
  type="email"
  required
  pattern="[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$"
  minLength={3}
  maxLength={100}
/>
```
❌ Manual attribute writing
❌ Duplicate validation logic
❌ Easy to make mistakes

### RHTMX r-field
```rust
#[email]
#[min_length(3)]
#[max_length(100)]
email: String,

<input r-field={form.email} />
```
✅ Define once
✅ Auto-generated
✅ Type-safe

## Running the Example

```bash
cargo run --example r_field_directive
```

## Working with Nutype Validated Types

RHTMX provides pre-built validated types in `rhtmx-form-types` that work seamlessly with `r-field`. These types enforce validation at the **type level**, eliminating redundant validation logic.

### Basic Nutype Usage

Simply use the Nutype type directly - no additional markers needed!

```rust
use rhtmx::{html, Html, Validate, FormField};
use rhtmx_form_types::{WorkEmailAddress, PasswordStrong, Username};
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct RegisterForm {
    email: WorkEmailAddress,      // ✅ Blocks public domains (Gmail, Yahoo, etc.)
    password: PasswordStrong,      // ✅ 10+ chars with complexity
    username: Username,            // ✅ 3-30 chars, alphanumeric + _/-
}

fn register_page(form: &RegisterForm) -> Html {
    html! {
        <form action="/register" method="post">
            <input type="email" r-field={form.email} placeholder="work@company.com" />
            <input type="password" r-field={form.password} />
            <input r-field={form.username} />
            <button type="submit">Register</button>
        </form>
    }
}
```

**What happens:**
- ✅ `WorkEmailAddress::try_new()` validates email format + blocks public/disposable domains
- ✅ `PasswordStrong::try_new()` validates 10+ chars + uppercase + lowercase + digit + special
- ✅ `Username::try_new()` validates 3-30 chars, alphanumeric + underscore/dash
- ✅ r-field generates clean HTML with `name` attributes
- ✅ No redundant validation logic!

### Available Nutype Types

#### Email Types

```rust
use rhtmx_form_types::{EmailAddress, WorkEmailAddress, BusinessEmailAddress};

#[derive(Validate, FormField, Deserialize)]
struct EmailExamples {
    // Any valid email (blocks disposable only)
    personal: EmailAddress,

    // No public domains (Gmail, Yahoo, Hotmail, etc.)
    work: WorkEmailAddress,

    // Strictest validation (corporate domains only)
    business: BusinessEmailAddress,
}

html! {
    <input type="email" r-field={form.personal} />
    <input type="email" r-field={form.work} />
    <input type="email" r-field={form.business} />
}
```

**Validation Rules:**
| Type | Valid | Invalid |
|------|-------|---------|
| `EmailAddress` | `user@gmail.com`, `user@company.com` | `user@tempmail.com` |
| `WorkEmailAddress` | `user@company.com` | `user@gmail.com`, `user@yahoo.com` |
| `BusinessEmailAddress` | `user@verified-corp.com` | Public or disposable domains |

#### Password Types

```rust
use rhtmx_form_types::{
    PasswordBasic,       // 6+ chars
    PasswordMedium,      // 8+ chars
    PasswordStrong,      // 10+ chars + complexity
    SuperStrongPassword, // 12+ chars + 2 special chars
    PasswordPhrase,      // 15+ chars (passphrase style)
    PasswordPhrase3,     // 20+ chars, 3+ words
    ModernPassword,      // 16+ chars (NIST 2024)
};

#[derive(Validate, FormField, Deserialize)]
struct SecurityLevels {
    // Basic (low security)
    basic: PasswordBasic,         // "secret"

    // Standard (medium security)
    medium: PasswordMedium,       // "password"

    // Strong (high security) - RECOMMENDED
    strong: PasswordStrong,       // "Password123!"

    // Super strong (very high security)
    super_strong: SuperStrongPassword,  // "Password123!@"

    // Passphrase (high security, user-friendly)
    phrase: PasswordPhrase,       // "BlueSky-Mountain"

    // Multi-word passphrase
    phrase3: PasswordPhrase3,     // "Correct-Horse-Battery-Staple"

    // Modern NIST guidelines
    modern: ModernPassword,       // "MyLongPassword2024"
}

html! {
    <input type="password" r-field={form.strong} placeholder="10+ chars, mixed case, digit, special" />
    <input type="password" r-field={form.modern} placeholder="16+ chars" />
}
```

**Choose the right password type for your security needs:**
- 🔒 **PasswordBasic** - Non-critical accounts
- 🔒🔒 **PasswordMedium/Strong** - Standard applications
- 🔒🔒🔒 **SuperStrongPassword** - Financial/admin accounts
- 🔑 **PasswordPhrase/ModernPassword** - User-friendly + secure

#### String Types

```rust
use rhtmx_form_types::{NonEmptyString, Username};

#[derive(Validate, FormField, Deserialize)]
struct ProfileForm {
    // Cannot be empty
    bio: NonEmptyString,

    // 3-30 chars, alphanumeric + _/-
    username: Username,
}

html! {
    <input r-field={form.username} placeholder="john_doe" />
    <textarea r-field={form.bio}></textarea>
}
```

#### Numeric Types

```rust
use rhtmx_form_types::{PositiveInt, NonNegativeInt, Age, Percentage, Port};

#[derive(Validate, FormField, Deserialize)]
struct NumericForm {
    age: Age,                    // 18-120
    discount: Percentage,        // 0-100
    quantity: PositiveInt,       // > 0
    rating: NonNegativeInt,      // >= 0
    server_port: Port,           // 1-65535
}

html! {
    <input type="number" r-field={form.age} placeholder="18-120" />
    <input type="number" r-field={form.discount} placeholder="0-100%" />
    <input type="number" r-field={form.server_port} placeholder="1-65535" />
}
```

#### URL Types

```rust
use rhtmx_form_types::{UrlAddress, HttpsUrl};

#[derive(Validate, FormField, Deserialize)]
struct LinkForm {
    // Any valid URL (http, https, ftp, etc.)
    website: UrlAddress,

    // HTTPS only (secure connections)
    api_endpoint: HttpsUrl,
}

html! {
    <input type="url" r-field={form.website} placeholder="https://example.com" />
    <input type="url" r-field={form.api_endpoint} placeholder="HTTPS only" />
}
```

#### Pattern Types (US-Specific)

```rust
use rhtmx_form_types::{PhoneNumber, ZipCode, IpAddress, Uuid};

#[derive(Validate, FormField, Deserialize)]
struct USAddressForm {
    phone: PhoneNumber,      // (123) 456-7890
    zip: ZipCode,            // 12345 or 12345-6789
    ip: IpAddress,           // 192.168.1.1
    id: Uuid,                // 550e8400-e29b-41d4-a716-446655440000
}

html! {
    <input type="tel" r-field={form.phone} placeholder="(123) 456-7890" />
    <input r-field={form.zip} placeholder="12345" />
    <input r-field={form.ip} placeholder="192.168.1.1" />
    <input r-field={form.id} placeholder="UUID v4" />
}
```

### Real-World Example: B2B Registration

```rust
use rhtmx::{html, Html, Validate, FormField};
use rhtmx_form_types::{WorkEmailAddress, PasswordStrong, Username, PhoneNumber, NonEmptyString};
use serde::Deserialize;

#[derive(Validate, FormField, Deserialize)]
struct B2BRegistration {
    username: Username,
    email: WorkEmailAddress,           // Only corporate emails
    password: PasswordStrong,

    #[equals_field = "password"]       // ✅ Form-level validator still works!
    confirm_password: PasswordStrong,

    company: NonEmptyString,
    phone: PhoneNumber,
}

fn registration_form(form: &B2BRegistration) -> Html {
    html! {
        <form action="/register" method="post" class="registration">
            <div class="form-group">
                <label>Username</label>
                <input r-field={form.username} class="form-control" />
                <small>3-30 characters, alphanumeric + underscore/dash</small>
            </div>

            <div class="form-group">
                <label>Work Email</label>
                <input type="email" r-field={form.email} class="form-control" />
                <small>Corporate email only (no Gmail, Yahoo, etc.)</small>
            </div>

            <div class="form-group">
                <label>Password</label>
                <input type="password" r-field={form.password} class="form-control" />
                <small>10+ chars, uppercase, lowercase, digit, special character</small>
            </div>

            <div class="form-group">
                <label>Confirm Password</label>
                <input type="password" r-field={form.confirm_password} class="form-control" />
            </div>

            <div class="form-group">
                <label>Company Name</label>
                <input r-field={form.company} class="form-control" />
            </div>

            <div class="form-group">
                <label>Phone</label>
                <input type="tel" r-field={form.phone} class="form-control" placeholder="(123) 456-7890" />
            </div>

            <button type="submit" class="btn-primary">Create Account</button>
        </form>
    }
}
```

### Combining Nutype with Form-Level Validators

If you need **additional** form-level validators on Nutype fields, use the `#[nutype]` marker to avoid duplication:

```rust
#[derive(Validate, FormField, Deserialize)]
struct LoginForm {
    #[nutype]                          // ← Skip base validators
    #[equals_field = "confirm_email"]  // ✅ Cross-field validation
    email: WorkEmailAddress,

    #[nutype]
    #[equals_field = "email"]
    confirm_email: WorkEmailAddress,
}
```

**Without `#[nutype]` marker:** If you add validators like `#[email]` to a `WorkEmailAddress` field, validation would run twice (redundant).

**With `#[nutype]` marker:** Base validators are skipped, only form-specific ones (like `#[equals_field]`) are kept.

### Benefits of Nutype Types

✅ **Type-Level Validation** - Impossible to construct invalid values
✅ **Zero Redundancy** - No duplicate validation logic
✅ **Business Rules in Types** - `WorkEmailAddress` IS the business rule
✅ **Reusable** - Use same types across multiple forms
✅ **Self-Documenting** - Type name explains validation rules
✅ **Compile-Time Safety** - Invalid types = compiler error
✅ **Works with r-field** - Seamless integration with zero boilerplate

### When to Use Nutype vs. Form Validators

**Use Nutype types when:**
- ✅ Validation is a fundamental domain constraint (e.g., "work emails only")
- ✅ You'll reuse the validation across multiple forms
- ✅ The type itself represents a business rule

**Use form-level validators when:**
- ✅ Validation is specific to one form (e.g., "age must be 21+ for this contest")
- ✅ Cross-field validation (e.g., `#[equals_field]`)
- ✅ Custom business logic (e.g., `#[custom]`)

**Best of both worlds:**
```rust
#[derive(Validate, FormField, Deserialize)]
struct ContestEntry {
    email: WorkEmailAddress,     // ← Type-level: work email only

    #[min(21)]                   // ← Form-level: contest requires 21+
    age: Age,                    // ← Type-level: 18-120 range
}
```

## Related Documentation

- [Unified Validation](./UNIFIED_VALIDATION.md) - Single source of truth for validation
- [FormField Trait](./crates/rhtmx/src/form_field.rs) - Field attribute generation
- [Validate Derive](./crates/RHTMX-Form/src/lib.rs) - Validation macro
- [Nutype Types](./crates/rhtmx-form-types/src/lib.rs) - Pre-built validated types

## Summary

The `r-field` directive eliminates form input boilerplate while maintaining complete type safety and security. Define your validation rules once in Rust, and let the compiler generate all the HTML and JavaScript validation attributes automatically.

**No more duplicate validation logic. No more forgotten attributes. Just pure, type-safe form generation.** 🎉
