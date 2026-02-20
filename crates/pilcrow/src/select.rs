use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
// use std::convert::Infallible;

// ════════════════════════════════════════════════════════════
// 1. The Unified Application Error
// ════════════════════════════════════════════════════════════

/// A unified error type so developers can use `?` inside their closures.
pub enum AppError {
    /// A standard 500 Internal Server Error (e.g., database failure)
    Internal(anyhow::Error),
    /// A 404 Not Found (e.g., requested user doesn't exist)
    NotFound(String),
}

// Map your custom AppError to standard Axum responses
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Internal(err) => {
                tracing::error!("Internal server error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
        }
    }
}

// Allows developers to use `?` on standard Result types (like SQLx or std::io)
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::Internal(err.into())
    }
}

// ════════════════════════════════════════════════════════════
// 2. The Responses Container (with Type-Safe Builder)
// ════════════════════════════════════════════════════════════

/// Holds the closures for each potential response format.
pub struct Responses<H, J, N> {
    html: Option<H>,
    json: Option<J>,
    navigate: Option<N>,
}

impl Responses<(), (), ()> {
    /// Starts an empty set of responses
    pub fn new() -> Self {
        Self {
            html: None,
            json: None,
            navigate: None,
        }
    }
}

impl<H, J, N> Responses<H, J, N> {
    pub fn html<NewH>(self, f: NewH) -> Responses<NewH, J, N> {
        Responses {
            html: Some(f),
            json: self.json,
            navigate: self.navigate,
        }
    }

    pub fn json<NewJ>(self, f: NewJ) -> Responses<H, NewJ, N> {
        Responses {
            html: self.html,
            json: Some(f),
            navigate: self.navigate,
        }
    }

    pub fn navigate<NewN>(self, f: NewN) -> Responses<H, J, NewN> {
        Responses {
            html: self.html,
            json: self.json,
            navigate: Some(f),
        }
    }
}

// ════════════════════════════════════════════════════════════
// 3. The Core Selector Implementation
// ════════════════════════════════════════════════════════════

// Assuming RequestMode is imported from Phase 1
use crate::extract::{RequestMode, SilcrowRequest};

impl SilcrowRequest {
    /// Evaluates the preferred mode and executes *only* the matching closure.
    pub fn select<H, J, N, THtml, TJson, TNav>(
        &self,
        responses: Responses<H, J, N>,
    ) -> Result<Response, AppError>
    where
        H: FnOnce() -> Result<THtml, AppError>,
        J: FnOnce() -> Result<TJson, AppError>,
        N: FnOnce() -> Result<TNav, AppError>,
        THtml: IntoResponse,
        TJson: IntoResponse,
        TNav: IntoResponse,
    {
        match self.preferred_mode() {
            RequestMode::Html => {
                if let Some(f) = responses.html {
                    Ok(f()?.into_response())
                } else {
                    Ok((
                        StatusCode::NOT_ACCEPTABLE,
                        "HTML representation not provided",
                    )
                        .into_response())
                }
            }
            RequestMode::Json => {
                if let Some(f) = responses.json {
                    Ok(f()?.into_response())
                } else {
                    Ok((
                        StatusCode::NOT_ACCEPTABLE,
                        "JSON representation not provided",
                    )
                        .into_response())
                }
            }
            RequestMode::Navigate => {
                if let Some(f) = responses.navigate {
                    Ok(f()?.into_response())
                } else {
                    Ok(
                        (StatusCode::NOT_ACCEPTABLE, "Navigation rule not provided")
                            .into_response(),
                    )
                }
            }
        }
    }
}
