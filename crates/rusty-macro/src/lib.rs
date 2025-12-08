// Procedural macros for rusty-sync

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for Syncable trait
///
/// # Example
///
/// ```ignore
/// #[derive(Syncable)]
/// #[sync(table = "users")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(Syncable, attributes(sync))]
pub fn derive_syncable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract the table name from #[sync(table = "...")] or use struct name
    let table_name = extract_table_name(&input).unwrap_or_else(|| {
        // Convert CamelCase to snake_case
        let struct_name = name.to_string();
        to_snake_case(&struct_name)
    });

    // Find the id field (must be present)
    let id_field = find_id_field(&input).expect("Syncable structs must have an 'id' field");

    // Check if the struct has version and modified_at fields
    let has_version = has_field(&input, "version") || has_field(&input, "_version");
    let has_modified_at = has_field(&input, "modified_at") || has_field(&input, "_modified_at");

    // Generate the trait implementation
    let version_impl = if has_version {
        quote! {
            fn version(&self) -> Option<i64> {
                self.version.or(self._version)
            }

            fn set_version(&mut self, version: i64) {
                if let Some(v) = self.version.as_mut() {
                    *v = version;
                } else if let Some(v) = self._version.as_mut() {
                    *v = version;
                }
            }
        }
    } else {
        quote! {}
    };

    let modified_at_impl = if has_modified_at {
        quote! {
            fn modified_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                self.modified_at.or(self._modified_at)
            }

            fn set_modified_at(&mut self, timestamp: chrono::DateTime<chrono::Utc>) {
                if let Some(ts) = self.modified_at.as_mut() {
                    *ts = timestamp;
                } else if let Some(ts) = self._modified_at.as_mut() {
                    *ts = timestamp;
                }
            }
        }
    } else {
        quote! {}
    };

    let has_metadata = has_version || has_modified_at;

    let expanded = quote! {
        impl #impl_generics rusty_sync::Syncable for #name #ty_generics #where_clause {
            fn entity_name() -> &'static str {
                #table_name
            }

            fn id(&self) -> String {
                self.#id_field.to_string()
            }

            #version_impl

            #modified_at_impl

            fn has_sync_metadata() -> bool {
                #has_metadata
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract table name from #[sync(table = "...")] attribute
fn extract_table_name(input: &DeriveInput) -> Option<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("sync") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args::<syn::Meta>() {
                if nv.path.is_ident("table") {
                    if let syn::Expr::Lit(lit) = nv.value {
                        if let syn::Lit::Str(s) = lit.lit {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Find the id field in the struct
fn find_id_field(input: &DeriveInput) -> Option<syn::Ident> {
    if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    if ident == "id" {
                        return Some(ident.clone());
                    }
                }
            }
        }
    }
    None
}

/// Check if a struct has a specific field
fn has_field(input: &DeriveInput, field_name: &str) -> bool {
    if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            return fields
                .named
                .iter()
                .any(|f| f.ident.as_ref().map(|i| i == field_name).unwrap_or(false));
        }
    }
    false
}

/// Convert CamelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}
