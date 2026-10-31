//! Integration test `commands/extras.rs` (issue #62): `read_log`,
//! `share_project`, `diagnostics`, `tm_search`, `qa_check` lewat mock app IPC.

mod common;

use std::fs::File;
use std::io::Read;
use serde_json::json;

#[test]
fn diagnostics_memuat_data_dir_tanpa_secret() {
    let (app, dir) = common::test_app();

    let val = common::invoke(&app, "diagnostics", json!({})).expect("diagnostics");
    let obj = val.as_object().expect("objek diagnostics");

    assert!(obj.get("app_version").is_some());
    assert!(obj.get("os").is_some());
    let data_dir = obj.get("data_dir").and_then(|v| v.as_str()).expect("data_dir ada");
    assert_eq!(data_dir, dir.0.to_string_lossy());

    let raw = val.to_string();
    assert!(!raw.to_lowercase().contains("bearer"), "tidak boleh bocor secret");
    assert!(!raw.to_lowercase().contains("api_key"), "tidak boleh bocor secret");
}

#[test]
fn read_log_hilang_dan_ekor() {
    let (app, dir) = common::test_app_tagged("read-log");

    // 1. File log belum dibuat -> kembalikan string kosong tanpa error
    let out = common::invoke(&app, "read_log", json!({"lines": null})).expect("log kosong");
    assert_eq!(out.as_str().expect("string"), "");

    // 2. Tulis 5 baris log
    let log_path = dir.join("kuron-studio.log");
    let lines = ["baris-1", "baris-2", "baris-3", "baris-4", "baris-5"];
    std::fs::write(&log_path, lines.join("\n")).expect("tulis log");

    // 3. Minta 3 baris terakhir
    let tail = common::invoke(&app, "read_log", json!({"lines": 3})).expect("baca tail 3");
    let text = tail.as_str().expect("string text");
    assert_eq!(text, "baris-3\nbaris-4\nbaris-5");

    // 4. Minta semua tanpa argumen lines
    let all = common::invoke(&app, "read_log", json!({"lines": null})).expect("baca all");
    assert_eq!(all.as_str().expect("string all"), lines.join("\n"));
}

#[test]
fn share_project_zip_berisi_manifest_dan_gambar() {
    let (app, dir) = common::test_app_tagged("share-proj");

    // 1. Setup: 1 project + 1 halaman PNG
    let p = common::invoke(&app, "create_project", json!({"name": "ShareMe"})).expect("buat project");
    let pid = p["id"].as_str().unwrap().to_string();

    let src_img = dir.join("orig.png");
    common::write_png(&src_img, 64, 64);
    let imported = common::invoke(
        &app,
        "import_pages",
        json!({"project_id": pid, "paths": [src_img.to_string_lossy()], "folder_id": null}),
    )
    .expect("impor halaman");
    assert_eq!(imported["pages"].as_array().unwrap().len(), 1);

    // 2. Error path: project tidak ada
    let out_zip = dir.join("export.zip");
    let err = common::invoke(
        &app,
        "share_project",
        json!({"project_id": "tidak-ada", "path": out_zip.to_string_lossy()}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("project not found"), "err: {err}");

    // 3. Happy path: share_project membuat zip
    let out_path = common::invoke(
        &app,
        "share_project",
        json!({"project_id": pid, "path": out_zip.to_string_lossy()}),
    )
    .expect("share sukses");
    assert_eq!(out_path.as_str().unwrap(), out_zip.to_string_lossy());
    assert!(out_zip.exists(), "file zip harus tercipta di disk");

    // 4. Periksa isi ZIP
    let file = File::open(&out_zip).expect("buka file zip");
    let mut archive = zip::ZipArchive::new(file).expect("baca zip archive");

    // Harus ada project.json
    let mut manifest_entry = archive.by_name("project.json").expect("project.json ada di zip");
    let mut manifest_str = String::new();
    manifest_entry.read_to_string(&mut manifest_str).expect("baca manifest");
    let manifest_val: serde_json::Value = serde_json::from_str(&manifest_str).expect("json valid");
    assert_eq!(manifest_val["id"], json!(pid));
    assert_eq!(manifest_val["name"], json!("ShareMe"));
    drop(manifest_entry);

    // Harus ada gambar halaman di folder pages/
    let mut img_entry = archive.by_name("pages/001.png").expect("pages/001.png ada di zip");
    let mut img_bytes = Vec::new();
    img_entry.read_to_end(&mut img_bytes).expect("baca gambar");
    assert!(!img_bytes.is_empty(), "gambar di dalam zip tidak boleh kosong");
}

#[test]
fn qa_check_dan_tm_search_smoke() {
    let (app, dir) = common::test_app_tagged("qa-tm");
    let p = common::invoke(&app, "create_project", json!({"name": "QATest"})).expect("buat project");
    let pid = p["id"].as_str().unwrap().to_string();

    let img = dir.join("qa.png");
    common::write_png(&img, 50, 50);
    common::invoke(
        &app,
        "import_pages",
        json!({"project_id": pid, "paths": [img.to_string_lossy()], "folder_id": null}),
    )
    .expect("impor");

    // qa_check pada project yang ada
    let qa = common::invoke(&app, "qa_check", json!({"project_id": pid})).expect("qa_check");
    assert!(qa.is_array());

    // tm_search query
    let tm = common::invoke(&app, "tm_search", json!({"query": "halo", "limit": 5})).expect("tm_search");
    assert!(tm.is_array());
}

/// `validate_provider` via mock HTTP bersama: probe `POST /chat/completions`
/// ke mock server lolos (ok=true). Gagal-aman: provider tak dikenal jadi Err,
/// bukan panic. (Dari target ini untuk memakai helper mock di semua target
/// issue #62; validasi adalah pre-flight sebelum translate batch project.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn validate_provider_mock_ok() {
    let (app, _dir) = common::test_app_tagged("validate-mock");
    let mock_base = common::spawn_mock("{}", 2);

    let prov = common::invoke(
        &app,
        "save_provider",
        json!({"input": {
            "providerType": "custom", "name": "Mock Validate",
            "baseUrl": mock_base, "apiKey": "k", "model": "mock-gpt-4o",
        }}),
    )
    .expect("simpan provider mock");
    let prov_id = prov["id"].as_str().unwrap().to_string();

    let out = common::invoke(&app, "validate_provider", json!({"provider_id": prov_id}))
        .expect("validate_provider terpanggil");
    assert_eq!(out["ok"], json!(true));
    assert_eq!(out["message"], json!("ok"));

    // Provider tak dikenal ditolak (Err, bukan ok=false).
    let err = common::invoke(&app, "validate_provider", json!({"provider_id": "tidak-ada"}))
        .unwrap_err();
    assert!(err.as_str().unwrap().contains("not found"), "err: {err}");
}
