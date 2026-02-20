// silcrow/src/response/json.rs
// silcrow/crates/silcrow/src/response/json.rs — Silcrow server-side JSON response builder
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
        let key = key.into();

        match serde_json::to_value(value) {
            Ok(v) => {
                self.data.insert(key, v);
            }
            Err(_) => {
                self.data.insert(key, serde_json::Value::Null);
            }
        }

        self
    }

    pub fn try_set(
        mut self,
        key: impl Into<String>,
        value: impl serde::Serialize,
    ) -> std::result::Result<Self, serde_json::Error> {
        let v = serde_json::to_value(value)?;
        self.data.insert(key.into(), v);
        Ok(self)
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

    pub fn try_from<T>(value: T) -> std::result::Result<Self, serde_json::Error>
    where
        T: serde::Serialize,
    {
        let mut res = JsonOkResponse::new();
        let v = serde_json::to_value(value)?;

        match v {
            serde_json::Value::Object(map) => {
                res.data = map;
            }
            other => {
                res.data.insert("data".into(), other);
            }
        }

        Ok(res)
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
        Self::try_from(value).unwrap_or_else(|err| {
            JsonOkResponse::new().set_value(
                "error",
                serde_json::json!({
                    "message": format!("Failed to serialize JSON: {err}"),
                }),
            )
        })
    }
}
