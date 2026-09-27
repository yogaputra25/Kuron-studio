use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::cache;
use crate::provider::config::{is_vision_capable, AiProviderType, ProviderRecord};
use crate::provider;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderInput {
    pub id: Option<String>,
    pub provider_type: String,
    pub name: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderView {
    pub id: String,
    pub provider_type: AiProviderType,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub has_key: bool,
    pub is_vision_capable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResult {
    pub ok: bool,
    pub message: String,
}

fn has_key_of(rec: &ProviderRecord) -> bool {
    // M4-1: keychain hit wins; sqlite legacy column is the fallback.
    crate::secrets::has_key(&rec.id, &rec.api_key)
}

fn view_of(rec: &ProviderRecord) -> ProviderView {
    ProviderView {
        id: rec.id.clone(),
        provider_type: rec.provider_type,
        name: rec.name.clone(),
        base_url: rec.effective_base_url(),
        model: rec.model.clone(),
        has_key: has_key_of(rec),
        is_vision_capable: is_vision_capable(&rec.model),
    }
}

fn parse_input(input: SaveProviderInput) -> Result<ProviderRecord, String> {
    let pt = AiProviderType::from_str(&input.provider_type)
        .ok_or_else(|| format!("unknown provider: {}", input.provider_type))?;
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("provider name is empty".to_string());
    }
    Ok(ProviderRecord {
        id: input.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        provider_type: pt,
        name,
        base_url: input.base_url.unwrap_or_default().trim().to_string(),
        api_key: input.api_key.unwrap_or_default(),
        model: input.model.unwrap_or_default().trim().to_string(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_provider(
    state: State<'_, AppState>,
    input: SaveProviderInput,
) -> Result<ProviderView, String> {
    let mut rec = parse_input(input)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // M4-1: edit tanpa key = pertahankan key lama (jangan wipe keychain).
    if rec.api_key.is_empty() {
        if let Some(old) = cache::get_provider(&db, &rec.id)? {
            rec.api_key = crate::secrets::read_key(&rec.id, &old.api_key)
                .unwrap_or_default();
        }
    }
    // Keychain write-through; sqlite keeps "" on success so no second
    // copy of the secret exists. Keychain failure → legacy sqlite fallback.
    let sqlite_key = match crate::secrets::save_key(&rec.id, &rec.api_key) {
        Ok(()) => String::new(),
        Err(_) => rec.api_key.clone(),
    };
    cache::insert_provider_with_key(&db, &rec, &sqlite_key)?;
    Ok(view_of(&rec))
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_providers(state: State<'_, AppState>) -> Result<Vec<ProviderView>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let recs = cache::list_providers(&db)?;
    Ok(recs.iter().map(view_of).collect())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_provider(state: State<'_, AppState>, provider_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::secrets::delete_key(&db, &provider_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_models(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<Vec<provider::config::AiModelOption>, String> {
    let (client, rec) = record_of(&state, &provider_id)?;
    provider::list_models(&client, &rec).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn validate_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ValidateResult, String> {
    let (client, rec) = record_of(&state, &provider_id)?;
    match provider::validate(&client, &rec).await {
        Ok(()) => Ok(ValidateResult { ok: true, message: "ok".to_string() }),
        Err(e) => Ok(ValidateResult { ok: false, message: e }),
    }
}

fn record_of(
    state: &State<'_, AppState>,
    provider_id: &str,
) -> Result<(reqwest::Client, ProviderRecord), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut rec = cache::get_provider(&db, provider_id)?
        .ok_or_else(|| format!("provider not found: {provider_id}"))?;
    // M4-1: keychain first, sqlite legacy fallback. Never logged.
    rec.api_key = crate::secrets::read_key(provider_id, &rec.api_key)?;
    Ok((state.http.clone(), rec))
}
