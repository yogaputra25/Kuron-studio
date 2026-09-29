pub mod bubble;
pub mod cache;
pub mod memory;
pub mod psd;
pub mod qa;
pub mod secrets;
pub mod detect_decode;
pub mod detector;
pub mod glossary;
pub mod image_ops;
pub mod logging;
pub mod parser;
pub mod prompt;
pub mod provider;
pub mod translation;
pub mod commands;

use std::collections::{HashMap, HashSet};
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
use commands::project::{
    PageStatus, ProjectStore, clean_pages, create_project, delete_project, get_project,
    import_pages, list_projects,
};
use commands::provider::{
    delete_provider, get_providers, list_models, list_models_draft, save_provider, validate_provider,
};
use commands::translate::{cancel_translate, clear_cache, save_translation, translate_page};

pub struct AppState {
    pub store: Mutex<ProjectStore>,
    pub detect: DetectState,
    pub http: reqwest::Client,
    pub db: Mutex<rusqlite::Connection>,
    /// Halaman yang diminta batal (opsi B cancel): diisi `cancel_translate`,
    /// dibaca `backoff_translate` tiap iterasi + saat sleep via `select!`.
    pub cancel: Mutex<HashSet<String>>,
    /// Status sebelum `claim_page` (Translating menimpa); path cancel restore
    /// ini via helper, bukan `fail_page`.
    pub prev_status: Mutex<HashMap<String, PageStatus>>,
    /// App data dir (`…/kuron-studio`). Disimpan di state supaya command
    /// bisa menemukan projects.json + log file tanpa menebak path.
    pub data_dir: std::path::PathBuf,
}

impl AppState {
    pub fn stash_prev(&self, page_id: &str, st: PageStatus) {
        if let Ok(mut m) = self.prev_status.lock() {
            m.insert(page_id.to_string(), st);
        }
    }
    pub fn take_prev(&self, page_id: &str) -> Option<PageStatus> {
        self.prev_status.lock().ok()?.remove(page_id)
    }
    pub fn clear_cancel(&self, page_id: &str) {
        if let Ok(mut c) = self.cancel.lock() {
            c.remove(page_id);
        }
    }
    pub fn request_cancel(&self, page_id: &str) {
        if let Ok(mut c) = self.cancel.lock() {
            c.insert(page_id.to_string());
        }
    }
    pub fn is_cancelled(&self, page_id: &str) -> bool {
        self.cancel.lock().map(|c| c.contains(page_id)).unwrap_or(false)
    }
    pub fn data_dir(&self) -> String {
        self.data_dir.to_string_lossy().to_string()
    }

    pub fn data_dir_path(&self) -> &std::path::Path {
        &self.data_dir
    }
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
                cancel: Mutex::new(HashSet::new()),
                prev_status: Mutex::new(HashMap::new()),
                data_dir: dir,
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
            clean_pages,
            delete_project,
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
            cancel_translate,
            clear_cache,
            diagnostics,
            read_log
        ])
        .run(tauri::generate_context!())
        .expect("kuron-studio failed to run");
}
