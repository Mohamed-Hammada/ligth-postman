use postman_client_lib::db;
use postman_client_lib::models::{HeaderEntry, NewProjectInput, NewRequestInput};
use postman_client_lib::resolver::{self, ScopeChain};
use postman_client_lib::script_engine;
use postman_client_lib::store::{project_store, request_store};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn scale_100_projects_and_1000_requests_in_sqlite() {
    let db_path = std::env::temp_dir().join(format!("scale_test_{}.db", uuid::Uuid::new_v4()));
    let conn = db::open(&db_path).expect("open sqlite db");

    let start_insert = Instant::now();

    // 1. Insert 100 projects
    let mut project_ids = Vec::with_capacity(100);
    for i in 1..=100 {
        let p = project_store::create_project(
            &conn,
            NewProjectInput {
                name: format!("Scale Project {i}"),
                workspace_id: "default".into(),
            },
        )
        .expect("create project");
        project_ids.push(p.id);
    }
    assert_eq!(project_ids.len(), 100);

    // 2. Insert 1,000 requests distributed evenly across the 100 projects (10 requests per project)
    for (idx, p_id) in project_ids.iter().enumerate() {
        for r in 1..=10 {
            let req_num = idx * 10 + r;
            request_store::create_request(
                &conn,
                NewRequestInput {
                    project_id: p_id.clone(),
                    name: format!("Request #{req_num}"),
                    method: if req_num % 2 == 0 { "GET".into() } else { "POST".into() },
                    url: format!("https://api.example.com/v1/items/{req_num}"),
                    headers: vec![HeaderEntry {
                        key: "Authorization".into(),
                        value: "Bearer {{token}}".into(),
                        enabled: true,
                        description: None,
                    }],
                    body: if req_num % 2 == 1 {
                        Some("{\"action\":\"test\"}".into())
                    } else {
                        None
                    },
                    ..Default::default()
                },
            )
            .expect("create request");
        }
    }

    let insert_duration = start_insert.elapsed();
    println!("Inserted 100 projects and 1,000 requests in {insert_duration:?}");

    // 3. Fast query verification: list requests for a specific project using indexed column
    let query_start = Instant::now();
    let sample_project_id = &project_ids[42];
    let project_requests = request_store::list_requests(&conn, sample_project_id)
        .expect("list requests by project");
    let query_duration = query_start.elapsed();

    assert_eq!(project_requests.len(), 10);
    assert!(
        query_duration.as_millis() < 50,
        "Indexed query should be sub-50ms, took: {query_duration:?}"
    );

    // 4. Verify system diagnostics reflect exact counts
    let diagnostics = postman_client_lib::diagnostics::collect_system_diagnostics(
        &conn,
        Some(&db_path),
        0,
        false,
        10,
    )
    .expect("collect diagnostics");

    assert_eq!(diagnostics.total_projects, 100);
    assert_eq!(diagnostics.total_requests, 1000);
    assert!(diagnostics.db_size_bytes > 0);
    assert!(diagnostics.process_rss_bytes > 0);

    // Clean up temp file
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn variable_scope_resolution_scale_benchmark() {
    let mut global_map = HashMap::new();
    let mut env_map = HashMap::new();
    let mut req_map = HashMap::new();

    // Populate 50 variables in each scope = 150 variables total
    for i in 1..=50 {
        global_map.insert(format!("g_var_{i}"), format!("g_val_{i}"));
        env_map.insert(format!("e_var_{i}"), format!("e_val_{i}"));
        req_map.insert(format!("r_var_{i}"), format!("r_val_{i}"));
    }

    let chain = ScopeChain {
        runtime: None,
        request: Some(&req_map),
        folder: None,
        collection: None,
        environment: Some(&env_map),
        global: Some(&global_map),
    };

    let template_url = "https://{{g_var_1}}.example.com/{{e_var_2}}/api/{{r_var_3}}?key={{g_var_4}}&filter={{e_var_5}}";
    let template_body = r#"{"id":"{{r_var_10}}","env":"{{e_var_20}}","global":"{{g_var_30}}","nested":{"a":"{{r_var_40}}"}}"#;

    let start = Instant::now();
    let iterations = 2_000;
    for _ in 0..iterations {
        let res_u = resolver::resolve_template(template_url, &chain);
        assert!(res_u.missing.is_empty());
        assert_eq!(
            res_u.resolved,
            "https://g_val_1.example.com/e_val_2/api/r_val_3?key=g_val_4&filter=e_val_5"
        );

        let res_b = resolver::resolve_template(template_body, &chain);
        assert!(res_b.missing.is_empty());
        assert_eq!(
            res_b.resolved,
            r#"{"id":"r_val_10","env":"e_val_20","global":"g_val_30","nested":{"a":"r_val_40"}}"#
        );
    }
    let elapsed = start.elapsed();
    println!("Completed {iterations} variable template resolutions across 150 scopes in {elapsed:?}");
    assert!(
        elapsed.as_millis() < 1500,
        "Variable resolution took too long: {elapsed:?}"
    );
}

#[test]
fn script_sandbox_scale_execution() {
    let mut env = HashMap::new();
    env.insert("counter".to_string(), "0".to_string());
    let vars = HashMap::new();

    let script = r#"
        let count = parseInt(pm.environment.get("counter") || "0");
        pm.environment.set("counter", (count + 1).toString());
        pm.test("Status check", function() {
            pm.response.to.have.status(200);
        });
    "#;

    let headers = vec![("Content-Type".into(), "application/json".into())];
    let body = r#"{"status":"ok","items":[1,2,3]}"#;

    let start = Instant::now();
    let executions = 20;

    for i in 0..executions {
        let result = script_engine::execute_post_request_script(
            script,
            &env,
            &vars,
            200,
            "OK",
            &headers,
            body,
            1000,
        );

        assert!(result.success);
        assert_eq!(result.tests.len(), 1);
        assert!(result.tests[0].passed);
        assert_eq!(result.environment.get("counter").unwrap(), &(i + 1).to_string());

        // Update env for next execution
        env = result.environment;
    }

    let elapsed = start.elapsed();
    println!("Completed {executions} sandboxed JS test script executions in {elapsed:?}");
    assert!(
        elapsed.as_millis() < 2000,
        "Sandbox executions took too long: {elapsed:?}"
    );
}
