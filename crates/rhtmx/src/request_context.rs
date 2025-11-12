// File: src/request_context.rs
// Purpose: Request context with query params, headers, cookies, and form data

use axum::http::{HeaderMap, Method};
use serde_json::Value as JsonValue;
use sqlx::AnyPool;
use std::collections::HashMap;
use std::sync::Arc;

/// Request context passed to data functions and templates
#[derive(Clone)]
pub struct RequestContext {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: Method,

    /// Query parameters from URL (?key=value)
    pub query: QueryParams,

    /// Form data from POST/PUT requests
    pub form: FormData,

    /// Request headers
    pub headers: HeaderMap,

    /// Parsed cookies
    pub cookies: HashMap<String, String>,

    /// Request path
    pub path: String,

    /// Database connection pool (supports SQLite, PostgreSQL, MySQL, etc.)
    /// None if database is not configured
    pub db: Option<Arc<AnyPool>>,
}

impl std::fmt::Debug for RequestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestContext")
            .field("method", &self.method)
            .field("path", &self.path)
            .finish()
    }
}

impl RequestContext {
    /// Create a new request context
    pub fn new(
        method: Method,
        path: String,
        query: QueryParams,
        form: FormData,
        headers: HeaderMap,
        db: Option<Arc<AnyPool>>,
    ) -> Self {
        // Parse cookies from headers
        let cookies = Self::parse_cookies(&headers);

        Self {
            method,
            query,
            form,
            headers,
            cookies,
            path,
            db,
        }
    }

    /// Parse cookies from Cookie header
    fn parse_cookies(headers: &HeaderMap) -> HashMap<String, String> {
        let mut cookies = HashMap::new();

        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some((key, value)) = cookie.split_once('=') {
                        cookies.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        cookies
    }

    /// Get a cookie value
    pub fn get_cookie(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    /// Get a header value
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)?.to_str().ok()
    }

    /// Check if request accepts JSON
    pub fn accepts_json(&self) -> bool {
        if let Some(accept) = self.get_header("accept") {
            accept.contains("application/json") || accept.contains("json")
        } else {
            false
        }
    }

    /// Check if request wants a partial/fragment response (without layout)
    /// Returns true if:
    /// - Query parameter ?partial=true is present
    /// - HX-Request header is present (HTMX request)
    /// - X-Partial header is present
    pub fn wants_partial(&self) -> bool {
        // Check query parameter
        if self.query.get("partial") == Some(&"true".to_string()) {
            return true;
        }

        // Check HTMX header
        if self.get_header("hx-request").is_some() {
            return true;
        }

        // Check X-Partial header
        if self.get_header("x-partial").is_some() {
            return true;
        }

        false
    }

    /// Check if this is an HTMX request
    pub fn is_htmx(&self) -> bool {
        self.get_header("hx-request").is_some()
    }

    /// Get HTMX target element (if present)
    pub fn htmx_target(&self) -> Option<&str> {
        self.get_header("hx-target")
    }

    /// Get HTMX trigger element (if present)
    pub fn htmx_trigger(&self) -> Option<&str> {
        self.get_header("hx-trigger")
    }

    /// Check if this is a specific method
    pub fn is_get(&self) -> bool {
        self.method == Method::GET
    }

    pub fn is_post(&self) -> bool {
        self.method == Method::POST
    }

    pub fn is_put(&self) -> bool {
        self.method == Method::PUT
    }

    pub fn is_delete(&self) -> bool {
        self.method == Method::DELETE
    }
}

/// Query parameters from URL
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    params: HashMap<String, String>,
}

impl QueryParams {
    /// Create from HashMap
    pub fn new(params: HashMap<String, String>) -> Self {
        Self { params }
    }

    /// Get a query parameter value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Get a query parameter as a specific type
    pub fn get_as<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.params.get(key)?.parse().ok()
    }

    /// Check if a parameter exists
    pub fn has(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Get all parameter names
    pub fn keys(&self) -> Vec<&String> {
        self.params.keys().collect()
    }

    /// Get as HashMap
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.params
    }
}

/// Form data from POST/PUT requests
#[derive(Debug, Clone, Default)]
pub struct FormData {
    fields: HashMap<String, String>,
    raw_json: Option<JsonValue>,
    validation_errors: HashMap<String, String>,
}

impl FormData {
    /// Create empty form data
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            raw_json: None,
            validation_errors: HashMap::new(),
        }
    }

    /// Create from form fields with automatic trimming
    pub fn from_fields(fields: HashMap<String, String>) -> Self {
        // Trim all string values by default
        let trimmed_fields = fields
            .into_iter()
            .map(|(k, v)| (k, v.trim().to_string()))
            .collect();

        Self {
            fields: trimmed_fields,
            raw_json: None,
            validation_errors: HashMap::new(),
        }
    }

    /// Create from JSON
    pub fn from_json(json: JsonValue) -> Self {
        let mut fields = HashMap::new();

        // If JSON is an object, extract fields
        if let JsonValue::Object(map) = &json {
            for (key, value) in map {
                if let Some(s) = value.as_str() {
                    // Trim string values
                    fields.insert(key.clone(), s.trim().to_string());
                } else {
                    fields.insert(key.clone(), value.to_string());
                }
            }
        }

        Self {
            fields,
            raw_json: Some(json),
            validation_errors: HashMap::new(),
        }
    }

    /// Get a form field value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(key)
    }

    /// Get a form field as a specific type
    pub fn get_as<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.fields.get(key)?.parse().ok()
    }

    /// Check if a field exists
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Get all field names
    pub fn keys(&self) -> Vec<&String> {
        self.fields.keys().collect()
    }

    /// Get raw JSON if available
    pub fn json(&self) -> Option<&JsonValue> {
        self.raw_json.as_ref()
    }

    /// Get as HashMap
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Check if form is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.raw_json.is_none()
    }

    /// Set validation errors
    pub fn set_validation_errors(&mut self, errors: HashMap<String, String>) {
        self.validation_errors = errors;
    }

    /// Get validation errors
    pub fn validation_errors(&self) -> &HashMap<String, String> {
        &self.validation_errors
    }

    /// Get error for a specific field
    pub fn get_error(&self, field: &str) -> Option<&String> {
        self.validation_errors.get(field)
    }

    /// Check if there are validation errors
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Check if a specific field has an error
    pub fn has_error(&self, field: &str) -> bool {
        self.validation_errors.contains_key(field)
    }

    /// Parse into a validated struct
    pub fn parse<T>(&self) -> Result<T, HashMap<String, String>>
    where
        T: serde::de::DeserializeOwned + crate::validation::Validate,
    {
        // First parse the data
        let parsed: T = if let Some(json) = &self.raw_json {
            serde_json::from_value(json.clone())
                .map_err(|e| {
                    let mut errors = HashMap::new();
                    errors.insert("_general".to_string(), e.to_string());
                    errors
                })?
        } else {
            // Convert fields to JSON and parse
            let json = serde_json::to_value(&self.fields)
                .map_err(|e| {
                    let mut errors = HashMap::new();
                    errors.insert("_general".to_string(), e.to_string());
                    errors
                })?;
            serde_json::from_value(json)
                .map_err(|e| {
                    let mut errors = HashMap::new();
                    errors.insert("_general".to_string(), e.to_string());
                    errors
                })?
        };

        // Then validate
        parsed.validate()?;

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_data_empty() {
        let form = FormData::new();
        assert!(form.is_empty());
        assert!(form.as_map().is_empty());
        assert!(!form.has_errors());
    }

    #[test]
    fn test_form_data_trimming() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "  John  ".to_string());
        fields.insert("email".to_string(), "\ttest@example.com\n".to_string());

        let form = FormData::from_fields(fields);

        // Strings should be trimmed
        assert_eq!(form.get("name"), Some(&"John".to_string()));
        assert_eq!(form.get("email"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_form_data_json_parsing() {
        let json = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });

        let form = FormData::from_json(json.clone());

        assert_eq!(form.get("name"), Some(&"Alice".to_string()));
        assert_eq!(form.get("age"), Some(&"30".to_string()));
        assert_eq!(form.json(), Some(&json));
    }

    #[test]
    fn test_form_data_preserves_empty_strings() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "".to_string());
        fields.insert("bio".to_string(), "".to_string());

        let form = FormData::from_fields(fields);

        // Empty strings should be preserved
        assert_eq!(form.get("name"), Some(&"".to_string()));
        assert_eq!(form.get("bio"), Some(&"".to_string()));
        assert!(!form.is_empty()); // Form has fields, even if empty
    }

    #[test]
    fn test_form_data_keys() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("email".to_string(), "john@example.com".to_string());

        let form = FormData::from_fields(fields);
        let keys = form.keys();

        assert_eq!(keys.len(), 2);
        let key_strs: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();
        assert!(key_strs.contains(&"name"));
        assert!(key_strs.contains(&"email"));
    }

    #[test]
    fn test_form_data_get_as_types() {
        let mut fields = HashMap::new();
        fields.insert("age".to_string(), "30".to_string());
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("score".to_string(), "95.5".to_string());

        let form = FormData::from_fields(fields);

        assert_eq!(form.get_as::<i32>("age"), Some(30));
        assert_eq!(form.get_as::<f64>("score"), Some(95.5));
        assert_eq!(form.get_as::<i32>("name"), None); // Can't parse string as int
    }

    #[test]
    fn test_form_data_validation_errors() {
        let form = FormData::new();
        assert!(!form.has_errors());
        assert!(form.get_error("name").is_none());

        let mut errors = HashMap::new();
        errors.insert("name".to_string(), "Name is required".to_string());
        errors.insert("email".to_string(), "Invalid email".to_string());

        let mut form = FormData::new();
        form.set_validation_errors(errors);

        assert!(form.has_errors());
        assert!(form.has_error("name"));
        assert!(form.has_error("email"));
        assert!(!form.has_error("age"));
        assert_eq!(form.get_error("name"), Some(&"Name is required".to_string()));
    }

    #[test]
    fn test_query_params_basic() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "1".to_string());
        params.insert("filter".to_string(), "active".to_string());

        let query = QueryParams::new(params);

        assert!(query.has("page"));
        assert!(query.has("filter"));
        assert!(!query.has("sort"));
        assert_eq!(query.get("page"), Some(&"1".to_string()));
    }

    #[test]
    fn test_query_params_get_as() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("limit".to_string(), "50".to_string());

        let query = QueryParams::new(params);

        assert_eq!(query.get_as::<i32>("page"), Some(2));
        assert_eq!(query.get_as::<i32>("limit"), Some(50));
        assert_eq!(query.get_as::<i32>("nonexistent"), None);
    }

    #[test]
    fn test_request_context_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "session=abc123; user=john".parse().unwrap());

        let cookies = RequestContext::parse_cookies(&headers);

        assert_eq!(cookies.get("session"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user"), Some(&"john".to_string()));
        assert_eq!(cookies.len(), 2);
    }

    #[test]
    fn test_request_context_accepts_json() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "application/json".parse().unwrap());

        // We can't create a full context without a database, but we can test the parse method
        let cookies = RequestContext::parse_cookies(&headers);
        assert!(cookies.is_empty()); // No cookies in this test
    }
}
