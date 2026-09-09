use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::schema::*;
use crate::error::AppError;
use crate::models::{
    Auth, HeaderEntry, NewEnvironmentInput, NewProjectInput, NewRequestInput,
    NewSampleResponseInput, NewVariableInput, QueryParam, VariableScope,
};
use crate::store::{environment_store, project_store, request_store, variable_store};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionImportReport {
    pub project_id: String,
    pub project_name: String,
    pub requests_count: usize,
    pub variables_count: usize,
    pub sample_responses_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentImportReport {
    pub environment_id: String,
    pub environment_name: String,
    pub variables_count: usize,
    pub warnings: Vec<String>,
}

pub fn import_collection(
    conn: &Connection,
    json_str: &str,
    target_project_id: Option<String>,
) -> Result<CollectionImportReport, AppError> {
    let collection: PostmanCollection = serde_json::from_str(json_str)
        .map_err(|err| AppError::Validation(format!("Invalid Postman Collection JSON: {err}")))?;

    let mut warnings = Vec::new();

    // Begin transaction for large collection import safety (LP-0503)
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|err| AppError::Storage(err.to_string()))?;

    let result = (|| -> Result<CollectionImportReport, AppError> {
        let (project_id, project_name) = match target_project_id {
            Some(pid) => {
                let existing = project_store::get_project(conn, &pid)?;
                (existing.id, existing.name)
            }
            None => {
                let name = if collection.info.name.trim().is_empty() {
                    "Imported Postman Collection".to_string()
                } else {
                    collection.info.name.trim().to_string()
                };
                let p = project_store::create_project(conn, NewProjectInput { name })?;
                (p.id, p.name)
            }
        };

        // 1. Import Collection variables as Global Project Variables
        let mut variables_count = 0;
        for var in &collection.variable {
            if let Some(key) = &var.key {
                if !key.trim().is_empty() {
                    let val = var.value.clone().unwrap_or_default();
                    let is_secret = var.r#type.as_deref() == Some("secret");
                    let _ = variable_store::create_variable(
                        conn,
                        NewVariableInput {
                            scope: VariableScope::Global,
                            project_id: Some(project_id.clone()),
                            environment_id: None,
                            request_id: None,
                            key: key.trim().to_string(),
                            value: val,
                            enabled: true,
                            is_secret,
                            is_local: false,
                            description: None,
                        },
                    );
                    variables_count += 1;
                }
            }
        }

        // 2. Recursively import items
        let mut requests_count = 0;
        let mut sample_responses_count = 0;
        let mut folder_stack = Vec::new();

        import_items_recursive(
            conn,
            &collection.item,
            &project_id,
            &mut folder_stack,
            &mut requests_count,
            &mut sample_responses_count,
            &mut warnings,
        )?;

        Ok(CollectionImportReport {
            project_id,
            project_name,
            requests_count,
            variables_count,
            sample_responses_count,
            warnings,
        })
    })();

    match result {
        Ok(report) => {
            conn.execute_batch("COMMIT;")
                .map_err(|err| AppError::Storage(err.to_string()))?;
            Ok(report)
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(err)
        }
    }
}

fn import_items_recursive(
    conn: &Connection,
    items: &[PostmanItem],
    project_id: &str,
    folder_stack: &mut Vec<String>,
    requests_count: &mut usize,
    sample_responses_count: &mut usize,
    warnings: &mut Vec<String>,
) -> Result<(), AppError> {
    for item in items {
        let item_name = item.name.as_deref().unwrap_or("Untitled");

        // If this item is a folder containing sub-items
        if let Some(sub_items) = &item.item {
            folder_stack.push(item_name.to_string());
            import_items_recursive(
                conn,
                sub_items,
                project_id,
                folder_stack,
                requests_count,
                sample_responses_count,
                warnings,
            )?;
            folder_stack.pop();
        }

        // If this item has a request
        if let Some(req) = &item.request {
            let full_name = if folder_stack.is_empty() {
                item_name.to_string()
            } else {
                format!("{} / {}", folder_stack.join(" / "), item_name)
            };

            let method = req.method.as_deref().unwrap_or("GET").to_uppercase();
            let (url, query_params) = parse_postman_url(&req.url);

            let headers = req
                .header
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter_map(|h| {
                    let key = h.key.as_deref()?.trim();
                    if key.is_empty() {
                        return None;
                    }
                    Some(HeaderEntry {
                        key: key.to_string(),
                        value: h.value.clone().unwrap_or_default(),
                        enabled: !h.disabled.unwrap_or(false),
                        description: h.description.clone(),
                    })
                })
                .collect();

            let auth = parse_postman_auth(req.auth.as_ref(), warnings, &full_name);
            let body = parse_postman_body(req.body.as_ref(), warnings, &full_name);

            let (pre_script, post_script) = parse_postman_scripts(item.event.as_deref());

            let description = req.description.clone().or_else(|| item.description.clone());

            let created_request = request_store::create_request(
                conn,
                NewRequestInput {
                    project_id: project_id.to_string(),
                    name: full_name,
                    method,
                    url,
                    headers,
                    query_params,
                    auth,
                    body,
                    description,
                    settings: None,
                    pre_request_script: pre_script,
                    post_request_script: post_script,
                },
            )?;
            *requests_count += 1;

            // Import sample responses if present
            if let Some(responses) = &item.response {
                for resp in responses {
                    let resp_name = resp.name.as_deref().unwrap_or("Sample Response");
                    let status = resp.code.unwrap_or(200);
                    let resp_headers = resp
                        .header
                        .as_deref()
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|h| {
                            let key = h.key.as_deref()?.trim();
                            if key.is_empty() {
                                return None;
                            }
                            Some(HeaderEntry {
                                key: key.to_string(),
                                value: h.value.clone().unwrap_or_default(),
                                enabled: true,
                                description: None,
                            })
                        })
                        .collect();

                    let _ = request_store::create_sample_response(
                        conn,
                        NewSampleResponseInput {
                            request_id: created_request.id.clone(),
                            name: resp_name.to_string(),
                            status,
                            status_text: resp.status.clone().unwrap_or_else(|| "OK".into()),
                            headers: resp_headers,
                            body: resp.body.clone(),
                            content_type: None,
                        },
                    );
                    *sample_responses_count += 1;
                }
            }
        }
    }

    Ok(())
}

fn parse_postman_url(url_val: &PostmanUrlOrString) -> (String, Vec<QueryParam>) {
    match url_val {
        PostmanUrlOrString::Plain(raw_str) => {
            extract_query_from_raw_url(raw_str)
        }
        PostmanUrlOrString::Structured(structured) => {
            let mut query_params = Vec::new();
            if let Some(queries) = &structured.query {
                for q in queries {
                    if let Some(key) = &q.key {
                        if !key.trim().is_empty() {
                            query_params.push(QueryParam {
                                key: key.trim().to_string(),
                                value: q.value.clone().unwrap_or_default(),
                                enabled: !q.disabled.unwrap_or(false),
                                description: q.description.clone(),
                            });
                        }
                    }
                }
            }

            let base_url = if let Some(raw) = &structured.raw {
                let (cleaned, extra_params) = extract_query_from_raw_url(raw);
                if query_params.is_empty() {
                    query_params = extra_params;
                }
                cleaned
            } else {
                // Synthesize from protocol, host, path
                let proto = structured.protocol.as_deref().unwrap_or("https");
                let host = match &structured.host {
                    Some(PostmanSegments::List(parts)) => parts.join("."),
                    Some(PostmanSegments::Single(s)) => s.clone(),
                    None => "localhost".to_string(),
                };
                let path = match &structured.path {
                    Some(PostmanSegments::List(parts)) => parts.join("/"),
                    Some(PostmanSegments::Single(s)) => s.clone(),
                    None => String::new(),
                };
                if path.is_empty() {
                    format!("{proto}://{host}")
                } else {
                    format!("{proto}://{host}/{path}")
                }
            };

            (base_url, query_params)
        }
    }
}

fn extract_query_from_raw_url(raw: &str) -> (String, Vec<QueryParam>) {
    if let Some(idx) = raw.find('?') {
        let base = raw[..idx].to_string();
        let query_str = &raw[idx + 1..];
        let mut params = Vec::new();
        for pair in query_str.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (k, v) = match pair.find('=') {
                Some(eq_idx) => (&pair[..eq_idx], &pair[eq_idx + 1..]),
                None => (pair, ""),
            };
            params.push(QueryParam {
                key: k.to_string(),
                value: v.to_string(),
                enabled: true,
                description: None,
            });
        }
        (base, params)
    } else {
        (raw.to_string(), Vec::new())
    }
}

fn parse_postman_auth(
    auth: Option<&PostmanAuth>,
    warnings: &mut Vec<String>,
    req_name: &str,
) -> Auth {
    let Some(auth) = auth else {
        return Auth::None;
    };

    match auth.r#type.to_lowercase().as_str() {
        "noauth" => Auth::None,
        "bearer" => {
            let token = auth
                .bearer
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .find(|p| p.key == "token")
                .and_then(|p| p.value.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            Auth::Bearer { token }
        }
        "basic" => {
            let params = auth.basic.as_deref().unwrap_or(&[]);
            let username = params
                .iter()
                .find(|p| p.key == "username")
                .and_then(|p| p.value.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let password = params
                .iter()
                .find(|p| p.key == "password")
                .and_then(|p| p.value.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            Auth::Basic { username, password }
        }
        "apikey" | "api_key" => {
            let params = auth.apikey.as_deref().unwrap_or(&[]);
            let key = params
                .iter()
                .find(|p| p.key == "key")
                .and_then(|p| p.value.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let value = params
                .iter()
                .find(|p| p.key == "value")
                .and_then(|p| p.value.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let location = params
                .iter()
                .find(|p| p.key == "in")
                .and_then(|p| p.value.as_str())
                .unwrap_or("header");
            let location = if location.eq_ignore_ascii_case("query") {
                crate::models::ApiKeyLocation::Query
            } else {
                crate::models::ApiKeyLocation::Header
            };
            Auth::ApiKey { key, value, location }
        }
        other => {
            warnings.push(format!(
                "Request '{req_name}': auth type '{other}' is unsupported; imported with No Auth."
            ));
            Auth::None
        }
    }
}

fn parse_postman_body(
    body: Option<&PostmanBody>,
    warnings: &mut Vec<String>,
    req_name: &str,
) -> Option<String> {
    let body = body?;
    let mode = body.mode.as_deref().unwrap_or("raw");

    match mode {
        "raw" => body.raw.clone(),
        "graphql" => {
            if let Some(gql) = &body.graphql {
                let mut map = serde_json::Map::new();
                if let Some(q) = &gql.query {
                    map.insert("query".to_string(), serde_json::Value::String(q.clone()));
                }
                if let Some(vars) = &gql.variables {
                    if let Ok(parsed_vars) = serde_json::from_str::<serde_json::Value>(vars) {
                        map.insert("variables".to_string(), parsed_vars);
                    } else if !vars.trim().is_empty() {
                        map.insert("variables".to_string(), serde_json::Value::String(vars.clone()));
                    }
                }
                Some(serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_default())
            } else {
                body.raw.clone()
            }
        }
        "urlencoded" => {
            if let Some(items) = &body.urlencoded {
                let mut parts = Vec::new();
                for item in items {
                    if !item.disabled.unwrap_or(false) && !item.key.is_empty() {
                        parts.push(format!(
                            "{}={}",
                            item.key,
                            item.value.as_deref().unwrap_or_default()
                        ));
                    }
                }
                Some(parts.join("&"))
            } else {
                None
            }
        }
        "formdata" => {
            if let Some(items) = &body.formdata {
                let mut parts = Vec::new();
                for item in items {
                    if !item.disabled.unwrap_or(false) && !item.key.is_empty() {
                        if item.r#type.as_deref() == Some("file") {
                            warnings.push(format!(
                                "Request '{req_name}': multipart file part '{}' references local file '{}'; file path noted.",
                                item.key,
                                item.src.as_deref().unwrap_or("<empty>")
                            ));
                            parts.push(format!("{}=<file:{}>", item.key, item.src.as_deref().unwrap_or_default()));
                        } else {
                            parts.push(format!("{}: {}", item.key, item.value.as_deref().unwrap_or_default()));
                        }
                    }
                }
                Some(parts.join("\n"))
            } else {
                None
            }
        }
        "file" => {
            warnings.push(format!("Request '{req_name}': binary file body mode imported as placeholder."));
            Some(String::new())
        }
        other => {
            warnings.push(format!("Request '{req_name}': body mode '{other}' imported as raw string."));
            body.raw.clone()
        }
    }
}

fn parse_postman_scripts(events: Option<&[PostmanEvent]>) -> (Option<String>, Option<String>) {
    let mut pre_script = None;
    let mut post_script = None;

    if let Some(events) = events {
        for ev in events {
            if let Some(script) = &ev.script {
                if let Some(exec) = &script.exec {
                    let content = exec.to_string_content();
                    if !content.trim().is_empty() {
                        if ev.listen == "prerequest" {
                            pre_script = Some(content);
                        } else if ev.listen == "test" {
                            post_script = Some(content);
                        }
                    }
                }
            }
        }
    }

    (pre_script, post_script)
}

pub fn import_environment(
    conn: &Connection,
    json_str: &str,
    target_project_id: &str,
) -> Result<EnvironmentImportReport, AppError> {
    let env: PostmanEnvironment = serde_json::from_str(json_str)
        .map_err(|err| AppError::Validation(format!("Invalid Postman Environment JSON: {err}")))?;

    // Verify project exists
    let _ = project_store::get_project(conn, target_project_id)?;

    let name = if env.name.trim().is_empty() {
        "Imported Environment".to_string()
    } else {
        env.name.trim().to_string()
    };

    let created_env = environment_store::create_environment(
        conn,
        NewEnvironmentInput {
            project_id: target_project_id.to_string(),
            name,
        },
    )?;
    let mut variables_count = 0;
    let warnings = Vec::new();

    for item in &env.values {
        if !item.key.trim().is_empty() {
            let is_secret = item.r#type.as_deref() == Some("secret");
            if variable_store::create_variable(
                conn,
                NewVariableInput {
                    scope: VariableScope::Environment,
                    project_id: None,
                    environment_id: Some(created_env.id.clone()),
                    request_id: None,
                    key: item.key.trim().to_string(),
                    value: item.value.clone(),
                    enabled: item.enabled,
                    is_secret,
                    is_local: false,
                    description: None,
                },
            ).is_ok() {
                variables_count += 1;
            }
        }
    }

    Ok(EnvironmentImportReport {
        environment_id: created_env.id,
        environment_name: created_env.name,
        variables_count,
        warnings,
    })
}
