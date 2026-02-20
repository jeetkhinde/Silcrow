// silcrow/src/response/html.rs

use super::base::{finalize_response, BaseResponse};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

pub struct HtmlOkResponse {
    base: BaseResponse,
    markup: String,
}

impl HtmlOkResponse {
    pub fn new(markup: impl Into<String>) -> Self {
        let mut base = BaseResponse::new();
        base.status(StatusCode::OK);

        Self {
            base,
            markup: markup.into(),
        }
    }
    pub fn status(mut self, status: StatusCode) -> Self {
        self.base.status(status);
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.base.header(key, value);
        self
    }

    pub fn no_cache(mut self) -> Self {
        self.base
            .insert_header(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        self
    }
}

impl IntoResponse for HtmlOkResponse {
    fn into_response(self) -> Response {
        finalize_response(self.base, axum::response::Html(self.markup))
    }
}
