use crate::error::AppError;

use super::config::{vision_capable, AiModelOption, AiProviderType};
use super::http;

/// Sufiks base lawas yang ditoleransi (default frontend lama).
const STRIP_SUFFIXES: &[&str] = &["/v1beta"];

pub async fn chat_completions_image(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg_bytes: &[u8],
) -> Result<String, AppError> {
    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        http::norm_base(base_url, STRIP_SUFFIXES),
        model,
        api_key
    );
    let body = serde_json::json!({
        "contents": [{"parts": [
            {"text": prompt},
            {"inlineData": {"mimeType": "image/jpeg", "data": http::b64(jpeg_bytes)}}
        ]}],
        "generationConfig": {"temperature": 0.3, "maxOutputTokens": 4096},
    });
    let res = http::post_with_retry(|| client.post(&url).json(&body), || {})
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
    let parts = v
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .ok_or_else(|| AppError::parse("empty candidates".to_string()))?;
    let text: String = parts.iter().filter_map(|p| p.get("text").and_then(|t| t.as_str())).collect();
    if text.is_empty() { Err(AppError::parse("empty candidates".to_string())) } else { Ok(text) }
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<AiModelOption>, AppError> {
    let url = format!("{}/v1beta/models?key={}", http::norm_base(base_url, STRIP_SUFFIXES), api_key);
    let res = client.get(&url).send().await.map_err(AppError::from)?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    let v: serde_json::Value = res.json().await.map_err(AppError::from)?;
    let ids: Vec<String> = v
        .get("models")
        .and_then(|m| m.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|n| n.strip_prefix("models/").unwrap_or(n).to_string())).collect())
        .unwrap_or_default();
    Ok(ids.into_iter().map(|id| AiModelOption { label: id.clone(), vision: vision_capable(AiProviderType::Gemini, &id), id }).collect())
}

pub async fn validate_minimal(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), AppError> {
    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        http::norm_base(base_url, STRIP_SUFFIXES),
        model,
        api_key
    );
    let body = serde_json::json!({
        "contents": [{"parts": [{"text": "Reply with exactly: ok"}]}],
        "generationConfig": {"temperature": 0.0, "maxOutputTokens": 8},
    });
    let res = client.post(&url).json(&body).send().await.map_err(AppError::from)?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(AppError::provider(http::err_preview(st, &b)));
    }
    Ok(())
}
