use tauri::State;

use crate::AppState;
use crate::glossary::GlossaryEntry;
use crate::glossary as gloss;

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_list(state: State<'_, AppState>) -> Result<Vec<GlossaryEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::list_entries(&db)
}

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_add(
    state: State<'_, AppState>,
    source: String,
    target: String,
) -> Result<GlossaryEntry, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::add_entry(&db, &source, &target)
}

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_update(
    state: State<'_, AppState>,
    id: String,
    source: String,
    target: String,
) -> Result<GlossaryEntry, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::update_entry(&db, &id, &source, &target)
}

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::delete_entry(&db, &id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_import_csv(state: State<'_, AppState>, csv: String) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::import_csv(&db, &csv)
}

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_export_csv(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::export_csv(&db)
}

#[tauri::command(rename_all = "snake_case")]
pub fn glossary_context(
    state: State<'_, AppState>,
    bubble_texts: Vec<String>,
) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    gloss::context_for(&db, &bubble_texts)
}
