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

/// Token masuk/keluar satu respons chat. Bentuk per provider beda
/// (OpenAI `usage`, Gemini `usageMetadata`, Cohere `tokens`) — dinormalisasi
/// di [`extract_usage`]. `None` = provider tak melaporkan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
}

/// Hasil satu chat completions: teks + pemakaian opsional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResult {
    pub text: String,
    pub usage: Option<TokenUsage>,
}

 /// Benar bila status minta retry-429 satu kali.
 pub fn is_429(status: reqwest::StatusCode) -> bool {
     status == reqwest::StatusCode::TOO_MANY_REQUESTS
 }
 
fn u64_at(v: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    let mut cur = v;
    for k in keys {
        cur = cur.get(*k)?;
    }
    cur.as_u64()
}

/// Ambil pemakaian dari body respons apa pun bentuk providernya.
/// OpenAI-shape (`usage.prompt_tokens/completion_tokens`), Gemini
/// (`usageMetadata.promptTokenCount/candidatesTokenCount`), Cohere v2
/// (`usage.tokens.input_tokens/output_tokens`, fallback `billed_units`).
pub fn extract_usage(v: &serde_json::Value) -> Option<TokenUsage> {
    if let Some(u) = v.get("usage") {
        // OpenAI-shape.
        if let (Some(i), Some(o)) = (
            u.get("prompt_tokens").and_then(|n| n.as_u64()),
            u.get("completion_tokens").and_then(|n| n.as_u64()),
        ) {
            return Some(TokenUsage { input: i, output: o });
        }
        // Cohere-shape.
        if let Some(t) = u.get("tokens") {
            if let (Some(i), Some(o)) = (
                u64_at(t, &["input_tokens"]),
                u64_at(t, &["output_tokens"]),
            ) {
                return Some(TokenUsage { input: i, output: o });
            }
        }
        if let Some(b) = u.get("billed_units") {
            if let (Some(i), Some(o)) = (
                u64_at(b, &["input_tokens"]),
                u64_at(b, &["output_tokens"]),
            ) {
                return Some(TokenUsage { input: i, output: o });
            }
        }
    }
    if let Some(m) = v.get("usageMetadata") {
        if let (Some(i), Some(o)) = (
            u64_at(m, &["promptTokenCount"]),
            u64_at(m, &["candidatesTokenCount"]),
        ) {
            return Some(TokenUsage { input: i, output: o });
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_usage_mengenali_tiga_bentuk_provider() {
        let openai: serde_json::Value = serde_json::json!({"usage": {"prompt_tokens": 10, "completion_tokens": 20}});
        assert_eq!(extract_usage(&openai), Some(TokenUsage { input: 10, output: 20 }));
        let gemini: serde_json::Value = serde_json::json!({"usageMetadata": {"promptTokenCount": 5, "candidatesTokenCount": 7}});
        assert_eq!(extract_usage(&gemini), Some(TokenUsage { input: 5, output: 7 }));
        let cohere: serde_json::Value = serde_json::json!({"usage": {"tokens": {"input_tokens": 3, "output_tokens": 4}}});
        assert_eq!(extract_usage(&cohere), Some(TokenUsage { input: 3, output: 4 }));
        let kosong: serde_json::Value = serde_json::json!({"choices": []});
        assert_eq!(extract_usage(&kosong), None);
    }
}
