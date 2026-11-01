//! Helper bersama integration test command wrapper (issue #62).
//!
//! Satu-satunya cara membuat `AppState` di integration test: direktori temp
//! unik per pemanggil + skema DB via `db::migrate` + HTTP timeout pendek
//! (gagal cepat, bukan 90 detik seperti produksi).
//!
//! Semua interaksi command lewat mock app IPC via `tauri::test`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use kuron_studio_lib::commands::detect::DetectState;
use kuron_studio_lib::preview_cache::PreviewCaches;
use kuron_studio_lib::store::ProjectStore;
use kuron_studio_lib::AppState;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Direktori temp unik — aman untuk test paralel dalam satu binary
/// (pid + nanos + counter, bukan nanos saja).
pub fn unique_dir(tag: &str) -> PathBuf {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("ks-cmd-{tag}-{}-{n}-{c}", std::process::id()))
}

/// RAII wrapper direktori temp yang otomatis membersihkan saat di-drop.
pub struct TempDir(pub PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for TempDir {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

/// Factory `test_app()` (issue #62 langkah 2): mengembalikan `(AppHandle, TempDir)`
/// dengan `AppState` terisolasi di temp dir baru.
pub fn test_app() -> (tauri::AppHandle<tauri::test::MockRuntime>, TempDir) {
    test_app_tagged("test")
}

/// Factory `test_app_tagged()` dengan tag prefix khusus untuk nama folder temp.
pub fn test_app_tagged(tag: &str) -> (tauri::AppHandle<tauri::test::MockRuntime>, TempDir) {
    let dir = TempDir(unique_dir(tag));
    std::fs::create_dir_all(&dir.0).expect("buat temp dir test");
    let store = ProjectStore::load(dir.0.clone()).expect("load store kosong");
    let db = kuron_studio_lib::db::open(&dir.0.join("test.db")).expect("buka sqlite test");
    kuron_studio_lib::db::migrate(&db).expect("migrasi skema test");
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("buat http client test");
    let state = AppState {
        store: Mutex::new(store),
        detect: DetectState::new(None, dir.0.clone()),
        http,
        db: Mutex::new(db),
        previews: Mutex::new(PreviewCaches::new()),
        data_dir: dir.0.clone(),
    };

    let app = tauri::test::mock_builder()
        .manage(state)
        .manage(kuron_studio_lib::translate::CancelState::default())
        .invoke_handler(tauri::generate_handler![
            // project
            kuron_studio_lib::commands::project::list_projects,
            kuron_studio_lib::commands::project::create_project,
            kuron_studio_lib::commands::project::get_project,
            kuron_studio_lib::commands::project::delete_project,
            kuron_studio_lib::commands::project::create_folder,
            kuron_studio_lib::commands::project::rename_folder,
            kuron_studio_lib::commands::project::delete_folder,
            kuron_studio_lib::commands::project::move_pages,
            kuron_studio_lib::commands::project::clean_pages,
            // import
            kuron_studio_lib::import::import_pages,
            // detect
            kuron_studio_lib::commands::detect::save_bubbles,
            kuron_studio_lib::commands::detect::detect_status,
            kuron_studio_lib::commands::detect::detect_bubbles,
            // extras
            kuron_studio_lib::commands::extras::diagnostics,
            kuron_studio_lib::commands::extras::read_log,
            kuron_studio_lib::commands::extras::tm_search,
            kuron_studio_lib::commands::extras::qa_check,
            kuron_studio_lib::commands::extras::share_project,
            // provider
            kuron_studio_lib::commands::provider::save_provider,
            kuron_studio_lib::commands::provider::get_providers,
            kuron_studio_lib::commands::provider::delete_provider,
            kuron_studio_lib::commands::provider::validate_provider,
            kuron_studio_lib::commands::provider::list_models,
            // translate
            kuron_studio_lib::commands::translate::translate_page,
            kuron_studio_lib::commands::translate::cancel_translate,
            // batch (issue #62): retry_bubble State-only sehingga bisa lewat
            // mock IPC. `translate_batch` butuh `AppHandle` konkret (=Wry)
            // yang tak memenuhi `CommandArg<MockRuntime>` — preseden sama
            // seperti `detect_bubbles_batch` yang memang tak didaftarkan di
            // sini; cakupannya diganti uji registry + retry di bawah.
            kuron_studio_lib::commands::batch::retry_bubble,
            kuron_studio_lib::commands::translate::save_translation,
            kuron_studio_lib::commands::translate::mark_page_completed,
            kuron_studio_lib::commands::translate::clear_cache,
            // glossary
            kuron_studio_lib::commands::glossary::glossary_list,
            kuron_studio_lib::commands::glossary::glossary_add,
            kuron_studio_lib::commands::glossary::glossary_update,
            kuron_studio_lib::commands::glossary::glossary_delete,
            kuron_studio_lib::commands::glossary::glossary_import_csv,
            kuron_studio_lib::commands::glossary::glossary_export_csv,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app harus bisa dibangun");

    let _window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview mock harus bisa dibangun");

    (app.handle().clone(), dir)
}

/// Panggil command nyata lewat IPC mock. `body` = objek argumen top-level snake_case
/// (key persis nama param Rust); struct input dibungkus `{"input": {...}}`
/// dengan field camelCase. Ok → `serde_json::Value`, Err → string error
/// command sebagai `Value`.
pub fn invoke(
    app: &tauri::AppHandle<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    use tauri::Manager;
    let window = app.get_webview_window("main").expect("window main harus ada");
    // Skema `tauri://` = origin lokal (bypass gate ACL untuk command app
    // tanpa prefix plugin saat tak ada app-manifest, seperti mock di sini).
    // Di Windows/Android origin lokal justru `http://tauri.localhost`.
    let url: tauri::Url = if cfg!(any(windows, target_os = "android")) {
        "http://tauri.localhost"
    } else {
        "tauri://localhost"
    }
    .parse()
    .expect("URL invoke valid");
    let req = tauri::webview::InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url,
        body: tauri::ipc::InvokeBody::Json(body),
        headers: Default::default(),
        invoke_key: tauri::test::INVOKE_KEY.to_string(),
    };
    tauri::test::get_ipc_response(&window, req)
        .map(|b| b.deserialize::<serde_json::Value>().expect("respons harus JSON"))
}

/// Tulis PNG valid ke `path` (untuk setup `import_pages`).
pub fn write_png(path: &Path, w: u32, h: u32) {
    let img = image::RgbImage::from_pixel(w, h, image::Rgb([200, 210, 220]));
    image::DynamicImage::ImageRgb8(img)
        .save(path)
        .expect("tulis PNG fixture");
}

/// Mock HTTP ala `tests/translate_mock.rs`: TcpListener std yang menyajikan
/// `body` mentah untuk `hits` koneksi, tanpa wiremock.
pub fn spawn_mock(body: &'static str, hits: usize) -> String {
    use std::io::{Read, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock");
    let addr = l.local_addr().expect("addr mock").to_string();
    std::thread::spawn(move || {
        for stream in l.incoming().take(hits) {
            let mut s = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut buf = [0u8; 65536];
            let _ = s.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });
    format!("http://{addr}")
}
