// silcrow/src/response/html.rs
// silcrow/src/response/html.rs — Silcrow server-side HTML response builder
use super::base::{finalize_response, BaseResponse};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use maud::Markup;

pub struct HtmlOkResponse {
    base: BaseResponse,
    markup: Markup,
}

impl HtmlOkResponse {
    pub fn new(markup: Markup) -> Self {
        let mut base = BaseResponse::new();
        base.status(StatusCode::OK);

        Self { base, markup }
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
        finalize_response(self.base, axum::response::Html(self.markup.into_string()))
    }
}
