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

fn insert_toast(headers: &mut HeaderMap, message: &str) {
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    let json = format!(r#"{{"showToast":{{"message":"{}"}}}}"#, escaped);
    if let Result::Ok(value) = HeaderValue::from_str(&json) {
        headers.insert("HX-Trigger", value);
    }
}

#[derive(Debug)]
pub struct OkResponse {
    content: Option<String>,
    headers: HeaderMap,
    toast_message: Option<String>,
    oob_updates: Vec<(String, String)>,
    status: StatusCode,
}

impl OkResponse {
    pub fn new() -> Self {
        Self {
            content: None,
            headers: HeaderMap::new(),
            toast_message: None,
            oob_updates: Vec::new(),
            status: StatusCode::OK,
        }
    }

    /// Set the response body. Accepts Maud Markup, String, or &str.
    pub fn html(mut self, content: impl IntoHtml) -> Self {
        self.content = Some(content.into_html());
        self
    }

    /// Add a toast notification via HX-Trigger header.
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.toast_message = Some(message.into());
        self
    }

    /// Add an out-of-band swap. Accepts Maud Markup, String, or &str.
    pub fn oob(mut self, target: impl Into<String>, content: impl IntoHtml) -> Self {
        self.oob_updates.push((target.into(), content.into_html()));
        self
    }

    /// Add a custom response header.
    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        insert_header(&mut self.headers, key.as_ref(), value.as_ref());
        self
    }

    /// Set the HTTP status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl Default for OkResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        let mut headers = self.headers;
        if let Some(msg) = self.toast_message {
            insert_toast(&mut headers, &msg);
        }
        let main = self.content.unwrap_or_default();
        let oob: String = self
            .oob_updates
            .iter()
            .map(|(target, html)| {
                format!(r#"<div id="{}" hx-swap-oob="true">{}</div>"#, target, html)
            })
            .collect();
        (self.status, headers, Html(format!("{}{}", main, oob))).into_response()
    }
}

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

    /// Set error message (rendered as `<div class="error">…</div>`).
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

#[derive(Debug)]
pub struct RedirectResponse {
    location: Option<String>,
    toast_message: Option<String>,
    status: StatusCode,
}

impl RedirectResponse {
    pub fn new() -> Self {
        Self {
            location: None,
            toast_message: None,
            status: StatusCode::SEE_OTHER,
        }
    }

    /// Set the redirect location.
    pub fn to(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Add a toast notification via HX-Trigger header.
    pub fn toast(mut self, message: impl Into<String>) -> Self {
        self.toast_message = Some(message.into());
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
            if let Result::Ok(value) = HeaderValue::from_str(location) {
                headers.insert(axum::http::header::LOCATION, value.clone());
                headers.insert("HX-Redirect", value);
            }
        }
        if let Some(msg) = self.toast_message {
            insert_toast(&mut headers, &msg);
        }
        (self.status, headers).into_response()
    }
}

// ============================================================================
// Constructor functions
// ============================================================================

#[allow(non_snake_case)]
pub fn Ok() -> OkResponse {
    OkResponse::new()
}

#[allow(non_snake_case)]
pub fn Error() -> ErrorResponse {
    ErrorResponse::new()
}

#[allow(non_snake_case)]
pub fn Redirect() -> RedirectResponse {
    RedirectResponse::new()
}
