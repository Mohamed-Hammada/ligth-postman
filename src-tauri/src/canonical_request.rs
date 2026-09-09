//! CanonicalRequest (task-pack LP-0303): the one fully-resolved request representation
//! shared by execution and code generation (README's "share the same canonical resolved
//! request representation" rule). Pure and DB-free, like `resolver.rs` and `http_engine.rs` —
//! it takes an already-loaded `RequestFull` plus a `ScopeChain` and produces the exact
//! method/url/headers/body that would go out over the wire, with zero network or SQL calls.
//!
//! Auth (README/LP-0107) is folded in here rather than kept as a separate concept the caller
//! has to remember to apply: `Auth::Bearer`/`Basic`/`ApiKey` become a header (or, for an
//! ApiKey in the query location, a query parameter) exactly once, in exactly one place.

use base64::Engine;

use crate::error::AppError;
use crate::models::{ApiKeyLocation, Auth, HeaderEntry, RequestFull};
use crate::resolver::{self, ScopeChain};

#[derive(Debug, Clone)]
pub struct CanonicalRequest {
    pub method: String,
    /// Fully resolved, with all enabled query params (including any ApiKey-in-query auth)
    /// already appended. Never written back to the stored request.
    pub url: String,
    /// Fully resolved, enabled-only, in stored order plus any auth-derived header appended last.
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
}

pub fn build(request: &RequestFull, chain: &ScopeChain) -> Result<CanonicalRequest, AppError> {
    let resolve = |template: &str| resolver::resolve_template(template, chain).resolved;

    let base_url = resolve(&request.url);

    let mut headers: Vec<HeaderEntry> = request
        .headers
        .iter()
        .filter(|h| h.enabled)
        .map(|h| HeaderEntry { key: h.key.clone(), value: resolve(&h.value), enabled: true })
        .collect();

    let mut query_params: Vec<(String, String)> = request
        .query_params
        .iter()
        .filter(|p| p.enabled)
        .map(|p| (resolve(&p.key), resolve(&p.value)))
        .collect();

    apply_auth(&request.auth, &resolve, &mut headers, &mut query_params)?;

    let url = append_query_params(&base_url, &query_params)?;
    let body = request.body.as_deref().map(resolve);

    Ok(CanonicalRequest { method: request.method.clone(), url, headers, body })
}

fn apply_auth(
    auth: &Auth,
    resolve: &impl Fn(&str) -> String,
    headers: &mut Vec<HeaderEntry>,
    query_params: &mut Vec<(String, String)>,
) -> Result<(), AppError> {
    match auth {
        Auth::None => {}
        Auth::Bearer { token } => {
            let token = resolve(token);
            headers.push(HeaderEntry { key: "Authorization".into(), value: format!("Bearer {token}"), enabled: true });
        }
        Auth::Basic { username, password } => {
            let username = resolve(username);
            let password = resolve(password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
            headers.push(HeaderEntry { key: "Authorization".into(), value: format!("Basic {encoded}"), enabled: true });
        }
        Auth::ApiKey { key, value, location } => {
            let key = resolve(key);
            let value = resolve(value);
            match location {
                ApiKeyLocation::Header => headers.push(HeaderEntry { key, value, enabled: true }),
                ApiKeyLocation::Query => query_params.push((key, value)),
            }
        }
    }
    Ok(())
}

/// Same encoding logic as `execution::append_query_params` — kept here so both the send
/// path and codegen path build the URL identically. (`execution.rs` now delegates to this
/// module entirely rather than duplicating it.)
pub fn append_query_params(base_url: &str, params: &[(String, String)]) -> Result<String, AppError> {
    if params.is_empty() {
        return Ok(base_url.to_string());
    }
    let mut url = reqwest::Url::parse(base_url)
        .map_err(|err| AppError::Validation(format!("invalid URL '{base_url}': {err}")))?;
    {
        let mut query = url.query_pairs_mut();
        for (key, value) in params {
            query.append_pair(key, value);
        }
    }
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Auth as A, HeaderEntry as H, QueryParam as QP};
    use std::collections::HashMap;

    fn request(auth: A) -> RequestFull {
        RequestFull {
            id: "r1".into(),
            project_id: "p1".into(),
            name: "Test".into(),
            method: "GET".into(),
            url: "https://api.example.com/users".into(),
            headers: vec![H { key: "X-Existing".into(), value: "1".into(), enabled: true }],
            query_params: vec![],
            auth,
            body: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn none_auth_adds_no_header() {
        let canonical = build(&request(A::None), &ScopeChain::default()).unwrap();
        assert_eq!(canonical.headers.len(), 1);
        assert!(!canonical.headers.iter().any(|h| h.key == "Authorization"));
    }

    #[test]
    fn bearer_auth_resolves_variable_into_header() {
        let vars: HashMap<String, String> = [("token".to_string(), "abc123".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let canonical = build(&request(A::Bearer { token: "{{token}}".into() }), &chain).unwrap();
        let auth_header = canonical.headers.iter().find(|h| h.key == "Authorization").unwrap();
        assert_eq!(auth_header.value, "Bearer abc123");
    }

    #[test]
    fn basic_auth_base64_encodes_username_password() {
        let canonical = build(
            &request(A::Basic { username: "alice".into(), password: "wonderland".into() }),
            &ScopeChain::default(),
        )
        .unwrap();
        let auth_header = canonical.headers.iter().find(|h| h.key == "Authorization").unwrap();
        // echo -n alice:wonderland | base64
        assert_eq!(auth_header.value, "Basic YWxpY2U6d29uZGVybGFuZA==");
    }

    #[test]
    fn api_key_in_header_location_adds_a_header() {
        let canonical = build(
            &request(A::ApiKey { key: "X-Api-Key".into(), value: "secret".into(), location: ApiKeyLocation::Header }),
            &ScopeChain::default(),
        )
        .unwrap();
        let header = canonical.headers.iter().find(|h| h.key == "X-Api-Key").unwrap();
        assert_eq!(header.value, "secret");
    }

    #[test]
    fn api_key_in_query_location_adds_a_query_param_not_a_header() {
        let canonical = build(
            &request(A::ApiKey { key: "api_key".into(), value: "secret".into(), location: ApiKeyLocation::Query }),
            &ScopeChain::default(),
        )
        .unwrap();
        assert!(!canonical.headers.iter().any(|h| h.key == "api_key"));
        assert!(canonical.url.contains("api_key=secret"));
    }

    #[test]
    fn disabled_headers_and_params_are_dropped() {
        let mut req = request(A::None);
        req.headers[0].enabled = false;
        req.query_params.push(QP { key: "skip".into(), value: "1".into(), enabled: false, description: None });
        req.query_params.push(QP { key: "keep".into(), value: "1".into(), enabled: true, description: None });

        let canonical = build(&req, &ScopeChain::default()).unwrap();
        assert!(canonical.headers.is_empty());
        assert!(canonical.url.contains("keep=1"));
        assert!(!canonical.url.contains("skip="));
    }

    #[test]
    fn body_is_resolved_through_the_scope_chain() {
        let vars: HashMap<String, String> = [("name".to_string(), "Ada".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request(A::None);
        req.body = Some(r#"{"name":"{{name}}"}"#.into());
        let canonical = build(&req, &chain).unwrap();
        assert_eq!(canonical.body, Some(r#"{"name":"Ada"}"#.into()));
    }
}
