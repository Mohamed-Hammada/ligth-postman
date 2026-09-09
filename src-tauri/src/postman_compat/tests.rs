use super::*;
use crate::db;
use crate::models::{
    Auth, HeaderEntry, NewProjectInput, NewRequestInput, NewSampleResponseInput, NewVariableInput,
    QueryParam, VariableScope, VariableView,
};
use crate::store::{project_store, request_store, variable_store};
use rusqlite::Connection;

fn setup_db() -> Connection {
    db::open_in_memory().unwrap()
}

#[test]
fn imports_basic_collection_with_variables() {
    let conn = setup_db();
    let json = r#"{
        "info": {
            "name": "Test API",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "variable": [
            { "key": "baseUrl", "value": "https://api.test.com" }
        ],
        "item": [
            {
                "name": "Get Status",
                "request": {
                    "method": "GET",
                    "url": "https://api.test.com/status?version=1",
                    "header": [
                        { "key": "Accept", "value": "application/json" }
                    ]
                }
            }
        ]
    }"#;

    let report = import_collection(&conn, json, None).unwrap();
    assert_eq!(report.project_name, "Test API");
    assert_eq!(report.requests_count, 1);
    assert_eq!(report.variables_count, 1);

    let requests = request_store::list_requests(&conn, &report.project_id).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].name, "Get Status");

    let req = request_store::get_request(&conn, &requests[0].id).unwrap();
    assert_eq!(req.method, "GET");
    assert_eq!(req.url, "https://api.test.com/status");
    assert_eq!(req.query_params.len(), 1);
    assert_eq!(req.query_params[0].key, "version");
    assert_eq!(req.query_params[0].value, "1");
    assert_eq!(req.headers.len(), 1);
    assert_eq!(req.headers[0].key, "Accept");

    let vars = variable_store::list_variables_for_scope(&conn, VariableScope::Global, &report.project_id).unwrap();
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].key, "baseUrl");
    assert_eq!(vars[0].value, "https://api.test.com");
}

#[test]
fn imports_nested_folders_auth_scripts_and_sample_responses() {
    let conn = setup_db();
    let json = r#"{
        "info": {
            "name": "Complex API"
        },
        "item": [
            {
                "name": "Users",
                "item": [
                    {
                        "name": "Authentication",
                        "item": [
                            {
                                "name": "Login",
                                "request": {
                                    "method": "POST",
                                    "url": {
                                        "raw": "https://api.test.com/v1/auth/login",
                                        "protocol": "https",
                                        "host": ["api", "test", "com"],
                                        "path": ["v1", "auth", "login"],
                                        "query": [
                                            { "key": "debug", "value": "true" }
                                        ]
                                    },
                                    "header": [
                                        { "key": "Content-Type", "value": "application/json" }
                                    ],
                                    "body": {
                                        "mode": "raw",
                                        "raw": "{\"user\": \"alice\"}"
                                    },
                                    "auth": {
                                        "type": "bearer",
                                        "bearer": [
                                            { "key": "token", "value": "my-token-123" }
                                        ]
                                    },
                                    "description": "Log into the API"
                                },
                                "event": [
                                    {
                                        "listen": "prerequest",
                                        "script": {
                                            "exec": ["console.log('before send');"]
                                        }
                                    },
                                    {
                                        "listen": "test",
                                        "script": {
                                            "exec": ["pm.test('200 ok', function() {});"]
                                        }
                                    }
                                ],
                                "response": [
                                    {
                                        "name": "Success 200",
                                        "code": 200,
                                        "header": [
                                            { "key": "Content-Type", "value": "application/json" }
                                        ],
                                        "body": "{\"token\": \"xyz\"}"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        ]
    }"#;

    let report = import_collection(&conn, json, None).unwrap();
    assert_eq!(report.requests_count, 1);
    assert_eq!(report.sample_responses_count, 1);

    let requests = request_store::list_requests(&conn, &report.project_id).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].name, "Users / Authentication / Login");

    let req = request_store::get_request(&conn, &requests[0].id).unwrap();
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://api.test.com/v1/auth/login");
    assert_eq!(req.query_params.len(), 1);
    assert_eq!(req.query_params[0].key, "debug");
    assert_eq!(req.body.as_deref(), Some("{\"user\": \"alice\"}"));
    assert_eq!(req.auth, Auth::Bearer { token: "my-token-123".into() });
    assert_eq!(req.pre_request_script.as_deref(), Some("console.log('before send');"));
    assert_eq!(req.post_request_script.as_deref(), Some("pm.test('200 ok', function() {});"));
    assert_eq!(req.description.as_deref(), Some("Log into the API"));

    let samples = request_store::list_sample_responses(&conn, &req.id).unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].name, "Success 200");
    assert_eq!(samples[0].status, 200);
    assert_eq!(samples[0].body.as_deref(), Some("{\"token\": \"xyz\"}"));
}

#[test]
fn imports_environment_with_secrets() {
    let conn = setup_db();
    let project = project_store::create_project(&conn, NewProjectInput { name: "Env Test Project".into() }).unwrap();

    let json = r#"{
        "name": "Staging Environment",
        "values": [
            {
                "key": "apiUrl",
                "value": "https://staging.api.com",
                "enabled": true,
                "type": "default"
            },
            {
                "key": "apiKey",
                "value": "secret-staging-key",
                "enabled": true,
                "type": "secret"
            }
        ]
    }"#;

    let report = import_environment(&conn, json, &project.id).unwrap();
    assert_eq!(report.environment_name, "Staging Environment");
    assert_eq!(report.variables_count, 2);

    let vars = variable_store::list_variables_for_scope(&conn, VariableScope::Environment, &report.environment_id).unwrap();
    assert_eq!(vars.len(), 2);

    let api_url = vars.iter().find(|v| v.key == "apiUrl").unwrap();
    assert_eq!(api_url.value, "https://staging.api.com");
    assert!(!api_url.is_secret);

    let api_key = vars.iter().find(|v| v.key == "apiKey").unwrap();
    assert!(api_key.is_secret);
    assert_eq!(api_key.value, "secret-staging-key");
    // VariableView masks secret value for frontend
    let view = VariableView::from(api_key.clone());
    assert_eq!(view.value, "••••••••");
}

#[test]
fn exports_collection_and_roundtrips() {
    let conn = setup_db();
    let project = project_store::create_project(&conn, NewProjectInput { name: "Exportable API".into() }).unwrap();

    let _ = variable_store::create_variable(
        &conn,
        NewVariableInput {
            scope: VariableScope::Global,
            project_id: Some(project.id.clone()),
            environment_id: None,
            request_id: None,
            key: "host".into(),
            value: "example.com".into(),
            enabled: true,
            is_secret: false,
            is_local: false,
            description: None,
        },
    );

    let req = request_store::create_request(
        &conn,
        NewRequestInput {
            project_id: project.id.clone(),
            name: "Search Users".into(),
            method: "GET".into(),
            url: "https://{{host}}/search".into(),
            headers: vec![HeaderEntry {
                key: "Accept".into(),
                value: "application/json".into(),
                enabled: true,
                description: None,
            }],
            query_params: vec![QueryParam {
                key: "q".into(),
                value: "alice".into(),
                enabled: true,
                description: None,
            }],
            auth: Auth::Bearer { token: "{{token}}".into() },
            body: Some("{}".into()),
            description: Some("Search endpoint".into()),
            settings: None,
            pre_request_script: Some("// pre script".into()),
            post_request_script: Some("// post script".into()),
        },
    ).unwrap();

    let _ = request_store::create_sample_response(
        &conn,
        NewSampleResponseInput {
            request_id: req.id.clone(),
            name: "Sample 200".into(),
            status: 200,
            status_text: "OK".into(),
            headers: vec![],
            body: Some("{\"results\": []}".into()),
            content_type: Some("application/json".into()),
        },
    ).unwrap();

    // Export to Postman Collection v2.1
    let exported_json = export_project_collection(&conn, &project.id).unwrap();
    assert!(exported_json.contains("Exportable API"));
    assert!(exported_json.contains("https://schema.getpostman.com/json/collection/v2.1.0/collection.json"));
    assert!(exported_json.contains("Search Users"));
    assert!(exported_json.contains("Sample 200"));

    // Now re-import the exported collection as a new project
    let import_report = import_collection(&conn, &exported_json, None).unwrap();
    assert_eq!(import_report.project_name, "Exportable API");
    assert_eq!(import_report.requests_count, 1);
    assert_eq!(import_report.variables_count, 1);
    assert_eq!(import_report.sample_responses_count, 1);

    let re_reqs = request_store::list_requests(&conn, &import_report.project_id).unwrap();
    let re_req = request_store::get_request(&conn, &re_reqs[0].id).unwrap();
    assert_eq!(re_req.name, "Search Users");
    assert_eq!(re_req.method, "GET");
    assert_eq!(re_req.url, "https://{{host}}/search");
    assert_eq!(re_req.query_params.len(), 1);
    assert_eq!(re_req.query_params[0].key, "q");
    assert_eq!(re_req.query_params[0].value, "alice");
    assert_eq!(re_req.headers.len(), 1);
    assert_eq!(re_req.headers[0].key, "Accept");
    assert_eq!(re_req.auth, Auth::Bearer { token: "{{token}}".into() });
    assert_eq!(re_req.pre_request_script.as_deref(), Some("// pre script"));
    assert_eq!(re_req.post_request_script.as_deref(), Some("// post script"));
}

#[test]
fn unsupported_auth_adds_warning_and_falls_back_to_none() {
    let conn = setup_db();
    let json = r#"{
        "info": { "name": "Legacy Auth Test" },
        "item": [
            {
                "name": "OAuth1 Request",
                "request": {
                    "method": "GET",
                    "url": "https://api.test.com",
                    "auth": {
                        "type": "oauth1"
                    }
                }
            }
        ]
    }"#;

    let report = import_collection(&conn, json, None).unwrap();
    assert_eq!(report.warnings.len(), 1);
    assert!(report.warnings[0].contains("oauth1"));

    let reqs = request_store::list_requests(&conn, &report.project_id).unwrap();
    let req = request_store::get_request(&conn, &reqs[0].id).unwrap();
    assert_eq!(req.auth, Auth::None);
}
