// RHTMX Procedural Macros for Form

use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

mod validation;

/// Derive macro for automatic validation
///
/// Generates a `Validate` trait implementation that validates struct fields
/// based on attributes like #[email], #[min], #[max], etc.
///
/// # Example
///
/// ```ignore
/// use rhtmx::Validate;
/// use serde::Deserialize;
///
/// #[derive(Validate, Deserialize)]
/// struct CreateUserRequest {
///     #[min_length(3)]
///     #[max_length(50)]
///     name: String,
///
///     #[email]
///     #[no_public_domains]
///     email: String,
///
///     #[password("strong")]
///     password: String,
///
///     #[min(18)]
///     #[max(120)]
///     age: i32,
///
///     bio: Option<String>,  // Optional fields
/// }
/// ```
///
/// # Available Validators
///
/// **Email Validators:**
/// - `#[email]` - Valid email format
/// - `#[no_public_domains]` - Reject gmail, yahoo, etc.
/// - `#[blocked_domains("a.com", "b.com")]` - Block specific domains
///
/// **Password Validators:**
/// - `#[password("strong")]` - 8+ chars, upper, lower, digit, special
/// - `#[password("medium")]` - 8+ chars, upper, lower, digit
/// - `#[password("basic")]` - 6+ chars
/// - `#[password(r"regex")]` - Custom regex pattern
///
/// **Numeric Validators:**
/// - `#[min(n)]` - Minimum value
/// - `#[max(n)]` - Maximum value
/// - `#[range(min, max)]` - Value range
///
/// **String Validators:**
/// - `#[min_length(n)]` - Minimum length
/// - `#[max_length(n)]` - Maximum length
/// - `#[length(min, max)]` - Length range
/// - `#[regex(r"pattern")]` - Custom regex
/// - `#[url]` - Valid URL format
///
/// **String Matching:**
/// - `#[contains("text")]` - String must contain substring
/// - `#[not_contains("text")]` - String must not contain substring
/// - `#[starts_with("prefix")]` - String must start with prefix
/// - `#[ends_with("suffix")]` - String must end with suffix
///
/// **Equality:**
/// - `#[equals("value")]` - Must equal exact value
/// - `#[not_equals("value")]` - Must not equal value
/// - `#[equals_field("other_field")]` - Must match another field
///
/// **Conditional:**
/// - `#[depends_on("field", "value")]` - Required when another field has specific value
///
/// **Collections:**
/// - `#[min_items(n)]` - Minimum number of items in Vec/HashSet
/// - `#[max_items(n)]` - Maximum number of items
/// - `#[unique]` - All items must be unique
///
/// **Enum/Values:**
/// - `#[enum_variant("val1", "val2")]` - Must be one of allowed values
///
/// **Custom:**
/// - `#[custom("func_name")]` - Call custom validation function
/// - `#[message = "text"]` - Override default error message
/// - `#[label("Name")]` - Use friendly name in errors
/// - `#[message_key("key")]` - i18n message key
///
/// **General:**
/// - `#[required]` - Required for Option<T> fields
/// - `#[allow_whitespace]` - Don't trim whitespace
///
#[proc_macro_derive(
    Validate,
    attributes(
        email,
        no_public_domains,
        blocked_domains,
        password,
        min,
        max,
        range,
        min_length,
        max_length,
        length,
        regex,
        url,
        allow_whitespace,
        required,
        contains,
        not_contains,
        starts_with,
        ends_with,
        equals,
        not_equals,
        equals_field,
        depends_on,
        min_items,
        max_items,
        unique,
        enum_variant,
        message,
        label,
        message_key,
        custom,
        query,
        form,
        path
    )
)]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    validation::impl_validate(&input).into()
}

/// Derive macro for automatic form field generation with validation attributes
///
/// Generates a `FormField` trait implementation that provides HTML5 validation attributes
/// and data-validate JSON for client-side validation, automatically derived from the
/// validation attributes on your struct.
///
/// This macro should be used together with `#[derive(Validate)]` to create a single
/// source of truth for validation rules.
///
/// # Example
///
/// ```ignore
/// use rhtmx::{Validate, FormField};
/// use serde::Deserialize;
///
/// #[derive(Validate, FormField, Deserialize)]
/// struct RegisterForm {
///     #[email]
///     #[no_public_domains]
///     #[required]
///     email: String,
///
///     #[min_length(8)]
///     #[password("strong")]
///     password: String,
///
///     #[min(18)]
///     #[max(120)]
///     age: i32,
/// }
///
/// // In your handler:
/// let form = RegisterForm { ... };
///
/// // Get attributes for a specific field
/// let email_attrs = form.field_attrs("email");
/// // email_attrs.html5_attrs contains: {"type": "email", "required": ""}
/// // email_attrs.data_validate contains: {"email":true,"noPublicDomains":true,"required":true}
///
/// // Render in HTML:
/// html! {
///     <input name="email" {email_attrs.render_all()} />
/// }
/// // Outputs: <input name="email" type="email" required data-validate='{"email":true,"noPublicDomains":true,"required":true}' />
/// ```
///
/// # Generated Methods
///
/// - `field_attrs(&self, field_name: &str) -> FieldAttrs` - Get attributes for a specific field
/// - `field_names(&self) -> Vec<&'static str>` - Get list of all field names
///
/// # Field Attributes
///
/// The following validation attributes are converted to HTML5 and data-validate:
///
/// - `#[email]` → `type="email"` + `"email": true`
/// - `#[required]` → `required` + `"required": true`
/// - `#[min_length(n)]` → `minlength="n"` + `"minLength": n`
/// - `#[max_length(n)]` → `maxlength="n"` + `"maxLength": n`
/// - `#[min(n)]` → `min="n"` + `"min": n`
/// - `#[max(n)]` → `max="n"` + `"max": n`
/// - `#[url]` → `type="url"` + `"url": true`
/// - `#[regex(pattern)]` → `pattern="..."` + `"pattern": "..."`
/// - And many more (see Validate documentation)
///
#[proc_macro_derive(
    FormField,
    attributes(
        email,
        no_public_domains,
        blocked_domains,
        password,
        min,
        max,
        range,
        min_length,
        max_length,
        length,
        regex,
        url,
        allow_whitespace,
        required,
        contains,
        not_contains,
        starts_with,
        ends_with,
        equals,
        not_equals,
        equals_field,
        depends_on,
        min_items,
        max_items,
        unique,
        enum_variant,
        message,
        label,
        message_key,
        custom,
        query,
        form,
        path
    )
)]
pub fn derive_form_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    validation::impl_form_field(&input).into()
}
