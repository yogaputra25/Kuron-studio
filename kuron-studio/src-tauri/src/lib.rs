pub mod bubble;
pub mod cache;
pub mod secrets;
pub mod detect_decode;
pub mod detector;
pub mod glossary;
pub mod image_ops;
pub mod parser;
pub mod prompt;
pub mod provider;
pub mod translation;
pub mod commands;

use std::sync::Mutex;
use tauri::Manager;

use commands::batch::{retry_bubble, translate_batch};
use commands::detect::{
    DetectState, detect_bubbles, detect_bubbles_batch, detect_status, save_bubbles,
};
use commands::export::export_project;
use commands::glossary::{
    glossary_add, glossary_context, glossary_delete, glossary_export_csv, glossary_import_csv,
    glossary_list, glossary_update,
};
use commands::image::get_image_preview;
use commands::project::{
    ProjectStore, create_project, get_project, import_pages, list_projects,
};
use commands::provider::{delete_provider, get_providers, list_models, save_provider, validate_provider};
use commands::translate::{clear_cache, save_translation, translate_page};

pub struct AppState {
    pub store: Mutex<ProjectStore>,
    pub detect: DetectState,
    pub http: reqwest::Client,
    pub db: Mutex<rusqlite::Connection>,
}

// ponytail: M0 holds projects only. BubbleDetector (M1), reqwest client (M2),
// rusqlite cache+glossary (M2/M3) join AppState with their tasks.
// ponytail: secrets (M4-1) live in OS keychain via `secrets`; sqlite keeps '' + fallback.
fn io_err<E: ToString>(e: E) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .map_err(|e| io_err(e.to_string()))?
                .join("kuron-studio");
            let store = ProjectStore::load(dir.clone()).map_err(io_err)?;
            let db_path = dir.join("kuron-studio.db");
            let db = rusqlite::Connection::open(&db_path).map_err(io_err)?;
            cache::init_db(&db).map_err(io_err)?;
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(90))
                .build()
                .map_err(|e| io_err(e.to_string()))?;
            app.manage(AppState {
                store: Mutex::new(store),
                detect: DetectState::new(app.path().resource_dir().ok(), app.path().app_data_dir().map_err(|e| io_err(e.to_string()))?),
                http,
                db: Mutex::new(db),
            });
            // M4-7: updater (desktop only); endpoints/pubkey in tauri.conf.json.
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build()).map_err(io_err)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            get_project,
            import_pages,
            get_image_preview,
            detect_status,
            detect_bubbles,
            detect_bubbles_batch,
            save_bubbles,
            save_provider,
            get_providers,
            delete_provider,
            list_models,
            validate_provider,
            translate_page,
            translate_batch,
            retry_bubble,
            glossary_list,
            glossary_add,
            glossary_update,
            glossary_delete,
            glossary_import_csv,
            glossary_export_csv,
            glossary_context,
            export_project,
            save_translation,
            clear_cache
        ])
        .run(tauri::generate_context!())
        .expect("kuron-studio failed to run");
}
