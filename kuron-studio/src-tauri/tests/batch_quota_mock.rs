//! Acceptance hemat-kuota batch-quota (issue #67) lewat mock IPC:
//! tepat-1-POST saat parse gagal, 0-POST untuk halaman selesai.
//! Mock HTTP menghitung request masuk (`spawn_mock_counted`), tanpa wiremock.

mod common;

use serde_json::json;
use std::sync::atomic::Ordering;

/// Respons canned OpenAI-compatible untuk 1 bubble mosaic.
const MOSAIC_1: &str = r#"{"choices":[{"message":{"role":"assistant","content":"{\"1\":{\"original\":\"konnichiwa\",\"reading\":\"\",\"translated\":\"Halo Dunia\"}}"}}]}"#;

/// Respons 200 berisi sampah tak-terbaca (bukan JSON).
const GARBAGE: &str = r#"{"choices":[{"message":{"role":"assistant","content":"ini bukan json {{{ terpotong"}}}]"#;

/// Mock HTTP ala `common::spawn_mock`, ditambah counter request POST masuk
/// (bukan hanya melayani N koneksi). `hits` = batas atas soket yang dilayani;
/// counter = jumlah POST utuh yang benar-benar tiba.
fn spawn_mock_counted(
    body: &'static str,
    hits: usize,
) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    use std::io::{Read, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock");
    let addr = l.local_addr().expect("addr mock").to_string();
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let c = std::sync::Arc::clone(&count);
    std::thread::spawn(move || {
        for stream in l.incoming().take(hits) {
            let mut s = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut buf = [0u8; 65536];
            let n = s.read(&mut buf).unwrap_or(0);
            // Hanya hitung request POST utuh (abaikan probe/read kosong).
            if n > 0 && buf[..n].starts_with(b"POST") {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });
    (format!("http://{addr}"), count)
}

fn setup_page(
    app: &tauri::AppHandle<tauri::test::MockRuntime>,
    dir: &std::path::Path,
    name: &str,
) -> String {
    let p = common::invoke(app, "create_project", json!({"name": name})).expect("buat project");
    let pid = p["id"].as_str().unwrap().to_string();
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
    page_id
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mock_dasar_tetap_berfungsi() {
    // Penjaga: helper mock polos tetap hidup (dipakai target ini).
    let (app, _dir) = common::test_app();
    let base = common::spawn_mock("{}", 1);
    let prov = common::invoke(
        &app,
        "save_provider",
        json!({
            "input": {
                "providerType": "custom",
                "name": "Mock Dasar",
                "baseUrl": base,
                "apiKey": "k",
                "model": "m"
            }
        }),
    )
    .expect("daftar provider");
    assert!(prov["id"].as_str().is_some());
}

fn setup_provider(
    app: &tauri::AppHandle<tauri::test::MockRuntime>,
    base: &str,
) -> String {
    let prov = common::invoke(
        app,
        "save_provider",
        json!({
            "input": {
                "providerType": "custom",
                "name": "Mock Kuota",
                "baseUrl": base,
                "apiKey": "k",
                "model": "mock-gpt-4o"
            }
        }),
    )
    .expect("daftar mock provider");
    prov["id"].as_str().unwrap().to_string()
}

fn translate_input(page_id: &str, prov_id: &str) -> serde_json::Value {
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
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn respons_valid_hanya_satu_post() {
    let (app, dir) = common::test_app_tagged("kuota-1post");
    let page_id = setup_page(&app, &dir, "satu_post");
    let (mock_base, hits) = spawn_mock_counted(MOSAIC_1, 4);
    let prov_id = setup_provider(&app, &mock_base);

    let tr = common::invoke(&app, "translate_page", translate_input(&page_id, &prov_id))
        .expect("translate berhasil");
    assert_eq!(tr["bubbles"][0]["translated"], json!("Halo Dunia"));
    assert_eq!(hits.load(Ordering::SeqCst), 1, "respons valid = tepat 1 POST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn respons_rusak_tetap_satu_post_dan_gagal_jelas() {
    let (app, dir) = common::test_app_tagged("kuota-gagal");
    let page_id = setup_page(&app, &dir, "sampah_post");
    let (mock_base, hits) = spawn_mock_counted(GARBAGE, 4);
    let prov_id = setup_provider(&app, &mock_base);

    let err = common::invoke(&app, "translate_page", translate_input(&page_id, &prov_id))
        .expect_err("respons rusak harus gagal");
    let msg = err.as_str().unwrap();
    assert!(msg.contains("parse"), "pesan menyebut parse gagal: {msg}");
    assert!(msg.contains("Retry manual"), "pesan menganjurkan Retry manual: {msg}");
    assert_eq!(hits.load(Ordering::SeqCst), 1, "tanpa POST kedua otomatis");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn halaman_selesai_nol_post_tanpa_force() {
    let (app, dir) = common::test_app_tagged("kuota-skip");
    let page_id = setup_page(&app, &dir, "lewati_selesai");
    let (mock_base, hits) = spawn_mock_counted(MOSAIC_1, 4);
    let prov_id = setup_provider(&app, &mock_base);

    // POST pertama: menerjemahkan.
    let tr = common::invoke(&app, "translate_page", translate_input(&page_id, &prov_id))
        .expect("translate pertama berhasil");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    // Panggilan non-paksa kedua: halaman sudah lengkap → 0 request baru.
    let tr2 = common::invoke(&app, "translate_page", translate_input(&page_id, &prov_id))
        .expect("halaman selesai dikembalikan tanpa POST");
    assert_eq!(hits.load(Ordering::SeqCst), 1, "halaman selesai = 0 POST tambahan");
    assert_eq!(tr2["bubbles"][0]["translated"], tr["bubbles"][0]["translated"]);
}
