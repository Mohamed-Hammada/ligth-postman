use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

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
#[derive(Debug, thiserror::Error)]
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

    /// The `kind` discriminant sent over IPC — kept stable for the frontend's `AppError.kind` union.
    fn kind_name(&self) -> &'static str {
        match self {
            AppError::Storage(_) => "Storage",
            AppError::NotFound(_) => "NotFound",
            AppError::Validation(_) => "Validation",
            AppError::Network(_) => "Network",
            AppError::Cancelled => "Cancelled",
            AppError::Ai(_) => "Ai",
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

/// Hand-written so the IPC payload can carry `hint` alongside `kind`/`message` — the frontend's
/// `describeError()` uses it to show an actionable message instead of a raw backend string
/// (e.g. "Unable to reach the server." + the hint above, rather than "network error: ...").
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 3)?;
        s.serialize_field("kind", self.kind_name())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("hint", self.remediation_hint())?;
        s.end()
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

    #[test]
    fn wire_format_carries_kind_message_and_hint() {
        let value = serde_json::to_value(AppError::Network("connection refused".into())).unwrap();
        assert_eq!(value["kind"], "Network");
        assert_eq!(value["message"], "network error: connection refused");
        assert!(value["hint"].as_str().unwrap().contains("proxy"));

        // Cancelled is a unit variant — confirm `message`/`hint` are still present (unlike the
        // old adjacently-tagged derive, which omitted `message` entirely for unit variants).
        let cancelled = serde_json::to_value(AppError::Cancelled).unwrap();
        assert_eq!(cancelled["kind"], "Cancelled");
        assert_eq!(cancelled["message"], "request cancelled");
    }
}

