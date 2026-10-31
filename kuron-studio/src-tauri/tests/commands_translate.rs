//! Integration test command provider + translate end-to-end (issue #62).
//! Provider `custom` menunjuk ke mock HTTP localhost (pola `tests/translate_mock.rs`).
//! Semua interaksi lewat command nyata via mock app IPC.

mod common;

use serde_json::json;

/// Respons canned OpenAI-compatible untuk 1 bubble mosaic.
const MOSAIC_1: &str = r#"{"choices":[{"message":{"role":"assistant","content":"{\"1\":{\"original\":\"konnichiwa\",\"reading\":\"\",\"translated\":\"Halo Dunia\"}}"}}]}"#;

fn fn_obj(v: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
    v.as_object().expect("respons harus objek JSON")
}

/// Setup helper: buat project + 1 halaman PNG + 1 bubble via command nyata.
fn setup_page(
    app: &tauri::AppHandle<tauri::test::MockRuntime>,
    dir: &std::path::Path,
    name: &str,
) -> (String, String) {
    let p = common::invoke(app, "create_project", json!({"name": name})).expect("buat project");
    let pid = fn_obj(&p)["id"].as_str().unwrap().to_string();

    let img_path = dir.join(format!("{name}.png"));
    common::write_png(&img_path, 100, 100);

    let imported = common::invoke(
        app,
        "import_pages",
        json!({"project_id": pid, "paths": [img_path.to_string_lossy()], "folder_id": null}),
    )
    .expect("impor halaman");
    let page_id = imported["pages"].as_array().unwrap()[0]["id"].as_str().unwrap().to_string();

    common::invoke(
        app,
        "save_bubbles",
        json!({
            "page_id": page_id,
            "bubbles": [{"x": 10, "y": 10, "w": 50, "h": 50, "confidence": 1.0}]
        }),
    )
    .expect("simpan bubble");

    (pid, page_id)
}

#[test]
fn provider_crud_dan_validasi() {
    let (app, _dir) = common::test_app_tagged("prov-crud");

    // 1. Awal kosong
    let list = common::invoke(&app, "get_providers", json!({})).expect("list awal");
    assert_eq!(list.as_array().unwrap().len(), 0);

    // 2. Simpan provider custom
    let saved = common::invoke(
        &app,
        "save_provider",
        json!({
            "input": {
                "providerType": "custom",
                "name": "Local LLM",
                "baseUrl": "http://127.0.0.1:1234/v1",
                "apiKey": "fake-key-test",
                "model": "local-model-v1"
            }
        }),
    )
    .expect("simpan provider");
    let prov_id = saved["id"].as_str().unwrap().to_string();
    assert_eq!(saved["name"], json!("Local LLM"));
    assert_eq!(saved["model"], json!("local-model-v1"));

    // 3. get_providers memuat provider yang tersimpan
    let list2 = common::invoke(&app, "get_providers", json!({})).expect("list isi");
    assert_eq!(list2.as_array().unwrap().len(), 1);
    assert_eq!(list2.as_array().unwrap()[0]["id"], json!(prov_id));

    // 4. delete_provider
    common::invoke(&app, "delete_provider", json!({"provider_id": prov_id})).expect("hapus provider");
    let list3 = common::invoke(&app, "get_providers", json!({})).expect("list setelah hapus");
    assert_eq!(list3.as_array().unwrap().len(), 0);

    // 5. Provider type tidak dikenal ditolak
    let err = common::invoke(
        &app,
        "save_provider",
        json!({
            "input": {
                "providerType": "unknown_ai",
                "name": "Bad",
                "baseUrl": null,
                "apiKey": null,
                "model": null
            }
        }),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("unknown provider"), "err: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn translate_page_mock_sampai_translated() {
    let (app, dir) = common::test_app_tagged("trans-page");
    let (pid, page_id) = setup_page(&app, &dir, "halaman_translate");

    // 1. Jalankan mock HTTP server std TcpListener
    let mock_base = common::spawn_mock(MOSAIC_1, 2);

    // 2. Daftarkan provider custom yang menunjuk ke mock server
    let prov = common::invoke(
        &app,
        "save_provider",
        json!({
            "input": {
                "providerType": "custom",
                "name": "Mock AI",
                "baseUrl": mock_base,
                "apiKey": "test-key-mock",
                "model": "mock-gpt-4o"
            }
        }),
    )
    .expect("daftar mock provider");
    let prov_id = prov["id"].as_str().unwrap().to_string();

    // 3. Panggil command nyata `translate_page`
    let tr_res = common::invoke(
        &app,
        "translate_page",
        json!({
            "input": {
                "pageId": page_id,
                "providerId": prov_id,
                "targetLang": "Indonesian",
                "style": "standard",
                "skipSfx": false,
                "mosaicQuality": "high",
                "readingDirection": "rtl",
                "glossary": null,
                "customPrompt": null,
                "model": null
            }
        }),
    )
    .expect("translate_page berhasil");

    // 4. Assert hasil kembalian: model tercatat dan teks bubble diterjemahkan
    assert_eq!(tr_res["pageId"], json!(page_id));
    assert_eq!(tr_res["model"], json!("mock-gpt-4o"));
    let bubbles = tr_res["bubbles"].as_array().expect("array bubbles");
    assert_eq!(bubbles.len(), 1);
    assert_eq!(bubbles[0]["translated"], json!("Halo Dunia"));

    // 5. Assert status akhir halaman di project store menjadi "translated".
    // `Project` bersarang (`folders[].pages[]`), jadi kumpulkan dulu.
    let proj = common::invoke(&app, "get_project", json!({"project_id": pid})).expect("get project");
    let pages: Vec<&serde_json::Value> = proj["folders"]
        .as_array()
        .expect("folders")
        .iter()
        .flat_map(|f| f["pages"].as_array().expect("pages").iter())
        .collect();
    let page = pages.iter().find(|p| p["id"] == json!(page_id)).expect("page ketemu");
    assert_eq!(page["status"], json!("translated"), "status akhir harus translated");

    // Translation model juga harus tercatat di metadata halaman project
    assert!(page.get("translation").is_some());
    assert_eq!(page["translation"]["model"], json!("mock-gpt-4o"));
}

#[test]
fn cancel_translate_idempoten() {
    let (app, dir) = common::test_app_tagged("cancel-tr");
    let (_pid, page_id) = setup_page(&app, &dir, "cancel_page");

    // Cancel halaman yang ada (idempoten)
    let res = common::invoke(&app, "cancel_translate", json!({"page_id": page_id}));
    assert!(res.is_ok());

    // Cancel halaman asing tetap Ok (idempoten)
    let res2 = common::invoke(&app, "cancel_translate", json!({"page_id": "tidak-ada"}));
    assert!(res2.is_ok());
}

#[test]
fn clear_cache_smoke() {
    let (app, _dir) = common::test_app();
    let res = common::invoke(&app, "clear_cache", json!({})).expect("clear cache");
    assert!(res.is_number());
}

/// Kontrak dedup batch fase 1 (`translate_batch` memakai `JobRegistry`):
/// halaman yang sama tak boleh diakui dua kali; setelah `release` bisa lagi.
/// Pengganti uji IPC `translate_batch` — command itu mengambil `AppHandle`
/// konkret sehingga tak bisa didaftarkan di mock IPC (`CommandArg` hanya
/// untuk runtime yang sama; preseden: `detect_bubbles_batch`).
#[test]
fn batch_registry_dedup_duplikat_ditolak() {
    use kuron_studio_lib::jobs::JobRegistry;
    let reg = JobRegistry::new(3);
    assert!(reg.admit("p1"), "admit pertama lolos");
    assert!(!reg.admit("p1"), "duplikat dalam batch ditolak (busy)");
    assert!(reg.admit("p2"), "halaman lain tetap lolos");
    reg.release("p1");
    assert!(reg.admit("p1"), "setelah release bisa diakui lagi");
}

/// `retry_bubble` setelah translate_page: bubble 0 diterjemahkan ulang via
/// mock; indeks di luar jangkauan ditolak dengan pesan jelas.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retry_bubble_setelah_translate() {
    let (app, dir) = common::test_app_tagged("retry-bub");
    let (_pid, page_id) = setup_page(&app, &dir, "halaman_retry");

    let mock_base = common::spawn_mock(MOSAIC_1, 4);
    let prov = common::invoke(
        &app,
        "save_provider",
        json!({"input": {
            "providerType": "custom", "name": "Mock Retry",
            "baseUrl": mock_base, "apiKey": "k", "model": "mock-gpt-4o",
        }}),
    )
    .expect("daftar mock provider");
    let prov_id = prov["id"].as_str().unwrap().to_string();

    // Prasyarat retry: halaman sudah punya terjemahan.
    common::invoke(
        &app,
        "translate_page",
        json!({"input": {
            "pageId": page_id, "providerId": prov_id,
            "targetLang": "Indonesian", "style": "standard", "skipSfx": false,
            "mosaicQuality": "high", "readingDirection": "rtl",
            "glossary": null, "customPrompt": null, "model": null,
        }}),
    )
    .expect("translate_page prasyarat");

    // Happy path: retry bubble 0.
    let out = common::invoke(
        &app,
        "retry_bubble",
        json!({"input": {
            "pageId": page_id, "bubbleIndex": 0, "providerId": prov_id,
            "targetLang": "Indonesian", "style": "standard", "skipSfx": false,
            "mosaicQuality": "high", "readingDirection": "rtl",
            "glossary": null, "customPrompt": null, "model": null,
        }}),
    )
    .expect("retry_bubble berhasil");
    assert_eq!(out["pageId"], json!(page_id));
    assert_eq!(out["bubbles"].as_array().unwrap()[0]["translated"], json!("Halo Dunia"));

    // Error path: indeks fiktif.
    let err = common::invoke(
        &app,
        "retry_bubble",
        json!({"input": {
            "pageId": page_id, "bubbleIndex": 99, "providerId": prov_id,
            "targetLang": "Indonesian", "style": "standard", "skipSfx": false,
            "mosaicQuality": "high", "readingDirection": "rtl",
            "glossary": null, "customPrompt": null, "model": null,
        }}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("di luar"), "err: {err}");
}

/// Edit manual + tandai-selesai lewat command nyata: `save_translation`
/// menyimpan teks manual, `mark_page_completed` membalik flag `manualDone`.
#[test]
fn save_translation_dan_mark_completed() {
    let (app, dir) = common::test_app_tagged("save-mark");
    let (_pid, page_id) = setup_page(&app, &dir, "halaman_manual");

    let saved = common::invoke(
        &app,
        "save_translation",
        json!({
            "page_id": page_id,
            "bubbles": [{
                "index": 0, "x": 10, "y": 10, "w": 50, "h": 50,
                "original": "konnichiwa", "reading": "", "translated": "Halo manual",
            }],
        }),
    )
    .expect("save_translation berhasil");
    assert_eq!(saved["pageId"], json!(page_id));
    assert_eq!(
        saved["bubbles"].as_array().unwrap()[0]["translated"],
        json!("Halo manual")
    );

    let done = common::invoke(
        &app,
        "mark_page_completed",
        json!({"page_id": page_id, "completed": true}),
    )
    .expect("tandai selesai");
    assert_eq!(done["manualDone"], json!(true));

    let undone = common::invoke(
        &app,
        "mark_page_completed",
        json!({"page_id": page_id, "completed": false}),
    )
    .expect("cabut tanda selesai");
    assert_eq!(undone["manualDone"], json!(false));

    // Halaman tak dikenal ditolak.
    let err = common::invoke(
        &app,
        "save_translation",
        json!({"page_id": "tidak-ada", "bubbles": []}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("not found"), "err: {err}");
}
