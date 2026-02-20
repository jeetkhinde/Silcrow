use axum::{
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════
// 1. Shared State & Modifiers
// ════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toast {
    pub message: String,
    pub level: String,
}

#[derive(Default)]
pub struct BaseResponse {
    pub headers: HeaderMap,
    pub cookies: Vec<Cookie<'static>>,
    pub toasts: Vec<Toast>, // Future-proof: multiple toasts
}

impl BaseResponse {
    /// Applies all headers and standard cookies to the Axum response.
    /// (Fix #4: Centralized emission logic)
    pub fn apply_to_response(&self, response: &mut Response) {
        // 1. Apply standard headers
        for (name, value) in &self.headers {
            response.headers_mut().insert(name.clone(), value.clone());
        }

        // 2. Apply standard cookies
        for cookie in &self.cookies {
            if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                response.headers_mut().append(SET_COOKIE, header_value);
            }
        }
    }

    /// Safely formats toasts as URL-encoded cookies for HTML/Navigate responses.
    /// (Fix #3: Safe Cookie formatting)
    pub fn apply_toast_cookies(&self, response: &mut Response) {
        // If we have multiple toasts, we serialize the array to JSON, then URL-encode it
        if !self.toasts.is_empty() {
            if let Ok(json_string) = serde_json::to_string(&self.toasts) {
                let encoded = urlencoding::encode(&json_string);

                let cookie = Cookie::build(("silcrow_toasts", encoded.into_owned()))
                    .path("/")
                    .same_site(SameSite::Lax)
                    .build();

                if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                    response.headers_mut().append(SET_COOKIE, header_value);
                }
            }
        }
    }
}

// ════════════════════════════════════════════════════════════
// 2. The Modifier Trait
// ════════════════════════════════════════════════════════════

pub trait ResponseExt: Sized {
    fn base_mut(&mut self) -> &mut BaseResponse;

    fn with_header(mut self, key: &'static str, value: &'static str) -> Self {
        if let Ok(val) = HeaderValue::from_str(value) {
            self.base_mut().headers.insert(key, val);
        }
        self
    }

    fn no_cache(self) -> Self {
        self.with_header("silcrow-cache", "no-cache")
    }

    fn with_toast(mut self, message: impl Into<String>, level: impl Into<String>) -> Self {
        self.base_mut().toasts.push(Toast {
            message: message.into(),
            level: level.into(),
        });
        self
    }
}

// ════════════════════════════════════════════════════════════
// 3. Response Wrappers & Transport Logic
// ════════════════════════════════════════════════════════════

// --- HTML ---
pub struct HtmlResponse {
    pub data: String,
    pub base: BaseResponse,
}

impl IntoResponse for HtmlResponse {
    fn into_response(self) -> Response {
        let mut response = axum::response::Html(self.data).into_response();
        self.base.apply_to_response(&mut response);
        self.base.apply_toast_cookies(&mut response);
        response
    }
}

// --- JSON ---
pub struct JsonResponse<T> {
    pub data: T,
    pub base: BaseResponse,
}

impl<T: serde::Serialize> IntoResponse for JsonResponse<T> {
    fn into_response(self) -> Response {
        // Fix #1: Never unwrap serialization. Return 500 if it fails.
        let mut json_payload = match serde_json::to_value(&self.data) {
            Ok(val) => val,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        // Fix #2: Handle toasts safely, even if the root isn't an Object
        if !self.base.toasts.is_empty() {
            let toasts_json = serde_json::json!(self.base.toasts);

            if let serde_json::Value::Object(mut map) = json_payload {
                map.insert("_toasts".to_string(), toasts_json);
                json_payload = serde_json::Value::Object(map);
            } else {
                // Option B Safe Wrap: If the user returned an array `json(vec![1, 2])`
                json_payload = serde_json::json!({
                    "data": json_payload,
                    "_toasts": toasts_json
                });
            }
        }

        let mut response = Json(json_payload).into_response();
        self.base.apply_to_response(&mut response); // Apply headers/cookies (but NOT toast cookies)
        response
    }
}

// --- NAVIGATE ---
pub struct NavigateResponse {
    pub path: String,
    pub base: BaseResponse,
}

impl IntoResponse for NavigateResponse {
    fn into_response(self) -> Response {
        // Fix #5: Explicitly using 303 See Other, which is best practice for client-side routers
        let mut response = Redirect::to(&self.path).into_response();

        // Ensure the status is explicitly 303 (Axum defaults to 303 for Redirect::to, but this guarantees it)
        *response.status_mut() = StatusCode::SEE_OTHER;

        self.base.apply_to_response(&mut response);
        self.base.apply_toast_cookies(&mut response);
        response
    }
}

// ════════════════════════════════════════════════════════════
// 4. Constructors & Trait Impls
// ════════════════════════════════════════════════════════════

pub fn html(data: impl Into<String>) -> HtmlResponse {
    HtmlResponse {
        data: data.into(),
        base: BaseResponse::default(),
    }
}

pub fn json<T>(data: T) -> JsonResponse<T> {
    JsonResponse {
        data,
        base: BaseResponse::default(),
    }
}

pub fn navigate(path: impl Into<String>) -> NavigateResponse {
    NavigateResponse {
        path: path.into(),
        base: BaseResponse::default(),
    }
}

impl ResponseExt for HtmlResponse {
    fn base_mut(&mut self) -> &mut BaseResponse {
        &mut self.base
    }
}
impl<T> ResponseExt for JsonResponse<T> {
    fn base_mut(&mut self) -> &mut BaseResponse {
        &mut self.base
    }
}
impl ResponseExt for NavigateResponse {
    fn base_mut(&mut self) -> &mut BaseResponse {
        &mut self.base
    }
}
