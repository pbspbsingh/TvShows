use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::info;

pub struct HttpError {
    inner: anyhow::Error,
}

impl From<anyhow::Error> for HttpError {
    fn from(inner: anyhow::Error) -> Self {
        HttpError { inner }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let json = json!({
            "error": self.inner.to_string(),
            "stack": self.inner.backtrace().to_string(),
        });
        info!("Returning http error: {json}");
        let response = serde_json::to_string_pretty(&json)
            .unwrap_or_else(|_| format!("Something is wrong: {}", self.inner.to_string()));
        (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
    }
}
