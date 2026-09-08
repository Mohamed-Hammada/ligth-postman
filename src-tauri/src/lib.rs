mod commands;
mod db;
mod error;
mod models;
mod store;

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

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::create_request,
            commands::list_requests,
            commands::get_request,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
