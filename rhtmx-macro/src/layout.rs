// File: rhtml-macro/src/layout.rs
// Purpose: Implement #[layout] macro for layout functions

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct, File};

use crate::layout_registry::SlotField;

/// Process #[layout] attribute macro
///
/// Marks a function as a layout function. The function should accept
/// a LayoutSlots parameter containing the slots to be rendered.
///
/// Example:
/// ```ignore
/// #[layout]
/// pub fn layout(slots: LayoutSlots) {
///     <html>...</html>
/// }
/// ```
///
/// Note: This is part of future layout system infrastructure
#[allow(dead_code)]
pub fn process_layout_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let layout_fn = parse_macro_input!(item as ItemFn);

    // Simply return the function as-is
    // The layout function will be processed by the RHTML parser
    let output = quote! {
        #layout_fn
    };

    output.into()
}

/// Parse LayoutSlots struct from file content
///
/// Looks for: pub struct LayoutSlots { ... }
/// Extracts field names, types, and whether they're optional
#[allow(dead_code)]
fn parse_layout_slots(file_content: &str) -> Result<Vec<SlotField>, String> {
    // Parse the entire file
    let file: File = syn::parse_str(file_content)
        .map_err(|e| format!("Failed to parse file: {}", e))?;

    // Find LayoutSlots struct
    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            if item_struct.ident == "LayoutSlots" {
                return extract_slot_fields(&item_struct);
            }
        }
    }

    Err("LayoutSlots struct not found in layout file".to_string())
}

/// Extract slot fields from LayoutSlots struct
fn extract_slot_fields(struct_item: &ItemStruct) -> Result<Vec<SlotField>, String> {
    let mut slots = Vec::new();

    if let syn::Fields::Named(fields) = &struct_item.fields {
        for field in &fields.named {
            let name = field
                .ident
                .as_ref()
                .ok_or("Field has no name")?
                .to_string();

            let type_str = quote!(#(field.ty)).to_string();

            // Check if field is Option<T>
            let is_optional = is_option_type(&field.ty);

            // Check if this is the content slot
            let is_content = name == "content";

            slots.push(SlotField {
                name,
                type_str,
                is_optional,
                is_content,
            });
        }
    }

    Ok(slots)
}

/// Check if a type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            return segment.ident == "Option";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_layout_slots() {
        let content = r#"
            pub struct LayoutSlots {
                pub content: impl Render,
                pub title: &str,
                pub description: Option<&str>,
            }
        "#;

        let slots = parse_layout_slots(content).unwrap();
        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0].name, "content");
        assert!(slots[0].is_content);
        assert_eq!(slots[1].name, "title");
        assert!(!slots[1].is_optional);
        assert_eq!(slots[2].name, "description");
        assert!(slots[2].is_optional);
    }
}
