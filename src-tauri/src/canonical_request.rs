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
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{ApiKeyLocation, Auth, HeaderEntry, RequestBody, RequestFull, ResolvedFormPart};
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
    /// Set instead of `body` when the request's body type is `FormData`. Kept as structured
    /// parts (not flattened into `body`) because multipart requires the HTTP layer to generate
    /// the boundary and stream file bytes — see `models::ResolvedFormPart`.
    pub multipart: Option<Vec<ResolvedFormPart>>,
    /// Set instead of `body` when the request's body type is `Binary` — a path to read raw
    /// bytes from at send time. Must never be treated as `body` text directly: doing so would
    /// send the literal path string instead of the file's contents (a real bug this replaced).
    pub body_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RequestDiagnostics {
    pub all_missing: Vec<String>,
    pub url_missing: Vec<String>,
    pub headers_missing: Vec<String>,
    pub query_params_missing: Vec<String>,
    pub auth_missing: Vec<String>,
    pub body_missing: Vec<String>,
}

pub fn diagnose(request: &RequestFull, chain: &ScopeChain) -> RequestDiagnostics {
    let check = |template: &str| resolver::resolve_template(template, chain).missing;

    let mut diag = RequestDiagnostics::default();
    diag.url_missing = check(&request.url);

    for h in request.headers.iter().filter(|h| h.enabled) {
        diag.headers_missing.extend(check(&h.key));
        diag.headers_missing.extend(check(&h.value));
    }

    for p in request.query_params.iter().filter(|p| p.enabled) {
        diag.query_params_missing.extend(check(&p.key));
        diag.query_params_missing.extend(check(&p.value));
    }

    match &request.auth {
        Auth::None => {}
        Auth::Bearer { token } => {
            diag.auth_missing.extend(check(token));
        }
        Auth::Basic { username, password } => {
            diag.auth_missing.extend(check(username));
            diag.auth_missing.extend(check(password));
        }
        Auth::ApiKey { key, value, .. } => {
            diag.auth_missing.extend(check(key));
            diag.auth_missing.extend(check(value));
        }
    }

    if let Some(raw) = &request.body {
        if let Ok(parsed) = serde_json::from_str::<RequestBody>(raw) {
            match parsed {
                RequestBody::None => {}
                RequestBody::Binary { file_path } => {
                    if let Some(p) = file_path {
                        diag.body_missing.extend(check(&p));
                    }
                }
                RequestBody::Raw { data, .. } => {
                    diag.body_missing.extend(check(&data));
                }
                RequestBody::UrlEncoded { items } => {
                    for item in items.into_iter().filter(|i| i.enabled) {
                        diag.body_missing.extend(check(&item.key));
                        diag.body_missing.extend(check(&item.value));
                    }
                }
                RequestBody::FormData { items } => {
                    for item in items.into_iter().filter(|i| i.enabled) {
                        diag.body_missing.extend(check(&item.key));
                        if item.is_file {
                            if let Some(p) = &item.file_path {
                                diag.body_missing.extend(check(p));
                            }
                        } else {
                            diag.body_missing.extend(check(&item.value));
                        }
                    }
                }
                RequestBody::GraphQL { query, variables } => {
                    diag.body_missing.extend(check(&query));
                    if let Some(v) = variables {
                        diag.body_missing.extend(check(&v));
                    }
                }
            }
        } else {
            diag.body_missing.extend(check(raw));
        }
    }

    let dedup = |v: &mut Vec<String>| {
        v.sort();
        v.dedup();
    };
    dedup(&mut diag.url_missing);
    dedup(&mut diag.headers_missing);
    dedup(&mut diag.query_params_missing);
    dedup(&mut diag.auth_missing);
    dedup(&mut diag.body_missing);

    let mut all = Vec::new();
    all.extend(diag.url_missing.clone());
    all.extend(diag.headers_missing.clone());
    all.extend(diag.query_params_missing.clone());
    all.extend(diag.auth_missing.clone());
    all.extend(diag.body_missing.clone());
    dedup(&mut all);
    diag.all_missing = all;

    diag
}

pub fn build(request: &RequestFull, chain: &ScopeChain) -> Result<CanonicalRequest, AppError> {
    let resolve = |template: &str| resolver::resolve_template(template, chain).resolved;

    let base_url = resolve(&request.url);

    let mut headers: Vec<HeaderEntry> = request
        .headers
        .iter()
        .filter(|h| h.enabled)
        .map(|h| HeaderEntry {
            key: h.key.clone(),
            value: resolve(&h.value),
            enabled: true,
            description: h.description.clone(),
        })
        .collect();

    let mut query_params: Vec<(String, String)> = request
        .query_params
        .iter()
        .filter(|p| p.enabled)
        .map(|p| (resolve(&p.key), resolve(&p.value)))
        .collect();

    apply_auth(&request.auth, &resolve, &mut headers, &mut query_params)?;

    let url = append_query_params(&base_url, &query_params)?;
    let (body, multipart, body_file_path, auto_ct) = resolve_body(request.body.as_deref(), &resolve);

    // Multipart's Content-Type (with its boundary parameter) must be generated by the HTTP
    // layer at send time (`http_engine::execute`, via `reqwest`'s `.multipart()`) — injecting
    // a boundary-less `multipart/form-data` header here would be exactly the "fake behavior"
    // this canonical model exists to avoid (see README/task-pack: never hand-roll a boundary).
    if let Some(ct) = auto_ct {
        if multipart.is_none() && !headers.iter().any(|h| h.key.eq_ignore_ascii_case("content-type")) {
            headers.push(HeaderEntry {
                key: "Content-Type".into(),
                value: ct,
                enabled: true,
                description: None,
            });
        }
    }

    Ok(CanonicalRequest { method: request.method.clone(), url, headers, body, multipart, body_file_path })
}

/// Returns `(body, multipart, body_file_path, auto_content_type)`. Exactly one of
/// `body`/`multipart`/`body_file_path` is set for a body-bearing request; `auto_content_type`
/// is `None` for multipart since its Content-Type (boundary included) is generated by the HTTP
/// layer, not decided here.
pub fn resolve_body(
    raw_body: Option<&str>,
    resolve: &impl Fn(&str) -> String,
) -> (Option<String>, Option<Vec<ResolvedFormPart>>, Option<String>, Option<String>) {
    let Some(raw) = raw_body else {
        return (None, None, None, None);
    };
    if raw.trim().is_empty() {
        return (None, None, None, None);
    }

    if let Ok(parsed) = serde_json::from_str::<RequestBody>(raw) {
        match parsed {
            RequestBody::None => (None, None, None, None),
            RequestBody::Raw { content_type, data } => {
                let resolved_data = resolve(&data);
                let ct = if content_type.trim().is_empty() {
                    Some("application/json".to_string())
                } else {
                    Some(content_type)
                };
                (Some(resolved_data), None, None, ct)
            }
            RequestBody::UrlEncoded { items } => {
                let mut serializer = url::form_urlencoded::Serializer::new(String::new());
                for item in items.into_iter().filter(|i| i.enabled) {
                    serializer.append_pair(&resolve(&item.key), &resolve(&item.value));
                }
                let encoded = serializer.finish();
                (Some(encoded), None, None, Some("application/x-www-form-urlencoded".to_string()))
            }
            RequestBody::GraphQL { query, variables } => {
                let resolved_query = resolve(&query);
                let resolved_vars = variables.map(|v| resolve(&v));
                let vars_val = resolved_vars
                    .as_deref()
                    .and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok())
                    .unwrap_or(serde_json::Value::Null);
                let payload = serde_json::json!({
                    "query": resolved_query,
                    "variables": vars_val
                });
                (Some(payload.to_string()), None, None, Some("application/json".to_string()))
            }
            RequestBody::FormData { items } => {
                let parts: Vec<ResolvedFormPart> = items
                    .into_iter()
                    .filter(|i| i.enabled)
                    .filter_map(|item| {
                        if item.is_file {
                            let file_path = resolve(item.file_path.as_deref().unwrap_or(""));
                            if file_path.is_empty() {
                                return None;
                            }
                            let file_name = std::path::Path::new(&file_path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| file_path.clone());
                            Some(ResolvedFormPart::File { key: resolve(&item.key), file_name, file_path })
                        } else {
                            Some(ResolvedFormPart::Text { key: resolve(&item.key), value: resolve(&item.value) })
                        }
                    })
                    .collect();
                (None, Some(parts), None, None)
            }
            RequestBody::Binary { file_path } => {
                // `file_path` must land in `body_file_path`, never `body` — putting it in
                // `body` would send the literal path string as the request body instead of the
                // file's actual bytes (the bug this replaces).
                let resolved_path = file_path.map(|p| resolve(&p)).filter(|p| !p.is_empty());
                let ct = resolved_path.as_ref().map(|_| "application/octet-stream".to_string());
                (None, None, resolved_path, ct)
            }
        }
    } else {
        (Some(resolve(raw)), None, None, None)
    }
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
            headers.push(HeaderEntry {
                key: "Authorization".into(),
                value: format!("Bearer {token}"),
                enabled: true,
                description: None,
            });
        }
        Auth::Basic { username, password } => {
            let username = resolve(username);
            let password = resolve(password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
            headers.push(HeaderEntry {
                key: "Authorization".into(),
                value: format!("Basic {encoded}"),
                enabled: true,
                description: None,
            });
        }
        Auth::ApiKey { key, value, location } => {
            let key = resolve(key);
            let value = resolve(value);
            match location {
                ApiKeyLocation::Header => headers.push(HeaderEntry {
                    key,
                    value,
                    enabled: true,
                    description: None,
                }),
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
            folder_id: None,
            name: "Test".into(),
            method: "GET".into(),
            url: "https://api.example.com/users".into(),
            headers: vec![H { key: "X-Existing".into(), value: "1".into(), enabled: true, description: None }],
            query_params: vec![],
            auth,
            body: None,
            description: None,
            settings: Default::default(),
            pre_request_script: None,
            post_request_script: None,
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
    fn duplicate_headers_preserve_order_and_descriptions_and_resolve_independently() {
        let vars: HashMap<String, String> = [("v1".to_string(), "val1".to_string()), ("v2".to_string(), "val2".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request(A::None);
        req.headers = vec![
            H { key: "Accept".into(), value: "application/json".into(), enabled: true, description: Some("JSON accept".into()) },
            H { key: "Accept".into(), value: "text/plain".into(), enabled: true, description: Some("Text accept".into()) },
            H { key: "X-Custom".into(), value: "{{v1}}".into(), enabled: true, description: None },
            H { key: "X-Custom".into(), value: "{{v2}}".into(), enabled: true, description: None },
        ];
        let canonical = build(&req, &chain).unwrap();
        assert_eq!(canonical.headers.len(), 4);
        assert_eq!(canonical.headers[0].key, "Accept");
        assert_eq!(canonical.headers[0].value, "application/json");
        assert_eq!(canonical.headers[0].description, Some("JSON accept".into()));

        assert_eq!(canonical.headers[1].key, "Accept");
        assert_eq!(canonical.headers[1].value, "text/plain");
        assert_eq!(canonical.headers[1].description, Some("Text accept".into()));

        assert_eq!(canonical.headers[2].key, "X-Custom");
        assert_eq!(canonical.headers[2].value, "val1");

        assert_eq!(canonical.headers[3].key, "X-Custom");
        assert_eq!(canonical.headers[3].value, "val2");
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

    #[test]
    fn urlencoded_body_resolves_and_sets_content_type() {
        let vars: HashMap<String, String> = [("username".to_string(), "john doe".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request(A::None);
        req.body = Some(r#"{"type":"url_encoded","items":[{"key":"user","value":"{{username}}","enabled":true},{"key":"ignored","value":"x","enabled":false}]}"#.into());
        let canonical = build(&req, &chain).unwrap();
        assert_eq!(canonical.body, Some("user=john+doe".into()));
        let ct = canonical.headers.iter().find(|h| h.key == "Content-Type").unwrap();
        assert_eq!(ct.value, "application/x-www-form-urlencoded");
    }

    #[test]
    fn graphql_body_resolves_and_sets_content_type() {
        let vars: HashMap<String, String> = [("id".to_string(), "42".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request(A::None);
        req.body = Some(r#"{"type":"graph_ql","query":"query { user(id: {{id}}) { name } }","variables":"{}"}"#.into());
        let canonical = build(&req, &chain).unwrap();
        assert!(canonical.body.as_ref().unwrap().contains("query { user(id: 42) { name } }"));
        let ct = canonical.headers.iter().find(|h| h.key == "Content-Type").unwrap();
        assert_eq!(ct.value, "application/json");
    }

    #[test]
    fn form_data_resolves_into_structured_multipart_parts_not_a_flattened_string() {
        let vars: HashMap<String, String> = [("username".to_string(), "ada".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request(A::None);
        req.body = Some(
            r#"{"type":"form_data","items":[
                {"key":"user","value":"{{username}}","enabled":true,"is_file":false},
                {"key":"ignored","value":"x","enabled":false,"is_file":false},
                {"key":"avatar","value":"","enabled":true,"is_file":true,"file_path":"/tmp/pic.png"}
            ]}"#
                .into(),
        );
        let canonical = build(&req, &chain).unwrap();

        // The old (broken) behavior flattened this into a single "user=ada" string. It must
        // now be structured parts, and `body` must stay empty so http_engine knows to use
        // `.multipart()` instead of `.body()`.
        assert!(canonical.body.is_none(), "FormData must not also set a flat body string");
        let parts = canonical.multipart.expect("FormData must populate `multipart`");
        assert_eq!(parts.len(), 2, "disabled item must be filtered out");

        match &parts[0] {
            ResolvedFormPart::Text { key, value } => {
                assert_eq!(key, "user");
                assert_eq!(value, "ada"); // variable resolved
            }
            ResolvedFormPart::File { .. } => panic!("expected the text part first"),
        }
        match &parts[1] {
            ResolvedFormPart::File { key, file_name, file_path } => {
                assert_eq!(key, "avatar");
                assert_eq!(file_name, "pic.png");
                assert_eq!(file_path, "/tmp/pic.png");
            }
            ResolvedFormPart::Text { .. } => panic!("expected the file part second"),
        }

        // No hand-rolled, boundary-less "multipart/form-data" Content-Type header — that's the
        // HTTP layer's job (reqwest's `.multipart()` generates the real boundary at send time).
        assert!(!canonical.headers.iter().any(|h| h.key.eq_ignore_ascii_case("content-type")));
    }

    #[test]
    fn diagnose_reports_missing_variables_across_all_locations() {
        let chain = ScopeChain::default(); // empty chain
        let mut req = request(A::Bearer { token: "{{missing_token}}".into() });
        req.url = "https://api.example.com/users/{{user_id}}".into();
        req.headers = vec![
            H { key: "X-Header".into(), value: "{{missing_header}}".into(), enabled: true, description: None },
            H { key: "X-Ignored".into(), value: "{{ignored_header}}".into(), enabled: false, description: None },
        ];
        req.query_params = vec![
            crate::models::QueryParam { key: "filter".into(), value: "{{missing_filter}}".into(), enabled: true, description: None },
            crate::models::QueryParam { key: "skip".into(), value: "{{ignored_param}}".into(), enabled: false, description: None },
        ];
        req.body = Some(r#"{"type":"raw","content_type":"application/json","data":"{\"bodyKey\":\"{{missing_body}}\"}"}"#.into());

        let diag = diagnose(&req, &chain);
        assert_eq!(diag.url_missing, vec!["user_id"]);
        assert_eq!(diag.headers_missing, vec!["missing_header"]);
        assert_eq!(diag.query_params_missing, vec!["missing_filter"]);
        assert_eq!(diag.auth_missing, vec!["missing_token"]);
        assert_eq!(diag.body_missing, vec!["missing_body"]);
        assert_eq!(diag.all_missing.len(), 5);
        assert!(diag.all_missing.contains(&"user_id".to_string()));
        assert!(diag.all_missing.contains(&"missing_header".to_string()));
        assert!(diag.all_missing.contains(&"missing_filter".to_string()));
        assert!(diag.all_missing.contains(&"missing_token".to_string()));
        assert!(diag.all_missing.contains(&"missing_body".to_string()));
    }
}
