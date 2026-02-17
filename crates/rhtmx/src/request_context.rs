use axum::http::{HeaderMap, Method};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Request context passed to handlers and templates
#[derive(Clone)]
pub struct RequestContext {
    pub method: Method,
    pub query: QueryParams,
    pub form: FormData,
    pub headers: HeaderMap,
    pub cookies: HashMap<String, String>,
    pub path: String,
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
    pub fn new(
        method: Method,
        path: String,
        query: QueryParams,
        form: FormData,
        headers: HeaderMap,
    ) -> Self {
        let cookies = Self::parse_cookies(&headers);
        Self { method, query, form, headers, cookies, path }
    }

    fn parse_cookies(headers: &HeaderMap) -> HashMap<String, String> {
        headers
            .get("cookie")
            .and_then(|h| h.to_str().ok())
            .map(|s| {
                s.split(';')
                    .filter_map(|c| {
                        c.trim().split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_cookie(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)?.to_str().ok()
    }

    pub fn accepts_json(&self) -> bool {
        self.get_header("accept")
            .map(|a| a.contains("application/json"))
            .unwrap_or(false)
    }

    pub fn wants_partial(&self) -> bool {
        self.query.get("partial") == Some(&"true".to_string())
            || self.get_header("hx-request").is_some()
            || self.get_header("x-partial").is_some()
    }

    pub fn is_htmx(&self) -> bool {
        self.get_header("hx-request").is_some()
    }

    pub fn htmx_target(&self) -> Option<&str> {
        self.get_header("hx-target")
    }

    pub fn htmx_trigger(&self) -> Option<&str> {
        self.get_header("hx-trigger")
    }

    pub fn is_get(&self) -> bool { self.method == Method::GET }
    pub fn is_post(&self) -> bool { self.method == Method::POST }
    pub fn is_put(&self) -> bool { self.method == Method::PUT }
    pub fn is_delete(&self) -> bool { self.method == Method::DELETE }
}

/// Query parameters from URL
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    params: HashMap<String, String>,
}

impl QueryParams {
    pub fn new(params: HashMap<String, String>) -> Self {
        Self { params }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn get_as<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.params.get(key)?.parse().ok()
    }

    pub fn has(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.params
    }
}

/// Form data from POST/PUT requests
#[derive(Debug, Clone, Default)]
pub struct FormData {
    pub fields: HashMap<String, String>,
    pub raw_json: Option<JsonValue>,
}

impl FormData {
    pub fn new() -> Self {
        Self { fields: HashMap::new(), raw_json: None }
    }

    pub fn from_fields(fields: HashMap<String, String>) -> Self {
        Self {
            fields: fields.into_iter().map(|(k, v)| (k, v.trim().to_string())).collect(),
            raw_json: None,
        }
    }

    pub fn from_json(json: JsonValue) -> Self {
        let fields = if let JsonValue::Object(map) = &json {
            map.iter()
                .map(|(key, value)| {
                    let v = value.as_str().map(|s| s.trim().to_string()).unwrap_or_else(|| value.to_string());
                    (key.clone(), v)
                })
                .collect()
        } else {
            HashMap::new()
        };
        Self { fields, raw_json: Some(json) }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(key)
    }

    pub fn get_as<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.fields.get(key)?.parse().ok()
    }

    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    pub fn json(&self) -> Option<&JsonValue> {
        self.raw_json.as_ref()
    }

    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.fields
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.raw_json.is_none()
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
    }

    #[test]
    fn test_form_data_trimming() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "  John  ".to_string());
        let form = FormData::from_fields(fields);
        assert_eq!(form.get("name"), Some(&"John".to_string()));
    }

    #[test]
    fn test_form_data_json_parsing() {
        let json = serde_json::json!({"name": "Alice", "age": 30});
        let form = FormData::from_json(json.clone());
        assert_eq!(form.get("name"), Some(&"Alice".to_string()));
        assert_eq!(form.get("age"), Some(&"30".to_string()));
        assert_eq!(form.json(), Some(&json));
    }

    #[test]
    fn test_query_params() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "1".to_string());
        let query = QueryParams::new(params);
        assert!(query.has("page"));
        assert_eq!(query.get_as::<i32>("page"), Some(1));
    }

    #[test]
    fn test_request_context_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "session=abc123; user=john".parse().unwrap());
        let cookies = RequestContext::parse_cookies(&headers);
        assert_eq!(cookies.get("session"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user"), Some(&"john".to_string()));
    }

    #[test]
    fn test_wants_partial() {
        let form = FormData::new();
        let mut params = HashMap::new();
        params.insert("partial".to_string(), "true".to_string());
        let query = QueryParams::new(params);
        let headers = HeaderMap::new();
        let ctx = RequestContext::new(Method::GET, "/test".to_string(), query, form, headers);
        assert!(ctx.wants_partial());
    }
}
