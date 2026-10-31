use crate::error::AppError;

use super::config::{vision_capable, AiModelOption, AiProviderType};
use super::http;

/// Sufiks base lawas yang ditoleransi (default frontend lama).
const STRIP_SUFFIXES: &[&str] = &["/v2", "/v1"];

pub async fn chat_completions_image(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg_bytes: &[u8],
) -> Result<String, AppError> {
    let url = format!("{}/v2/chat", http::norm_base(base_url, STRIP_SUFFIXES));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": [
            {"type": "text", "text": prompt},
            {"type": "image_url", "image_url": {"url": http::data_url("image/jpeg", jpeg_bytes)}}
        ]}],
        "temperature": 0.3,
    });
    let send = || {
        client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
    };
    let res = http::post_with_retry(send, || {})
        .await
        .map_err(|(_, e)| AppError::provider(e.to_string()))?;
    if http::is_429(res.status()) {
        return Err(AppError::provider(http::rate_limited()));
    }
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    let v: serde_json::Value = res.json().await.map_err(AppError::from)?;
    extract(&v)
}

fn extract(v: &serde_json::Value) -> Result<String, AppError> {
    let content = v
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
        .ok_or_else(|| AppError::parse("empty message".to_string()))?;
    let text: String = content
        .iter()
        .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
        .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
        .collect();
    if text.is_empty() { Err(AppError::parse("empty message".to_string())) } else { Ok(text) }
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<AiModelOption>, AppError> {
    let url = format!("{}/v1/models", http::norm_base(base_url, STRIP_SUFFIXES));
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(AppError::from)?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    let v: serde_json::Value = res.json().await.map_err(AppError::from)?;
    let ids: Vec<String> = v
        .get("models")
        .and_then(|m| m.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    Ok(ids.into_iter().map(|id| AiModelOption { label: id.clone(), vision: vision_capable(AiProviderType::Cohere, &id), id }).collect())
}

pub async fn validate_minimal(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), AppError> {
    let url = format!("{}/v2/chat", http::norm_base(base_url, STRIP_SUFFIXES));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "Reply with exactly: ok"}],
        "temperature": 0.0,
    });
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(AppError::from)?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    Ok(())
}
