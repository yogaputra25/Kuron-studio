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

fn view_of(rec: &ProviderRecord) -> ProviderView {
    ProviderView {
        id: rec.id.clone(),
        provider_type: rec.provider_type,
        name: rec.name.clone(),
        base_url: rec.effective_base_url(),
        model: rec.model.clone(),
        has_key: !rec.api_key.is_empty(),
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

#[tauri::command]
pub fn save_provider(
    state: State<'_, AppState>,
    input: SaveProviderInput,
) -> Result<ProviderView, String> {
    let rec = parse_input(input)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::insert_provider(&db, &rec)?;
    Ok(view_of(&rec))
}

#[tauri::command]
pub fn get_providers(state: State<'_, AppState>) -> Result<Vec<ProviderView>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let recs = cache::list_providers(&db)?;
    Ok(recs.iter().map(view_of).collect())
}

#[tauri::command]
pub fn delete_provider(state: State<'_, AppState>, provider_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::delete_provider(&db, &provider_id)
}

#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<Vec<provider::config::AiModelOption>, String> {
    let (client, rec) = record_of(&state, &provider_id)?;
    provider::list_models(&client, &rec).await
}

#[tauri::command]
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
    let rec = cache::get_provider(&db, provider_id)?
        .ok_or_else(|| format!("provider not found: {provider_id}"))?;
    Ok((state.http.clone(), rec))
}
