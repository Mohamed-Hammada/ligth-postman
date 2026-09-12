// Import for Postman's newer "local files" workspace format — an on-disk directory tree
// (collections/<name>/**/*.request.yaml, environments/*.environment.yaml), NOT the single
// Postman Collection v2.1 JSON blob that `importer.rs` handles. Every file is parsed
// defensively via a generic `serde_yaml::Value` walk (never a strict typed deserialize) so one
// malformed/unusual file among thousands degrades to a warning instead of aborting the run.
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_yaml::Value as Yaml;

use crate::error::AppError;
use crate::models::{
    ApiKeyLocation, Auth, FormDataPart, HeaderEntry, NewEnvironmentInput, NewFolderInput,
    NewProjectInput, NewRequestInput, NewSampleResponseInput, NewVariableInput, QueryParam,
    RequestBody, UrlEncodedItem, VariableScope, VALID_METHODS,
};
use crate::store::{environment_store, folder_store, project_store, request_store, variable_store};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWorkspaceImportReport {
    pub projects_created: usize,
    pub folders_created: usize,
    pub requests_imported: usize,
    pub samples_imported: usize,
    pub environments_imported: usize,
    pub variables_imported: usize,
    pub warnings: Vec<String>,
}

// Heuristic only — the source format doesn't mark variables as secret, but real workspaces
// routinely carry live tokens/passwords in plain values. Flagging obviously-named ones as
// secret means they render masked by default instead of in plaintext.
const SECRET_KEY_HINTS: [&str; 8] = [
    "secret", "password", "passwd", "token", "apikey", "api_key", "private", "credential",
];

fn looks_like_secret(key: &str) -> bool {
    let lower = key.to_lowercase();
    SECRET_KEY_HINTS.iter().any(|hint| lower.contains(hint))
}

fn yaml_scalar_to_string(value: Option<&Yaml>) -> String {
    match value {
        Some(Yaml::String(s)) => s.clone(),
        Some(Yaml::Number(n)) => n.to_string(),
        Some(Yaml::Bool(b)) => b.to_string(),
        Some(Yaml::Null) | None => String::new(),
        Some(other) => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

fn yaml_get<'a>(doc: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    doc.as_mapping()?.get(Yaml::String(key.to_string()))
}

pub fn import_local_workspace(
    conn: &Connection,
    root_path: &str,
    workspace_id: &str,
) -> Result<LocalWorkspaceImportReport, AppError> {
    let root = Path::new(root_path);
    if !root.exists() || !root.is_dir() {
        return Err(AppError::Validation(format!(
            "Directory does not exist: {root_path}"
        )));
    }
    let collections_dir = root.join("collections");
    let environments_dir = root.join("environments");
    if !collections_dir.is_dir() && !environments_dir.is_dir() {
        return Err(AppError::Validation(
            "Not a recognized local Postman workspace — expected a 'collections' and/or 'environments' subfolder here.".into(),
        ));
    }

    let mut warnings = Vec::new();
    let mut projects_created = 0usize;
    let mut folders_created = 0usize;
    let mut requests_imported = 0usize;
    let mut samples_imported = 0usize;
    let mut environments_imported = 0usize;
    let mut variables_imported = 0usize;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|err| AppError::Storage(err.to_string()))?;

    let result = (|| -> Result<(), AppError> {
        if collections_dir.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(&collections_dir)
                .map_err(|err| AppError::Storage(err.to_string()))?
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let collection_name = entry.file_name().to_string_lossy().to_string();
                if collection_name.starts_with('.') {
                    continue;
                }

                let project = project_store::create_project(
                    conn,
                    NewProjectInput { name: collection_name.clone(), workspace_id: workspace_id.to_string() },
                )?;
                projects_created += 1;

                let mut folder_cache: HashMap<String, String> = HashMap::new();
                import_collection_dir(
                    conn,
                    &path,
                    &project.id,
                    "",
                    &mut folder_cache,
                    &mut folders_created,
                    &mut requests_imported,
                    &mut samples_imported,
                    &mut warnings,
                );
            }
        }

        if environments_dir.is_dir() {
            let mut env_files: Vec<_> = fs::read_dir(&environments_dir)
                .map_err(|err| AppError::Storage(err.to_string()))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    name.ends_with(".yaml") || name.ends_with(".yml")
                })
                .collect();
            env_files.sort_by_key(|e| e.file_name());

            if !env_files.is_empty() {
                let holder = project_store::create_project(
                    conn,
                    NewProjectInput { name: "Imported Environments".to_string(), workspace_id: workspace_id.to_string() },
                )?;
                projects_created += 1;

                for entry in env_files {
                    let path = entry.path();
                    let content = match fs::read_to_string(&path) {
                        Ok(c) => c,
                        Err(err) => {
                            warnings.push(format!("Could not read {}: {err}", path.display()));
                            continue;
                        }
                    };
                    let doc: Yaml = match serde_yaml::from_str(&content) {
                        Ok(d) => d,
                        Err(err) => {
                            warnings.push(format!(
                                "Skipped environment file '{}': invalid YAML ({err})",
                                path.display()
                            ));
                            continue;
                        }
                    };

                    let fallback_name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Imported Environment".to_string())
                        .trim_end_matches(".environment")
                        .to_string();
                    let name = yaml_get(&doc, "name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or(fallback_name);

                    let env = match environment_store::create_environment(
                        conn,
                        NewEnvironmentInput { project_id: holder.id.clone(), name: name.clone() },
                    ) {
                        Ok(e) => e,
                        Err(err) => {
                            warnings.push(format!("Could not create environment '{name}': {err}"));
                            continue;
                        }
                    };
                    environments_imported += 1;

                    if let Some(values) = yaml_get(&doc, "values").and_then(|v| v.as_sequence()) {
                        for item in values {
                            let key = yaml_get(item, "key")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if key.is_empty() {
                                continue;
                            }
                            let value = yaml_scalar_to_string(yaml_get(item, "value"));
                            let enabled = yaml_get(item, "enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                            let is_secret = looks_like_secret(&key);

                            let outcome = variable_store::create_variable(
                                conn,
                                NewVariableInput {
                                    scope: VariableScope::Environment,
                                    project_id: None,
                                    environment_id: Some(env.id.clone()),
                                    request_id: None,
                                    key: key.clone(),
                                    value,
                                    enabled,
                                    is_secret,
                                    is_local: false,
                                    description: None,
                                },
                            );
                            match outcome {
                                Ok(_) => variables_imported += 1,
                                Err(err) => warnings.push(format!(
                                    "Environment '{name}': could not import variable '{key}': {err}"
                                )),
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .map_err(|err| AppError::Storage(err.to_string()))?;
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK;");
            return Err(err);
        }
    }

    warnings.push(
        "Nested Postman folders were flattened into single-level folders named e.g. 'Parent / Child'."
            .to_string(),
    );

    Ok(LocalWorkspaceImportReport {
        projects_created,
        folders_created,
        requests_imported,
        samples_imported,
        environments_imported,
        variables_imported,
        warnings,
    })
}

#[allow(clippy::too_many_arguments)]
fn import_collection_dir(
    conn: &Connection,
    dir: &Path,
    project_id: &str,
    folder_path: &str,
    folder_cache: &mut HashMap<String, String>,
    folders_created: &mut usize,
    requests_imported: &mut usize,
    samples_imported: &mut usize,
    warnings: &mut Vec<String>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            warnings.push(format!("Could not read directory {}: {err}", dir.display()));
            return;
        }
    };
    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    // Pass 1: create every request directly in this directory first, keyed by its filename
    // stem — `.resources/<stem>.resources/examples/*.example.yaml` (pass 2 below) references
    // requests by that same stem, not by whatever the request's own internal `name:` says.
    let mut request_ids_by_stem: HashMap<String, String> = HashMap::new();
    let mut resources_dir: Option<std::path::PathBuf> = None;
    let mut subdirs: Vec<(std::path::PathBuf, String)> = Vec::new();

    for entry in &sorted {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if file_name == ".resources" {
                resources_dir = Some(path);
            } else {
                subdirs.push((path, file_name));
            }
        } else if file_name.ends_with(".request.yaml") || file_name.ends_with(".request.yml") {
            let stem = file_name
                .strip_suffix(".request.yaml")
                .or_else(|| file_name.strip_suffix(".request.yml"))
                .unwrap_or(&file_name)
                .to_string();
            let folder_id = if folder_path.is_empty() {
                None
            } else {
                get_or_create_folder(conn, project_id, folder_path, folder_cache, folders_created, warnings)
            };
            if let Some(id) = import_request_file(conn, &path, project_id, folder_id, requests_imported, warnings) {
                request_ids_by_stem.insert(stem, id);
            }
        }
    }

    // Pass 2: saved example responses, one `.resources/<stem>.resources/examples/` dir per request.
    if let Some(resources_dir) = resources_dir {
        import_examples_in_resources_dir(conn, &resources_dir, &request_ids_by_stem, samples_imported, warnings);
    }

    // Pass 3: recurse into real Postman sub-folders.
    for (path, file_name) in subdirs {
        let child_path = if folder_path.is_empty() {
            file_name
        } else {
            format!("{folder_path} / {file_name}")
        };
        import_collection_dir(
            conn,
            &path,
            project_id,
            &child_path,
            folder_cache,
            folders_created,
            requests_imported,
            samples_imported,
            warnings,
        );
    }
}

fn import_examples_in_resources_dir(
    conn: &Connection,
    resources_dir: &Path,
    request_ids_by_stem: &HashMap<String, String>,
    samples_imported: &mut usize,
    warnings: &mut Vec<String>,
) {
    let entries = match fs::read_dir(resources_dir) {
        Ok(e) => e,
        Err(err) => {
            warnings.push(format!("Could not read {}: {err}", resources_dir.display()));
            return;
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let Some(request_stem) = dir_name.strip_suffix(".resources") else { continue };
        let Some(request_id) = request_ids_by_stem.get(request_stem) else { continue };

        let examples_dir = path.join("examples");
        if !examples_dir.is_dir() {
            continue;
        }
        let mut example_files: Vec<_> = match fs::read_dir(&examples_dir) {
            Ok(e) => e.filter_map(|e| e.ok()).collect(),
            Err(err) => {
                warnings.push(format!("Could not read {}: {err}", examples_dir.display()));
                continue;
            }
        };
        example_files.sort_by_key(|e| e.file_name());

        for example_entry in example_files {
            let example_path = example_entry.path();
            let example_name_raw = example_entry.file_name().to_string_lossy().to_string();
            if !example_name_raw.ends_with(".example.yaml") && !example_name_raw.ends_with(".example.yml") {
                continue;
            }
            import_example_file(conn, &example_path, request_id, request_stem, samples_imported, warnings);
        }
    }
}

fn import_example_file(
    conn: &Connection,
    path: &Path,
    request_id: &str,
    request_stem: &str,
    samples_imported: &mut usize,
    warnings: &mut Vec<String>,
) {
    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let example_name = file_name
        .strip_suffix(".example.yaml")
        .or_else(|| file_name.strip_suffix(".example.yml"))
        .unwrap_or(&file_name)
        .to_string();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            warnings.push(format!("Could not read example '{example_name}' for '{request_stem}': {err}"));
            return;
        }
    };
    let doc: Yaml = match serde_yaml::from_str(&content) {
        Ok(d) => d,
        Err(err) => {
            warnings.push(format!("Skipped example '{example_name}' for '{request_stem}': invalid YAML ({err})"));
            return;
        }
    };

    let kind = yaml_get(&doc, "$kind").and_then(|v| v.as_str()).unwrap_or("http-example");
    if kind != "http-example" {
        warnings.push(format!("Skipped example '{example_name}': unsupported kind '{kind}'."));
        return;
    }

    let Some(response) = yaml_get(&doc, "response") else {
        warnings.push(format!("Skipped example '{example_name}' for '{request_stem}': no saved response."));
        return;
    };

    let status = yaml_get(response, "statusCode").and_then(|v| v.as_u64()).unwrap_or(200) as u16;
    let status_text = yaml_get(response, "statusText").and_then(|v| v.as_str()).unwrap_or("OK").to_string();

    let mut headers = Vec::new();
    let mut content_type = None;
    if let Some(seq) = yaml_get(response, "headers").and_then(|v| v.as_sequence()) {
        for item in seq {
            let Some(key) = yaml_get(item, "key").and_then(|v| v.as_str()) else { continue };
            if key.trim().is_empty() {
                continue;
            }
            let value = yaml_scalar_to_string(yaml_get(item, "value"));
            if key.eq_ignore_ascii_case("content-type") {
                content_type = Some(value.clone());
            }
            headers.push(HeaderEntry {
                key: key.to_string(),
                value,
                enabled: true,
                description: None,
            });
        }
    }

    let body = yaml_get(response, "body").and_then(|b| yaml_get(b, "content")).and_then(|v| v.as_str()).map(|s| s.to_string());

    let outcome = request_store::create_sample_response(
        conn,
        NewSampleResponseInput {
            request_id: request_id.to_string(),
            name: example_name.clone(),
            status,
            status_text,
            headers,
            body,
            content_type,
        },
    );
    match outcome {
        Ok(_) => *samples_imported += 1,
        Err(err) => warnings.push(format!("Could not import example '{example_name}' for '{request_stem}': {err}")),
    }
}

fn get_or_create_folder(
    conn: &Connection,
    project_id: &str,
    folder_path: &str,
    folder_cache: &mut HashMap<String, String>,
    folders_created: &mut usize,
    warnings: &mut Vec<String>,
) -> Option<String> {
    if let Some(id) = folder_cache.get(folder_path) {
        return Some(id.clone());
    }
    match folder_store::create_folder(
        conn,
        NewFolderInput {
            project_id: project_id.to_string(),
            name: folder_path.to_string(),
            ..Default::default()
        },
    ) {
        Ok(f) => {
            folder_cache.insert(folder_path.to_string(), f.id.clone());
            *folders_created += 1;
            Some(f.id)
        }
        Err(err) => {
            warnings.push(format!("Could not create folder '{folder_path}': {err}"));
            None
        }
    }
}

fn import_request_file(
    conn: &Connection,
    path: &Path,
    project_id: &str,
    folder_id: Option<String>,
    requests_imported: &mut usize,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let file_stem = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let name_from_file = file_stem
        .strip_suffix(".request.yaml")
        .or_else(|| file_stem.strip_suffix(".request.yml"))
        .unwrap_or(&file_stem)
        .to_string();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            warnings.push(format!("Could not read '{name_from_file}': {err}"));
            return None;
        }
    };
    let doc: Yaml = match serde_yaml::from_str(&content) {
        Ok(d) => d,
        Err(err) => {
            warnings.push(format!("Skipped '{name_from_file}': invalid YAML ({err})"));
            return None;
        }
    };

    let kind = yaml_get(&doc, "$kind").and_then(|v| v.as_str()).unwrap_or("http-request");
    if kind != "http-request" {
        warnings.push(format!("Skipped '{name_from_file}': unsupported kind '{kind}'."));
        return None;
    }

    let name = yaml_get(&doc, "name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(name_from_file.clone());

    let url = yaml_get(&doc, "url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let method_raw = yaml_get(&doc, "method").and_then(|v| v.as_str()).unwrap_or("GET").to_uppercase();
    let method = if VALID_METHODS.contains(&method_raw.as_str()) {
        method_raw
    } else {
        warnings.push(format!(
            "Request '{name}': unsupported method '{method_raw}' — imported as GET."
        ));
        "GET".to_string()
    };

    let mut headers = Vec::new();
    if let Some(map) = yaml_get(&doc, "headers").and_then(|v| v.as_mapping()) {
        for (k, v) in map {
            if let Some(key) = k.as_str() {
                if key.trim().is_empty() {
                    continue;
                }
                headers.push(HeaderEntry {
                    key: key.to_string(),
                    value: yaml_scalar_to_string(Some(v)),
                    enabled: true,
                    description: None,
                });
            }
        }
    }

    let mut query_params = Vec::new();
    if let Some(map) = yaml_get(&doc, "queryParams").and_then(|v| v.as_mapping()) {
        for (k, v) in map {
            if let Some(key) = k.as_str() {
                if key.trim().is_empty() {
                    continue;
                }
                query_params.push(QueryParam {
                    key: key.to_string(),
                    value: yaml_scalar_to_string(Some(v)),
                    enabled: true,
                    description: None,
                });
            }
        }
    }

    let description = yaml_get(&doc, "description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let body = parse_local_body(yaml_get(&doc, "body"), &name, warnings);
    let auth = parse_local_auth(yaml_get(&doc, "auth"), &name, warnings);
    let (pre_request_script, post_request_script) = parse_local_scripts(yaml_get(&doc, "scripts"));

    let outcome = request_store::create_request(
        conn,
        NewRequestInput {
            project_id: project_id.to_string(),
            folder_id,
            name,
            method,
            url,
            headers,
            query_params,
            auth,
            body,
            description,
            settings: None,
            pre_request_script,
            post_request_script,
        },
    );
    match outcome {
        Ok(created) => {
            *requests_imported += 1;
            Some(created.id)
        }
        Err(err) => {
            warnings.push(format!("Could not import request '{name_from_file}': {err}"));
            None
        }
    }
}

fn parse_local_auth(auth: Option<&Yaml>, req_name: &str, warnings: &mut Vec<String>) -> Auth {
    let Some(auth) = auth else { return Auth::None };
    let auth_type = yaml_get(auth, "type").and_then(|v| v.as_str()).unwrap_or("noauth");
    let credentials = yaml_get(auth, "credentials");

    match auth_type {
        "noauth" | "none" => Auth::None,
        "bearer" => {
            let token = credentials
                .and_then(|c| yaml_get(c, "token"))
                .map(|v| yaml_scalar_to_string(Some(v)))
                .unwrap_or_default();
            Auth::Bearer { token }
        }
        "basic" => {
            let username = credentials
                .and_then(|c| yaml_get(c, "username"))
                .map(|v| yaml_scalar_to_string(Some(v)))
                .unwrap_or_default();
            let password = credentials
                .and_then(|c| yaml_get(c, "password"))
                .map(|v| yaml_scalar_to_string(Some(v)))
                .unwrap_or_default();
            Auth::Basic { username, password }
        }
        "apikey" | "api_key" => {
            let key = credentials
                .and_then(|c| yaml_get(c, "key"))
                .map(|v| yaml_scalar_to_string(Some(v)))
                .unwrap_or_default();
            let value = credentials
                .and_then(|c| yaml_get(c, "value"))
                .map(|v| yaml_scalar_to_string(Some(v)))
                .unwrap_or_default();
            let location = credentials
                .and_then(|c| yaml_get(c, "in"))
                .and_then(|v| v.as_str())
                .unwrap_or("header");
            let location = if location.eq_ignore_ascii_case("query") {
                ApiKeyLocation::Query
            } else {
                ApiKeyLocation::Header
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

fn parse_local_body(body: Option<&Yaml>, req_name: &str, warnings: &mut Vec<String>) -> Option<String> {
    let body = body?;
    let body_type = yaml_get(body, "type").and_then(|v| v.as_str()).unwrap_or("none").to_lowercase();
    let content = yaml_get(body, "content");

    match body_type.as_str() {
        "none" | "" => None,
        "urlencoded" => {
            if let Some(map) = content.and_then(|v| v.as_mapping()) {
                let items: Vec<UrlEncodedItem> = map
                    .iter()
                    .filter_map(|(k, v)| {
                        let key = k.as_str()?.to_string();
                        if key.trim().is_empty() {
                            return None;
                        }
                        Some(UrlEncodedItem {
                            key,
                            value: yaml_scalar_to_string(Some(v)),
                            enabled: true,
                            description: None,
                        })
                    })
                    .collect();
                serde_json::to_string(&RequestBody::UrlEncoded { items }).ok()
            } else {
                warnings.push(format!(
                    "Request '{req_name}': urlencoded body had an unexpected shape; imported empty."
                ));
                None
            }
        }
        "formdata" => {
            if let Some(seq) = content.and_then(|v| v.as_sequence()) {
                let items: Vec<FormDataPart> = seq
                    .iter()
                    .filter_map(|item| {
                        let key = yaml_get(item, "key").and_then(|v| v.as_str())?.to_string();
                        if key.trim().is_empty() {
                            return None;
                        }
                        let is_file = yaml_get(item, "type").and_then(|v| v.as_str()) == Some("file");
                        if is_file {
                            warnings.push(format!(
                                "Request '{req_name}': multipart file part '{key}' references a local file — re-select it on this machine."
                            ));
                        }
                        Some(FormDataPart {
                            key,
                            value: yaml_scalar_to_string(yaml_get(item, "value")),
                            enabled: !yaml_get(item, "disabled").and_then(|v| v.as_bool()).unwrap_or(false),
                            description: None,
                            is_file,
                            file_path: yaml_get(item, "src").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        })
                    })
                    .collect();
                serde_json::to_string(&RequestBody::FormData { items }).ok()
            } else {
                warnings.push(format!(
                    "Request '{req_name}': formdata body had an unexpected shape; imported empty."
                ));
                None
            }
        }
        _ => {
            // text, json, html, javascript, xml, graphql, or anything unrecognized — the app's
            // "raw" body editor handles all of these fine as plain text.
            content.and_then(|v| v.as_str()).map(|s| s.to_string())
        }
    }
}

/// Postman's local format lists scripts as `[{type: beforeRequest|afterResponse, code, language}]`
/// (sometimes prefixed `http:`). Multiple scripts of the same phase are concatenated.
fn parse_local_scripts(scripts: Option<&Yaml>) -> (Option<String>, Option<String>) {
    let mut pre_parts = Vec::new();
    let mut post_parts = Vec::new();

    if let Some(seq) = scripts.and_then(|v| v.as_sequence()) {
        for item in seq {
            let script_type = yaml_get(item, "type").and_then(|v| v.as_str()).unwrap_or("");
            let code = yaml_get(item, "code").and_then(|v| v.as_str()).unwrap_or("");
            if code.trim().is_empty() {
                continue;
            }
            if script_type.contains("beforeRequest") {
                pre_parts.push(code.to_string());
            } else if script_type.contains("afterResponse") {
                post_parts.push(code.to_string());
            }
        }
    }

    let pre = if pre_parts.is_empty() { None } else { Some(pre_parts.join("\n\n")) };
    let post = if post_parts.is_empty() { None } else { Some(post_parts.join("\n\n")) };
    (pre, post)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    struct ScratchDir(std::path::PathBuf);
    impl ScratchDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("lp-local-ws-test-{label}-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }
    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn imports_a_flat_collection_with_one_request() {
        let scratch = ScratchDir::new("flat");
        let collections = scratch.0.join("collections").join("MyCollection");
        fs::create_dir_all(&collections).unwrap();
        fs::write(
            collections.join("Ping.request.yaml"),
            "$kind: http-request\nurl: \"{{host}}/ping\"\nmethod: GET\nheaders:\n  Accept: application/json\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.projects_created, 1);
        assert_eq!(report.requests_imported, 1);
        assert_eq!(report.folders_created, 0);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "MyCollection");

        let requests = request_store::list_requests(&conn, &projects[0].id).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].name, "Ping");
        assert_eq!(requests[0].method, "GET");
    }

    #[test]
    fn imports_saved_examples_as_sample_responses_on_their_matching_request() {
        let scratch = ScratchDir::new("examples");
        let collection = scratch.0.join("collections").join("WithExamples");
        fs::create_dir_all(&collection).unwrap();
        fs::write(
            collection.join("Login.request.yaml"),
            "$kind: http-request\nurl: \"{{host}}/login\"\nmethod: POST\n",
        )
        .unwrap();

        let examples_dir = collection.join(".resources").join("Login.resources").join("examples");
        fs::create_dir_all(&examples_dir).unwrap();
        fs::write(
            examples_dir.join("Login Success.example.yaml"),
            "$kind: http-example\n\
             request:\n  url: \"{{host}}/login\"\n  method: POST\n\
             response:\n  statusCode: 200\n  statusText: OK\n  headers:\n    - key: Content-Type\n      value: application/json\n  body:\n    type: json\n    content: '{\"ok\":true}'\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.requests_imported, 1);
        assert_eq!(report.samples_imported, 1);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let requests = request_store::list_requests(&conn, &projects[0].id).unwrap();
        let samples = request_store::list_sample_responses(&conn, &requests[0].id).unwrap();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "Login Success");
        assert_eq!(samples[0].status, 200);
        assert_eq!(samples[0].content_type.as_deref(), Some("application/json"));
        assert_eq!(samples[0].body.as_deref(), Some("{\"ok\":true}"));
    }

    #[test]
    fn flattens_nested_folders_with_joined_names() {
        let scratch = ScratchDir::new("nested");
        let nested = scratch.0.join("collections").join("Nested").join("A").join("B");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("Deep.request.yaml"),
            "$kind: http-request\nurl: https://example.com\nmethod: POST\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.requests_imported, 1);
        assert_eq!(report.folders_created, 1);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let folders = folder_store::list_folders(&conn, &projects[0].id).unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "A / B");
    }

    #[test]
    fn imports_environment_values_into_a_dedicated_project() {
        let scratch = ScratchDir::new("env");
        let envs = scratch.0.join("environments");
        fs::create_dir_all(&envs).unwrap();
        fs::write(
            envs.join("Prod.environment.yaml"),
            "name: Prod\nvalues:\n  - key: host\n    value: https://api.example.com\n    enabled: true\n  - key: api_secret\n    value: shh\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.environments_imported, 1);
        assert_eq!(report.variables_imported, 2);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Imported Environments");

        let env_id = environment_store::list_environments(&conn, &projects[0].id).unwrap()[0].id.clone();
        let vars = variable_store::list_variables_for_scope(&conn, VariableScope::Environment, &env_id).unwrap();
        assert_eq!(vars.len(), 2);
        assert!(vars.iter().any(|v| v.key == "api_secret" && v.is_secret));
        assert!(vars.iter().any(|v| v.key == "host" && !v.is_secret));
    }

    #[test]
    fn rejects_missing_directory() {
        let conn = db::open_in_memory().unwrap();
        let err = import_local_workspace(&conn, r"Z:\definitely\not\a\real\path\for\this\test", "default").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn rejects_directory_that_is_not_a_postman_workspace() {
        let scratch = ScratchDir::new("not-a-workspace");
        let conn = db::open_in_memory().unwrap();
        let err = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
