use crate::error::AppError;

use super::config::{vision_capable, AiProviderType};
use super::http;
/// Catat request/response HTTP sebagai JSON Lines. Tanpa header auth — hanya
/// status + body terpotong, jadi body error dari proxy lokal bisa dibaca utuh
/// di log.
fn trace(label: &str, url: &str, status: reqwest::StatusCode, body: &str) {
    crate::logging::info(
        "http",
        serde_json::json!({
            "label": label,
            "url": url,
            "status": status.as_u16(),
            "body": crate::logging::redact(body),
        }),
    );
}

fn content_to_string(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

async fn post_chat(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg: &[u8],
    max_tokens: u32,
) -> Result<String, AppError> {
    let url = format!("{}/chat/completions", http::norm_base(base_url, &[]));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": [
            {"type": "text", "text": prompt},
            {"type": "image_url", "image_url": {"url": http::data_url("image/jpeg", jpeg)}}
        ]}],
        "temperature": 0.3,
        "max_tokens": max_tokens,
    });
    let res = http::post_with_retry(
        || {
            client
                .post(&url)
                .bearer_auth(api_key)
                .json(&body)
        },
        || crate::logging::warn("http_429_retry_once", serde_json::json!({ "url": &url })),
    )
    .await
    .map_err(|(attempt, e)| {
        if attempt == 0 {
            crate::logging::error(
                "http_send_failed",
                serde_json::json!({
                    "url": &url, "model": model, "error": crate::logging::redact(&e.to_string()),
                }),
            );
            AppError::provider(format!("send ke {url} gagal: {e}"))
        } else {
            AppError::provider(e.to_string())
        }
    })?;
    if http::is_429(res.status()) {
        trace("chat_completions_retry", &url, res.status(), "");
        return Err(AppError::provider(http::rate_limited()));
    }
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        trace("chat_completions", &url, st, &b);
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    let v: serde_json::Value = res
        .json()
        .await
        .map_err(|e| {
            crate::logging::error(
                "http_json_decode_failed",
                serde_json::json!({ "url": &url, "error": crate::logging::redact(&e.to_string()) }),
            );
            AppError::parse(e.to_string())
        })?;
    extract_content(&v)
}

fn extract_content(v: &serde_json::Value) -> Result<String, AppError> {
    v.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .map(content_to_string)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::parse("empty choices".to_string()))
}

pub async fn chat_completions_image(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg_bytes: &[u8],
) -> Result<String, AppError> {
    post_chat(client, base_url, api_key, model, prompt, jpeg_bytes, 4096).await
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    provider: AiProviderType,
) -> Result<Vec<super::config::AiModelOption>, AppError> {
    let base = http::norm_base(base_url, &[]);
    let url = format!("{base}/models");
    let res = client
        .get(&url)
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| {
            crate::logging::error(
                "list_models_send_failed",
                serde_json::json!({ "url": &url, "error": crate::logging::redact(&e.to_string()) }),
            );
            AppError::provider(format!("request ke {url} gagal: {e}"))
        })?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        trace("list_models", &url, st, &b);
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    let v: serde_json::Value = res.json().await.map_err(AppError::from)?;
    let ids: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    Ok(ids.into_iter().map(|id| super::config::AiModelOption { label: id.clone(), vision: vision_capable(provider, &id), id }).collect())
}

pub async fn validate_minimal(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), AppError> {
    // Tiny text-only probe (no image bytes needed).
    let base = http::norm_base(base_url, &[]);
    let url = format!("{base}/chat/completions");
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "Reply with exactly: ok"}],
        "temperature": 0.0,
        "max_tokens": 8,
    });
    let res = client.post(&url).bearer_auth(api_key).json(&body).send().await.map_err(AppError::from)?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    Ok(())
}
