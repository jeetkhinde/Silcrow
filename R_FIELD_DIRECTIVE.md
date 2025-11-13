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

## Related Documentation

- [Unified Validation](./UNIFIED_VALIDATION.md) - Single source of truth for validation
- [FormField Trait](./crates/rhtmx/src/form_field.rs) - Field attribute generation
- [Validate Derive](./crates/RHTMX-Form/src/lib.rs) - Validation macro

## Summary

The `r-field` directive eliminates form input boilerplate while maintaining complete type safety and security. Define your validation rules once in Rust, and let the compiler generate all the HTML and JavaScript validation attributes automatically.

**No more duplicate validation logic. No more forgotten attributes. Just pure, type-safe form generation.** 🎉
