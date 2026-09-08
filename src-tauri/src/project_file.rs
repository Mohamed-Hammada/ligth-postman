use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
    workspace_id: &str,
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
        let existing_projects = project_store::list_projects(&tx, workspace_id)?;
        let mut name = base_name.clone();
        let mut suffix = 1;
        while existing_projects.iter().any(|p| p.name == name) {
            name = format!("{base_name} ({suffix})");
            suffix += 1;
        }
        project_store::create_project(&tx, NewProjectInput { name, workspace_id: workspace_id.to_string() })?
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
                // Folders aren't part of the light-postman.json schema yet — imported requests
                // land at the project root, same as any other request-store caller that hasn't
                // been taught about folders.
                folder_id: None,
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

// ---------------------------------------------------------------------------
// Workspace-level sync: one shared repo covering every project in a workspace
// ---------------------------------------------------------------------------

pub const WORKSPACE_PROJECTS_SUBDIR: &str = "projects";

/// Filesystem-safe folder name for one project inside a workspace repo: the project's name,
/// sanitized, with a short id suffix appended only on collision (two projects sharing a name) —
/// keeps the common case human-readable in diffs/PRs while staying unique.
fn project_folder_slug(name: &str, id: &str, used: &mut HashSet<String>) -> String {
    let mut slug: String = name
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if slug.is_empty() {
        slug = "project".to_string();
    }
    if used.contains(&slug) {
        let short_id: String = id.chars().take(8).collect();
        slug = format!("{slug}-{short_id}");
    }
    used.insert(slug.clone());
    slug
}

/// Writes every project in the workspace into its own `projects/<slug>/light-postman.json`,
/// under one shared repo directory — the workspace-level counterpart to `export_project_to_json`
/// (used per-project). Returns the file paths written, for the caller to `git add`.
pub fn export_workspace_to_repo(
    conn: &Connection,
    workspace_id: &str,
    dir: &Path,
    include_secrets: bool,
) -> Result<Vec<PathBuf>, AppError> {
    let projects = project_store::list_projects(conn, workspace_id)?;
    let projects_dir = dir.join(WORKSPACE_PROJECTS_SUBDIR);
    std::fs::create_dir_all(&projects_dir)
        .map_err(|e| AppError::Storage(format!("Failed to create projects folder: {e}")))?;

    let mut used = HashSet::new();
    let mut written = Vec::new();
    for p in projects {
        let slug = project_folder_slug(&p.name, &p.id, &mut used);
        let proj_dir = projects_dir.join(&slug);
        std::fs::create_dir_all(&proj_dir)
            .map_err(|e| AppError::Storage(format!("Failed to create project folder: {e}")))?;
        let json = export_project_to_json(conn, &p.id, include_secrets)?;
        let file_path = proj_dir.join(PROJECT_FILE_DEFAULT_NAME);
        std::fs::write(&file_path, json)
            .map_err(|e| AppError::Storage(format!("Failed to write project file: {e}")))?;
        written.push(file_path);
    }
    Ok(written)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceImportReport {
    pub updated_projects: Vec<Project>,
    pub created_projects: Vec<Project>,
    pub warnings: Vec<String>,
}

/// Reads every `projects/*/light-postman.json` under the repo. A file whose embedded `id`
/// matches a project already in this workspace updates it in place (pulling on the machine that
/// made the export); anything else creates a new project, same as importing a single
/// light-postman.json with no target (pulling onto a fresh machine/workspace) — so this never
/// silently overwrites an unrelated project that just happens to share a name. One bad/unreadable
/// file is recorded as a warning and skipped, not a fatal error for the whole sync.
pub fn import_workspace_from_repo(
    conn: &mut Connection,
    workspace_id: &str,
    dir: &Path,
) -> Result<WorkspaceImportReport, AppError> {
    let projects_dir = dir.join(WORKSPACE_PROJECTS_SUBDIR);
    let mut report = WorkspaceImportReport {
        updated_projects: vec![],
        created_projects: vec![],
        warnings: vec![],
    };
    if !projects_dir.is_dir() {
        return Ok(report);
    }

    let existing_ids: HashSet<String> = project_store::list_projects(conn, workspace_id)?
        .into_iter()
        .map(|p| p.id)
        .collect();

    let mut entries: Vec<_> = std::fs::read_dir(&projects_dir)
        .map_err(|e| AppError::Storage(format!("Failed to read projects folder: {e}")))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let file_path = path.join(PROJECT_FILE_DEFAULT_NAME);
        if !file_path.is_file() {
            continue;
        }
        let json_content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                report.warnings.push(format!("Could not read {}: {e}", file_path.display()));
                continue;
            }
        };
        let parsed: ProjectFile = match serde_json::from_str(&json_content) {
            Ok(p) => p,
            Err(e) => {
                report
                    .warnings
                    .push(format!("Skipped {}: invalid JSON ({e})", file_path.display()));
                continue;
            }
        };
        let target_id = parsed.id.as_deref().filter(|id| existing_ids.contains(*id));
        match import_project_from_json(conn, &json_content, target_id, workspace_id) {
            Ok(project) => {
                if target_id.is_some() {
                    report.updated_projects.push(project);
                } else {
                    report.created_projects.push(project);
                }
            }
            Err(e) => report
                .warnings
                .push(format!("Failed to import {}: {e}", file_path.display())),
        }
    }

    Ok(report)
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
                workspace_id: "default".into(),
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
                folder_id: None,
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
        let imported = import_project_from_json(&mut conn, &exported_full, None, "default").unwrap();
        assert_ne!(imported.id, project.id);
        assert!(imported.name.starts_with("Test Sync Project"));

        let imported_requests = request_store::list_requests(&conn, &imported.id).unwrap();
        assert_eq!(imported_requests.len(), 1);
        assert_eq!(imported_requests[0].name, "Get Users");

        let imported_envs = environment_store::list_environments(&conn, &imported.id).unwrap();
        assert_eq!(imported_envs.len(), 1);
        assert_eq!(imported_envs[0].name, "Staging");
    }

    fn make_project(conn: &Connection, workspace_id: &str, name: &str) -> Project {
        project_store::create_project(
            conn,
            NewProjectInput { name: name.into(), workspace_id: workspace_id.into() },
        )
        .unwrap()
    }

    #[test]
    fn export_workspace_writes_one_file_per_project_and_import_creates_them_on_a_fresh_workspace() {
        let conn = db::open_in_memory().unwrap();
        let ws = crate::store::workspace_store::create_workspace(
            &conn,
            crate::models::NewWorkspaceInput { name: "Team A".into() },
        )
        .unwrap();
        make_project(&conn, &ws.id, "Payments API");
        make_project(&conn, &ws.id, "Auth Service");

        let dir = std::env::temp_dir().join(format!("lp-ws-export-{}", uuid::Uuid::new_v4()));
        let written = export_workspace_to_repo(&conn, &ws.id, &dir, true).unwrap();
        assert_eq!(written.len(), 2);
        assert!(dir.join(WORKSPACE_PROJECTS_SUBDIR).join("Payments_API").join(PROJECT_FILE_DEFAULT_NAME).exists());
        assert!(dir.join(WORKSPACE_PROJECTS_SUBDIR).join("Auth_Service").join(PROJECT_FILE_DEFAULT_NAME).exists());

        // Fresh workspace (simulating a different machine/db) — importing creates both projects.
        let mut conn2 = db::open_in_memory().unwrap();
        let ws2 = crate::store::workspace_store::create_workspace(
            &conn2,
            crate::models::NewWorkspaceInput { name: "Team A copy".into() },
        )
        .unwrap();
        let report = import_workspace_from_repo(&mut conn2, &ws2.id, &dir).unwrap();
        assert_eq!(report.created_projects.len(), 2);
        assert!(report.updated_projects.is_empty());
        assert!(report.warnings.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reimporting_into_the_same_workspace_updates_in_place_instead_of_duplicating() {
        let mut conn = db::open_in_memory().unwrap();
        let ws = crate::store::workspace_store::create_workspace(
            &conn,
            crate::models::NewWorkspaceInput { name: "Team A".into() },
        )
        .unwrap();
        make_project(&conn, &ws.id, "Payments API");

        let dir = std::env::temp_dir().join(format!("lp-ws-reimport-{}", uuid::Uuid::new_v4()));
        export_workspace_to_repo(&conn, &ws.id, &dir, true).unwrap();

        // Re-importing the same export back into the SAME workspace must update the existing
        // project (matched by the id embedded in light-postman.json), not create a duplicate.
        let report = import_workspace_from_repo(&mut conn, &ws.id, &dir).unwrap();
        assert_eq!(report.updated_projects.len(), 1);
        assert!(report.created_projects.is_empty());

        let projects = project_store::list_projects(&conn, &ws.id).unwrap();
        assert_eq!(projects.len(), 1, "must still be exactly one project, not a duplicate");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_projects_with_the_same_name_get_distinct_slugs() {
        let conn = db::open_in_memory().unwrap();
        let ws = crate::store::workspace_store::create_workspace(
            &conn,
            crate::models::NewWorkspaceInput { name: "Team A".into() },
        )
        .unwrap();
        make_project(&conn, &ws.id, "API");
        make_project(&conn, &ws.id, "API");

        let dir = std::env::temp_dir().join(format!("lp-ws-slug-{}", uuid::Uuid::new_v4()));
        let written = export_workspace_to_repo(&conn, &ws.id, &dir, true).unwrap();
        assert_eq!(written.len(), 2);
        assert_ne!(written[0], written[1]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
