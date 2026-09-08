//! Developer Console Subsystem (task-pack LP-0412 - LP-0421).
//!
//! Provides a dedicated, high-performance, bounded chronological diagnostic stream for
//! request and response lifecycles.
//!
//! Features:
//! - Correlation IDs across request start, headers, cookies, redirects, and responses (LP-0419).
//! - Automatic secret redaction for Authorization, API keys, passwords, cookies, and tokens (LP-0418).
//! - Bounded ring-buffer storage with strict memory limits to avoid unbounded growth (LP-0420).
//! - Filtering by event level, request ID, and export capabilities (LP-0415, LP-0417).

use std::collections::VecDeque;
use std::sync::Mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_MAX_EVENTS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Info,
    Debug,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEvent {
    pub id: String,
    pub correlation_id: String,
    #[serde(default)]
    pub request_id: Option<String>,
    pub timestamp: String,
    pub level: ConsoleLevel,
    pub event_type: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

pub struct ConsoleBuffer {
    max_capacity: usize,
    events: Mutex<VecDeque<ConsoleEvent>>,
}

impl ConsoleBuffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            events: Mutex::new(VecDeque::with_capacity(max_capacity.min(100))),
        }
    }

    pub fn record(&self, event: ConsoleEvent) {
        let mut buf = self.events.lock().expect("console mutex poisoned");
        if buf.len() >= self.max_capacity {
            buf.pop_front();
        }
        buf.push_back(event);
    }

    pub fn log(
        &self,
        correlation_id: &str,
        request_id: Option<&str>,
        level: ConsoleLevel,
        event_type: &str,
        message: &str,
        details: Option<serde_json::Value>,
    ) {
        let event = ConsoleEvent {
            id: Uuid::new_v4().to_string(),
            correlation_id: correlation_id.to_string(),
            request_id: request_id.map(|s| s.to_string()),
            timestamp: Utc::now().to_rfc3339(),
            level,
            event_type: event_type.to_string(),
            message: message.to_string(),
            details,
        };
        self.record(event);
    }

    pub fn get_events(
        &self,
        limit: Option<usize>,
        level: Option<&str>,
        request_id: Option<&str>,
    ) -> Vec<ConsoleEvent> {
        let buf = self.events.lock().expect("console mutex poisoned");
        let parsed_level = level.and_then(|l| match l.to_lowercase().as_str() {
            "info" => Some(ConsoleLevel::Info),
            "debug" => Some(ConsoleLevel::Debug),
            "warn" | "warning" => Some(ConsoleLevel::Warn),
            "error" => Some(ConsoleLevel::Error),
            _ => None,
        });

        let mut filtered: Vec<ConsoleEvent> = buf
            .iter()
            .filter(|e| {
                if let Some(target_lvl) = parsed_level {
                    if e.level != target_lvl {
                        return false;
                    }
                }
                if let Some(target_req) = request_id {
                    if e.request_id.as_deref() != Some(target_req) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        if let Some(lim) = limit {
            if filtered.len() > lim {
                let start = filtered.len() - lim;
                filtered = filtered.split_off(start);
            }
        }

        filtered
    }

    pub fn clear(&self) {
        let mut buf = self.events.lock().expect("console mutex poisoned");
        buf.clear();
    }

    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let buf = self.events.lock().expect("console mutex poisoned");
        let slice: Vec<&ConsoleEvent> = buf.iter().collect();
        serde_json::to_string_pretty(&slice)
    }

    pub fn len(&self) -> usize {
        let buf = self.events.lock().expect("console mutex poisoned");
        buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ConsoleBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_EVENTS)
    }
}

/// Redacts sensitive headers and query strings before logging (LP-0418).
pub fn redact_header_value(name: &str, value: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("auth")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("key")
        || lower == "cookie"
        || lower == "set-cookie"
    {
        if value.is_empty() {
            String::new()
        } else {
            "[REDACTED]".to_string()
        }
    } else {
        value.to_string()
    }
}

pub fn redact_url(raw_url: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(raw_url) {
        let pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .map(|(k, v)| {
                let lower_k = k.to_ascii_lowercase();
                if lower_k.contains("key")
                    || lower_k.contains("token")
                    || lower_k.contains("secret")
                    || lower_k.contains("auth")
                    || lower_k.contains("password")
                {
                    (k.to_string(), "REDACTED".to_string())
                } else {
                    (k.to_string(), v.to_string())
                }
            })
            .collect();

        if !pairs.is_empty() {
            parsed.set_query(None);
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            for (k, v) in pairs {
                serializer.append_pair(&k, &v);
            }
            parsed.set_query(Some(&serializer.finish()));
        }
        parsed.to_string()
    } else {
        raw_url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_bounds_capacity_cleanly() {
        let buffer = ConsoleBuffer::new(5);
        for i in 0..10 {
            buffer.log("corr-1", None, ConsoleLevel::Info, "test", &format!("event {i}"), None);
        }

        assert_eq!(buffer.len(), 5);
        let events = buffer.get_events(None, None, None);
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].message, "event 5");
        assert_eq!(events[4].message, "event 9");
    }

    #[test]
    fn filter_by_level_and_request_id() {
        let buffer = ConsoleBuffer::new(100);
        buffer.log("c1", Some("req-1"), ConsoleLevel::Info, "start", "starting", None);
        buffer.log("c1", Some("req-1"), ConsoleLevel::Error, "error", "failed", None);
        buffer.log("c2", Some("req-2"), ConsoleLevel::Info, "start", "starting 2", None);

        let req1_events = buffer.get_events(None, None, Some("req-1"));
        assert_eq!(req1_events.len(), 2);

        let error_events = buffer.get_events(None, Some("error"), None);
        assert_eq!(error_events.len(), 1);
        assert_eq!(error_events[0].message, "failed");
    }

    #[test]
    fn secret_redaction_masks_sensitive_headers_and_urls() {
        assert_eq!(redact_header_value("Authorization", "Bearer eyJhbGciOi..."), "[REDACTED]");
        assert_eq!(redact_header_value("X-Api-Key", "secret-12345"), "[REDACTED]");
        assert_eq!(redact_header_value("Cookie", "session=abcdef"), "[REDACTED]");
        assert_eq!(redact_header_value("Content-Type", "application/json"), "application/json");

        let masked_url = redact_url("https://api.example.com/data?api_key=secret123&user=42");
        assert!(masked_url.contains("api_key=REDACTED"));
        assert!(masked_url.contains("user=42"));
    }

    #[test]
    fn clear_and_export_roundtrip() {
        let buffer = ConsoleBuffer::new(10);
        buffer.log("c1", None, ConsoleLevel::Info, "init", "hello", None);
        let exported = buffer.export_json().unwrap();
        assert!(exported.contains("hello"));

        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }
}
