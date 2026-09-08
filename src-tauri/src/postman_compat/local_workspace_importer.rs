// Import for Postman's newer "local files" workspace format — an on-disk directory tree
// (collections/<name>/**/*.request.yaml, environments/*.environment.yaml), NOT the single
// Postman Collection v2.1 JSON blob that `importer.rs` handles. Every file is parsed
// defensively via a generic `serde_yaml::Value` walk (never a strict typed deserialize) so one
// malformed/unusual file among thousands degrades to a warning instead of aborting the run.
use std::cmp::Ordering;
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
    /// Every project created by this import, in the order they were created (which is the
    /// alphabetical-by-directory-name order this importer reads `collections/` in — the closest
    /// available proxy for "source order" this file format carries, since the on-disk YAML tree
    /// has no explicit position field). The frontend uses this to default each of these
    /// projects' sort preference to "Custom order" so a many-project import doesn't get silently
    /// re-alphabetized by the sidebar's default Name sort.
    pub project_ids: Vec<String>,
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

/// Directory-entry sort key for reading `collections/`, a project's request/folder files, and
/// `.resources/*/examples/` — approximates Postman's source item order (see `project_ids`' doc
/// comment; the on-disk tree carries no explicit position field). Plain byte/lexicographic
/// sorting puts "10" before "2", so a folder numbered "1", "2", ... "10", "11" would import out
/// of order the moment it passed 9 items — invisible at the top level (few, distinctly-named
/// collections) but common inside a single collection's numbered request files. Digit runs are
/// compared as numbers instead so "2" sorts before "10".
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum NaturalChunk {
    Number(u128),
    Text(String),
}

fn natural_sort_key(name: &std::ffi::OsStr) -> Vec<NaturalChunk> {
    let name = name.to_string_lossy();
    let mut chunks = Vec::new();
    let mut chars = name.chars().peekable();
    while chars.peek().is_some() {
        if chars.peek().unwrap().is_ascii_digit() {
            let mut digits = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    digits.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            chunks.push(NaturalChunk::Number(digits.parse().unwrap_or(u128::MAX)));
        } else {
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    break;
                }
                text.push(c);
                chars.next();
            }
            chunks.push(NaturalChunk::Text(text));
        }
    }
    chunks
}

/// Best-effort peek at a YAML file's top-level `order:` and `name:` fields.
///
/// A local-files-synced workspace stamps every request file, and every folder's own
/// `.resources/definition.yaml`, with an `order:` — Postman's real drag-and-drop position among
/// its siblings, on a scale (1000, 2000, ...) that has nothing to do with the file/directory
/// name. Directory *listing* order (what `natural_sort_key` approximates) is only a fallback for
/// the rare item that predates this field; where `order` exists, it's ground truth and must win.
/// `name:` is the second reason to open this file — Postman truncates long folder names when it
/// writes the directory on disk, but keeps the untruncated name here.
///
/// Errors are swallowed: this is a secondary signal, and the file gets its own real warning
/// wherever it's actually parsed for import.
fn peek_order_and_name(path: &Path) -> (Option<f64>, Option<String>) {
    let Ok(content) = fs::read_to_string(path) else { return (None, None) };
    let Ok(doc) = serde_yaml::from_str::<Yaml>(&content) else { return (None, None) };
    let order = yaml_get(&doc, "order").and_then(|v| match v {
        Yaml::Number(n) => n.as_f64(),
        _ => None,
    });
    let name = yaml_get(&doc, "name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty());
    (order, name)
}

/// Sorts a source-order candidate list: real `order` wins when present (missing values sort
/// last), a natural-sort of the fallback key breaks ties or covers items with no `order` at all.
fn sort_by_source_order<T>(items: &mut [T], order: impl Fn(&T) -> Option<f64>, fallback_key: impl Fn(&T) -> Vec<NaturalChunk>) {
    items.sort_by(|a, b| match (order(a), order(b)) {
        (Some(oa), Some(ob)) => oa.partial_cmp(&ob).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => fallback_key(a).cmp(&fallback_key(b)),
    });
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
    let mut project_ids: Vec<String> = Vec::new();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|err| AppError::Storage(err.to_string()))?;

    let result = (|| -> Result<(), AppError> {
        if collections_dir.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(&collections_dir)
                .map_err(|err| AppError::Storage(err.to_string()))?
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| natural_sort_key(&e.file_name()));

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
                project_ids.push(project.id.clone());

                let mut folder_cache: HashMap<String, String> = HashMap::new();
                import_collection_dir(
                    conn,
                    &path,
                    &project.id,
                    &[],
                    &[],
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
            env_files.sort_by_key(|e| natural_sort_key(&e.file_name()));

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
                        let parsed = values.iter().filter_map(|item| {
                            let key = yaml_get(item, "key")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if key.is_empty() {
                                return None;
                            }
                            let value = yaml_scalar_to_string(yaml_get(item, "value"));
                            let enabled = yaml_get(item, "enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                            Some((key, (value, enabled)))
                        });
                        for (key, (value, enabled)) in super::dedupe_keys_last_wins(parsed) {
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

    Ok(LocalWorkspaceImportReport {
        projects_created,
        folders_created,
        requests_imported,
        samples_imported,
        environments_imported,
        variables_imported,
        warnings,
        project_ids,
    })
}

#[allow(clippy::too_many_arguments)]
fn import_collection_dir(
    conn: &Connection,
    dir: &Path,
    project_id: &str,
    folder_segments: &[String],
    folder_display_segments: &[String],
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
    let sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();

    // Pass 1: create every request directly in this directory first, keyed by its filename
    // stem — `.resources/<stem>.resources/examples/*.example.yaml` (pass 2 below) references
    // requests by that same stem, not by whatever the request's own internal `name:` says.
    let mut resources_dir: Option<std::path::PathBuf> = None;
    // (path, raw dir name, display name, real Postman `order`)
    let mut subdirs: Vec<(std::path::PathBuf, String, String, Option<f64>)> = Vec::new();
    // (dir entry, real Postman `order`)
    let mut request_files: Vec<(fs::DirEntry, Option<f64>)> = Vec::new();

    for entry in sorted {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if file_name == ".resources" {
                resources_dir = Some(path);
            } else {
                let (order, display_name) = peek_order_and_name(&path.join(".resources").join("definition.yaml"));
                subdirs.push((path, file_name.clone(), display_name.unwrap_or(file_name), order));
            }
        } else if file_name.ends_with(".request.yaml") || file_name.ends_with(".request.yml") {
            let (order, _) = peek_order_and_name(&path);
            request_files.push((entry, order));
        }
    }

    // A local-files sync stamps every request/folder with its real drag-and-drop position (see
    // `peek_order_and_name`) — that's what a many-item collection actually needs, since plain
    // directory-listing order has nothing to do with how the user arranged the collection in
    // Postman. Requests and folders each carry their own separate sort_order sequence (see
    // folder_store/request_store), so sorting the two lists independently by that same `order`
    // reproduces the source arrangement within each.
    sort_by_source_order(&mut request_files, |(_, order)| *order, |(entry, _)| natural_sort_key(&entry.file_name()));
    sort_by_source_order(&mut subdirs, |(_, _, _, order)| *order, |(_, raw_name, _, _)| natural_sort_key(std::ffi::OsStr::new(raw_name)));

    let mut request_ids_by_stem: HashMap<String, String> = HashMap::new();
    for (entry, _) in &request_files {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let stem = file_name
            .strip_suffix(".request.yaml")
            .or_else(|| file_name.strip_suffix(".request.yml"))
            .unwrap_or(&file_name)
            .to_string();
        let folder_id = if folder_segments.is_empty() {
            None
        } else {
            get_or_create_folder(conn, project_id, folder_segments, folder_display_segments, folder_cache, folders_created, warnings)
        };
        if let Some(id) = import_request_file(conn, &path, project_id, folder_id, requests_imported, warnings) {
            request_ids_by_stem.insert(stem, id);
        }
    }

    // Pass 2: saved example responses, one `.resources/<stem>.resources/examples/` dir per request.
    if let Some(resources_dir) = resources_dir {
        import_examples_in_resources_dir(conn, &resources_dir, &request_ids_by_stem, samples_imported, warnings);
    }

    // Pass 3: recurse into real Postman sub-folders, in the same source order.
    for (path, raw_name, display_name, _) in subdirs {
        let mut child_segments = folder_segments.to_vec();
        child_segments.push(raw_name);
        let mut child_display_segments = folder_display_segments.to_vec();
        child_display_segments.push(display_name);
        import_collection_dir(
            conn,
            &path,
            project_id,
            &child_segments,
            &child_display_segments,
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
        example_files.sort_by_key(|e| natural_sort_key(&e.file_name()));

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

/// Walks `segments` (one per directory level) and returns the id of the deepest folder,
/// creating any level that doesn't exist yet and hanging it off its real parent.
///
/// Folders used to be flattened into a single level named "Parent / Child", which made deep
/// collections read as a wall of long sibling names. The app's own model supports arbitrary
/// nesting (`NewFolderInput::parent_folder_id`, and the sidebar renders folders recursively),
/// so the source hierarchy is kept intact instead. Segments are passed as a slice rather than a
/// pre-joined path so a folder whose own name contains the separator can't be split by mistake.
fn get_or_create_folder(
    conn: &Connection,
    project_id: &str,
    segments: &[String],
    display_segments: &[String],
    folder_cache: &mut HashMap<String, String>,
    folders_created: &mut usize,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let mut parent_folder_id: Option<String> = None;
    for depth in 0..segments.len() {
        // NUL can't occur in a path segment, so this cache key can never collide with a
        // different-but-similarly-named hierarchy. Keyed by the raw on-disk segment (not the
        // display one) since that's the stable, collision-free identity — the display name
        // (from `.resources/definition.yaml`, see `peek_order_and_name`) is only what gets
        // shown, for the case where Postman truncated the real name when writing the directory.
        let cache_key = segments[..=depth].join("\u{0}");
        if let Some(id) = folder_cache.get(&cache_key) {
            parent_folder_id = Some(id.clone());
            continue;
        }
        let name = display_segments.get(depth).cloned().unwrap_or_else(|| segments[depth].clone());
        match folder_store::create_folder(
            conn,
            NewFolderInput {
                project_id: project_id.to_string(),
                name: name.clone(),
                parent_folder_id: parent_folder_id.clone(),
            },
        ) {
            Ok(f) => {
                folder_cache.insert(cache_key, f.id.clone());
                *folders_created += 1;
                parent_folder_id = Some(f.id);
            }
            Err(err) => {
                warnings.push(format!("Could not create folder '{name}': {err}"));
                return None;
            }
        }
    }
    parent_folder_id
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

    // The local workspace format writes `headers` as a flat mapping (`Key: value`) when no
    // header needs extra metadata, but switches to a sequence of `{key, value, disabled,
    // description}` objects — the only way to express a disabled header or a description — the
    // moment one does. Real exports mix both across files, so both shapes must be handled or
    // every header on a sequence-shaped request (not just a missing Content-Type) silently
    // vanishes on import.
    let headers = match yaml_get(&doc, "headers") {
        Some(Yaml::Mapping(map)) => map
            .iter()
            .filter_map(|(k, v)| {
                let key = k.as_str()?.trim();
                if key.is_empty() {
                    return None;
                }
                Some(HeaderEntry {
                    key: key.to_string(),
                    value: yaml_scalar_to_string(Some(v)),
                    enabled: true,
                    description: None,
                })
            })
            .collect(),
        Some(Yaml::Sequence(seq)) => seq
            .iter()
            .filter_map(|item| {
                let key = yaml_get(item, "key").and_then(|v| v.as_str())?.trim().to_string();
                if key.is_empty() {
                    return None;
                }
                Some(HeaderEntry {
                    key,
                    value: yaml_scalar_to_string(yaml_get(item, "value")),
                    enabled: !yaml_get(item, "disabled").and_then(|v| v.as_bool()).unwrap_or(false),
                    description: yaml_get(item, "description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                })
            })
            .collect(),
        _ => Vec::new(),
    };
    let mut headers: Vec<HeaderEntry> = headers;

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

    if let Some(mime) = infer_local_raw_content_type_header(yaml_get(&doc, "body")) {
        let has_content_type = headers.iter().any(|h| h.key.eq_ignore_ascii_case("content-type"));
        if !has_content_type {
            headers.push(HeaderEntry {
                key: "Content-Type".to_string(),
                value: mime.to_string(),
                enabled: true,
                description: None,
            });
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

/// Same gap as the single-collection-file importer: the local workspace format tags a raw body
/// with its own `type` (json/xml/html/javascript) separately from any Content-Type header, and
/// real exports frequently have the type tag but no matching header — Postman adds that header
/// client-side at send-time, not in the saved file. Without translating the tag into a real
/// header here, the imported request's raw-type dropdown falls back to "Text".
fn infer_local_raw_content_type_header(body: Option<&Yaml>) -> Option<&'static str> {
    let body_type = yaml_get(body?, "type").and_then(|v| v.as_str())?.to_lowercase();
    Some(match body_type.as_str() {
        "json" => "application/json",
        "xml" => "application/xml",
        "html" => "text/html",
        "javascript" => "application/javascript",
        _ => return None,
    })
}

fn parse_local_body(body: Option<&Yaml>, req_name: &str, warnings: &mut Vec<String>) -> Option<String> {
    let body = body?;
    let body_type = yaml_get(body, "type").and_then(|v| v.as_str()).unwrap_or("none").to_lowercase();
    let content = yaml_get(body, "content");

    match body_type.as_str() {
        "none" | "" => None,
        // A missing/empty `content` is not a malformed file — plenty of real requests declare a
        // body mode while passing every field via the query string instead. Warning about those
        // trains the user to ignore the compatibility notes, so only a genuinely unreadable
        // shape gets flagged.
        "urlencoded" if content.is_none() || matches!(content, Some(Yaml::Null)) => None,
        "formdata" if content.is_none() || matches!(content, Some(Yaml::Null)) => None,
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
                        let file_path = yaml_get(item, "src")
                            .and_then(|v| v.as_str())
                            .map(super::normalize_local_file_src);
                        if is_file {
                            let still_here = file_path.as_deref().is_some_and(|p| Path::new(p).is_file());
                            if !still_here {
                                warnings.push(format!(
                                    "Request '{req_name}': multipart file part '{key}' references a local file — re-select it on this machine."
                                ));
                            }
                        }
                        Some(FormDataPart {
                            key,
                            value: yaml_scalar_to_string(yaml_get(item, "value")),
                            enabled: !yaml_get(item, "disabled").and_then(|v| v.as_bool()).unwrap_or(false),
                            description: None,
                            is_file,
                            file_path,
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

    /// The local-workspace format writes `headers` as a flat mapping only when no header needs
    /// extra metadata — the moment one is disabled or carries a description, real exports switch
    /// to a sequence of `{key, value, disabled, description}` objects instead (there's no way to
    /// express "disabled" in the flat mapping shape). Before this fix, only the mapping shape was
    /// recognized, so every header on a sequence-shaped request — not just a missing
    /// Content-Type — silently vanished on import.
    #[test]
    fn imports_sequence_shaped_headers_including_disabled_ones() {
        let scratch = ScratchDir::new("seq-headers");
        let collections = scratch.0.join("collections").join("SeqHeaders");
        fs::create_dir_all(&collections).unwrap();
        fs::write(
            collections.join("Get Status.request.yaml"),
            "$kind: http-request\nurl: \"{{host}}/status\"\nmethod: GET\nheaders:\n  - key: Content-Type\n    value: application/json\n  - key: X-Device\n    value: \"{{device}}\"\n    disabled: true\n  - key: X-Trace\n    value: \"1\"\n    description: for debugging\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();
        assert_eq!(report.requests_imported, 1);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let requests = request_store::list_requests(&conn, &projects[0].id).unwrap();
        let req = request_store::get_request(&conn, &requests[0].id).unwrap();

        assert_eq!(req.headers.len(), 3);
        let content_type = req.headers.iter().find(|h| h.key == "Content-Type").unwrap();
        assert_eq!(content_type.value, "application/json");
        assert!(content_type.enabled);

        let device = req.headers.iter().find(|h| h.key == "X-Device").unwrap();
        assert!(!device.enabled, "disabled: true must be preserved, not dropped");

        let trace = req.headers.iter().find(|h| h.key == "X-Trace").unwrap();
        assert_eq!(trace.description.as_deref(), Some("for debugging"));
    }

    /// The local-workspace `.request.yaml` format tags a raw body's format via `body.type`
    /// (json/xml/html/...), separate from any Content-Type header — real exports commonly have
    /// the tag but no matching header. Without inferring the header from the tag on import, the
    /// request editor's raw-type dropdown falls back to "Text" for a body that's actually JSON.
    #[test]
    fn imports_raw_json_body_type_as_content_type_header() {
        let scratch = ScratchDir::new("body-type");
        let collection = scratch.0.join("collections").join("BodyType");
        fs::create_dir_all(&collection).unwrap();
        fs::write(
            collection.join("Update Status.request.yaml"),
            "$kind: http-request\nurl: \"{{host}}/status\"\nmethod: POST\nbody:\n  type: json\n  content: '{\"approved_ids\":[\"1\"]}'\n",
        )
        .unwrap();
        fs::write(
            collection.join("Already Typed.request.yaml"),
            "$kind: http-request\nurl: \"{{host}}/status\"\nmethod: POST\nheaders:\n  Content-Type: application/vnd.custom+json\nbody:\n  type: json\n  content: '{\"x\":1}'\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();
        assert_eq!(report.requests_imported, 2);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let requests = request_store::list_requests(&conn, &projects[0].id).unwrap();

        let untyped = requests.iter().find(|r| r.name == "Update Status").unwrap();
        let untyped = request_store::get_request(&conn, &untyped.id).unwrap();
        assert_eq!(untyped.headers.len(), 1);
        assert_eq!(untyped.headers[0].key, "Content-Type");
        assert_eq!(untyped.headers[0].value, "application/json");

        // An explicit header in the source file must never be overridden by the inferred one.
        let already_typed = requests.iter().find(|r| r.name == "Already Typed").unwrap();
        let already_typed = request_store::get_request(&conn, &already_typed.id).unwrap();
        assert_eq!(already_typed.headers.len(), 1);
        assert_eq!(already_typed.headers[0].value, "application/vnd.custom+json");
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
    fn preserves_nested_folder_hierarchy() {
        let scratch = ScratchDir::new("nested");
        let nested = scratch.0.join("collections").join("Nested").join("A").join("B");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("Deep.request.yaml"),
            "$kind: http-request\nurl: https://example.com\nmethod: POST\n",
        )
        .unwrap();
        // A sibling under the same parent must reuse "A" rather than creating a second one.
        let sibling = scratch.0.join("collections").join("Nested").join("A").join("C");
        fs::create_dir_all(&sibling).unwrap();
        fs::write(
            sibling.join("Other.request.yaml"),
            "$kind: http-request\nurl: https://example.com/other\nmethod: GET\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.requests_imported, 2);
        assert_eq!(report.folders_created, 3, "A, A/B and A/C — not one per leaf path");

        let projects = project_store::list_projects(&conn, "default").unwrap();
        assert_eq!(report.project_ids, vec![projects[0].id.clone()]);
        let folders = folder_store::list_folders(&conn, &projects[0].id).unwrap();
        assert_eq!(folders.len(), 3);

        let a = folders.iter().find(|f| f.name == "A").unwrap();
        assert!(a.parent_folder_id.is_none(), "top-level folder sits at the project root");

        let b = folders.iter().find(|f| f.name == "B").unwrap();
        assert_eq!(b.parent_folder_id.as_deref(), Some(a.id.as_str()));

        let c = folders.iter().find(|f| f.name == "C").unwrap();
        assert_eq!(c.parent_folder_id.as_deref(), Some(a.id.as_str()));

        // The deep request belongs to B, not to a flattened "A / B" folder at the root.
        let requests = request_store::list_requests(&conn, &projects[0].id).unwrap();
        let deep = requests.iter().find(|r| r.name == "Deep").unwrap();
        assert_eq!(deep.folder_id.as_deref(), Some(b.id.as_str()));
    }

    /// The real bug report this pins: a synced local workspace stamps every request file, and
    /// every folder's own `.resources/definition.yaml`, with an `order:` field carrying the
    /// user's actual drag-and-drop position in Postman — on a scale (1000, 2000, ...) that has
    /// nothing to do with alphabetical filename order. A collection arranged as
    /// "Zebra, Apple, Mango" on disk (alphabetically: Apple, Mango, Zebra) must still import as
    /// Zebra, Apple, Mango. The same file also carries the folder's untruncated `name:` — Windows
    /// truncates a long directory name when Postman writes it to disk, but keeps the real name
    /// here, so the importer must prefer it over the (possibly truncated) directory name.
    #[test]
    fn respects_explicit_order_and_name_from_resources_definition_yaml() {
        let scratch = ScratchDir::new("explicit-order");
        let root = scratch.0.join("collections").join("Proj");
        fs::create_dir_all(&root).unwrap();

        // Three sibling requests at the project root, written to disk in alphabetical order but
        // stamped with `order:` values that say the real arrangement is Zebra, Apple, Mango.
        for (name, order) in [("Apple", 2000), ("Mango", 3000), ("Zebra", 1000)] {
            fs::write(
                root.join(format!("{name}.request.yaml")),
                format!("$kind: http-request\nurl: https://example.com\nmethod: GET\norder: {order}\n"),
            )
            .unwrap();
        }

        // Two sibling folders, same idea: "B Folder" is written before "A Folder" on disk but
        // stamped to come second. "A Folder" also has its on-disk name truncated relative to its
        // real name, which only `.resources/definition.yaml` carries.
        let a_folder = root.join("A Folder (truncated");
        fs::create_dir_all(a_folder.join(".resources")).unwrap();
        fs::write(
            a_folder.join(".resources").join("definition.yaml"),
            "$kind: collection\nname: A Folder (truncated on disk)\norder: 1000\n",
        )
        .unwrap();
        fs::write(
            a_folder.join("req.request.yaml"),
            "$kind: http-request\nurl: https://example.com\nmethod: GET\n",
        )
        .unwrap();

        let b_folder = root.join("B Folder");
        fs::create_dir_all(b_folder.join(".resources")).unwrap();
        fs::write(b_folder.join(".resources").join("definition.yaml"), "$kind: collection\norder: 2000\n").unwrap();
        fs::write(
            b_folder.join("req.request.yaml"),
            "$kind: http-request\nurl: https://example.com\nmethod: GET\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let mut root_requests = request_store::list_requests(&conn, &projects[0].id)
            .unwrap()
            .into_iter()
            .filter(|r| r.folder_id.is_none())
            .collect::<Vec<_>>();
        root_requests.sort_by_key(|r| r.sort_order);
        let names: Vec<&str> = root_requests.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["Zebra", "Apple", "Mango"]);

        let mut root_folders = folder_store::list_folders(&conn, &projects[0].id)
            .unwrap()
            .into_iter()
            .filter(|f| f.parent_folder_id.is_none())
            .collect::<Vec<_>>();
        root_folders.sort_by_key(|f| f.sort_order);
        let folder_names: Vec<&str> = root_folders.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(folder_names, vec!["A Folder (truncated on disk)", "B Folder"]);
    }

    /// A folder numbered "1", "2", ... "10", "11" sorts "10" and "11" before "2" under plain
    /// byte/lexicographic comparison — invisible with a handful of top-level collections, but a
    /// real problem inside a single collection's numbered request files, which is exactly what
    /// this importer's "Custom order" default (see `project_ids`' doc comment) is supposed to
    /// preserve. Digit runs must compare as numbers instead.
    #[test]
    fn sorts_numbered_requests_and_folders_naturally_not_lexicographically() {
        let scratch = ScratchDir::new("natural-order");
        let sub = scratch.0.join("collections").join("Proj").join("Sub");
        fs::create_dir_all(&sub).unwrap();
        for i in [1, 2, 3, 9, 10, 11, 20] {
            fs::write(
                sub.join(format!("{i} req.request.yaml")),
                "$kind: http-request\nurl: https://example.com\nmethod: GET\n",
            )
            .unwrap();
        }
        // Sibling folders under the same parent must also sort naturally, not just requests.
        fs::create_dir_all(sub.join("2 Folder")).unwrap();
        fs::write(
            sub.join("2 Folder").join("nested.request.yaml"),
            "$kind: http-request\nurl: https://example.com\nmethod: GET\n",
        )
        .unwrap();
        fs::create_dir_all(sub.join("10 Folder")).unwrap();
        fs::write(
            sub.join("10 Folder").join("nested.request.yaml"),
            "$kind: http-request\nurl: https://example.com\nmethod: GET\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let folders = folder_store::list_folders(&conn, &projects[0].id).unwrap();
        let sub_folder = folders.iter().find(|f| f.name == "Sub").unwrap();

        let mut requests = request_store::list_requests(&conn, &projects[0].id)
            .unwrap()
            .into_iter()
            .filter(|r| r.folder_id.as_deref() == Some(sub_folder.id.as_str()))
            .collect::<Vec<_>>();
        requests.sort_by_key(|r| r.sort_order);
        let names: Vec<&str> = requests.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["1 req", "2 req", "3 req", "9 req", "10 req", "11 req", "20 req"]);

        let mut child_folders = folders
            .iter()
            .filter(|f| f.parent_folder_id.as_deref() == Some(sub_folder.id.as_str()))
            .collect::<Vec<_>>();
        child_folders.sort_by_key(|f| f.sort_order);
        let folder_names: Vec<&str> = child_folders.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(folder_names, vec!["2 Folder", "10 Folder"]);
    }

    /// `report.project_ids` is what the frontend uses to default a many-project import's sort
    /// preference to "Custom order" (see +page.svelte's `importLocalWorkspaceAction`) — it must
    /// list every created project, in the same alphabetical-by-directory-name order they were
    /// actually created in, not just contain the right ids in any order.
    #[test]
    fn report_lists_every_created_project_id_in_creation_order() {
        let scratch = ScratchDir::new("multi-project-order");
        for name in ["Zeta", "Alpha", "Mid"] {
            let dir = scratch.0.join("collections").join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("Ping.request.yaml"),
                "$kind: http-request\nurl: https://example.com\nmethod: GET\n",
            )
            .unwrap();
        }

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.projects_created, 3);
        assert_eq!(report.project_ids.len(), 3);

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let expected_ids: Vec<String> = {
            let mut by_name: HashMap<String, String> =
                projects.iter().map(|p| (p.name.clone(), p.id.clone())).collect();
            // Directory entries are read in natural sort order (see `natural_sort_key`), which
            // for these non-numeric names is just alphabetical: Alpha, then Mid, then Zeta.
            ["Alpha", "Mid", "Zeta"]
                .iter()
                .map(|name| by_name.remove(*name).unwrap())
                .collect()
        };
        assert_eq!(report.project_ids, expected_ids);
    }

    /// Duplicate keys in one environment: Postman folds its ordered list into a map when
    /// resolving `{{key}}`, so the last definition is the live one. Importing in order and
    /// letting the store reject duplicates kept the *first* — typically a stale value the user
    /// had explicitly switched off.
    #[test]
    fn duplicate_environment_keys_keep_the_last_definition() {
        let scratch = ScratchDir::new("dupe-vars");
        let envs = scratch.0.join("environments");
        fs::create_dir_all(&envs).unwrap();
        fs::write(
            envs.join("Dupes.environment.yaml"),
            "name: Dupes\nvalues:\n  - key: host\n    value: https://old.example.com\n    enabled: false\n  - key: host\n    value: https://live.example.com\n    enabled: true\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.variables_imported, 1);
        assert!(
            report.warnings.is_empty(),
            "a duplicate key is normal in real exports, not something to warn about: {:?}",
            report.warnings
        );

        let projects = project_store::list_projects(&conn, "default").unwrap();
        let env_id = environment_store::list_environments(&conn, &projects[0].id).unwrap()[0].id.clone();
        let vars = variable_store::list_variables_for_scope(&conn, VariableScope::Environment, &env_id).unwrap();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].value, "https://live.example.com");
        assert!(vars[0].enabled);
    }

    /// A body mode with no `content` at all is a perfectly normal request (everything passed via
    /// the query string instead) — it must not be reported as a malformed file.
    #[test]
    fn empty_urlencoded_body_imports_without_a_warning() {
        let scratch = ScratchDir::new("empty-urlencoded");
        let collection = scratch.0.join("collections").join("EmptyBody");
        fs::create_dir_all(&collection).unwrap();
        fs::write(
            collection.join("Token.request.yaml"),
            "$kind: http-request\nurl: \"{{host}}/oauth/token?grant_type=client_credentials\"\nmethod: POST\nbody:\n  type: urlencoded\n",
        )
        .unwrap();

        let conn = db::open_in_memory().unwrap();
        let report = import_local_workspace(&conn, scratch.0.to_str().unwrap(), "default").unwrap();

        assert_eq!(report.requests_imported, 1);
        assert!(report.warnings.is_empty(), "unexpected warnings: {:?}", report.warnings);
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
