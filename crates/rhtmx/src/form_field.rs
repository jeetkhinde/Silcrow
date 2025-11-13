// File: rhtmx/src/form_field.rs
// Purpose: Form field generation with validation attributes from struct metadata

use std::collections::HashMap;

/// Metadata for a form field including HTML5 and client-side validation attributes
#[derive(Debug, Clone)]
pub struct FieldAttrs {
    /// HTML5 native attributes (e.g., "required", "minlength", "type")
    pub html5_attrs: HashMap<String, String>,
    /// JSON string for data-validate attribute (client-side WASM validation)
    pub data_validate: String,
    /// Field label for display
    pub label: String,
}

impl FieldAttrs {
    /// Create a new FieldAttrs with default values
    pub fn new() -> Self {
        Self {
            html5_attrs: HashMap::new(),
            data_validate: "{}".to_string(),
            label: String::new(),
        }
    }

    /// Render HTML5 attributes as a string
    pub fn render_html5_attrs(&self) -> String {
        self.html5_attrs
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    k.clone()
                } else {
                    format!("{}=\"{}\"", k, v)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Render data-validate attribute
    pub fn render_data_validate(&self) -> String {
        format!("data-validate='{}'", self.data_validate)
    }

    /// Render all attributes (HTML5 + data-validate)
    pub fn render_all(&self) -> String {
        let html5 = self.render_html5_attrs();
        let validate = self.render_data_validate();

        if html5.is_empty() {
            validate
        } else {
            format!("{} {}", html5, validate)
        }
    }
}

impl Default for FieldAttrs {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for structs that can generate form fields with validation
///
/// This trait is automatically implemented when you derive `Validate` and `FormField`.
/// It provides methods to get field attributes that include both HTML5 validation
/// and data-validate JSON for client-side validation.
///
/// # Example
///
/// ```ignore
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
/// }
///
/// // In your template:
/// let form = RegisterForm { ... };
/// let email_attrs = form.field_attrs("email");
///
/// html! {
///     <input name="email" {email_attrs.render_all()} />
/// }
/// // Renders:
/// // <input name="email" type="email" required data-validate='{"email":true,"noPublicDomains":true,"required":true}' />
/// ```
pub trait FormField {
    /// Get field attributes for the specified field name
    fn field_attrs(&self, field_name: &str) -> FieldAttrs;

    /// Get all field names
    fn field_names(&self) -> Vec<&'static str>;
}
