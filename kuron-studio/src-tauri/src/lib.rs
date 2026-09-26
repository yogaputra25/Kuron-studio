pub mod bubble;
mod commands;

use std::sync::Mutex;
use tauri::Manager;

use commands::image::get_image_preview;
use commands::project::{
    ProjectStore, create_project, get_project, import_pages, list_projects,
};

pub struct AppState {
    pub store: Mutex<ProjectStore>,
}

// ponytail: M0 holds projects only. BubbleDetector (M1), reqwest client (M2),
// rusqlite cache+glossary (M2/M3) join AppState with their tasks.
fn io_err(msg: String) -> std::io::Error {
    std::io::Error::other(msg)
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
            let store = ProjectStore::load(dir).map_err(io_err)?;
            app.manage(AppState {
                store: Mutex::new(store),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            get_project,
            import_pages,
            get_image_preview
        ])
        .run(tauri::generate_context!())
        .expect("kuron-studio failed to run");
}
