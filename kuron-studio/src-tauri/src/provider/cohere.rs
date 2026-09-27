use base64::Engine;

use super::config::{is_vision_capable, AiModelOption};

/// Tolerate legacy saved bases ending in /v2 (old frontend default).
fn norm_base(base_url: &str) -> &str {
    let t = base_url.trim_end_matches('/');
    t.strip_suffix("/v2").or_else(|| t.strip_suffix("/v1")).unwrap_or(t)
}

fn err_preview(status: reqwest::StatusCode, body: &str) -> String {
    let p: String = body.chars().take(200).collect();
    format!("http {status}: {p}")
}

pub async fn chat_completions_image(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg_bytes: &[u8],
) -> Result<String, String> {
    let url = format!("{}/v2/chat", norm_base(base_url));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": [
            {"type": "text", "text": prompt},
            {"type": "image_url", "image_url": {"url": format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(jpeg_bytes))}}
        ]}],
        "temperature": 0.3,
    });
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let res2 = client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
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
    let content = v
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
        .ok_or_else(|| "empty message".to_string())?;
    let text: String = content
        .iter()
        .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
        .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
        .collect();
    if text.is_empty() { Err("empty message".to_string()) } else { Ok(text) }
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<AiModelOption>, String> {
    let url = format!("{}/v1/models", norm_base(base_url));
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let ids: Vec<String> = v
        .get("models")
        .and_then(|m| m.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    Ok(ids.into_iter().map(|id| AiModelOption { label: id.clone(), vision: is_vision_capable(&id), id }).collect())
}

pub async fn validate_minimal(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), String> {
    let url = format!("{}/v2/chat", norm_base(base_url));
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
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    Ok(())
}
