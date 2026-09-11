pub mod ai;
pub mod background_jobs;
pub mod canonical_request;
pub mod codegen;
pub mod commands;
pub mod console;
pub mod curl_importer;
pub mod db;
pub mod diagnostics;
pub mod error;
pub mod execution;
pub mod git_sync;
pub mod github_auth;
pub mod http_engine;
pub mod models;
pub mod postman_compat;
pub mod project_file;
pub mod resolver;
pub mod script_engine;
pub mod source_analyzer;
pub mod store;

use std::collections::HashMap;
use std::sync::Mutex;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("create app data dir");

            let db_path = data_dir.join("app.db");
            log::info!("opening database at {}", db_path.display());
            let conn = db::open(&db_path).expect("open sqlite database");

            let http_client = reqwest::Client::builder()
                .build()
                .expect("build reqwest client");

            let ai_provider = ai::ClaudeProvider::from_env(http_client.clone());
            log::info!("AI provider configured: {}", ai_provider.is_some());

            let job_manager = std::sync::Arc::new(background_jobs::BackgroundJobManager::default());
            let start_time = std::time::Instant::now();

            app.manage(AppState {
                db: Mutex::new(conn),
                http_client,
                response_body_dir: data_dir.join("response_bodies"),
                cancel_signals: Mutex::new(HashMap::new()),
                ai_provider,
                console: std::sync::Arc::new(console::ConsoleBuffer::default()),
                job_manager,
                start_time,
                db_path: Some(db_path),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::get_project_request_counts,
            commands::update_project,
            commands::delete_project,
            commands::create_workspace,
            commands::list_workspaces,
            commands::update_workspace,
            commands::delete_workspace,
            commands::create_request,
            commands::list_requests,
            commands::get_request,
            commands::update_request,
            commands::delete_request,
            commands::create_folder,
            commands::list_folders,
            commands::update_folder,
            commands::delete_folder,
            commands::create_environment,
            commands::list_environments,
            commands::list_all_environments,
            commands::update_environment,
            commands::delete_environment,
            commands::create_variable,
            commands::list_variables_for_scope,
            commands::update_variable,
            commands::delete_variable,
            commands::reveal_variable_value,
            commands::resolve_preview,
            commands::diagnose_request,
            commands::send_request,
            commands::cancel_send,
            commands::list_response_summaries,
            commands::list_project_history,
            commands::get_response,
            commands::get_response_body,
            commands::delete_response,
            commands::is_ai_configured,
            commands::get_ai_settings,
            commands::save_ai_settings,
            commands::test_ai_connection,
            commands::generate_api_with_ai,
            commands::generate_api_with_project_context,
            commands::generate_sample_response_with_ai,
            commands::generate_tests_and_docs_with_ai,
            commands::scan_source_project,
            commands::get_project_source_directory,
            commands::set_project_source_directory,
            commands::import_discovered_endpoint,
            commands::generate_curl_snippet,
            commands::import_curl,
            commands::create_sample_response,
            commands::list_sample_responses,
            commands::delete_sample_response,
            commands::update_sample_response,
            commands::create_cookie,
            commands::list_cookies_for_project,
            commands::delete_cookie,
            commands::import_postman_collection,
            commands::import_local_postman_workspace,
            commands::import_postman_environment,
            commands::export_postman_collection,
            commands::export_postman_environment,
            commands::get_console_events,
            commands::clear_console_events,
            commands::export_console_events,
            commands::export_project_file,
            commands::import_project_file,
            commands::save_project_to_repo,
            commands::load_project_from_repo,
            commands::get_git_status,
            commands::git_init_repository,
            commands::git_commit_changes,
            commands::git_get_diff,
            commands::git_get_log,
            commands::git_pull_repository,
            commands::git_push_repository,
            commands::git_resolve_conflict,
            commands::git_get_conflict_versions,
            commands::get_project_git_settings,
            commands::save_project_git_settings,
            commands::verify_github_token,
            commands::get_github_repo_info,
            commands::run_script_sandbox,
            commands::get_system_diagnostics,
            commands::submit_background_job,
            commands::list_background_jobs,
            commands::cancel_background_job,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
