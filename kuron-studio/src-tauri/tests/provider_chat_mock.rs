//! Mock validate + chat per provider (#26/#27): TcpListener standar, tanpa dev-dep.
//! Assert path endpoint, auth, dan bentuk body request.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use kuron_studio_lib::provider::{cohere, gemini, openai};
use kuron_studio_lib::provider::config::AiProviderType;

fn read_request(s: &mut std::net::TcpStream) -> (String, String) {
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = s.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&buf[..n]);
        if raw.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let head_end = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4).unwrap_or(raw.len());
    let head = String::from_utf8_lossy(&raw[..head_end]).to_string();
    let len: usize = head
        .lines()
        .find_map(|l| l.strip_prefix("Content-Length:").or_else(|| l.strip_prefix("content-length:")))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0);
    let mut body = raw[head_end..].to_vec();
    while body.len() < len {
        let n = s.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buf[..n]);
    }
    (head, String::from_utf8_lossy(&body).to_string())
}

/// Layani `bodies` berurutan; kirim (head, body) request ke test.
fn spawn_capture(bodies: Vec<&'static str>) -> (String, mpsc::Receiver<(String, String)>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap().to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for body in bodies {
            let (mut s, _) = l.accept().unwrap();
            let (head, req_body) = read_request(&mut s);
            let _ = tx.send((head, req_body));
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

const OPENAI_OK: &str = r#"{"choices":[{"message":{"content":"ok"}}]}"#;
const GEMINI_OK: &str = r#"{"candidates":[{"content":{"parts":[{"text":"ok"}]}}]}"#;
const COHERE_OK: &str = r#"{"message":{"content":[{"type":"text","text":"ok"}]}}"#;

#[tokio::test]
async fn openai_validate_path_auth_body() {
    let (base, rx) = spawn_capture(vec![OPENAI_OK]);
    openai::validate_minimal(&client(), &base, "secret", "m").await.unwrap();
    let (head, body) = rx.recv().unwrap();
    assert!(head.starts_with("POST /chat/completions "), "path: {head}");
    assert!(head.to_lowercase().contains("authorization: bearer secret"), "auth: {head}");
    assert!(body.contains("Reply with exactly: ok"), "probe: {body}");
    assert!(body.contains("max_tokens"), "max_tokens: {body}");
}

#[tokio::test]
async fn openai_chat_sends_image_url() {
    let (base, rx) = spawn_capture(vec![OPENAI_OK]);
    let out = openai::chat_completions_image(&client(), &base, "k", "m", "prompt", b"fake-jpeg").await.unwrap();
    assert_eq!(out, "ok");
    let (head, body) = rx.recv().unwrap();
    assert!(head.starts_with("POST /chat/completions "), "path: {head}");
    assert!(body.contains("image_url"), "image block: {body}");
    assert!(body.contains("data:image/jpeg;base64,"), "data url: {body}");
}

#[tokio::test]
async fn gemini_validate_and_chat_paths() {
    let (base, rx) = spawn_capture(vec![GEMINI_OK, GEMINI_OK]);
    let c = client();
    gemini::validate_minimal(&c, &base, "key123", "gemini-2.0-flash").await.unwrap();
    gemini::chat_completions_image(&c, &base, "key123", "gemini-2.0-flash", "p", b"j").await.unwrap();
    let (h1, b1) = rx.recv().unwrap();
    let (h2, b2) = rx.recv().unwrap();
    assert!(h1.starts_with("POST /v1beta/models/gemini-2.0-flash:generateContent?key=key123"), "v: {h1}");
    assert!(b1.contains("Reply with exactly: ok"), "probe: {b1}");
    assert!(h2.starts_with("POST /v1beta/models/gemini-2.0-flash:generateContent?key="), "c: {h2}");
    assert!(b2.contains("inlineData"), "inline: {b2}");
    assert!(b2.contains("image/jpeg"), "mime: {b2}");
}

#[tokio::test]
async fn cohere_v2_chat_bearer_image_url() {
    let (base, rx) = spawn_capture(vec![COHERE_OK, COHERE_OK]);
    let c = client();
    cohere::validate_minimal(&c, &base, "tok", "command-a-vision-07-2025").await.unwrap();
    let out = cohere::chat_completions_image(&c, &base, "tok", "command-a-vision-07-2025", "p", b"j").await.unwrap();
    assert_eq!(out, "ok");
    let (h1, b1) = rx.recv().unwrap();
    let (h2, b2) = rx.recv().unwrap();
    assert!(h1.starts_with("POST /v2/chat "), "validate path: {h1}");
    assert!(h1.to_lowercase().contains("authorization: bearer tok"), "auth: {h1}");
    assert!(b1.contains("Reply with exactly: ok"), "probe: {b1}");
    assert!(h2.starts_with("POST /v2/chat "), "chat path: {h2}");
    assert!(b2.contains("image_url"), "image block: {b2}");
}
