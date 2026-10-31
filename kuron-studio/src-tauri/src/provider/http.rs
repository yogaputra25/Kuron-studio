//! Helper HTTP bersama semua provider (#56).
//!
//! Sebelumnya `openai.rs`, `gemini.rs`, `cohere.rs` masing-masing mendefinisikan
//! `err_preview`, perapih base URL, blok retry-429, dan encode data-URL/base64
//! sendiri-sendiri. Satu definisi di sini dipakai ketiga modul + `commands/image`.
//! Tanpa ubah perilaku request/response: URL, header, body, log, dan string error
//! sama persis seperti salinan lama.

use base64::Engine;

/// Panjang potongan body error — sama seperti ketiga salinan lama (200 char).
pub const ERR_PREVIEW_LEN: usize = 200;

/// Jeda sebelum kirim ulang sekali saat kena 429 — sama seperti blok lama.
pub const RETRY_429_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

/// Potong teks ke `n` char pertama.
pub fn preview(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// Ringkasan error HTTP: status + potongan body. Format dibaca test mock
/// (`provider_chat_mock`, `provider_list_mock`) — jangan diubah.
pub fn err_preview(status: reqwest::StatusCode, body: &str) -> String {
    format!("http {status}: {}", preview(body, ERR_PREVIEW_LEN))
}

/// Rapikan base URL: buang `/` akhir lalu satu sufiks versi lama bila cocok.
/// Sufiks per provider — gemini `&["/v1beta"]`, cohere `&["/v2", "/v1"]`,
/// openai `&[]` (tanpa sufiks, hanya buang `/` seperti inline lama).
pub fn norm_base<'a>(base_url: &'a str, strip: &[&str]) -> &'a str {
    let t = base_url.trim_end_matches('/');
    strip.iter().find_map(|s| t.strip_suffix(s)).unwrap_or(t)
}

/// Encode mentah base64 standar.
pub fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Data-URL `data:<mime>;base64,...`.
pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", b64(bytes))
}

/// Benar bila status minta retry-429 satu kali.
pub fn is_429(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
}

/// Error saat 429 bertahan setelah kirim ulang — string sama seperti blok lama.
pub fn rate_limited() -> String {
    "rate_limited: retry later".to_string()
}

/// Kirim request sekali; bila kena 429, panggil `on_retry` (mis. log warn),
/// tunggu [`RETRY_429_DELAY`], lalu kirim ulang sekali memakai request yang
/// dibangun ulang oleh `send`.
///
/// `Err((attempt, e))`: `attempt` 0 = kirim pertama gagal, 1 = kirim ulang
/// gagal — caller memetakan ke pesannya masing-masing agar perilaku
/// log/error tiap provider tidak berubah.
pub async fn post_with_retry(
    send: impl Fn() -> reqwest::RequestBuilder,
    on_retry: impl FnOnce(),
) -> Result<reqwest::Response, (u32, reqwest::Error)> {
    let res = send().send().await.map_err(|e| (0u32, e))?;
    if !is_429(res.status()) {
        return Ok(res);
    }
    on_retry();
    tokio::time::sleep(RETRY_429_DELAY).await;
    send().send().await.map_err(|e| (1u32, e))
}
