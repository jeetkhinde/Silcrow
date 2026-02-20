// silcrow/src/response/json.rs

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::base::{finalize_response, BaseResponse};

pub struct JsonOkResponse {
    base: BaseResponse,
    data: serde_json::Map<String, serde_json::Value>,
}

impl JsonOkResponse {
    pub fn new() -> Self {
        let mut base = BaseResponse::new();
        base.status(StatusCode::OK);

        Self {
            base,
            data: serde_json::Map::new(),
        }
    }

    pub fn set(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        let v = serde_json::to_value(value).expect("Failed to serialize JSON");
        self.data.insert(key.into(), v);
        self
    }

    pub fn set_value(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
    pub fn merge(mut self, map: serde_json::Map<String, serde_json::Value>) -> Self {
        self.data.extend(map);
        self
    }
    pub fn toast(self, message: impl Into<String>, kind: impl Into<String>) -> Self {
        self.set(
            "_toast",
            serde_json::json!({
                "message": message.into(),
                "type": kind.into()
            }),
        )
    }
    pub fn set_opt<T: serde::Serialize>(
        mut self,
        key: impl Into<String>,
        value: Option<T>,
    ) -> Self {
        if let Some(v) = value {
            self = self.set(key, v);
        }
        self
    }
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.base.header(key, value);
        self
    }

    pub fn no_cache(mut self) -> Self {
        self.base.no_cache();
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.base.status(status);
        self
    }
    pub fn ok() -> Self {
        Self::new()
    }

    pub fn created() -> Self {
        let mut res = Self::new();
        res.base.status(StatusCode::CREATED);
        res
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
        finalize_response(self.base, axum::Json(body))
    }
}
impl<T> From<T> for JsonOkResponse
where
    T: serde::Serialize,
{
    fn from(value: T) -> Self {
        let mut res = JsonOkResponse::new();
        let v = serde_json::to_value(value).expect("Failed to serialize JSON");

        match v {
            serde_json::Value::Object(map) => {
                res.data = map;
            }
            other => {
                res.data.insert("data".into(), other);
            }
        }

        res
    }
}
