use serde::Serialize;

/// Error taxonomy classifying faults into three distinct architectural tiers (LP-0906).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorTier {
    /// Validation and entity state invariant violations.
    Domain,
    /// User action workflows, cancellations, and orchestration errors.
    Application,
    /// Operating system, network sockets, file I/O, and database failures.
    Infrastructure,
}

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

impl AppError {
    /// Categorizes the error into Domain, Application, or Infrastructure tier.
    pub fn tier(&self) -> ErrorTier {
        match self {
            AppError::Validation(_) | AppError::NotFound(_) => ErrorTier::Domain,
            AppError::Ai(_) | AppError::Cancelled => ErrorTier::Application,
            AppError::Storage(_) | AppError::Network(_) => ErrorTier::Infrastructure,
        }
    }

    /// Provides user-facing remediation advice for actionable UI feedback.
    pub fn remediation_hint(&self) -> &'static str {
        match self {
            AppError::Network(_) => "Verify target URL, ensure server is active, and inspect proxy or firewall settings.",
            AppError::Validation(_) => "Check required fields, ensure names and URLs are non-empty, and format variables correctly.",
            AppError::NotFound(_) => "The resource does not exist or was deleted. Please refresh the view.",
            AppError::Ai(_) => "Verify your Anthropic Claude API key in AI Settings or check network connectivity.",
            AppError::Storage(_) => "SQLite database I/O error. Verify disk space and write permissions.",
            AppError::Cancelled => "Request was cancelled by user action.",
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Storage(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_tier_classification() {
        assert_eq!(AppError::Validation("empty name".into()).tier(), ErrorTier::Domain);
        assert_eq!(AppError::NotFound("no request".into()).tier(), ErrorTier::Domain);
        assert_eq!(AppError::Cancelled.tier(), ErrorTier::Application);
        assert_eq!(AppError::Ai("key invalid".into()).tier(), ErrorTier::Application);
        assert_eq!(AppError::Network("connection refused".into()).tier(), ErrorTier::Infrastructure);
        assert_eq!(AppError::Storage("disk full".into()).tier(), ErrorTier::Infrastructure);
    }

    #[test]
    fn remediation_hints_are_actionable() {
        assert!(AppError::Network("err".into()).remediation_hint().contains("proxy"));
        assert!(AppError::Ai("err".into()).remediation_hint().contains("Settings"));
        assert!(AppError::Validation("err".into()).remediation_hint().contains("required fields"));
    }
}

