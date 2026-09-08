use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::error::AppError;
use crate::models::{NewProjectInput, NewRequestInput, Project, RequestFull, RequestSummary};
use crate::store::{project_store, request_store};

pub struct AppState {
    pub db: Mutex<Connection>,
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
