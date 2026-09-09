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

use crate::error::AppError;
use crate::http_engine::{self, HttpRequestSpec};
use crate::models::{HeaderEntry, ResponseMeta};
use crate::resolver::{self, ScopeChain};
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
    input: ExecutionInput,
) -> Result<ResponseMeta, AppError> {
    // ---- Load + resolve (synchronous DB section — the guard must never cross an .await,
    // or every other DB-touching command would block for the duration of the network call). ----
    let (spec, request_id) = {
        let conn = db.lock().expect("db mutex poisoned");
        let request = request_store::get_request(&conn, &input.request_id)?;

        // Extension point: a pre-request script runs here, with read/write access to the
        // same variable maps `load_scope_maps` builds below, before resolution happens.
        let (global, environment, request_vars) = variable_store::load_scope_maps(
            &conn,
            &request.project_id,
            input.environment_id.as_deref(),
            Some(&request.id),
        )?;
        let chain = ScopeChain {
            global: Some(&global),
            environment: Some(&environment),
            request: Some(&request_vars),
            ..Default::default()
        };

        let base_url = resolver::resolve_template(&request.url, &chain).resolved;
        let resolved_params: Vec<(String, String)> = request
            .query_params
            .iter()
            .filter(|p| p.enabled)
            .map(|p| {
                (
                    resolver::resolve_template(&p.key, &chain).resolved,
                    resolver::resolve_template(&p.value, &chain).resolved,
                )
            })
            .collect();
        let url = append_query_params(&base_url, &resolved_params)?;
        let headers: Vec<HeaderEntry> = request
            .headers
            .iter()
            .map(|h| HeaderEntry {
                key: h.key.clone(),
                value: resolver::resolve_template(&h.value, &chain).resolved,
                enabled: h.enabled,
            })
            .collect();
        let body = request.body.as_deref().map(|b| resolver::resolve_template(b, &chain).resolved);

        (
            HttpRequestSpec { method: request.method, url, headers, body, timeout_ms: input.timeout_ms },
            request.id,
        )
    };

    // ---- Network call (no DB lock held across this await) ----
    let result = http_engine::execute(client, spec, MAX_INLINE_BODY_BYTES, response_body_dir).await?;

    // Extension point: a post-request/test script runs here, with the resolved request and
    // this `result` available, before the response is persisted.

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

/// Appends `params` to `base_url`'s query string (URL-encoded), preserving any query string
/// already present in `base_url` rather than clobbering it. Runtime-only — the caller never
/// writes the result back to the stored request (README's query-parameter serialization rule).
fn append_query_params(base_url: &str, params: &[(String, String)]) -> Result<String, AppError> {
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
                let request_line = String::from_utf8_lossy(&received)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                *captured_clone.lock().unwrap() = Some(request_line);
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
        let project = project_store::create_project(&conn, NewProjectInput { name: "Demo".into() }).unwrap();
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
                body: None,
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
            ExecutionInput { request_id: "ghost".into(), environment_id: None, timeout_ms: 1000 },
        )
        .await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn append_query_params_preserves_existing_query_string_and_encodes_values() {
        let url = append_query_params(
            "https://api.example.com/search?existing=1",
            &[("q".to_string(), "hello world".to_string())],
        )
        .unwrap();
        assert_eq!(url, "https://api.example.com/search?existing=1&q=hello+world");
    }

    #[test]
    fn append_query_params_is_a_no_op_for_an_empty_list() {
        let url = append_query_params("https://api.example.com/users", &[]).unwrap();
        assert_eq!(url, "https://api.example.com/users");
    }

    #[test]
    fn append_query_params_rejects_an_unparseable_base_url() {
        let result = append_query_params("not a url", &[("a".to_string(), "b".to_string())]);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
