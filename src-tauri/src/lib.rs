pub mod commands;
mod error;
mod infra;
mod portfolio;

use commands::*;
use sqlx::SqlitePool;
use tauri::Manager;

use crate::infra::persistence::setup::{init_db, resolve_db_url};

pub struct AppState {
    pub db: SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_url = resolve_db_url(app.handle())?;
            let db = tauri::async_runtime::block_on(init_db(&db_url))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            app.manage(AppState { db });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_file_dialog,
            process_csv_report,
            suggest_rebalance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
