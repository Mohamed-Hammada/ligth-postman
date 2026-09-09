use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::ai::{AiProvider, ClaudeProvider, GeneratedApiDefinition};
use crate::error::AppError;
use crate::execution::{self, ExecutionInput};
use crate::models::{
    Environment, NewEnvironmentInput, NewProjectInput, NewRequestInput, NewVariableInput,
    Project, RequestFull, RequestSummary, ResponseBodyPayload, ResponseMeta, ResponseSummary,
    UpdateEnvironmentInput, UpdateProjectInput, UpdateRequestInput, UpdateVariableInput,
    VariableScope, VariableView,
};
use crate::resolver::{self, ScopeChain};
use crate::store::{environment_store, project_store, request_store, response_store, variable_store};

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

/// Lets the frontend show/hide the AI entry point without ever exposing whether (or what)
/// key is configured.
#[tauri::command]
pub fn is_ai_configured(state: State<AppState>) -> bool {
    state.ai_provider.is_some()
}

/// Prompt -> structured definition -> validated (LP-0805). Nothing is persisted here — the
/// frontend previews the result and only calls `create_request` if the user explicitly
/// approves it ("Add to Project"), reusing the exact same validation every manual create goes through.
#[tauri::command]
pub async fn generate_api_with_ai(
    state: State<'_, AppState>,
    prompt: String,
) -> Result<GeneratedApiDefinition, AppError> {
    let provider = state.ai_provider.as_ref().ok_or_else(|| {
        AppError::Validation("AI is not configured — set the ANTHROPIC_API_KEY environment variable".into())
    })?;
    let definition = provider.generate_api(&prompt).await?;
    log::info!("AI generated an API definition: {}", definition.name);
    Ok(definition)
}

/// LP-0605 (snippet UI backend). Builds the request's own scope chain the same way
/// `resolve_preview`/`send_request` do, then hands it to `codegen::generate_snippet`.
#[tauri::command]
pub fn generate_curl_snippet(
    state: State<AppState>,
    request_id: String,
    environment_id: Option<String>,
    mode: crate::codegen::SnippetMode,
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
    crate::codegen::generate_snippet(&request, &chain, mode)
}
