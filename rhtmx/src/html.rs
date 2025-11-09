// File: src/html.rs
// Purpose: Html type and response builders for the html! macro

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use std::fmt;

/// Html wrapper type for compile-time generated HTML
///
/// This type is returned by functions using the html! macro.
/// It provides type safety and ensures functions return valid HTML.
#[derive(Clone, Debug, PartialEq)]
pub struct Html(pub String);

impl Html {
    /// Create a new Html instance
    pub fn new(content: impl Into<String>) -> Self {
        Self(content.into())
    }

    /// Get the inner HTML string
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get a reference to the inner HTML string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Html {
    fn from(s: String) -> Self {
        Html(s)
    }
}

impl From<&str> for Html {
    fn from(s: &str) -> Self {
        Html(s.to_string())
    }
}

impl IntoResponse for Html {
    fn into_response(self) -> Response {
        (
            [(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            )],
            self.0,
        )
            .into_response()
    }
}

/// Ok response builder for action handlers
///
/// # Example
/// ```ignore
/// #[post]
/// fn create_user(req: CreateUserRequest) {
///     let user = db.create_user(req)?;
///     Ok()
///         .render(user_card, &user)
///         .toast("User created!")
/// }
/// ```
#[derive(Debug)]
pub struct OkResponse {
    content: Option<Html>,
    headers: HeaderMap,
    toast_message: Option<String>,
    oob_updates: Vec<(String, Html)>,
    status: StatusCode,
}

impl OkResponse {
    /// Create a new Ok response
    pub fn new() -> Self {
        Self {
            content: None,
            headers: HeaderMap::new(),
            toast_message: None,
            oob_updates: Vec::new(),
            status: StatusCode::OK,
        }
    }

    /// Render a component with data
    ///
    /// The function must return Html (typically using html! macro)
    pub fn render<F, P>(mut self, func: F, props: P) -> Self
    where
        F: FnOnce(P) -> Html,
    {
        self.content = Some(func(props));
        self
    }

    /// Render HTML directly
    pub fn render_html(mut self, html: Html) -> Self {
        self.content = Some(html);
        self
    }

    /// Add a toast notification
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.toast_message = Some(message.into());
        self
    }

    /// Add an out-of-band update with a component
    pub fn render_oob<F, P>(mut self, target: impl Into<String>, func: F, props: P) -> Self
    where
        F: FnOnce(P) -> Html,
    {
        self.oob_updates.push((target.into(), func(props)));
        self
    }

    /// Add an out-of-band update with raw HTML
    pub fn oob(mut self, target: impl Into<String>, html: Html) -> Self {
        self.oob_updates.push((target.into(), html));
        self
    }

    /// Add an out-of-band update with a simple value (for backward compatibility)
    pub fn oob_value<T: ToString>(mut self, target: impl Into<String>, value: T) -> Self {
        self.oob_updates.push((target.into(), Html(value.to_string())));
        self
    }

    /// Add a custom header
    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let std::result::Result::Ok(header_name) = axum::http::HeaderName::from_bytes(key.as_ref().as_bytes()) {
            if let std::result::Result::Ok(header_value) = HeaderValue::from_str(value.as_ref()) {
                self.headers.insert(header_name, header_value);
            }
        }
        self
    }

    /// Set the HTTP status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Build the final response
    pub fn build(self) -> (StatusCode, HeaderMap, String) {
        let mut headers = self.headers;

        // Add HX-Trigger header for toast
        if let Some(message) = self.toast_message {
            let trigger = serde_json::json!({
                "showToast": {
                    "message": message
                }
            });
            if let std::result::Result::Ok(value) = HeaderValue::from_str(&trigger.to_string()) {
                headers.insert("HX-Trigger", value);
            }
        }

        // Build content with OOB updates
        let mut content = String::new();

        // Add main content
        if let Some(html) = self.content {
            content.push_str(&html.0);
        }

        // Add OOB updates
        for (target, html) in self.oob_updates {
            content.push_str(&format!(
                r#"<div id="{}" hx-swap-oob="true">{}</div>"#,
                target, html.0
            ));
        }

        (self.status, headers, content)
    }
}

impl Default for OkResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        let (status, headers, content) = self.build();
        (status, headers, Html(content)).into_response()
    }
}

/// Convenient function to create an Ok response
#[allow(non_snake_case)]
pub fn Ok() -> OkResponse {
    OkResponse::new()
}

/// Convenient function to create an ok response (lowercase variant to avoid conflicts)
pub fn ok() -> OkResponse {
    OkResponse::new()
}

/// Error response builder for action handlers
///
/// # Example
/// ```ignore
/// #[post]
/// fn create_user(req: CreateUserRequest) {
///     let errors = validate(&req);
///     if !errors.is_empty() {
///         return Error()
///             .render(validation_errors, errors)
///             .status(StatusCode::BAD_REQUEST);
///     }
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct ErrorResponse {
    content: Option<Html>,
    headers: HeaderMap,
    message: Option<String>,
    status: StatusCode,
}

impl ErrorResponse {
    /// Create a new Error response
    pub fn new() -> Self {
        Self {
            content: None,
            headers: HeaderMap::new(),
            message: None,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Render an error component
    pub fn render<F, P>(mut self, func: F, props: P) -> Self
    where
        F: FnOnce(P) -> Html,
    {
        self.content = Some(func(props));
        self
    }

    /// Render HTML directly
    pub fn render_html(mut self, html: Html) -> Self {
        self.content = Some(html);
        self
    }

    /// Set error message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the HTTP status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add a custom header
    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let std::result::Result::Ok(header_name) = axum::http::HeaderName::from_bytes(key.as_ref().as_bytes()) {
            if let std::result::Result::Ok(header_value) = HeaderValue::from_str(value.as_ref()) {
                self.headers.insert(header_name, header_value);
            }
        }
        self
    }

    /// Build the final response
    pub fn build(self) -> (StatusCode, HeaderMap, String) {
        let content = if let Some(html) = self.content {
            html.0
        } else if let Some(msg) = self.message {
            format!("<div class=\"error\">{}</div>", msg)
        } else {
            String::from("<div class=\"error\">An error occurred</div>")
        };

        (self.status, self.headers, content)
    }
}

impl Default for ErrorResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let (status, headers, content) = self.build();
        (status, headers, Html(content)).into_response()
    }
}

/// Convenient function to create an Error response
#[allow(non_snake_case)]
pub fn Error() -> ErrorResponse {
    ErrorResponse::new()
}

/// Convenient function to create an error response (lowercase variant to avoid conflicts)
pub fn error() -> ErrorResponse {
    ErrorResponse::new()
}

/// Redirect response builder
///
/// # Example
/// ```ignore
/// #[post]
/// fn login(req: LoginRequest) {
///     if authenticate(&req) {
///         Redirect().to("/dashboard")
///     } else {
///         Error().message("Invalid credentials")
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RedirectResponse {
    location: Option<String>,
    toast_message: Option<String>,
    status: StatusCode,
}

impl RedirectResponse {
    /// Create a new Redirect response
    pub fn new() -> Self {
        Self {
            location: None,
            toast_message: None,
            status: StatusCode::SEE_OTHER, // 303 redirect
        }
    }

    /// Set the redirect location
    pub fn to(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Add a toast notification
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.toast_message = Some(message.into());
        self
    }

    /// Set the HTTP status code (301, 302, 303, 307, 308)
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Build the final response
    pub fn build(self) -> (StatusCode, HeaderMap, ()) {
        let mut headers = HeaderMap::new();

        // Add Location and HX-Redirect headers
        if let Some(ref location) = self.location {
            // Add Location header
            if let std::result::Result::Ok(value) = HeaderValue::from_str(location) {
                headers.insert(axum::http::header::LOCATION, value.clone());
                // Add HX-Redirect for HTMX requests
                headers.insert("HX-Redirect", value);
            }
        }

        // Add toast if present
        if let Some(message) = self.toast_message {
            let trigger = serde_json::json!({
                "showToast": {
                    "message": message
                }
            });
            if let std::result::Result::Ok(value) = HeaderValue::from_str(&trigger.to_string()) {
                headers.insert("HX-Trigger", value);
            }
        }

        (self.status, headers, ())
    }
}

impl Default for RedirectResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for RedirectResponse {
    fn into_response(self) -> Response {
        let (status, headers, _) = self.build();
        (status, headers).into_response()
    }
}

/// Convenient function to create a Redirect response
#[allow(non_snake_case)]
pub fn Redirect() -> RedirectResponse {
    RedirectResponse::new()
}

/// Convenient function to create a redirect response (lowercase variant to avoid conflicts)
pub fn redirect() -> RedirectResponse {
    RedirectResponse::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_type() {
        let html = Html::new("<div>Test</div>");
        assert_eq!(html.as_str(), "<div>Test</div>");
        assert_eq!(html.to_string(), "<div>Test</div>");
    }

    #[test]
    fn test_ok_response() {
        let response = Ok()
            .render_html(Html::new("<div>Content</div>"))
            .toast("Success!");

        let (status, headers, content) = response.build();

        assert_eq!(status, StatusCode::OK);
        assert!(headers.contains_key("HX-Trigger"));
        assert!(content.contains("<div>Content</div>"));
    }

    #[test]
    fn test_error_response() {
        let response = Error()
            .message("Something went wrong")
            .status(StatusCode::BAD_REQUEST);

        let (status, _, content) = response.build();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(content.contains("Something went wrong"));
    }

    #[test]
    fn test_redirect_response() {
        let response = Redirect()
            .to("/dashboard")
            .toast("Logged in!");

        let (status, headers, _) = response.build();

        assert_eq!(status, StatusCode::SEE_OTHER);
        assert!(headers.contains_key(axum::http::header::LOCATION));
        assert!(headers.contains_key("HX-Redirect"));
    }

    #[test]
    fn test_oob_updates() {
        let response = Ok()
            .render_html(Html::new("<div>Main</div>"))
            .oob("counter", Html::new("42"))
            .oob_value("status", "active");

        let (_, _, content) = response.build();

        assert!(content.contains("<div>Main</div>"));
        assert!(content.contains(r#"id="counter""#));
        assert!(content.contains("42"));
        assert!(content.contains(r#"id="status""#));
        assert!(content.contains("active"));
    }
}
