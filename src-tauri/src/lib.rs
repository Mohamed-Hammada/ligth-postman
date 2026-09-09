mod ai;
mod canonical_request;
mod codegen;
mod commands;
mod db;
mod error;
mod execution;
mod http_engine;
mod models;
mod resolver;
mod store;

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

            app.manage(AppState {
                db: Mutex::new(conn),
                http_client,
                response_body_dir: data_dir.join("response_bodies"),
                cancel_signals: Mutex::new(HashMap::new()),
                ai_provider,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::update_project,
            commands::delete_project,
            commands::create_request,
            commands::list_requests,
            commands::get_request,
            commands::update_request,
            commands::delete_request,
            commands::create_environment,
            commands::list_environments,
            commands::update_environment,
            commands::delete_environment,
            commands::create_variable,
            commands::list_variables_for_scope,
            commands::update_variable,
            commands::delete_variable,
            commands::reveal_variable_value,
            commands::resolve_preview,
            commands::send_request,
            commands::cancel_send,
            commands::list_response_summaries,
            commands::get_response,
            commands::get_response_body,
            commands::delete_response,
            commands::is_ai_configured,
            commands::generate_api_with_ai,
            commands::generate_curl_snippet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
