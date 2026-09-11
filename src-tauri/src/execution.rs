//! Request execution pipeline (README §15/§16):
//!
//!   Load request -> Load variables -> [pre-request script] -> Resolve variables
//!     -> HTTP request -> Response -> [post-request/test script] -> Persist result
//!
//! The two bracketed steps aren't implemented (the scripting sandbox is explicitly deferred)
//! but are marked as exact hook points so adding a script engine later means inserting code
//! here, not reshaping this function.

use std::path::Path;
use std::sync::Mutex;

use reqwest::Client;
use rusqlite::Connection;

use crate::canonical_request;
use crate::error::AppError;
use crate::http_engine::{self, HttpRequestSpec};
use crate::models::{HeaderEntry, ResponseMeta};
use crate::resolver::ScopeChain;
use crate::store::{request_store, response_store, variable_store};

/// Bodies larger than this spill to disk instead of growing the in-memory buffer further
/// (README's large-response-handling requirement — see `http_engine::execute`).
const MAX_INLINE_BODY_BYTES: usize = 256 * 1024;

pub struct ExecutionInput {
    pub request_id: String,
    pub environment_id: Option<String>,
    pub timeout_ms: u64,
}

pub async fn execute_request(
    db: &Mutex<Connection>,
    client: &Client,
    response_body_dir: &Path,
    console: Option<&crate::console::ConsoleBuffer>,
    input: ExecutionInput,
) -> Result<ResponseMeta, AppError> {
    let correlation_id = uuid::Uuid::new_v4().to_string();

    // ---- Load + resolve (synchronous DB section — the guard must never cross an .await,
    // or every other DB-touching command would block for the duration of the network call). ----
    let (spec, request_id, project_id_for_cookies, target_domain, post_request_script, post_script_env, post_script_vars, auth_type_label) = {
        let conn = db.lock().expect("db mutex poisoned");
        let request = request_store::get_request(&conn, &input.request_id)?;
        let auth_type_label = match &request.auth {
            crate::models::Auth::None => "none",
            crate::models::Auth::Bearer { .. } => "bearer",
            crate::models::Auth::Basic { .. } => "basic",
            crate::models::Auth::ApiKey { .. } => "api_key",
        };

        let (global, mut environment, mut request_vars) = variable_store::load_scope_maps(
            &conn,
            &request.project_id,
            input.environment_id.as_deref(),
            Some(&request.id),
        )?;

        // Pre-request script execution (LP-0901, LP-0902)
        if let Some(ref script) = request.pre_request_script {
            let res = crate::script_engine::execute_pre_request_script(
                script,
                &environment,
                &request_vars,
                1500,
            );
            if let Some(c) = console {
                for log in &res.logs {
                    c.log(&correlation_id, Some(&request.id), crate::console::ConsoleLevel::Info, "pre_script_log", log, None);
                }
                if let Some(ref err) = res.error {
                    c.log(&correlation_id, Some(&request.id), crate::console::ConsoleLevel::Warn, "pre_script_error", err, None);
                }
            }
            for (k, v) in res.environment {
                environment.insert(k, v);
            }
            for (k, v) in res.variables {
                request_vars.insert(k, v);
            }
        }

        let chain = ScopeChain {
            global: Some(&global),
            environment: Some(&environment),
            request: Some(&request_vars),
            ..Default::default()
        };

        // The single resolved representation (LP-0303) — codegen (curl, etc.) builds the
        // exact same struct from the exact same function, so "what gets sent" and "what
        // gets shown as a snippet" can never drift apart.
        let canonical = canonical_request::build(&request, &chain)?;

        let mut headers = canonical.headers;
        let project_id_for_cookies = request.project_id.clone();
        let target_url_parsed = reqwest::Url::parse(&canonical.url).ok();
        let target_domain = target_url_parsed.as_ref().and_then(|u| u.host_str()).map(|h| h.to_string());

        // Cookie jar integration (LP-0310): inject matching cookies for request's domain/path
        if !headers.iter().any(|h| h.key.eq_ignore_ascii_case("cookie")) {
            if let Ok(project_cookies) = request_store::list_cookies_for_project(&conn, &request.project_id) {
                if let Some(u) = &target_url_parsed {
                    let host = u.host_str().unwrap_or("");
                    let path = u.path();
                    let matched_pairs: Vec<String> = project_cookies
                        .iter()
                        .filter(|c| {
                            let domain_match = host.eq_ignore_ascii_case(&c.domain)
                                || (host.to_lowercase().ends_with(&format!(".{}", c.domain.to_lowercase())));
                            let path_match = path.starts_with(&c.path);
                            domain_match && path_match
                        })
                        .map(|c| format!("{}={}", c.name, c.value))
                        .collect();

                    if !matched_pairs.is_empty() {
                        headers.push(HeaderEntry {
                            key: "Cookie".into(),
                            value: matched_pairs.join("; "),
                            enabled: true,
                            description: None,
                        });
                        if let Some(c) = console {
                            c.log(
                                &correlation_id,
                                Some(&request.id),
                                crate::console::ConsoleLevel::Info,
                                "cookie_injected",
                                &format!("Injected {} cookies matching domain/path", matched_pairs.len()),
                                Some(serde_json::json!({ "cookies": matched_pairs })),
                            );
                        }
                    }
                }
            }
        }

        (
            HttpRequestSpec {
                method: canonical.method,
                url: canonical.url,
                headers,
                body: canonical.body,
                multipart: canonical.multipart,
                body_file_path: canonical.body_file_path,
                timeout_ms: request
                    .settings
                    .as_ref()
                    .and_then(|s| s.timeout_ms)
                    .unwrap_or(input.timeout_ms),
                settings: request.settings.clone(),
            },
            request.id,
            project_id_for_cookies,
            target_domain,
            request.post_request_script,
            environment,
            request_vars,
            auth_type_label,
        )
    };

    if let Some(c) = console {
        let redacted_url = crate::console::redact_url(&spec.url);
        let redacted_headers: Vec<serde_json::Value> = spec
            .headers
            .iter()
            .map(|h| {
                serde_json::json!({
                    "key": h.key,
                    "value": crate::console::redact_header_value(&h.key, &h.value),
                    "enabled": h.enabled,
                })
            })
            .collect();
        c.log(
            &correlation_id,
            Some(&request_id),
            crate::console::ConsoleLevel::Info,
            "request_start",
            &format!("{} {}", spec.method, redacted_url),
            Some(serde_json::json!({
                "method": spec.method,
                "url": redacted_url,
                "headers": redacted_headers,
                "body_bytes": spec.body.as_ref().map(|b| b.len()).unwrap_or(0),
                "project_id": project_id_for_cookies,
                "environment_id": input.environment_id,
                "auth_type": auth_type_label,
                "timeout_ms": spec.timeout_ms,
                "follow_redirects": spec.settings.as_ref().and_then(|s| s.follow_redirects).unwrap_or(true),
                "verify_ssl": spec.settings.as_ref().and_then(|s| s.verify_ssl).unwrap_or(true),
                "proxy": spec.settings.as_ref().and_then(|s| s.proxy_url.clone()),
                "http_version": spec.settings.as_ref().and_then(|s| s.http_version.clone()),
            })),
        );
    }

    // ---- Network call (no DB lock held across this await) ----
    let exec_res = http_engine::execute(client, spec, MAX_INLINE_BODY_BYTES, response_body_dir).await;
    let result = match exec_res {
        Ok(res) => {
            if let Some(c) = console {
                let redacted_resp_headers: Vec<serde_json::Value> = res
                    .headers
                    .iter()
                    .map(|h| {
                        serde_json::json!({
                            "key": h.key,
                            "value": crate::console::redact_header_value(&h.key, &h.value),
                        })
                    })
                    .collect();
                // Cookie values may carry session/auth material — same redaction stance as the
                // Set-Cookie header above (name/domain/path/flags stay visible for debugging,
                // the value never appears in the console).
                let redacted_cookies: Vec<serde_json::Value> = res
                    .cookies
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "name": c.name,
                            "value": if c.value.is_empty() { "" } else { "[REDACTED]" },
                            "domain": c.domain,
                            "path": c.path,
                            "http_only": c.http_only,
                            "secure": c.secure,
                        })
                    })
                    .collect();
                c.log(
                    &correlation_id,
                    Some(&request_id),
                    if res.status >= 400 {
                        crate::console::ConsoleLevel::Warn
                    } else {
                        crate::console::ConsoleLevel::Info
                    },
                    "response_received",
                    &format!(
                        "{} {} ({}ms, {} bytes)",
                        res.status, res.status_text, res.duration_ms, res.body_size
                    ),
                    Some(serde_json::json!({
                        "status": res.status,
                        "status_text": res.status_text,
                        "duration_ms": res.duration_ms,
                        "body_size": res.body_size,
                        "headers": redacted_resp_headers,
                        "cookies": redacted_cookies,
                        "content_type": res.content_type,
                    })),
                );
            }
            res
        }
        Err(err) => {
            if let Some(c) = console {
                c.log(
                    &correlation_id,
                    Some(&request_id),
                    crate::console::ConsoleLevel::Error,
                    "request_error",
                    &format!("Request failed: {err}"),
                    Some(serde_json::json!({ "error": err.to_string() })),
                );
            }
            return Err(err);
        }
    };

    // Post-request / test script execution (LP-0901, LP-0902)
    if let Some(ref script) = post_request_script {
        let resp_body_text = match &result.body {
            crate::http_engine::BodyCapture::Inline(bytes) => {
                String::from_utf8_lossy(bytes).to_string()
            }
            crate::http_engine::BodyCapture::Spilled { path, .. } => {
                std::fs::read_to_string(path).unwrap_or_default()
            }
        };

        let headers_tuples: Vec<(String, String)> = result
            .headers
            .iter()
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();

        let script_res = crate::script_engine::execute_post_request_script(
            script,
            &post_script_env,
            &post_script_vars,
            result.status,
            &result.status_text,
            &headers_tuples,
            &resp_body_text,
            2000,
        );

        if let Some(c) = console {
            for log in &script_res.logs {
                c.log(&correlation_id, Some(&request_id), crate::console::ConsoleLevel::Info, "test_script_log", log, None);
            }
            for t in &script_res.tests {
                let lvl = if t.passed {
                    crate::console::ConsoleLevel::Info
                } else {
                    crate::console::ConsoleLevel::Error
                };
                let msg = if t.passed {
                    format!("PASS: {}", t.name)
                } else {
                    format!("FAIL: {} - {}", t.name, t.error.as_deref().unwrap_or("assertion failed"))
                };
                c.log(&correlation_id, Some(&request_id), lvl, "test_assertion", &msg, Some(serde_json::json!({
                    "name": t.name,
                    "passed": t.passed,
                    "error": t.error
                })));
            }
            if let Some(ref err) = script_res.error {
                c.log(&correlation_id, Some(&request_id), crate::console::ConsoleLevel::Warn, "test_script_error", err, None);
            }
        }
    }

    // Persist received cookies into project cookie jar (LP-0310)
    if !result.cookies.is_empty() {
        let conn = db.lock().expect("db mutex poisoned");
        for rc in &result.cookies {
            let domain = rc.domain.clone().or_else(|| target_domain.clone()).unwrap_or_default();
            let path = rc.path.clone().unwrap_or_else(|| "/".into());
            let _ = request_store::upsert_cookie(
                &conn,
                crate::models::NewCookieInput {
                    project_id: project_id_for_cookies.clone(),
                    domain,
                    path,
                    name: rc.name.clone(),
                    value: rc.value.clone(),
                    expires: rc.expires.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&chrono::Utc))),
                    secure: rc.secure,
                    http_only: rc.http_only,
                    same_site: rc.same_site.clone(),
                },
            );
        }
    }

    // ---- Persist ----
    let conn = db.lock().expect("db mutex poisoned");
    response_store::create_response(
        &conn,
        response_store::NewResponseInput {
            request_id,
            status: result.status,
            status_text: result.status_text,
            headers: result.headers,
            duration_ms: result.duration_ms,
            body_size: result.body_size,
            body: result.body,
        },
    )
}

// Query-param URL-encoding now lives in `canonical_request::append_query_params` (its own
// tests cover it there); `build()` above is the only call site left in this module.

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex as StdMutex};

    use crate::models::{
        NewEnvironmentInput, NewProjectInput, NewRequestInput, NewVariableInput, QueryParam,
        VariableScope,
    };
    use crate::store::{environment_store, project_store, variable_store};

    /// Records the exact request line it received so the test can assert the URL was
    /// actually resolved (not just that *some* request went out).
    fn spawn_recording_server(response: Vec<u8>) -> (u16, Arc<StdMutex<Option<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let port = listener.local_addr().unwrap().port();
        let captured = Arc::new(StdMutex::new(None));
        let captured_clone = captured.clone();

        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_nodelay(true);
                let mut received = Vec::new();
                let mut buf = [0u8; 1024];
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            received.extend_from_slice(&buf[..n]);
                            if received.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                // Full request line + headers (not just the first line) so tests can also
                // assert on things like the resolved Authorization header.
                let request_text = String::from_utf8_lossy(&received).to_string();
                *captured_clone.lock().unwrap() = Some(request_text);
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });
        (port, captured)
    }

    #[tokio::test]
    async fn full_pipeline_resolves_variables_sends_request_and_persists_response() {
        let (port, captured_request) = spawn_recording_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
                .to_vec(),
        );

        let conn = crate::db::open_in_memory().unwrap();
        let project = project_store::create_project(&conn, NewProjectInput { name: "Demo".into(), workspace_id: "default".into() }).unwrap();
        let env = environment_store::create_environment(
            &conn,
            NewEnvironmentInput { project_id: project.id.clone(), name: "Local".into() },
        )
        .unwrap();
        variable_store::create_variable(
            &conn,
            NewVariableInput {
                scope: VariableScope::Environment,
                project_id: None,
                environment_id: Some(env.id.clone()),
                request_id: None,
                key: "port".into(),
                value: port.to_string(),
                enabled: true,
                is_secret: false,
                is_local: false,
                description: None,
            },
        )
        .unwrap();
        let request = crate::store::request_store::create_request(
            &conn,
            NewRequestInput {
                project_id: project.id,
                name: "Ping".into(),
                method: "GET".into(),
                url: "http://127.0.0.1:{{port}}/health".into(),
                headers: vec![],
                query_params: vec![
                    QueryParam { key: "verbose".into(), value: "{{port}}".into(), enabled: true, description: None },
                    QueryParam { key: "skip".into(), value: "1".into(), enabled: false, description: None },
                ],
                auth: crate::models::Auth::Bearer { token: "secret-{{port}}".into() },
                body: None,
                description: None,
                ..Default::default()
            },
        )
        .unwrap();

        let db = Mutex::new(conn);
        let client = Client::new();
        let spill_dir = std::env::temp_dir().join("postman-client-test-execution");

        let meta = execute_request(
            &db,
            &client,
            &spill_dir,
            None,
            ExecutionInput {
                request_id: request.id.clone(),
                environment_id: Some(env.id),
                timeout_ms: 5000,
            },
        )
        .await
        .unwrap();

        assert_eq!(meta.status, 200);
        assert_eq!(meta.request_id, request.id);

        // Proves the {{port}} template was actually substituted before the request went out,
        // and that query params were appended (enabled one present, disabled one omitted).
        let request_line = captured_request.lock().unwrap().clone().unwrap();
        assert!(request_line.contains("/health"), "unexpected request line: {request_line}");
        assert!(
            request_line.contains(&format!("verbose={port}")),
            "unexpected request line: {request_line}"
        );
        assert!(!request_line.contains("skip="), "disabled query param must be omitted: {request_line}");
        assert!(
            request_line.to_lowercase().contains(&format!("authorization: bearer secret-{port}")),
            "expected resolved Authorization header, got: {request_line}"
        );

        // And that it was actually persisted, not just returned.
        let conn = db.lock().unwrap();
        let history = response_store::list_summaries(&conn, &request.id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, 200);
    }

    #[tokio::test]
    async fn missing_request_returns_not_found_without_making_a_network_call() {
        let conn = crate::db::open_in_memory().unwrap();
        let db = Mutex::new(conn);
        let client = Client::new();
        let spill_dir = std::env::temp_dir().join("postman-client-test-execution-missing");

        let result = execute_request(
            &db,
            &client,
            &spill_dir,
            None,
            ExecutionInput { request_id: "ghost".into(), environment_id: None, timeout_ms: 1000 },
        )
        .await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn cookie_jar_injects_matching_cookie_and_persists_response_set_cookie() {
        let (port, captured_request) = spawn_recording_server(
            b"HTTP/1.1 200 OK\r\nSet-Cookie: session_id=sess_999; Path=/\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
                .to_vec(),
        );

        let conn = crate::db::open_in_memory().unwrap();
        let project = project_store::create_project(&conn, NewProjectInput { name: "Cookie Test".into(), workspace_id: "default".into() }).unwrap();

        // Pre-populate an existing cookie
        request_store::create_cookie(
            &conn,
            crate::models::NewCookieInput {
                project_id: project.id.clone(),
                domain: "127.0.0.1".into(),
                path: "/".into(),
                name: "auth_token".into(),
                value: "token_123".into(),
                expires: None,
                secure: false,
                http_only: true,
                same_site: None,
            },
        )
        .unwrap();

        let request = request_store::create_request(
            &conn,
            NewRequestInput {
                project_id: project.id.clone(),
                name: "Cookie Ping".into(),
                method: "GET".into(),
                url: format!("http://127.0.0.1:{port}/api"),
                ..Default::default()
            },
        )
        .unwrap();

        let db = Mutex::new(conn);
        let client = Client::new();
        let spill_dir = std::env::temp_dir().join("postman-client-test-cookies");

        let _ = execute_request(
            &db,
            &client,
            &spill_dir,
            None,
            ExecutionInput {
                request_id: request.id.clone(),
                environment_id: None,
                timeout_ms: 5000,
            },
        )
        .await
        .unwrap();

        // 1. Verify outgoing request had Cookie: auth_token=token_123 injected
        let request_text = captured_request.lock().unwrap().clone().unwrap();
        assert!(
            request_text.to_lowercase().contains("cookie: auth_token=token_123"),
            "expected cookie header in request, got: {request_text}"
        );

        // 2. Verify response Set-Cookie was persisted into DB
        let conn = db.lock().unwrap();
        let saved_cookies = request_store::list_cookies_for_project(&conn, &project.id).unwrap();
        let session_cookie = saved_cookies.iter().find(|c| c.name == "session_id");
        assert!(session_cookie.is_some(), "session_id cookie should have been persisted");
        assert_eq!(session_cookie.unwrap().value, "sess_999");
    }

    #[tokio::test]
    async fn console_logs_request_and_response_lifecycle_with_redaction() {
        let (port, _) = spawn_recording_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nSet-Cookie: session_id=sess_abc; Path=/\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}"
                .to_vec(),
        );

        let conn = crate::db::open_in_memory().unwrap();
        let project = project_store::create_project(&conn, NewProjectInput { name: "Console Test".into(), workspace_id: "default".into() }).unwrap();

        let request = request_store::create_request(
            &conn,
            NewRequestInput {
                project_id: project.id.clone(),
                name: "Console Ping".into(),
                method: "GET".into(),
                url: format!("http://127.0.0.1:{port}/secure?token=secret123"),
                auth: crate::models::Auth::Bearer { token: "super_secret_bearer".into() },
                ..Default::default()
            },
        )
        .unwrap();

        let db = Mutex::new(conn);
        let client = Client::new();
        let spill_dir = std::env::temp_dir().join("postman-client-test-console");
        let console_buffer = crate::console::ConsoleBuffer::new(50);

        let meta = execute_request(
            &db,
            &client,
            &spill_dir,
            Some(&console_buffer),
            ExecutionInput {
                request_id: request.id.clone(),
                environment_id: None,
                timeout_ms: 5000,
            },
        )
        .await
        .unwrap();

        assert_eq!(meta.status, 200);

        let events = console_buffer.get_events(None, None, Some(&request.id));
        assert!(events.len() >= 2, "expected at least request_start and response_received events");

        let start_evt = events.iter().find(|e| e.event_type == "request_start").unwrap();
        let resp_evt = events.iter().find(|e| e.event_type == "response_received").unwrap();

        // Correlation ID is identical
        assert_eq!(start_evt.correlation_id, resp_evt.correlation_id);

        // Sensitive url parameter is redacted
        assert!(start_evt.message.contains("REDACTED"));
        assert!(!start_evt.message.contains("secret123"));

        // Sensitive authorization header in details is redacted
        let details_str = serde_json::to_string(&start_evt.details).unwrap();
        assert!(details_str.contains("[REDACTED]"));
        assert!(!details_str.contains("super_secret_bearer"));

        // Debugging metadata the spec explicitly asks for is present on request_start.
        let start_details = start_evt.details.as_ref().unwrap();
        assert_eq!(start_details["auth_type"], "bearer");
        assert_eq!(start_details["project_id"], project.id);
        assert!(start_details["environment_id"].is_null());
        assert_eq!(start_details["timeout_ms"], 5000);

        // Response metadata: content type surfaced, cookie name/flags visible but value redacted.
        let resp_details = resp_evt.details.as_ref().unwrap();
        assert_eq!(resp_details["content_type"], "application/json");
        let cookies = resp_details["cookies"].as_array().unwrap();
        assert_eq!(cookies[0]["name"], "session_id");
        assert_eq!(cookies[0]["value"], "[REDACTED]");
        let resp_details_str = serde_json::to_string(resp_details).unwrap();
        assert!(!resp_details_str.contains("sess_abc"));
    }
}
