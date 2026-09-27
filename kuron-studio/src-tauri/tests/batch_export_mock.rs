//! M3-12: batch 10 halaman (mock LLM) → progress → export JSON/PNG/CBZ.
//! Pola sama seperti translate_mock.rs: std TcpListener, tanpa wiremock.

use std::io::{Read, Write};
use std::net::TcpListener;

use image::{Rgb, RgbImage};
use kuron_studio_lib::commands::export::{export_cbz_bytes, export_json_bytes, render_page_png};
use kuron_studio_lib::translation::{order_indices, PageTranslation};

fn spawn_mock(body: &'static str, hits: usize) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap().to_string();
    std::thread::spawn(move || {
        for stream in l.incoming().take(hits) {
            let mut s = stream.unwrap();
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

fn page_png() -> Vec<u8> {
    let img = RgbImage::from_pixel(400, 600, Rgb([210, 210, 210]));
    let mut buf = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut buf);
    use image::ImageEncoder;
    enc.write_image(img.as_raw(), 400, 600, image::ExtendedColorType::Rgb8)
        .unwrap();
    buf
}

/// Semaphore-3 runner generik: tiru pola commands::batch fase 2/3
/// (mpsc + Semaphore 3, progress done=1..N) di atas mock translate fn.
async fn run_batch_10() -> (Vec<PageTranslation>, Vec<(String, usize, bool)>) {
    use std::sync::{Arc, Mutex};
    let canned = r#"{"choices":[{"message":{"role":"assistant","content":"{\"1\":{\"original\":\"a\",\"reading\":\"\",\"translated\":\"A\"}}"}}]}"#;
    // 10 halaman x (1 POST + cadangan 1 retry) = 20 slot.
    let base = spawn_mock(canned, 20);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let sem = Arc::new(tokio::sync::Semaphore::new(3));
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let progress: Arc<Mutex<Vec<(String, usize, bool)>>> = Arc::new(Mutex::new(Vec::new()));

    for i in 0..10usize {
        let tx = tx.clone();
        let sem = sem.clone();
        let (client, base) = (client.clone(), base.clone());
        tokio::spawn(async move {
            let _p = sem.acquire_owned().await.unwrap();
            let page_id = format!("p{i}");
            let out = kuron_studio_lib::provider::openai::chat_completions_image(
                &client, &base, "k", "m", "prompt", b"fake-jpeg",
            )
            .await
            .and_then(|text| {
                let b = kuron_studio_lib::bubble::BubbleBox {
                    x: 10, y: 20, w: 60, h: 40, confidence: 1.0,
                    shape: None, kind: None, tail: None,
                };
                let ordered = order_indices(std::slice::from_ref(&b), "rtl");
                let results = kuron_studio_lib::parser::parse_mosaic_json(&text)
                    .map_err(|e| e.to_string())?;
                let mapped = kuron_studio_lib::translation::map_mosaic(
                    std::slice::from_ref(&b), &ordered, &results, 400, 600,
                );
                Ok::<_, String>(PageTranslation {
                    page_id: page_id.clone(),
                    target_lang: "Indonesian".into(),
                    style: "standard".into(),
                    model: "m".into(),
                    bubbles: mapped,
                })
            });
            let _ = tx.send((page_id, out)).await;
        });
    }
    drop(tx);
    let mut pages = Vec::new();
    let mut done = 0;
    while let Some((id, r)) = rx.recv().await {
        done += 1;
        let ok = r.is_ok();
        progress.lock().unwrap().push((id.clone(), done, ok));
        if let Ok(t) = r {
            pages.push(t);
        }
    }
    pages.sort_by(|a, b| a.page_id.cmp(&b.page_id));
    let prog = progress.lock().unwrap().clone();
    (pages, prog)
}

#[tokio::test]
async fn batch_10_progress_export() {
    let (pages, prog) = run_batch_10().await;
    assert_eq!(pages.len(), 10);
    // Progress: 10 event, done 1..=10, semua ok.
    assert_eq!(prog.len(), 10);
    let mut dones: Vec<usize> = prog.iter().map(|(_, d, _)| *d).collect();
    dones.sort_unstable();
    assert_eq!(dones, (1..=10).collect::<Vec<_>>());
    assert!(prog.iter().all(|(_, _, ok)| *ok));

    // Export JSON: 10 PageTranslation valid.
    let raw = serde_json::to_vec_pretty(&pages).unwrap();
    let back: Vec<PageTranslation> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(back.len(), 10);
    assert_eq!(back[0].bubbles[0].translated, "A");

    // Export PNG per halaman: decode OK + teks bitmap ada.
    let png = page_png();
    let shapes = [None];
    for t in &pages {
        let out = render_page_png(&png, t, &shapes).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (400, 600));
        assert!(img.pixels().any(|p| p.0 == [0, 0, 0]));
    }

    // CBZ: zip berisi 10 PNG filename order.
    let buf: Vec<u8> = {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (i, t) in pages.iter().enumerate() {
            let name = format!("{:03}.png", i + 1);
            zip.start_file(&name, zip::write::SimpleFileOptions::default())
                .unwrap();
            let bytes = render_page_png(&png, t, &shapes).unwrap();
            use std::io::Write;
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    };
    let cur = std::io::Cursor::new(&buf);
    let mut zip = zip::ZipArchive::new(cur).unwrap();
    assert_eq!(zip.len(), 10);
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert_eq!(names[0], "001.png");
}

/// export_*_bytes bekerja di atas AppState penuh — diuji via command layer
/// di sini hanya memastikan signature terpanggil (compile-guard).
#[test]
fn export_fns_linked() {
    let _ = export_json_bytes as fn(&tauri::State<'_, kuron_studio_lib::AppState>, &str) -> Result<Vec<u8>, String>;
    let _ = export_cbz_bytes as fn(&tauri::State<'_, kuron_studio_lib::AppState>, &str) -> Result<Vec<u8>, String>;
}
