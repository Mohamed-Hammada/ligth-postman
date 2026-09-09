use rusqlite::Connection;
use serde_json::json;

use super::schema::*;
use crate::error::AppError;
use crate::models::{ApiKeyLocation, Auth, VariableScope};
use crate::store::{environment_store, project_store, request_store, variable_store};

pub fn export_project_collection(
    conn: &Connection,
    project_id: &str,
) -> Result<String, AppError> {
    let project = project_store::get_project(conn, project_id)?;
    let request_summaries = request_store::list_requests(conn, project_id)?;
    let variables = variable_store::list_variables_for_scope(conn, VariableScope::Global, project_id)?;

    let postman_variables = variables
        .into_iter()
        .filter(|v| !v.is_local)
        .map(|v| PostmanVariable {
            key: Some(v.key),
            value: Some(v.value),
            r#type: if v.is_secret {
                Some("secret".to_string())
            } else {
                Some("string".to_string())
            },
        })
        .collect();

    let mut items = Vec::new();

    for summary in request_summaries {
        let req = request_store::get_request(conn, &summary.id)?;
        let sample_responses = request_store::list_sample_responses(conn, &req.id)?;

        let mut postman_queries = Vec::new();
        let mut full_url = req.url.clone();
        if !req.query_params.is_empty() {
            let mut query_strings = Vec::new();
            for p in &req.query_params {
                postman_queries.push(PostmanQueryParam {
                    key: Some(p.key.clone()),
                    value: Some(p.value.clone()),
                    disabled: Some(!p.enabled),
                    description: p.description.clone(),
                });
                if p.enabled {
                    query_strings.push(format!("{}={}", p.key, p.value));
                }
            }
            if !query_strings.is_empty() {
                let separator = if full_url.contains('?') { "&" } else { "?" };
                full_url = format!("{full_url}{separator}{}", query_strings.join("&"));
            }
        }

        let postman_headers = req
            .headers
            .into_iter()
            .map(|h| PostmanHeader {
                key: Some(h.key),
                value: Some(h.value),
                disabled: Some(!h.enabled),
                description: h.description,
            })
            .collect();

        let postman_auth = match req.auth {
            Auth::None => None,
            Auth::Bearer { token } => Some(PostmanAuth {
                r#type: "bearer".to_string(),
                bearer: Some(vec![PostmanAuthParam {
                    key: "token".to_string(),
                    value: json!(token),
                }]),
                ..Default::default()
            }),
            Auth::Basic { username, password } => Some(PostmanAuth {
                r#type: "basic".to_string(),
                basic: Some(vec![
                    PostmanAuthParam {
                        key: "username".to_string(),
                        value: json!(username),
                    },
                    PostmanAuthParam {
                        key: "password".to_string(),
                        value: json!(password),
                    },
                ]),
                ..Default::default()
            }),
            Auth::ApiKey { key, value, location } => Some(PostmanAuth {
                r#type: "apikey".to_string(),
                apikey: Some(vec![
                    PostmanAuthParam {
                        key: "key".to_string(),
                        value: json!(key),
                    },
                    PostmanAuthParam {
                        key: "value".to_string(),
                        value: json!(value),
                    },
                    PostmanAuthParam {
                        key: "in".to_string(),
                        value: json!(match location {
                            ApiKeyLocation::Header => "header",
                            ApiKeyLocation::Query => "query",
                        }),
                    },
                ]),
                ..Default::default()
            }),
        };

        let postman_body = req.body.map(|b| PostmanBody {
            mode: Some("raw".to_string()),
            raw: Some(b),
            ..Default::default()
        });

        let mut events = Vec::new();
        if let Some(pre) = req.pre_request_script {
            if !pre.trim().is_empty() {
                events.push(PostmanEvent {
                    listen: "prerequest".to_string(),
                    script: Some(PostmanScript {
                        r#type: Some("text/javascript".to_string()),
                        exec: Some(PostmanScriptExec::Lines(
                            pre.lines().map(String::from).collect(),
                        )),
                    }),
                });
            }
        }
        if let Some(post) = req.post_request_script {
            if !post.trim().is_empty() {
                events.push(PostmanEvent {
                    listen: "test".to_string(),
                    script: Some(PostmanScript {
                        r#type: Some("text/javascript".to_string()),
                        exec: Some(PostmanScriptExec::Lines(
                            post.lines().map(String::from).collect(),
                        )),
                    }),
                });
            }
        }

        let postman_responses = sample_responses
            .into_iter()
            .map(|s| PostmanResponse {
                name: Some(s.name),
                code: Some(s.status),
                header: Some(
                    s.headers
                        .into_iter()
                        .map(|h| PostmanHeader {
                            key: Some(h.key),
                            value: Some(h.value),
                            disabled: Some(!h.enabled),
                            description: h.description,
                        })
                        .collect(),
                ),
                body: s.body,
                ..Default::default()
            })
            .collect();

        items.push(PostmanItem {
            name: Some(req.name),
            description: req.description,
            item: None,
            request: Some(PostmanRequest {
                url: PostmanUrlOrString::Structured(PostmanUrl {
                    raw: Some(full_url),
                    query: if postman_queries.is_empty() {
                        None
                    } else {
                        Some(postman_queries)
                    },
                    ..Default::default()
                }),
                method: Some(req.method),
                header: Some(postman_headers),
                body: postman_body,
                auth: postman_auth,
                description: None,
            }),
            response: Some(postman_responses),
            event: if events.is_empty() { None } else { Some(events) },
        });
    }

    let collection = PostmanCollection {
        info: PostmanInfo {
            name: project.name,
            description: None,
            schema: Some("https://schema.getpostman.com/json/collection/v2.1.0/collection.json".to_string()),
        },
        item: items,
        variable: postman_variables,
        auth: None,
        event: Vec::new(),
    };

    serde_json::to_string_pretty(&collection)
        .map_err(|err| AppError::Storage(format!("Failed to serialize Postman Collection: {err}")))
}

pub fn export_environment(
    conn: &Connection,
    environment_id: &str,
) -> Result<String, AppError> {
    let env = environment_store::get_environment(conn, environment_id)?;
    let variables = variable_store::list_variables_for_scope(conn, VariableScope::Environment, environment_id)?;

    let values = variables
        .into_iter()
        .filter(|v| !v.is_local)
        .map(|v| PostmanEnvValue {
            key: v.key,
            value: v.value,
            enabled: v.enabled,
            r#type: if v.is_secret {
                Some("secret".to_string())
            } else {
                Some("default".to_string())
            },
        })
        .collect();

    let postman_env = PostmanEnvironment {
        id: Some(env.id),
        name: env.name,
        values,
    };

    serde_json::to_string_pretty(&postman_env)
        .map_err(|err| AppError::Storage(format!("Failed to serialize Postman Environment: {err}")))
}
