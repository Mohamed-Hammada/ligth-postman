use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::ai::{AiProvider, ClaudeProvider, GeneratedApiDefinition};
use crate::canonical_request;
use crate::error::AppError;
use crate::execution::{self, ExecutionInput};
use crate::models::{
    Auth, Environment, Folder, HeaderEntry, NewEnvironmentInput, NewFolderInput, NewProjectInput,
    NewRequestInput, NewVariableInput, Project, RequestFull, RequestSummary, ResponseBodyPayload,
    ResponseMeta, ResponseSummary, UpdateEnvironmentInput, UpdateFolderInput, UpdateProjectInput,
    UpdateRequestInput, UpdateVariableInput, VariableScope, VariableView,
};
use crate::resolver::{self, ScopeChain};
use crate::store::{
    environment_store, folder_store, project_store, request_store, response_store, variable_store,
};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub http_client: reqwest::Client,
    pub response_body_dir: PathBuf,
    /// Keyed by request_id so `cancel_send` can find the in-flight send for a request.
    /// Cancellation works by racing the execution future against this channel with
    /// `tokio::select!` (see `send_request`) — dropping the losing future is what actually
    /// aborts the underlying HTTP call. Sending the same request twice concurrently means
    /// the second send's sender overwrites the first's — a documented simplification, not
    /// a real per-execution id yet (see PROJECT_MAP).
    pub cancel_signals: Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>,
    /// `None` when `ANTHROPIC_API_KEY` isn't set — AI is an opt-in feature, its absence is
    /// not a startup failure (LP-0803).
    pub ai_provider: Option<ClaudeProvider>,
    pub console: std::sync::Arc<crate::console::ConsoleBuffer>,
    pub job_manager: std::sync::Arc<crate::background_jobs::BackgroundJobManager>,
    pub start_time: std::time::Instant,
    pub db_path: Option<PathBuf>,
}

#[tauri::command]
pub fn create_project(state: State<AppState>, name: String) -> Result<Project, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let project = project_store::create_project(&conn, NewProjectInput { name })?;
    log::info!("created project {}", project.id);
    Ok(project)
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    project_store::list_projects(&conn)
}

#[tauri::command]
pub fn get_project(state: State<AppState>, id: String) -> Result<Project, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    project_store::get_project(&conn, &id)
}

#[tauri::command]
pub fn get_project_request_counts(
    state: State<AppState>,
) -> Result<std::collections::HashMap<String, usize>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    project_store::request_counts_by_project(&conn)
}

#[tauri::command]
pub fn update_project(state: State<AppState>, input: UpdateProjectInput) -> Result<Project, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let project = project_store::update_project(&conn, input)?;
    log::info!("updated project {}", project.id);
    Ok(project)
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    project_store::delete_project(&conn, &id)?;
    log::info!("deleted project {id}");
    Ok(())
}

#[tauri::command]
pub fn create_request(
    state: State<AppState>,
    input: NewRequestInput,
) -> Result<RequestFull, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let request = request_store::create_request(&conn, input)?;
    log::info!("created request {} in project {}", request.id, request.project_id);
    Ok(request)
}

#[tauri::command]
pub fn list_requests(
    state: State<AppState>,
    project_id: String,
) -> Result<Vec<RequestSummary>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::list_requests(&conn, &project_id)
}

#[tauri::command]
pub fn get_request(state: State<AppState>, id: String) -> Result<RequestFull, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::get_request(&conn, &id)
}

#[tauri::command]
pub fn update_request(state: State<AppState>, input: UpdateRequestInput) -> Result<RequestFull, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let request = request_store::update_request(&conn, input)?;
    log::info!("updated request {}", request.id);
    Ok(request)
}

#[tauri::command]
pub fn delete_request(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::delete_request(&conn, &id)?;
    log::info!("deleted request {id}");
    Ok(())
}

#[tauri::command]
pub fn create_folder(state: State<AppState>, input: NewFolderInput) -> Result<Folder, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let folder = folder_store::create_folder(&conn, input)?;
    log::info!("created folder {} in project {}", folder.id, folder.project_id);
    Ok(folder)
}

#[tauri::command]
pub fn list_folders(state: State<AppState>, project_id: String) -> Result<Vec<Folder>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    folder_store::list_folders(&conn, &project_id)
}

#[tauri::command]
pub fn update_folder(state: State<AppState>, input: UpdateFolderInput) -> Result<Folder, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let folder = folder_store::update_folder(&conn, input)?;
    log::info!("updated folder {}", folder.id);
    Ok(folder)
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    folder_store::delete_folder(&conn, &id)?;
    log::info!("deleted folder {id}");
    Ok(())
}

#[tauri::command]
pub fn create_environment(
    state: State<AppState>,
    input: NewEnvironmentInput,
) -> Result<Environment, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let environment = environment_store::create_environment(&conn, input)?;
    log::info!("created environment {} in project {}", environment.id, environment.project_id);
    Ok(environment)
}

#[tauri::command]
pub fn list_environments(
    state: State<AppState>,
    project_id: String,
) -> Result<Vec<Environment>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    environment_store::list_environments(&conn, &project_id)
}

#[tauri::command]
pub fn update_environment(
    state: State<AppState>,
    input: UpdateEnvironmentInput,
) -> Result<Environment, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let environment = environment_store::update_environment(&conn, input)?;
    log::info!("updated environment {}", environment.id);
    Ok(environment)
}

#[tauri::command]
pub fn delete_environment(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    environment_store::delete_environment(&conn, &id)?;
    log::info!("deleted environment {id}");
    Ok(())
}

#[tauri::command]
pub fn create_variable(
    state: State<AppState>,
    input: NewVariableInput,
) -> Result<VariableView, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let variable = variable_store::create_variable(&conn, input)?;
    log::info!("created variable {} (key='{}', scope={:?})", variable.id, variable.key, variable.scope);
    Ok(variable.into())
}

/// Lists variables for one concrete scope instance, e.g. `(Global, <project_id>)` or
/// `(Environment, <environment_id>)`. Always masked — see `VariableView`.
#[tauri::command]
pub fn list_variables_for_scope(
    state: State<AppState>,
    scope: VariableScope,
    scope_ref: String,
) -> Result<Vec<VariableView>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let variables = variable_store::list_variables_for_scope(&conn, scope, &scope_ref)?;
    Ok(variables.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub fn update_variable(
    state: State<AppState>,
    input: UpdateVariableInput,
) -> Result<VariableView, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let variable = variable_store::update_variable(&conn, input)?;
    log::info!("updated variable {}", variable.id);
    Ok(variable.into())
}

#[tauri::command]
pub fn delete_variable(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    variable_store::delete_variable(&conn, &id)?;
    log::info!("deleted variable {id}");
    Ok(())
}

/// Explicit, user-initiated reveal of a secret value — never returned by the normal list/get
/// commands (README §41 "never expose secrets unnecessarily to the frontend").
#[tauri::command]
pub fn reveal_variable_value(state: State<AppState>, id: String) -> Result<String, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let variable = variable_store::get_variable(&conn, &id)?;
    log::info!("revealed variable {id}");
    Ok(variable.value)
}

/// Executes a request end-to-end (load → resolve variables → HTTP → persist) and returns
/// its metadata. Async, so it never blocks the UI thread; `cancel_send` can abort it by id.
#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    request_id: String,
    environment_id: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<ResponseMeta, AppError> {
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
    {
        let mut signals = state.cancel_signals.lock().expect("mutex poisoned");
        signals.insert(request_id.clone(), cancel_tx);
    }

    let execution = execution::execute_request(
        &state.db,
        &state.http_client,
        &state.response_body_dir,
        Some(&state.console),
        ExecutionInput {
            request_id: request_id.clone(),
            environment_id,
            timeout_ms: timeout_ms.unwrap_or(30_000),
        },
    );

    // Dropping the losing branch here is what actually cancels the in-flight HTTP call —
    // reqwest/hyper abort the underlying connection when their future is dropped.
    let result = tokio::select! {
        res = execution => res,
        _ = cancel_rx => Err(AppError::Cancelled),
    };

    state.cancel_signals.lock().expect("mutex poisoned").remove(&request_id);

    if let Ok(response) = &result {
        log::info!("sent request {request_id}: status {}", response.status);
    }
    result
}

#[tauri::command]
pub fn cancel_send(state: State<AppState>, request_id: String) -> Result<(), AppError> {
    let mut signals = state.cancel_signals.lock().expect("mutex poisoned");
    match signals.remove(&request_id) {
        Some(tx) => {
            let _ = tx.send(());
            log::info!("cancelled request {request_id}");
            Ok(())
        }
        None => Err(AppError::NotFound(format!("no in-flight request '{request_id}'"))),
    }
}

#[tauri::command]
pub fn list_response_summaries(
    state: State<AppState>,
    request_id: String,
) -> Result<Vec<ResponseSummary>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    response_store::list_summaries(&conn, &request_id)
}

#[tauri::command]
pub fn list_project_history(
    state: State<AppState>,
    project_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::models::ProjectHistoryEntry>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    response_store::list_history_for_project(&conn, &project_id, limit.unwrap_or(200))
}

#[tauri::command]
pub fn get_response(state: State<AppState>, id: String) -> Result<ResponseMeta, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    response_store::get_response(&conn, &id)
}

#[tauri::command]
pub fn get_response_body(state: State<AppState>, id: String) -> Result<ResponseBodyPayload, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    response_store::get_response_body(&conn, &id)
}

#[tauri::command]
pub fn delete_response(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    response_store::delete_response(&conn, &id)?;
    log::info!("deleted response {id}");
    Ok(())
}

/// Resolves a template string (e.g. a request URL) against the live scope chain for a
/// project/environment/request combination. Pure preview — does not persist anything and
/// is the same code path the future HTTP engine will use to build the final request.
#[tauri::command]
pub fn resolve_preview(
    state: State<AppState>,
    project_id: String,
    environment_id: Option<String>,
    request_id: Option<String>,
    template: String,
) -> Result<resolver::ResolvedTemplate, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let (global, environment, request) =
        variable_store::load_scope_maps(&conn, &project_id, environment_id.as_deref(), request_id.as_deref())?;
    let chain = ScopeChain {
        global: Some(&global),
        environment: Some(&environment),
        request: Some(&request),
        ..Default::default()
    };
    Ok(resolver::resolve_template(&template, &chain))
}

/// Evaluates unresolved variables across all request locations (URL, headers, query params, auth, body) (LP-0208).
#[tauri::command]
pub fn diagnose_request(
    state: State<AppState>,
    request_id: String,
    environment_id: Option<String>,
) -> Result<canonical_request::RequestDiagnostics, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let req = request_store::get_request(&conn, &request_id)?;
    let (global, environment, request) =
        variable_store::load_scope_maps(&conn, &req.project_id, environment_id.as_deref(), Some(&req.id))?;
    let chain = ScopeChain {
        global: Some(&global),
        environment: Some(&environment),
        request: Some(&request),
        ..Default::default()
    };
    Ok(canonical_request::diagnose(&req, &chain))
}

fn resolve_ai_provider(state: &AppState) -> Result<ClaudeProvider, AppError> {
    if let Some(ref p) = state.ai_provider {
        return Ok(p.clone());
    }
    let conn = state.db.lock().expect("db mutex poisoned");
    ClaudeProvider::from_db_or_env(state.http_client.clone(), &conn).ok_or_else(|| {
        AppError::Validation(
            "AI is not configured. Please configure your Anthropic API key in AI Settings or set ANTHROPIC_API_KEY.".into(),
        )
    })
}

/// Lets the frontend show/hide the AI entry point without ever exposing whether (or what)
/// key is configured.
#[tauri::command]
pub fn is_ai_configured(state: State<AppState>) -> bool {
    if state.ai_provider.is_some() {
        return true;
    }
    let conn = state.db.lock().expect("db mutex poisoned");
    ClaudeProvider::from_db_or_env(state.http_client.clone(), &conn).is_some()
}

#[tauri::command]
pub fn get_ai_settings(state: State<AppState>) -> Result<crate::ai::AiSettings, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::ai::get_ai_settings(&conn)
}

#[tauri::command]
pub fn save_ai_settings(state: State<AppState>, input: crate::ai::UpdateAiSettingsInput) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::ai::save_ai_settings(&conn, &input)
}

#[tauri::command]
pub async fn test_ai_connection(state: State<'_, AppState>) -> Result<String, AppError> {
    let provider = resolve_ai_provider(&state)?;
    let test_def = provider.generate_api("ping endpoint").await?;
    Ok(format!("AI connection verified: generated '{}' ({})", test_def.name, test_def.method))
}

/// Prompt -> structured definition -> validated (LP-0805). Nothing is persisted here — the
/// frontend previews the result and only calls `create_request` if the user explicitly
/// approves it ("Add to Project"), reusing the exact same validation every manual create goes through.
#[tauri::command]
pub async fn generate_api_with_ai(
    state: State<'_, AppState>,
    prompt: String,
) -> Result<GeneratedApiDefinition, AppError> {
    let provider = resolve_ai_provider(&state)?;
    let definition = provider.generate_api(&prompt).await?;
    log::info!("AI generated an API definition: {}", definition.name);
    Ok(definition)
}

#[tauri::command]
pub async fn generate_api_with_project_context(
    state: State<'_, AppState>,
    project_id: String,
    prompt: String,
    include_existing_requests: bool,
    include_variable_names: bool,
) -> Result<GeneratedApiDefinition, AppError> {
    let context = {
        let conn = state.db.lock().expect("db mutex poisoned");
        let proj = project_store::get_project(&conn, &project_id)?;

        let existing_endpoints = if include_existing_requests {
            let reqs = request_store::list_requests(&conn, &project_id)?;
            reqs.into_iter().map(|r| format!("{} {} ({})", r.method, r.url, r.name)).collect()
        } else {
            Vec::new()
        };

        let variable_keys = if include_variable_names {
            let vars = variable_store::list_variables_for_scope(&conn, VariableScope::Global, &project_id)?;
            vars.into_iter()
                .filter(|v| !v.is_secret)
                .map(|v| v.key)
                .collect()
        } else {
            Vec::new()
        };

        crate::ai::ProjectAiContext {
            project_name: proj.name,
            existing_endpoints,
            variable_keys,
        }
    };

    let enriched_prompt = crate::ai::build_project_context_prompt(&prompt, &context);
    let provider = resolve_ai_provider(&state)?;
    let def = provider.generate_api(&enriched_prompt).await?;
    log::info!("AI generated API with project context: {}", def.name);
    Ok(def)
}

#[tauri::command]
pub async fn generate_sample_response_with_ai(
    state: State<'_, AppState>,
    request_id: String,
) -> Result<crate::models::SampleResponse, AppError> {
    let (method, url, body, desc) = {
        let conn = state.db.lock().expect("db mutex poisoned");
        let req = request_store::get_request(&conn, &request_id)?;
        (req.method, req.url, req.body, req.description)
    };

    let provider = resolve_ai_provider(&state)?;
    let sample = provider
        .generate_sample_response(&method, &url, body.as_deref(), desc.as_deref())
        .await?;

    let conn = state.db.lock().expect("db mutex poisoned");
    let input = crate::models::NewSampleResponseInput {
        request_id,
        name: sample.description.clone().unwrap_or_else(|| format!("Sample {}", sample.status)),
        status: sample.status,
        status_text: format!("HTTP {}", sample.status),
        headers: sample.headers,
        body: Some(sample.body),
        content_type: Some("application/json".to_string()),
    };
    let created = request_store::create_sample_response(&conn, input)?;
    Ok(created)
}

#[tauri::command]
pub async fn generate_tests_and_docs_with_ai(
    state: State<'_, AppState>,
    request_id: String,
) -> Result<crate::ai::GeneratedTestsAndDocs, AppError> {
    let (method, url, body) = {
        let conn = state.db.lock().expect("db mutex poisoned");
        let req = request_store::get_request(&conn, &request_id)?;
        (req.method, req.url, req.body)
    };

    let provider = resolve_ai_provider(&state)?;
    let result = provider
        .generate_tests_and_docs(&method, &url, body.as_deref())
        .await?;
    Ok(result)
}

#[tauri::command]
pub fn scan_source_project(directory: String) -> Result<crate::source_analyzer::SourceProjectReport, AppError> {
    crate::source_analyzer::scan_source_project(&directory)
}

#[tauri::command]
pub fn get_project_source_directory(
    state: State<AppState>,
    project_id: String,
) -> Result<Option<String>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::source_analyzer::get_project_source_association(&conn, &project_id)
}

#[tauri::command]
pub fn set_project_source_directory(
    state: State<AppState>,
    project_id: String,
    directory: String,
    framework: Option<String>,
) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::source_analyzer::set_project_source_association(&conn, &project_id, &directory, framework.as_deref())
}

#[tauri::command]
pub fn import_discovered_endpoint(
    state: State<AppState>,
    project_id: String,
    endpoint: crate::source_analyzer::DiscoveredEndpoint,
) -> Result<RequestSummary, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let url = if endpoint.path.starts_with("http") {
        endpoint.path
    } else {
        format!("{{{{baseUrl}}}}{}", endpoint.path)
    };

    let mut headers = vec![
        HeaderEntry {
            key: "Accept".to_string(),
            value: "application/json".to_string(),
            enabled: true,
            description: None,
        }
    ];

    if let Some(hint) = endpoint.auth_hint {
        headers.push(HeaderEntry {
            key: "Authorization".to_string(),
            value: "Bearer {{token}}".to_string(),
            enabled: true,
            description: Some(hint),
        });
    }

    let input = NewRequestInput {
        project_id,
        folder_id: None,
        name: endpoint.name,
        method: endpoint.method,
        url,
        headers,
        query_params: Vec::new(),
        auth: Auth::None,
        body: None,
        description: endpoint.description,
        settings: None,
        pre_request_script: None,
        post_request_script: None,
    };

    let created = request_store::create_request(&conn, input)?;
    Ok(RequestSummary {
        id: created.id,
        project_id: created.project_id,
        folder_id: created.folder_id,
        name: created.name,
        method: created.method,
        url: created.url,
        updated_at: created.updated_at,
    })
}

/// LP-0605 (snippet UI backend). Builds the request's own scope chain the same way
/// `resolve_preview`/`send_request` do, then hands it to `codegen::generate_snippet`.
#[tauri::command]
pub fn generate_curl_snippet(
    state: State<AppState>,
    request_id: String,
    environment_id: Option<String>,
    mode: crate::codegen::SnippetMode,
    target: Option<crate::codegen::SnippetTarget>,
) -> Result<String, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let request = request_store::get_request(&conn, &request_id)?;
    let (global, environment, request_vars) =
        variable_store::load_scope_maps(&conn, &request.project_id, environment_id.as_deref(), Some(&request.id))?;
    let chain = ScopeChain {
        global: Some(&global),
        environment: Some(&environment),
        request: Some(&request_vars),
        ..Default::default()
    };
    crate::codegen::generate_snippet(&request, &chain, mode, target.unwrap_or_default())
}

#[tauri::command]
pub fn import_curl(command: String) -> Result<crate::curl_importer::ParsedCurlRequest, AppError> {
    crate::curl_importer::parse_curl(&command)
}

#[tauri::command]
pub fn create_sample_response(
    state: State<AppState>,
    input: crate::models::NewSampleResponseInput,
) -> Result<crate::models::SampleResponse, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::create_sample_response(&conn, input)
}

#[tauri::command]
pub fn list_sample_responses(
    state: State<AppState>,
    request_id: String,
) -> Result<Vec<crate::models::SampleResponse>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::list_sample_responses(&conn, &request_id)
}

#[tauri::command]
pub fn delete_sample_response(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::delete_sample_response(&conn, &id)
}

#[tauri::command]
pub fn create_cookie(
    state: State<AppState>,
    input: crate::models::NewCookieInput,
) -> Result<crate::models::Cookie, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::create_cookie(&conn, input)
}

#[tauri::command]
pub fn list_cookies_for_project(
    state: State<AppState>,
    project_id: String,
) -> Result<Vec<crate::models::Cookie>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::list_cookies_for_project(&conn, &project_id)
}

#[tauri::command]
pub fn delete_cookie(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::delete_cookie(&conn, &id)
}

#[tauri::command]
pub fn import_postman_collection(
    state: State<AppState>,
    collection_json: String,
    project_id: Option<String>,
) -> Result<crate::postman_compat::CollectionImportReport, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::postman_compat::import_collection(&conn, &collection_json, project_id)
}

#[tauri::command]
pub fn import_postman_environment(
    state: State<AppState>,
    environment_json: String,
    project_id: String,
) -> Result<crate::postman_compat::EnvironmentImportReport, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::postman_compat::import_environment(&conn, &environment_json, &project_id)
}

#[tauri::command]
pub fn export_postman_collection(
    state: State<AppState>,
    project_id: String,
) -> Result<String, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::postman_compat::export_project_collection(&conn, &project_id)
}

#[tauri::command]
pub fn export_postman_environment(
    state: State<AppState>,
    environment_id: String,
) -> Result<String, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::postman_compat::export_environment(&conn, &environment_id)
}

#[tauri::command]
pub fn get_console_events(
    state: State<AppState>,
    limit: Option<usize>,
    level: Option<String>,
    request_id: Option<String>,
) -> Result<Vec<crate::console::ConsoleEvent>, AppError> {
    Ok(state.console.get_events(limit, level.as_deref(), request_id.as_deref()))
}

#[tauri::command]
pub fn clear_console_events(state: State<AppState>) -> Result<(), AppError> {
    state.console.clear();
    Ok(())
}

#[tauri::command]
pub fn export_console_events(state: State<AppState>) -> Result<String, AppError> {
    state.console.export_json().map_err(|e| AppError::Storage(e.to_string()))
}

// ---------------------------------------------------------------------------
// Phase 07: Git, GitHub, Project File & Sync Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn export_project_file(
    state: State<AppState>,
    project_id: String,
    include_secrets: Option<bool>,
) -> Result<String, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::project_file::export_project_to_json(&conn, &project_id, include_secrets.unwrap_or(false))
}

#[tauri::command]
pub fn import_project_file(
    state: State<AppState>,
    file_content: String,
    target_project_id: Option<String>,
) -> Result<crate::models::Project, AppError> {
    let mut conn = state.db.lock().expect("db mutex poisoned");
    crate::project_file::import_project_from_json(&mut conn, &file_content, target_project_id.as_deref())
}

#[tauri::command]
pub fn save_project_to_repo(
    state: State<AppState>,
    project_id: String,
    directory: String,
    include_secrets: Option<bool>,
) -> Result<String, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let path = std::path::Path::new(&directory);
    let saved = crate::git_sync::GitService::export_to_repo(
        &conn,
        &project_id,
        path,
        include_secrets.unwrap_or(false),
    )?;
    Ok(saved.to_string_lossy().to_string())
}

#[tauri::command]
pub fn load_project_from_repo(
    state: State<AppState>,
    directory: String,
    target_project_id: Option<String>,
) -> Result<crate::models::Project, AppError> {
    let mut conn = state.db.lock().expect("db mutex poisoned");
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::import_from_repo(&mut conn, path, target_project_id.as_deref())
}

#[tauri::command]
pub fn get_git_status(directory: String) -> Result<crate::git_sync::GitStatus, AppError> {
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::get_status(path)
}

#[tauri::command]
pub fn git_init_repository(directory: String) -> Result<(), AppError> {
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::init_repo(path)
}

#[tauri::command]
pub fn git_commit_changes(directory: String, message: String) -> Result<String, AppError> {
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::stage_all(path)?;
    crate::git_sync::GitService::commit(path, &message)
}

#[tauri::command]
pub fn git_get_diff(directory: String) -> Result<String, AppError> {
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::diff(path)
}

#[tauri::command]
pub fn git_get_log(
    directory: String,
    limit: Option<usize>,
) -> Result<Vec<crate::git_sync::GitCommit>, AppError> {
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::log(path, limit.unwrap_or(20))
}

#[tauri::command]
pub fn git_pull_repository(
    directory: String,
    remote: Option<String>,
    branch: Option<String>,
) -> Result<String, AppError> {
    let path = std::path::Path::new(&directory);
    let r = remote.as_deref().unwrap_or("origin");
    let b = branch.as_deref().unwrap_or("main");
    crate::git_sync::GitService::pull(path, r, b)
}

#[tauri::command]
pub fn git_push_repository(
    directory: String,
    remote: Option<String>,
    branch: Option<String>,
) -> Result<String, AppError> {
    let path = std::path::Path::new(&directory);
    let r = remote.as_deref().unwrap_or("origin");
    let b = branch.as_deref().unwrap_or("main");
    crate::git_sync::GitService::push(path, r, b)
}

#[tauri::command]
pub fn git_resolve_conflict(
    directory: String,
    file: String,
    choice: String,
) -> Result<(), AppError> {
    let path = std::path::Path::new(&directory);
    crate::git_sync::GitService::resolve_conflict(path, &file, &choice)
}

#[tauri::command]
pub fn git_get_conflict_versions(
    directory: String,
    file: String,
) -> Result<crate::git_sync::ConflictVersions, AppError> {
    let path = std::path::Path::new(&directory);
    Ok(crate::git_sync::GitService::get_conflict_versions(path, &file))
}

#[tauri::command]
pub fn get_project_git_settings(
    state: State<AppState>,
    project_id: String,
) -> Result<Option<crate::git_sync::ProjectGitSettings>, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::git_sync::get_project_git_settings(&conn, &project_id)
}

#[tauri::command]
pub fn save_project_git_settings(
    state: State<AppState>,
    settings: crate::git_sync::ProjectGitSettings,
) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    crate::git_sync::save_project_git_settings(&conn, &settings)
}

#[tauri::command]
pub async fn verify_github_token(token: String) -> Result<crate::github_auth::GitHubUser, AppError> {
    crate::github_auth::GitHubService::verify_token(&token).await
}

#[tauri::command]
pub async fn get_github_repo_info(
    token: String,
    owner: String,
    repo: String,
) -> Result<crate::github_auth::GitHubRepoInfo, AppError> {
    crate::github_auth::GitHubService::get_repo_info(&token, &owner, &repo).await
}

#[tauri::command]
pub fn run_script_sandbox(
    script: String,
    environment: Option<HashMap<String, String>>,
    variables: Option<HashMap<String, String>>,
    response_status: Option<u16>,
    response_status_text: Option<String>,
    response_headers: Option<Vec<(String, String)>>,
    response_body: Option<String>,
) -> Result<crate::script_engine::ScriptExecutionResult, String> {
    let env = environment.unwrap_or_default();
    let vars = variables.unwrap_or_default();

    if let Some(status) = response_status {
        let status_text = response_status_text.unwrap_or_else(|| "OK".into());
        let headers = response_headers.unwrap_or_default();
        let body = response_body.unwrap_or_default();
        Ok(crate::script_engine::execute_post_request_script(
            &script, &env, &vars, status, &status_text, &headers, &body, 2000,
        ))
    } else {
        Ok(crate::script_engine::execute_pre_request_script(&script, &env, &vars, 2000))
    }
}

#[tauri::command]
pub fn get_system_diagnostics(
    state: State<AppState>,
) -> Result<crate::diagnostics::SystemDiagnostics, AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    let console_count = state.console.get_events(None, None, None).len();
    let ai_configured = state.ai_provider.is_some() || crate::ai::get_ai_settings(&conn)?.is_configured;
    let uptime = state.start_time.elapsed().as_secs();

    crate::diagnostics::collect_system_diagnostics(
        &conn,
        state.db_path.as_deref(),
        console_count,
        ai_configured,
        uptime,
    )
}

#[tauri::command]
pub fn submit_background_job(
    state: State<AppState>,
    name: String,
    priority_level: u8,
) -> Result<String, AppError> {
    let priority = match priority_level {
        2 => crate::background_jobs::JobPriority::High,
        0 => crate::background_jobs::JobPriority::Low,
        _ => crate::background_jobs::JobPriority::Normal,
    };
    let (id, _) = state.job_manager.submit_job(name, priority);
    Ok(id)
}

#[tauri::command]
pub fn list_background_jobs(
    state: State<AppState>,
) -> Result<Vec<crate::background_jobs::JobSummary>, AppError> {
    Ok(state.job_manager.list_jobs())
}

#[tauri::command]
pub fn cancel_background_job(
    state: State<AppState>,
    job_id: String,
) -> Result<bool, AppError> {
    Ok(state.job_manager.cancel_job(&job_id))
}


