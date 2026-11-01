pub mod bubble;
pub mod cache;
pub mod db;
pub mod error;
pub mod memory;
pub mod qa;
pub mod secrets;
pub mod store;
pub mod domain;
pub mod glossary;
pub mod import;
pub mod jobs;
pub mod logging;
#[cfg(desktop)]
pub mod menu;
pub mod preview_cache;
pub mod parser;
pub mod prompt;
pub mod provider;
pub mod render;
pub mod translation;
pub mod translate;
pub mod commands;

pub use error::AppError;
use std::sync::Mutex;
use tauri::Manager;

use commands::batch::{retry_bubble, translate_batch};
use commands::detect::{
    DetectState, detect_bubbles, detect_bubbles_batch, detect_status, save_bubbles,
};
use commands::export::export_project;
use commands::extras::{diagnostics, qa_check, read_log, share_project, tm_search};
use commands::glossary::{
    glossary_add, glossary_context, glossary_delete, glossary_export_csv, glossary_import_csv,
    glossary_list, glossary_update,
};
use commands::image::get_image_preview;
use crate::import::import_pages;
use crate::store::ProjectStore;
use commands::project::{
    clean_pages, create_folder, create_project, create_sample_project,
    delete_folder, delete_project, get_project, list_projects, move_pages, rename_folder,
};
use commands::provider::{
    delete_provider, get_providers, list_models, list_models_draft, save_provider, validate_provider,
};
use commands::translate::{cancel_translate, clear_cache, mark_page_completed, save_translation, translate_page};

pub struct AppState {
    pub store: Mutex<ProjectStore>,
    pub detect: DetectState,
    pub http: reqwest::Client,
    pub db: Mutex<rusqlite::Connection>,
    /// Cache preview gambar sesi (grid-preview-perf D4): tier thumb + full.
    pub previews: Mutex<preview_cache::PreviewCaches>,
    /// App data dir (`…/kuron-studio`). Disimpan di state supaya command
    /// bisa menemukan projects.json + log file tanpa menebak path.
    pub data_dir: std::path::PathBuf,
}

impl AppState {
    pub fn data_dir(&self) -> String {
        self.data_dir.to_string_lossy().to_string()
    }

    pub fn data_dir_path(&self) -> &std::path::Path {
        &self.data_dir
    }
}

// ponytail: M0 holds projects only. BubbleDetector (M1), reqwest client (M2),
// rusqlite cache+glossary (M2/M3) join AppState with their tasks.
// ponytail: secrets file-encrypted (AES-GCM, DEK keys/master.key) via `secrets`;
// sqlite menyimpan ciphertext `enc1:`. Keychain hanya dibaca saat migrasi.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use error::AppError;
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .map_err(AppError::store)?
                .join("kuron-studio");
            let store = ProjectStore::load(dir.clone())?;
            // Log JSON Lines: stderr (untuk `pnpm tauri dev`) + file persisten
            // di app data dir, supaya error yang terjadi belakangan masih bisa
            // dibaca tanpa terminal yang sedang terbuka.
            if let Some(log_path) = logging::init(&dir) {
                logging::info(
                    "app_start",
                    serde_json::json!({
                        "version": env!("CARGO_PKG_VERSION"),
                        "log_file": log_path.display().to_string(),
                    }),
                );
            } else {
                logging::warn(
                    "log_init_failed",
                    serde_json::json!({ "note": "stderr only" }),
                );
            }
            let db_path = dir.join("kuron-studio.db");
            let db = rusqlite::Connection::open(&db_path)?;
            cache::init_db(&db)?;
            // Migrasi satu arah keychain → sqlite terenkripsi (best-effort;
            // tidak menggagalkan startup). Setelah ini keychain tak tersentuh.
            let migrated = commands::provider::migrate_provider_keys(&db, &dir);
            if migrated > 0 {
                logging::info(
                    "provider_keys_migrated",
                    serde_json::json!({ "count": migrated }),
                );
            }
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(90))
                .tls_built_in_root_certs(true)
                .build()?;
            app.manage(AppState {
                store: Mutex::new(store),
                detect: DetectState::new(app.path().resource_dir().ok(), app.path().app_data_dir().map_err(AppError::store)?),
                http,
                db: Mutex::new(db),
                previews: Mutex::new(preview_cache::PreviewCaches::new()),
                data_dir: dir,
            });
            app.manage(translate::CancelState::default());
            // M4-7: updater (desktop only); endpoints/pubkey in tauri.conf.json.
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build()).map_err(AppError::store)?;
            // Menu kustom (desktop): About bawaan macOS tak memberi event,
            // jadi About kita = item kustom -> event `show-about`.
            #[cfg(desktop)]
            app.handle().set_menu(menu::build(app.handle()).map_err(AppError::store)?).map_err(AppError::store)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            create_sample_project,
            get_project,
            import_pages,
            clean_pages,
            delete_project,
            create_folder,
            rename_folder,
            delete_folder,
            move_pages,
            get_image_preview,
            detect_status,
            detect_bubbles,
            detect_bubbles_batch,
            save_bubbles,
            save_provider,
            get_providers,
            delete_provider,
            list_models,
            list_models_draft,
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
            tm_search,
            qa_check,
            share_project,
            save_translation,
            mark_page_completed,
            cancel_translate,
            clear_cache,
            diagnostics,
            read_log
        ]);
        // Menu About kustom -> frontend (AboutPage). Predefined lain
        // ditangani OS, tak perlu listener.
        #[cfg(desktop)]
        let builder = builder.on_menu_event(|app, event| {
            use tauri::Emitter;
            if event.id() == menu::SHOW_ABOUT_ID {
                let _ = app.emit(menu::SHOW_ABOUT_ID, ());
            }
        });
        builder
        .run(tauri::generate_context!())
        .expect("kuron-studio failed to run");
}
