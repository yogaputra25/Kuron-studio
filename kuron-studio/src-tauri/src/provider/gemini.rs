use base64::Engine;

use super::config::{is_vision_capable, AiModelOption};

/// Tolerate legacy saved bases ending in /v1beta (old frontend default).
fn norm_base(base_url: &str) -> &str {
    let t = base_url.trim_end_matches('/');
    t.strip_suffix("/v1beta").unwrap_or(t)
}

fn err_preview(status: reqwest::StatusCode, body: &str) -> String {
    let p: String = body.chars().take(200).collect();
    format!("http {status}: {p}")
}

fn b64(jpeg: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(jpeg)
}

pub async fn chat_completions_image(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg_bytes: &[u8],
) -> Result<String, String> {
    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        norm_base(base_url),
        model,
        api_key
    );
    let body = serde_json::json!({
        "contents": [{"parts": [
            {"text": prompt},
            {"inlineData": {"mimeType": "image/jpeg", "data": b64(jpeg_bytes)}}
        ]}],
        "generationConfig": {"temperature": 0.3, "maxOutputTokens": 4096},
    });
    let res = client.post(&url).json(&body).send().await.map_err(|e| e.to_string())?;
    if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let res2 = client.post(&url).json(&body).send().await.map_err(|e| e.to_string())?;
        if res2.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err("rate_limited: retry later".to_string());
        }
        if !res2.status().is_success() {
            let st = res2.status();
            let b = res2.text().await.unwrap_or_default();
            return Err(err_preview(st, &b));
        }
        let v: serde_json::Value = res2.json().await.map_err(|e| e.to_string())?;
        return extract(&v);
    }
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    extract(&v)
}

fn extract(v: &serde_json::Value) -> Result<String, String> {
    let parts = v
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .ok_or_else(|| "empty candidates".to_string())?;
    let text: String = parts.iter().filter_map(|p| p.get("text").and_then(|t| t.as_str())).collect();
    if text.is_empty() { Err("empty candidates".to_string()) } else { Ok(text) }
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<AiModelOption>, String> {
    let url = format!("{}/v1beta/models?key={}", norm_base(base_url), api_key);
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let ids: Vec<String> = v
        .get("models")
        .and_then(|m| m.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|n| n.strip_prefix("models/").unwrap_or(n).to_string())).collect())
        .unwrap_or_default();
    Ok(ids.into_iter().map(|id| AiModelOption { label: id.clone(), vision: is_vision_capable(&id), id }).collect())
}

pub async fn validate_minimal(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        norm_base(base_url),
        model,
        api_key
    );
    let body = serde_json::json!({
        "contents": [{"parts": [{"text": "Reply with exactly: ok"}]}],
        "generationConfig": {"temperature": 0.0, "maxOutputTokens": 8},
    });
    let res = client.post(&url).json(&body).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    Ok(())
}
