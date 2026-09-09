//! Code snippet generation (task-pack LP-0601/0602/0605/0606).
//!
//! Every generator consumes a `CanonicalRequest` — never the UI, never raw request text —
//! so a snippet can never show something different from what actually gets sent (LP-0601's
//! "generate from CanonicalRequest, never from UI text").
//!
//! Only one target exists today (Bash `curl`); PowerShell/CMD/other languages (LP-0603,
//! 0604, 0607) are not implemented — add them as siblings to `generate_curl_bash` and a new
//! `SnippetTarget` variant, not by touching `SnippetMode`/`generate_snippet`.

use crate::canonical_request;
use crate::error::AppError;
use crate::models::RequestFull;
use crate::resolver::ScopeChain;

/// LP-0606 secret-safe modes. `Masked` (show real values except secrets) is NOT implemented —
/// it would need the resolver to track which output bytes came from a secret variable, which
/// `resolver::resolve_template` doesn't do today. Rather than fake it, only the two modes that
/// are fully honest are offered: never resolve anything, or resolve everything explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnippetMode {
    /// Default. Variables stay as literal `{{name}}` — a secret can never leak into a
    /// snippet unless the user explicitly switches to `Resolved`.
    Placeholder,
    /// Explicit opt-in: every `{{var}}` (including secrets) is resolved to its real value,
    /// exactly as it would be sent over the wire.
    Resolved,
}

pub fn generate_snippet(
    request: &RequestFull,
    chain: &ScopeChain,
    mode: SnippetMode,
) -> Result<String, AppError> {
    let canonical = match mode {
        SnippetMode::Resolved => canonical_request::build(request, chain)?,
        // An empty chain resolves nothing — every {{var}} is left exactly as stored. This
        // reuses the real resolver/canonical-request path instead of a separate "don't
        // resolve" code path, so placeholder mode can't silently drift from resolved mode.
        SnippetMode::Placeholder => canonical_request::build(request, &ScopeChain::default())?,
    };
    Ok(generate_curl_bash(&canonical))
}

pub fn generate_curl_bash(canonical: &canonical_request::CanonicalRequest) -> String {
    let mut parts = vec![format!("curl -X {}", canonical.method), shell_single_quote(&canonical.url)];
    for header in &canonical.headers {
        parts.push(format!(
            "-H {}",
            shell_single_quote(&format!("{}: {}", header.key, header.value))
        ));
    }
    if let Some(body) = &canonical.body {
        parts.push(format!("--data-raw {}", shell_single_quote(body)));
    }
    parts.join(" \\\n  ")
}

/// POSIX single-quoting: wrap in `'...'`, and turn any literal `'` into `'\''` (close quote,
/// escaped literal quote, reopen quote) — the standard, minimal-surprise way to make an
/// arbitrary string safe inside single quotes in `sh`/`bash`.
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Auth, HeaderEntry, QueryParam};
    use std::collections::HashMap;

    fn request() -> RequestFull {
        RequestFull {
            id: "r1".into(),
            project_id: "p1".into(),
            name: "Get user".into(),
            method: "GET".into(),
            url: "https://api.example.com/users/{{userId}}".into(),
            headers: vec![HeaderEntry { key: "Accept".into(), value: "application/json".into(), enabled: true }],
            query_params: vec![],
            auth: Auth::Bearer { token: "{{token}}".into() },
            body: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn shell_single_quote_escapes_embedded_quotes() {
        assert_eq!(shell_single_quote("it's fine"), r#"'it'\''s fine'"#);
        assert_eq!(shell_single_quote("no quotes"), "'no quotes'");
    }

    #[test]
    fn placeholder_mode_never_resolves_variables_including_secrets() {
        let snippet = generate_snippet(&request(), &ScopeChain::default(), SnippetMode::Placeholder).unwrap();
        assert!(snippet.contains("{{userId}}"));
        assert!(snippet.contains("Bearer {{token}}"));
        assert!(!snippet.contains("Bearer sk-"));
    }

    #[test]
    fn resolved_mode_substitutes_every_variable() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "sk-live-123".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let snippet = generate_snippet(&request(), &chain, SnippetMode::Resolved).unwrap();
        assert!(snippet.contains("https://api.example.com/users/42"));
        assert!(snippet.contains("Bearer sk-live-123"));
        assert!(!snippet.contains("{{"));
    }

    #[test]
    fn generated_snippet_has_expected_curl_structure() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "abc".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let snippet = generate_snippet(&request(), &chain, SnippetMode::Resolved).unwrap();
        assert_eq!(
            snippet,
            "curl -X GET \\\n  'https://api.example.com/users/42' \\\n  -H 'Accept: application/json' \\\n  -H 'Authorization: Bearer abc'"
        );
    }

    #[test]
    fn body_is_included_with_data_raw_and_query_params_are_appended() {
        let mut req = request();
        req.method = "POST".into();
        req.body = Some(r#"{"name":"Ada"}"#.into());
        req.query_params.push(QueryParam { key: "verbose".into(), value: "true".into(), enabled: true, description: None });

        let snippet = generate_snippet(&req, &ScopeChain::default(), SnippetMode::Placeholder).unwrap();
        assert!(snippet.contains("curl -X POST"));
        assert!(snippet.contains("verbose=true"));
        assert!(snippet.ends_with(r#"--data-raw '{"name":"Ada"}'"#));
    }
}
