use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Methods we accept today. README §13 will grow this alongside auth/scripts later.
pub const VALID_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewProjectInput {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderEntry {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// Lightweight row for lists — never carries headers/body (README §4/§20 lazy loading).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub updated_at: DateTime<Utc>,
}

/// Fully hydrated request — only fetched when a tab is actually activated (README §4/§5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFull {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewRequestInput {
    pub project_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub body: Option<String>,
}
