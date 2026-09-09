use serde::Serialize;

/// Structured, domain-level errors. Never leak raw rusqlite/IO errors to the UI (README §34).
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("request cancelled")]
    Cancelled,
    #[error("AI error: {0}")]
    Ai(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Storage(err.to_string())
    }
}
