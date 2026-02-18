use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};

pub trait IntoHtml {
    fn into_html(self) -> String;
}

impl IntoHtml for maud::Markup {
    fn into_html(self) -> String {
        self.into_string()
    }
}

impl IntoHtml for String {
    fn into_html(self) -> String {
        self
    }
}

impl IntoHtml for &str {
    fn into_html(self) -> String {
        self.to_string()
    }
}

// -- Shared helpers --

fn insert_header(headers: &mut HeaderMap, key: &str, value: &str) {
    if let (Result::Ok(name), Result::Ok(val)) = (
        HeaderName::from_bytes(key.as_bytes()),
        HeaderValue::from_str(value),
    ) {
        headers.insert(name, val);
    }
}

// ============================================================================
// OkResponse — dispatcher with convenience constructors
// ============================================================================

pub struct OkResponse;

impl OkResponse {
    /// Create an HTML fragment response (for `s-html` requests).
    pub fn html(content: impl IntoHtml) -> HtmlOkResponse {
        HtmlOkResponse::new().html(content)
    }

    /// Create a JSON patch response (for default `s-action` requests).
    pub fn json() -> JsonOkResponse {
        JsonOkResponse::new()
    }
}

// ============================================================================
// HtmlOkResponse — HTML fragment for s-html requests
// ============================================================================

#[derive(Debug)]
pub struct HtmlOkResponse {
    content: Option<String>,
    headers: HeaderMap,
    status: StatusCode,
}

impl HtmlOkResponse {
    pub fn new() -> Self {
        Self {
            content: None,
            headers: HeaderMap::new(),
            status: StatusCode::OK,
        }
    }

    /// Set the response body. Accepts Maud Markup, String, or &str.
    pub fn html(mut self, content: impl IntoHtml) -> Self {
        self.content = Some(content.into_html());
        self
    }

    /// Add a custom response header.
    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        insert_header(&mut self.headers, key.as_ref(), value.as_ref());
        self
    }

    /// Set `silcrow-cache: no-cache` to prevent client-side caching.
    pub fn no_cache(mut self) -> Self {
        insert_header(&mut self.headers, "silcrow-cache", "no-cache");
        self
    }

    /// Set the HTTP status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl Default for HtmlOkResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for HtmlOkResponse {
    fn into_response(self) -> Response {
        let content = self.content.unwrap_or_default();
        (self.status, self.headers, Html(content)).into_response()
    }
}

// ============================================================================
// JsonOkResponse — JSON patch data for Silcrow.patch()
// ============================================================================

#[derive(Debug)]
pub struct JsonOkResponse {
    data: serde_json::Map<String, serde_json::Value>,
    headers: HeaderMap,
    status: StatusCode,
}

impl JsonOkResponse {
    pub fn new() -> Self {
        Self {
            data: serde_json::Map::new(),
            headers: HeaderMap::new(),
            status: StatusCode::OK,
        }
    }

    /// Insert a key-value pair into the JSON response.
    /// The value must implement `serde::Serialize`.
    pub fn set(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        let v = serde_json::to_value(value).expect("Failed to serialize value for JSON response");
        self.data.insert(key.into(), v);
        self
    }

    /// Insert a pre-built `serde_json::Value`.
    pub fn set_value(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Add a toast notification under the `_toast` key.
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.data.insert(
            "_toast".into(),
            serde_json::json!({ "message": message.into() }),
        );
        self
    }

    /// Add a custom response header.
    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        insert_header(&mut self.headers, key.as_ref(), value.as_ref());
        self
    }

    /// Set `silcrow-cache: no-cache` to prevent client-side caching.
    pub fn no_cache(mut self) -> Self {
        insert_header(&mut self.headers, "silcrow-cache", "no-cache");
        self
    }

    /// Set the HTTP status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl Default for JsonOkResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for JsonOkResponse {
    fn into_response(self) -> Response {
        let body = serde_json::Value::Object(self.data);
        (self.status, self.headers, axum::Json(body)).into_response()
    }
}

// ============================================================================
// ErrorResponse
// ============================================================================

#[derive(Debug)]
pub struct ErrorResponse {
    content: Option<String>,
    headers: HeaderMap,
    message: Option<String>,
    status: StatusCode,
}

impl ErrorResponse {
    pub fn new() -> Self {
        Self {
            content: None,
            headers: HeaderMap::new(),
            message: None,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Set the response body. Accepts Maud Markup, String, or &str.
    pub fn html(mut self, content: impl IntoHtml) -> Self {
        self.content = Some(content.into_html());
        self
    }

    /// Set error message (rendered as `<div class="error">…</div>` for HTML responses).
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the HTTP status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add a custom response header.
    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        insert_header(&mut self.headers, key.as_ref(), value.as_ref());
        self
    }

    /// Return the error as a JSON response instead of HTML.
    pub fn json(self) -> Response {
        let message = self.message.unwrap_or_else(|| "An error occurred".into());
        let body = serde_json::json!({ "error": message });
        (self.status, self.headers, axum::Json(body)).into_response()
    }
}

impl Default for ErrorResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let content = self
            .content
            .or_else(|| {
                self.message
                    .map(|msg| format!(r#"<div class="error">{}</div>"#, msg))
            })
            .unwrap_or_else(|| r#"<div class="error">An error occurred</div>"#.into());
        (self.status, self.headers, Html(content)).into_response()
    }
}

// ============================================================================
// RedirectResponse
// ============================================================================

#[derive(Debug)]
pub struct RedirectResponse {
    location: Option<String>,
    status: StatusCode,
}

impl RedirectResponse {
    pub fn new() -> Self {
        Self {
            location: None,
            status: StatusCode::SEE_OTHER,
        }
    }

    /// Set the redirect location.
    pub fn to(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set the HTTP status code (301, 302, 303, 307, 308).
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl Default for RedirectResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for RedirectResponse {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        if let Some(ref location) = self.location {
            let value = HeaderValue::from_str(location)
                .expect("Redirect location contains invalid characters for a header value");
            headers.insert(axum::http::header::LOCATION, value);
        }
        (self.status, headers).into_response()
    }
}

// ============================================================================
// Constructor functions
// ============================================================================

#[allow(non_snake_case)]
pub fn Ok() -> OkResponse {
    OkResponse
}

#[allow(non_snake_case)]
pub fn HtmlOk(content: impl IntoHtml) -> HtmlOkResponse {
    HtmlOkResponse::new().html(content)
}

#[allow(non_snake_case)]
pub fn JsonOk() -> JsonOkResponse {
    JsonOkResponse::new()
}

#[allow(non_snake_case)]
pub fn Error() -> ErrorResponse {
    ErrorResponse::new()
}

#[allow(non_snake_case)]
pub fn Redirect() -> RedirectResponse {
    RedirectResponse::new()
}
