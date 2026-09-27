//! M2 mock: std TcpListener serving canned OpenAI-compatible JSON.
//! No new dev-deps (no wiremock).

use std::io::{Read, Write};
use std::net::TcpListener;

use kuron_studio_lib::cache;
use kuron_studio_lib::parser::parse_mosaic_json;
use kuron_studio_lib::prompt::{build_mosaic_prompt, TranslateStyle};
use kuron_studio_lib::translation::{map_mosaic, order_indices};

fn spawn_mock(body: &'static str) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap().to_string();
    std::thread::spawn(move || {
        for stream in l.incoming().take(2) {
            let mut s = stream.unwrap();
            let mut buf = [0u8; 8192];
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

#[tokio::test]
async fn mock_openai_chat_parse_map_cache() {
    let canned = r#"{"choices":[{"message":{"role":"assistant","content":"{\"1\":{\"original\":\"a\",\"reading\":\"\",\"translated\":\"A\"}}"}}]}"#;
    let base = spawn_mock(canned);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().unwrap();
    let out = kuron_studio_lib::provider::openai::chat_completions_image(
        &client, &base, "secret", "m", "prompt", b"fake-jpeg",
    )
    .await
    .unwrap();
    assert!(out.contains("\"1\""));

    let results = parse_mosaic_json(&out).unwrap();
    assert_eq!(results[0].1.translated, "A");

    let b = kuron_studio_lib::bubble::BubbleBox { x: 10, y: 20, w: 60, h: 40, confidence: 1.0, shape: None, kind: None, tail: None };
    let ordered = order_indices(std::slice::from_ref(&b), "rtl");
    let mapped = map_mosaic(std::slice::from_ref(&b), &ordered, &results, 800, 1200);
    assert_eq!(mapped[0].translated, "A");
    assert_eq!(mapped[0].x, 10);

    // Cache roundtrip.
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    cache::init_db(&conn).unwrap();
    let key = cache::cache_key(cache::CacheKey {
        image: b"img",
        bubbles_json: "[]",
        target_lang: "id",
        style: "standard",
        model: "m",
        skip_sfx: true,
        reading_dir: "rtl",
        glossary: None,
    });
    cache::cache_set(&conn, &key, "{\"a\":1}").unwrap();
    assert_eq!(cache::cache_get(&conn, &key).unwrap().as_deref(), Some("{\"a\":1}"));

    // Prompt sanity.
    let p = build_mosaic_prompt("Indonesian", TranslateStyle::Standard, true, 1);
    assert!(p.contains("Indonesian"));
}
