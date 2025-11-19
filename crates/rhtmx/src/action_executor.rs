// File: src/action_executor.rs
// Purpose: Execute action functions and handle parameter binding and validation

use crate::request_context::FormData;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

/// Result of executing an action
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Successful response with HTML content
    Html {
        content: String,
        headers: HeaderMap,
    },
    /// Validation errors - re-render form with errors
    ValidationError {
        form_data: FormData,
        original_content: String,
    },
    /// Error response
    Error {
        status: u16,
        message: String,
    },
    /// Empty response (e.g., for DELETE)
    Empty {
        headers: HeaderMap,
    },
}

impl IntoResponse for ActionResult {
    fn into_response(self) -> Response {
        match self {
            ActionResult::Html { content, headers } => {
                let mut response = Html(content).into_response();
                *response.headers_mut() = headers;
                response
            }
            ActionResult::Empty { headers } => {
                let mut response = "".into_response();
                *response.headers_mut() = headers;
                response
            }
            ActionResult::ValidationError {
                form_data: _form_data,
                original_content,
            } => {
                // Store errors in form data and re-render template with errors
                // For now, return the original content with error information
                let mut response = Html(original_content).into_response();
                if let Ok(header_value) = "true".parse() {
                    response.headers_mut().insert("X-Validation-Failed", header_value);
                }
                response
            }
            ActionResult::Error { status, message } => {
                let response = Html(format!(
                    "<div class='error'><h1>Error {}</h1><p>{}</p></div>",
                    status, message
                ));
                let mut resp = response.into_response();
                *resp.status_mut() = axum::http::StatusCode::from_u16(status)
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                resp
            }
        }
    }
}

/// Pure function to parse a string value into appropriate JSON type
fn parse_value_to_json(value: &str) -> JsonValue {
    // Try parsing as i32 first (most common integer type)
    value.parse::<i32>()
        .map(|num| json!(num))
        .or_else(|_| {
            // If i32 fails, try f64
            value.parse::<f64>().map(|num| json!(num))
        })
        .unwrap_or_else(|_| {
            // If both numeric types fail, keep as string
            json!(value)
        })
}

/// Helper to convert form data to JSON for deserialization
/// Uses functional programming approach with iterator methods
pub fn form_to_json(form_data: &FormData) -> JsonValue {
    let map: serde_json::Map<String, JsonValue> = form_data
        .as_map()
        .iter()
        .map(|(key, value)| (key.clone(), parse_value_to_json(value)))
        .collect();

    JsonValue::Object(map)
}

/// Helper to deserialize form data into a typed struct
pub fn deserialize_form<T: serde::de::DeserializeOwned>(
    form_data: &FormData,
) -> Result<T, serde_json::Error> {
    // If we have raw JSON, use that directly
    form_data
        .json()
        .map(|raw_json| serde_json::from_value(raw_json.clone()))
        .unwrap_or_else(|| {
            // Otherwise, reconstruct from fields
            let json = form_to_json(form_data);
            serde_json::from_value(json)
        })
}

/// Helper to validate a struct using the Validate trait
pub fn validate_request<T: crate::validation::Validate>(
    request: &T,
) -> Result<(), HashMap<String, Vec<String>>> {
    request.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestUser {
        name: String,
        age: i32,
    }

    #[test]
    fn test_form_to_json() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("age".to_string(), "30".to_string());

        let form = FormData::from_fields(fields);
        let json = form_to_json(&form);

        assert_eq!(json["name"], "John");
        assert_eq!(json["age"], 30);
    }

    #[test]
    fn test_deserialize_from_fields() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("age".to_string(), "30".to_string());

        let form = FormData::from_fields(fields);
        let user: TestUser = deserialize_form(&form).expect("Failed to deserialize");
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json_value = serde_json::json!({
            "name": "Jane",
            "age": 25
        });

        let form = FormData::from_json(json_value);
        let user: TestUser = deserialize_form(&form).expect("Failed to deserialize");
        assert_eq!(user.name, "Jane");
        assert_eq!(user.age, 25);
    }

    #[test]
    fn test_parse_value_to_json_integer() {
        assert_eq!(parse_value_to_json("42"), json!(42));
        assert_eq!(parse_value_to_json("-100"), json!(-100));
    }

    #[test]
    fn test_parse_value_to_json_float() {
        assert_eq!(parse_value_to_json("3.14"), json!(3.14));
        assert_eq!(parse_value_to_json("99.99"), json!(99.99));
    }

    #[test]
    fn test_parse_value_to_json_string() {
        assert_eq!(parse_value_to_json("hello"), json!("hello"));
        assert_eq!(parse_value_to_json(""), json!(""));
        assert_eq!(parse_value_to_json("not_a_number"), json!("not_a_number"));
    }
}
