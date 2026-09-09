//! Script sandbox and Postman-compatible execution engine (LP-0901, LP-0902).
//!
//! Provides a secure, sandboxed JavaScript runtime with:
//! - Strict isolation: zero filesystem, process, network, or OS access.
//! - Controlled APIs: `pm.environment`, `pm.variables`, `pm.response`, `pm.test`, and `console.log`.
//! - Execution limits: timeout protection on a dedicated worker thread.
//! - Clean structured results: variable mutations, assertions, and console logs.

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use boa_engine::{Context, Source};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub success: bool,
    pub environment: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub tests: Vec<TestResult>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct ScriptOutputPayload {
    environment: HashMap<String, String>,
    variables: HashMap<String, String>,
    tests: Vec<TestResult>,
    logs: Vec<String>,
}

/// Executes a pre-request script in the isolated sandbox.
pub fn execute_pre_request_script(
    script: &str,
    environment: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    timeout_ms: u64,
) -> ScriptExecutionResult {
    execute_script_internal(script, environment, variables, None, timeout_ms)
}

/// Executes a post-request/tests script in the isolated sandbox with response context.
pub fn execute_post_request_script(
    script: &str,
    environment: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    response_status: u16,
    response_status_text: &str,
    response_headers: &[(String, String)],
    response_body: &str,
    timeout_ms: u64,
) -> ScriptExecutionResult {
    let resp_ctx = ResponseContext {
        status: response_status,
        status_text: response_status_text.to_string(),
        headers: response_headers.to_vec(),
        body: response_body.to_string(),
    };
    execute_script_internal(script, environment, variables, Some(resp_ctx), timeout_ms)
}

struct ResponseContext {
    status: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    body: String,
}

fn execute_script_internal(
    script: &str,
    environment: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    response: Option<ResponseContext>,
    timeout_ms: u64,
) -> ScriptExecutionResult {
    let script = script.trim().to_string();
    if script.is_empty() {
        return ScriptExecutionResult {
            success: true,
            environment: environment.clone(),
            variables: variables.clone(),
            tests: Vec::new(),
            logs: Vec::new(),
            error: None,
        };
    }

    let env_json = serde_json::to_string(environment).unwrap_or_else(|_| "{}".into());
    let vars_json = serde_json::to_string(variables).unwrap_or_else(|_| "{}".into());

    let (status_code, status_text_json, headers_json, body_json) = if let Some(r) = response {
        let headers_vec: Vec<serde_json::Value> = r
            .headers
            .iter()
            .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
            .collect();
        (
            r.status,
            serde_json::to_string(&r.status_text).unwrap_or_else(|_| "\"\"".into()),
            serde_json::to_string(&headers_vec).unwrap_or_else(|_| "[]".into()),
            serde_json::to_string(&r.body).unwrap_or_else(|_| "\"\"".into()),
        )
    } else {
        (0, "\"\"".into(), "[]".into(), "\"\"".into())
    };

    let wrapped_source = format!(
        r#"(function() {{
    const _env = {env_json};
    const _vars = {vars_json};
    const _tests = [];
    const _logs = [];

    const console = {{
        log: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')),
        info: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')),
        warn: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')),
        error: (...args) => _logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '))
    }};

    const pm = {{
        environment: {{
            get: (key) => _env[key] !== undefined ? _env[key] : null,
            set: (key, val) => {{ _env[key] = String(val); }},
            unset: (key) => {{ delete _env[key]; }},
            has: (key) => key in _env
        }},
        variables: {{
            get: (key) => _vars[key] !== undefined ? _vars[key] : (_env[key] !== undefined ? _env[key] : null),
            set: (key, val) => {{ _vars[key] = String(val); }}
        }},
        test: (name, fn) => {{
            try {{
                fn();
                _tests.push({{ name: String(name), passed: true, error: null }});
            }} catch (err) {{
                _tests.push({{ name: String(name), passed: false, error: String(err && err.message ? err.message : err) }});
            }}
        }},
        response: {{
            code: {status_code},
            status: {status_code},
            statusText: {status_text_json},
            headers: {headers_json},
            text: () => {body_json},
            json: () => JSON.parse({body_json}),
            to: {{
                have: {{
                    status: (expected) => {{
                        if (pm.response.code !== expected) {{
                            throw new Error(`expected status ${{expected}} but got ${{pm.response.code}}`);
                        }}
                    }},
                    header: (key, expectedValue) => {{
                        const found = pm.response.headers.find(h => h.key.toLowerCase() === key.toLowerCase());
                        if (!found) throw new Error(`expected header "${{key}}" to be present`);
                        if (expectedValue !== undefined && found.value !== expectedValue) {{
                            throw new Error(`expected header "${{key}}" to equal "${{expectedValue}}" but got "${{found.value}}"`);
                        }}
                    }}
                }}
            }}
        }}
    }};

    // User script body
    {script}

    return JSON.stringify({{
        environment: _env,
        variables: _vars,
        tests: _tests,
        logs: _logs
    }});
}})()"#
    );

    let (tx, rx) = mpsc::channel();
    let timeout = Duration::from_millis(timeout_ms.max(100));

    std::thread::spawn(move || {
        let mut context = Context::default();
        let eval_res = context.eval(Source::from_bytes(wrapped_source.as_bytes()));
        let res: Result<String, String> = match eval_res {
            Ok(val) => match val.to_string(&mut context) {
                Ok(s) => Ok(s.to_std_string_escaped()),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(res);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(parsed_str)) => {
            match serde_json::from_str::<ScriptOutputPayload>(&parsed_str) {
                Ok(payload) => ScriptExecutionResult {
                    success: true,
                    environment: payload.environment,
                    variables: payload.variables,
                    tests: payload.tests,
                    logs: payload.logs,
                    error: None,
                },
                Err(err) => ScriptExecutionResult {
                    success: false,
                    environment: environment.clone(),
                    variables: variables.clone(),
                    tests: Vec::new(),
                    logs: Vec::new(),
                    error: Some(format!("Failed to parse script execution payload: {err}")),
                },
            }
        }
        Ok(Err(err)) => ScriptExecutionResult {
            success: false,
            environment: environment.clone(),
            variables: variables.clone(),
            tests: Vec::new(),
            logs: Vec::new(),
            error: Some(format!("Script error: {err}")),
        },
        Err(_) => ScriptExecutionResult {
            success: false,
            environment: environment.clone(),
            variables: variables.clone(),
            tests: Vec::new(),
            logs: Vec::new(),
            error: Some(format!("Script execution timed out after {timeout_ms}ms")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_request_script_mutates_environment_and_logs() {
        let mut env = HashMap::new();
        env.insert("baseUrl".into(), "https://api.example.com".into());
        let vars = HashMap::new();

        let script = r#"
            console.log("Setting up session");
            pm.environment.set("token", "secret-token-123");
            pm.variables.set("tempId", "42");
        "#;

        let result = execute_pre_request_script(script, &env, &vars, 1000);
        assert!(result.success, "Script should succeed: {:?}", result.error);
        assert_eq!(result.environment.get("token").map(String::as_str), Some("secret-token-123"));
        assert_eq!(result.variables.get("tempId").map(String::as_str), Some("42"));
        assert!(result.logs.contains(&"Setting up session".to_string()));
    }

    #[test]
    fn post_request_script_runs_tests_and_validates_response() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let headers = vec![("Content-Type".into(), "application/json".into())];
        let body = r#"{"id": 99, "status": "active"}"#;

        let script = r#"
            pm.test("Status code is 200", function () {
                pm.response.to.have.status(200);
            });
            pm.test("Response body has id 99", function () {
                var json = pm.response.json();
                if (json.id !== 99) throw new Error("wrong id");
            });
            pm.test("Intentional failure", function () {
                pm.response.to.have.status(404);
            });
            var data = pm.response.json();
            pm.environment.set("extractedId", data.id);
        "#;

        let result = execute_post_request_script(script, &env, &vars, 200, "OK", &headers, body, 1000);
        assert!(result.success);
        assert_eq!(result.environment.get("extractedId").map(String::as_str), Some("99"));
        assert_eq!(result.tests.len(), 3);
        assert!(result.tests[0].passed);
        assert!(result.tests[1].passed);
        assert!(!result.tests[2].passed);
        assert!(result.tests[2].error.as_ref().unwrap().contains("expected status 404 but got 200"));
    }

    #[test]
    fn infinite_loop_times_out_safely() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let script = "while (true) {}";

        let result = execute_pre_request_script(script, &env, &vars, 200);
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("timed out"));
    }

    #[test]
    fn sandbox_has_no_process_or_filesystem_access() {
        let env = HashMap::new();
        let vars = HashMap::new();
        let script = r#"
            if (typeof process !== "undefined") throw new Error("process exposed");
            if (typeof require !== "undefined") throw new Error("require exposed");
            if (typeof fetch !== "undefined") throw new Error("fetch exposed");
        "#;

        let result = execute_pre_request_script(script, &env, &vars, 1000);
        assert!(result.success, "Sandbox check failed: {:?}", result.error);
    }
}
