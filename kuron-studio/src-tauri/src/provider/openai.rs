use base64::Engine;

use super::config::is_vision_capable;

fn data_url(jpeg: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(jpeg);
    format!("data:image/jpeg;base64,{b64}")
}

fn err_preview(status: reqwest::StatusCode, body: &str) -> String {
    let p: String = body.chars().take(200).collect();
    format!("http {status}: {p}")
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
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": [
            {"type": "text", "text": prompt},
            {"type": "image_url", "image_url": {"url": data_url(jpeg)}}
        ]}],
        "temperature": 0.3,
        "max_tokens": max_tokens,
    });
    let res = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let res2 = client
            .post(&url)
            .bearer_auth(api_key)
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
        return extract_content(&v);
    }
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    extract_content(&v)
}

fn extract_content(v: &serde_json::Value) -> Result<String, String> {
    v.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .map(content_to_string)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "empty choices".to_string())
}

pub async fn chat_completions_image(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    jpeg_bytes: &[u8],
) -> Result<String, String> {
    post_chat(client, base_url, api_key, model, prompt, jpeg_bytes, 4096).await
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<super::config::AiModelOption>, String> {
    let base = base_url.trim_end_matches('/');
    let url = format!("{base}/models");
    let res = client.get(&url).bearer_auth(api_key).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let ids: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.iter().filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    Ok(ids.into_iter().map(|id| super::config::AiModelOption { label: id.clone(), vision: is_vision_capable(&id), id }).collect())
}

pub async fn validate_minimal(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), String> {
    // Tiny text-only probe (no image bytes needed).
    let base = base_url.trim_end_matches('/');
    let url = format!("{base}/chat/completions");
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "Reply with exactly: ok"}],
        "temperature": 0.0,
        "max_tokens": 8,
    });
    let res = client.post(&url).bearer_auth(api_key).json(&body).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let st = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(err_preview(st, &b));
    }
    Ok(())
}
