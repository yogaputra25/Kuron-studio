//! Mock list_models per provider (#26/#27): TcpListener standar, tanpa dev-dep.
//! Fixture = bentuk respons asli (diverifikasi dev.meta.ai / docs.cohere.com, Okt 2026).

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use kuron_studio_lib::provider::config::AiProviderType;
use kuron_studio_lib::provider::{cohere, gemini, openai};

/// Mock HTTP: layani `bodies` berurutan, kirim request head (request-line +
/// header) ke test via channel supaya path/auth bisa diassert.
fn spawn_mock(bodies: Vec<&'static str>) -> (String, mpsc::Receiver<String>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap().to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for body in bodies {
            let (mut s, _) = l.accept().unwrap();
            let mut buf = [0u8; 8192];
            let n = s.read(&mut buf).unwrap_or(0);
            let head = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = tx.send(head);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });
    (format!("http://{addr}"), rx)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().unwrap()
}

#[tokio::test]
async fn meta_list_models_openai_shape_vision_flags() {
    let canned = r#"{"object":"list","data":[{"id":"muse-spark-1.3"},{"id":"muse-image-1.0"}]}"#;
    let (base, _rx) = spawn_mock(vec![canned]);
    let out = openai::list_models(&client(), &base, "k", AiProviderType::MetaAi).await.unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!((out[0].id.as_str(), out[0].vision), ("muse-spark-1.3", true));
    assert_eq!((out[1].id.as_str(), out[1].vision), ("muse-image-1.0", false));
}

#[tokio::test]
async fn gemini_list_models_strips_prefix_vision_flags() {
    let canned = r#"{"models":[{"name":"models/gemini-2.0-flash"},{"name":"models/text-embedding-004"}]}"#;
    let (base, _rx) = spawn_mock(vec![canned]);
    let out = gemini::list_models(&client(), &base, "k").await.unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!((out[0].id.as_str(), out[0].vision), ("gemini-2.0-flash", true));
    assert_eq!((out[1].id.as_str(), out[1].vision), ("text-embedding-004", false));
}

#[tokio::test]
async fn cohere_list_models_vision_flags() {
    let canned = r#"{"models":[{"name":"command-a-vision-07-2025"},{"name":"command-r-plus-08-2024"}]}"#;
    let (base, _rx) = spawn_mock(vec![canned]);
    let out = cohere::list_models(&client(), &base, "k").await.unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!((out[0].id.as_str(), out[0].vision), ("command-a-vision-07-2025", true));
    assert_eq!((out[1].id.as_str(), out[1].vision), ("command-r-plus-08-2024", false));
}

#[tokio::test]
async fn list_models_hits_expected_paths() {
    let list = r#"{"data":[]}"#;
    let (base, rx) = spawn_mock(vec![
        list,                                                // openai/meta
        r#"{"models":[]}"#,                                  // gemini
        r#"{"models":[]}"#,                                  // cohere
    ]);
    let c = client();
    let _ = openai::list_models(&c, &base, "k", AiProviderType::OpenAi).await.unwrap();
    let _ = gemini::list_models(&c, &base, "k").await.unwrap();
    let _ = cohere::list_models(&c, &base, "k").await.unwrap();
    let h1 = rx.recv().unwrap();
    let h2 = rx.recv().unwrap();
    let h3 = rx.recv().unwrap();
    assert!(h1.starts_with("GET /models "), "openai/meta: {h1}");
    assert!(h2.starts_with("GET /v1beta/models?key="), "gemini: {h2}");
    assert!(h3.starts_with("GET /v1/models "), "cohere: {h3}");
    assert!(h1.contains("Bearer") || h1.contains("authorization"), "auth header: {h1}");
}
