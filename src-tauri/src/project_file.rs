use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{
    Auth, HeaderEntry, NewEnvironmentInput, NewProjectInput, NewRequestInput, NewVariableInput,
    Project, QueryParam, RequestSettings, VariableScope,
};
use crate::store::{environment_store, project_store, request_store, variable_store};

pub const PROJECT_FILE_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_FILE_DEFAULT_NAME: &str = "light-postman.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<ProjectFileVariable>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<ProjectFileEnvironment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requests: Vec<ProjectFileRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileEnvironment {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<ProjectFileVariable>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileVariable {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<HeaderEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_params: Vec<QueryParam>,
    #[serde(default)]
    pub auth: Auth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_request_script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_request_script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<RequestSettings>,
}

fn default_true() -> bool {
    true
}

/// Serializes the given project into the canonical version-controlled JSON format (LP-0701).
///
/// Local-only variables (`is_local == true`) are strictly excluded to avoid leaking device data.
/// If `include_secrets` is false, secret variable values are replaced with masked placeholders.
pub fn export_project_to_json(
    conn: &Connection,
    project_id: &str,
    include_secrets: bool,
) -> Result<String, AppError> {
    let project = project_store::get_project(conn, project_id)?;

    // Global variables (excluding local-only)
    let global_vars = variable_store::list_variables_for_scope(
        conn,
        VariableScope::Global,
        project_id,
    )?;
    let mut file_vars: Vec<ProjectFileVariable> = global_vars
        .into_iter()
        .filter(|v| !v.is_local)
        .map(|v| ProjectFileVariable {
            key: v.key,
            value: if v.is_secret && !include_secrets {
                "{{$secret}}".to_string()
            } else {
                v.value
            },
            enabled: v.enabled,
            is_secret: v.is_secret,
            description: v.description,
        })
        .collect();
    file_vars.sort_by(|a, b| a.key.cmp(&b.key));

    // Environments & their non-local variables
    let envs = environment_store::list_environments(conn, project_id)?;
    let mut file_envs: Vec<ProjectFileEnvironment> = Vec::new();
    for env in envs {
        let env_vars = variable_store::list_variables_for_scope(
            conn,
            VariableScope::Environment,
            &env.id,
        )?;
        let mut vars: Vec<ProjectFileVariable> = env_vars
            .into_iter()
            .filter(|v| !v.is_local)
            .map(|v| ProjectFileVariable {
                key: v.key,
                value: if v.is_secret && !include_secrets {
                    "{{$secret}}".to_string()
                } else {
                    v.value
                },
                enabled: v.enabled,
                is_secret: v.is_secret,
                description: v.description,
            })
            .collect();
        vars.sort_by(|a, b| a.key.cmp(&b.key));
        file_envs.push(ProjectFileEnvironment {
            name: env.name,
            variables: vars,
        });
    }
    file_envs.sort_by(|a, b| a.name.cmp(&b.name));

    // Requests
    let request_summaries = request_store::list_requests(conn, project_id)?;
    let mut file_requests: Vec<ProjectFileRequest> = Vec::new();
    for sum in request_summaries {
        let full = request_store::get_request(conn, &sum.id)?;
        file_requests.push(ProjectFileRequest {
            name: full.name,
            method: full.method,
            url: full.url,
            headers: full.headers,
            query_params: full.query_params,
            auth: full.auth,
            body: full.body,
            description: full.description,
            pre_request_script: full.pre_request_script,
            post_request_script: full.post_request_script,
            settings: full.settings,
        });
    }
    file_requests.sort_by(|a, b| a.name.cmp(&b.name));

    let file = ProjectFile {
        schema_version: PROJECT_FILE_SCHEMA_VERSION,
        id: Some(project.id),
        name: project.name,
        description: None,
        variables: file_vars,
        environments: file_envs,
        requests: file_requests,
    };

    serde_json::to_string_pretty(&file).map_err(|e| AppError::Storage(e.to_string()))
}

/// Imports a project from the canonical project file JSON format.
///
/// If `target_project_id` is provided, merges/updates into the existing project.
/// Otherwise, creates a new project.
pub fn import_project_from_json(
    conn: &mut Connection,
    json_content: &str,
    target_project_id: Option<&str>,
) -> Result<Project, AppError> {
    let file: ProjectFile = serde_json::from_str(json_content)
        .map_err(|e| AppError::Validation(format!("Invalid light-postman.json: {e}")))?;

    let tx = conn.transaction()?;

    let project = if let Some(existing_id) = target_project_id {
        project_store::get_project(&tx, existing_id)?
    } else {
        let base_name = if file.name.trim().is_empty() {
            "Imported Project".to_string()
        } else {
            file.name.trim().to_string()
        };
        let existing_projects = project_store::list_projects(&tx)?;
        let mut name = base_name.clone();
        let mut suffix = 1;
        while existing_projects.iter().any(|p| p.name == name) {
            name = format!("{base_name} ({suffix})");
            suffix += 1;
        }
        project_store::create_project(&tx, NewProjectInput { name })?
    };

    // Import Global Variables
    for v in file.variables {
        let _ = variable_store::create_variable(
            &tx,
            NewVariableInput {
                scope: VariableScope::Global,
                project_id: Some(project.id.clone()),
                environment_id: None,
                request_id: None,
                key: v.key,
                value: v.value,
                enabled: v.enabled,
                is_secret: v.is_secret,
                is_local: false,
                description: v.description,
            },
        );
    }

    // Import Environments
    for env in file.environments {
        let created_env = match environment_store::create_environment(
            &tx,
            NewEnvironmentInput {
                project_id: project.id.clone(),
                name: env.name.clone(),
            },
        ) {
            Ok(e) => e,
            Err(_) => {
                let existing = environment_store::list_environments(&tx, &project.id)?;
                if let Some(found) = existing.into_iter().find(|e| e.name == env.name) {
                    found
                } else {
                    continue;
                }
            }
        };

        for v in env.variables {
            let _ = variable_store::create_variable(
                &tx,
                NewVariableInput {
                    scope: VariableScope::Environment,
                    project_id: None,
                    environment_id: Some(created_env.id.clone()),
                    request_id: None,
                    key: v.key,
                    value: v.value,
                    enabled: v.enabled,
                    is_secret: v.is_secret,
                    is_local: false,
                    description: v.description,
                },
            );
        }
    }

    // Import Requests
    for req in file.requests {
        let _ = request_store::create_request(
            &tx,
            NewRequestInput {
                project_id: project.id.clone(),
                name: req.name,
                method: req.method,
                url: req.url,
                headers: req.headers,
                query_params: req.query_params,
                auth: req.auth,
                body: req.body,
                description: req.description,
                settings: req.settings,
                pre_request_script: req.pre_request_script,
                post_request_script: req.post_request_script,
            },
        )?;
    }

    tx.commit()?;
    Ok(project)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn project_file_roundtrip_with_environments_and_requests() {
        let mut conn = db::open_in_memory().unwrap();
        let project = project_store::create_project(
            &conn,
            NewProjectInput {
                name: "Test Sync Project".into(),
            },
        )
        .unwrap();

        // Add a global variable
        variable_store::create_variable(
            &conn,
            NewVariableInput {
                scope: VariableScope::Global,
                project_id: Some(project.id.clone()),
                environment_id: None,
                request_id: None,
                key: "baseUrl".into(),
                value: "https://api.example.com".into(),
                enabled: true,
                is_secret: false,
                is_local: false,
                description: None,
            },
        )
        .unwrap();

        // Add a local variable (should NOT be exported)
        variable_store::create_variable(
            &conn,
            NewVariableInput {
                scope: VariableScope::Global,
                project_id: Some(project.id.clone()),
                environment_id: None,
                request_id: None,
                key: "myLocalToken".into(),
                value: "super_secret_local".into(),
                enabled: true,
                is_secret: true,
                is_local: true,
                description: None,
            },
        )
        .unwrap();

        // Add an environment
        let env = environment_store::create_environment(
            &conn,
            NewEnvironmentInput {
                project_id: project.id.clone(),
                name: "Staging".into(),
            },
        )
        .unwrap();
        variable_store::create_variable(
            &conn,
            NewVariableInput {
                scope: VariableScope::Environment,
                project_id: None,
                environment_id: Some(env.id.clone()),
                request_id: None,
                key: "apiKey".into(),
                value: "stage-key-123".into(),
                enabled: true,
                is_secret: true,
                is_local: false,
                description: None,
            },
        )
        .unwrap();

        // Add a request
        request_store::create_request(
            &conn,
            NewRequestInput {
                project_id: project.id.clone(),
                name: "Get Users".into(),
                method: "GET".into(),
                url: "{{baseUrl}}/users".into(),
                headers: vec![HeaderEntry {
                    key: "Accept".into(),
                    value: "application/json".into(),
                    enabled: true,
                    description: None,
                }],
                query_params: vec![QueryParam {
                    key: "limit".into(),
                    value: "10".into(),
                    enabled: true,
                    description: None,
                }],
                auth: Auth::Bearer {
                    token: "{{apiKey}}".into(),
                },
                body: None,
                description: None,
                settings: None,
                pre_request_script: None,
                post_request_script: None,
            },
        )
        .unwrap();

        // Export without secrets
        let exported_masked = export_project_to_json(&conn, &project.id, false).unwrap();
        assert!(exported_masked.contains("\"baseUrl\""));
        assert!(exported_masked.contains("\"Get Users\""));
        assert!(exported_masked.contains("\"{{$secret}}\""));
        assert!(!exported_masked.contains("super_secret_local")); // local var excluded!

        // Export with secrets
        let exported_full = export_project_to_json(&conn, &project.id, true).unwrap();
        assert!(exported_full.contains("stage-key-123"));
        assert!(!exported_full.contains("super_secret_local")); // local var still excluded!

        // Import into new project
        let imported = import_project_from_json(&mut conn, &exported_full, None).unwrap();
        assert_ne!(imported.id, project.id);
        assert!(imported.name.starts_with("Test Sync Project"));

        let imported_requests = request_store::list_requests(&conn, &imported.id).unwrap();
        assert_eq!(imported_requests.len(), 1);
        assert_eq!(imported_requests[0].name, "Get Users");

        let imported_envs = environment_store::list_environments(&conn, &imported.id).unwrap();
        assert_eq!(imported_envs.len(), 1);
        assert_eq!(imported_envs[0].name, "Staging");
    }
}
