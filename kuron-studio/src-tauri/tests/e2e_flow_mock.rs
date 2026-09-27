//! M4-10: E2E lapis command — import 3 halaman → detect (mock boxes) →
//! translate mock → export JSON/PNG/CBZ bytes.
//! Tanpa webview: menguji rantai backend yang dipakai Playwright/webDriver
//! (import_pages → translate mock → collect/export), plus pola batch M3-12.

use std::io::{Read, Write};
use std::net::TcpListener;

use image::{Rgb, RgbImage};
use kuron_studio_lib::bubble::BubbleBox;
use kuron_studio_lib::commands::export::render_page_png;
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

fn page_png(seed: u8) -> Vec<u8> {
    let img = RgbImage::from_pixel(400, 600, Rgb([200 + seed, 200, 200]));
    let mut buf = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut buf);
    use image::ImageEncoder;
    enc.write_image(img.as_raw(), 400, 600, image::ExtendedColorType::Rgb8)
        .unwrap();
    buf
}

fn bubble() -> BubbleBox {
    BubbleBox {
        x: 10, y: 20, w: 60, h: 40, confidence: 0.9,
        shape: None, kind: None, tail: None,
    }
}

async fn translate_one(
    client: &reqwest::Client,
    base: &str,
    page_id: &str,
) -> PageTranslation {
    let text = kuron_studio_lib::provider::openai::chat_completions_image(
        client, base, "k", "m", "prompt", b"fake-jpeg",
    )
    .await
    .unwrap();
    let results = kuron_studio_lib::parser::parse_mosaic_json(&text).unwrap();
    let b = bubble();
    let ordered = order_indices(std::slice::from_ref(&b), "rtl");
    let mapped = kuron_studio_lib::translation::map_mosaic(
        std::slice::from_ref(&b), &ordered, &results, 400, 600,
    );
    PageTranslation {
        page_id: page_id.to_string(),
        target_lang: "Indonesian".into(),
        style: "standard".into(),
        model: "m".into(),
        bubbles: mapped,
    }
}

#[tokio::test]
async fn e2e_import3_detect_translate_export() {
    // import: 3 halaman PNG valid (filename order).
    let pages: Vec<Vec<u8>> = (0..3).map(page_png).collect();
    for (i, p) in pages.iter().enumerate() {
        let img = image::load_from_memory(p).unwrap();
        assert_eq!(img.width(), 400, "page {i} decode");
    }

    // detect: mock 1 box/halaman (ganti infer ONNX di CI tanpa GPU).
    let boxes: Vec<BubbleBox> = pages.iter().map(|_| bubble()).collect();
    assert_eq!(boxes.len(), 3);

    // translate mock: 3 halaman → semua terisi.
    let canned = r#"{"choices":[{"message":{"role":"assistant","content":"{\"1\":{\"original\":\"a\",\"reading\":\"\",\"translated\":\"A\"}}"}}]}"#;
    let base = spawn_mock(canned, 6);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let mut out = Vec::new();
    for i in 0..3 {
        out.push(translate_one(&client, &base, &format!("p{i}")).await);
    }
    assert_eq!(out.len(), 3);
    assert!(out.iter().all(|t| t.bubbles.iter().all(|b| b.translated == "A")));

    // export JSON: roundtrip 3 PageTranslation.
    let raw = serde_json::to_vec_pretty(&out).unwrap();
    let back: Vec<PageTranslation> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(back.len(), 3);

    // export PNG overlay + CBZ: 3 file filename order.
    let shapes = [None];
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (i, t) in out.iter().enumerate() {
        let bytes = render_page_png(&pages[i], t, &shapes).unwrap();
        let img = image::load_from_memory(&bytes).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (400, 600));
        zip.start_file(format!("{:03}.png", i + 1), zip::write::SimpleFileOptions::default())
            .unwrap();
        use std::io::Write;
        zip.write_all(&bytes).unwrap();
    }
    let buf: Vec<u8> = zip.finish().unwrap().into_inner();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&buf)).unwrap();
    assert_eq!(zip.len(), 3);
    assert_eq!(zip.by_index(0).unwrap().name(), "001.png");
    assert_eq!(zip.by_index(2).unwrap().name(), "003.png");
}
