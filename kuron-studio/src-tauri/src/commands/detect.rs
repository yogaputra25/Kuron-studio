use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::bubble::{model_search_paths, post_process, sanitize_bubbles, BubbleBox};
use crate::detector::DetectorState;
use crate::AppState;

/// Status ONNX untuk banner frontend — model belum ada = mode manual penuh.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectStatus {
    pub engine: String,
    pub model_found: bool,
    pub model_path: Option<String>,
}

#[derive(Debug)]
pub struct DetectState {
    resource_dir: Option<PathBuf>,
    data_dir: PathBuf,
    detector: Arc<DetectorState>,
}

impl Default for DetectState {
    fn default() -> Self {
        Self {
            resource_dir: None,
            data_dir: std::env::temp_dir(),
            detector: Arc::new(DetectorState::new()),
        }
    }
}

impl DetectState {
    pub fn new(resource_dir: Option<PathBuf>, data_dir: PathBuf) -> Self {
        Self {
            resource_dir,
            data_dir,
            detector: Arc::new(DetectorState::new()),
        }
    }

    fn model_path(&self) -> Option<PathBuf> {
        model_search_paths(self.resource_dir.clone(), &self.data_dir)
            .into_iter()
            .find(|p| p.exists())
    }

    pub fn status(&self) -> DetectStatus {
        let found = self.model_path();
        DetectStatus {
            // "ort" = session nyata; "manual" = tanpa model (canvas manual).
            engine: if found.is_some() { "ort".to_string() } else { "manual".to_string() },
            model_found: found.is_some(),
            model_path: found.map(|p| p.to_string_lossy().to_string()),
        }
    }
}

impl Clone for DetectState {
    fn clone(&self) -> Self {
        Self {
            resource_dir: self.resource_dir.clone(),
            data_dir: self.data_dir.clone(),
            detector: Arc::clone(&self.detector),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectProgress {
    pub page_id: String,
    pub done: usize,
    pub total: usize,
}

#[tauri::command]
pub fn detect_status(state: State<'_, AppState>) -> Result<DetectStatus, String> {
    Ok(state.detect.status())
}

/// Detect nyata (ort) bila model ada; tanpa model → Ok([]) seperti dulu.
#[tauri::command]
pub async fn detect_bubbles(
    state: State<'_, AppState>,
    page_id: String,
) -> Result<Vec<BubbleBox>, String> {
    set_page_status(state.clone(), &page_id, "detecting")?;
    let status = state.detect.status();
    let Some(model_path) = status.model_path.map(PathBuf::from) else {
        set_page_status(state, &page_id, "idle")?;
        return Ok(vec![]);
    };
    let page_path = page_path_of(&state, &page_id)?;
    let detector = state.detect.detector.clone();
    let boxes = tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&page_path).map_err(|e| format!("read: {e}"))?;
        detector.detect_bytes(&model_path, &bytes)
    })
    .await
    .map_err(|e| format!("infer task: {e}"))??;
    let cleaned = post_process(sanitize_bubbles(boxes));
    set_page_status(state.clone(), &page_id, "idle")?;
    persist_bubbles(&state, &page_id, cleaned.clone())?;
    Ok(cleaned)
}

#[tauri::command]
pub async fn detect_bubbles_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    page_ids: Vec<String>,
) -> Result<Vec<Vec<BubbleBox>>, String> {
    let total = page_ids.len();
    let mut all = Vec::with_capacity(total);
    for (i, id) in page_ids.iter().enumerate() {
        let boxes = detect_bubbles(state.clone(), id.clone()).await?;
        all.push(boxes);
        let _ = app.emit(
            "detect_progress",
            DetectProgress {
                page_id: id.clone(),
                done: i + 1,
                total,
            },
        );
    }
    Ok(all)
}

/// Simpan bubble hasil edit canvas + update status page:
/// >=1 bubble = detected, 0 = noBubbles.
#[tauri::command]
pub fn save_bubbles(
    state: State<'_, AppState>,
    page_id: String,
    bubbles: Vec<BubbleBox>,
) -> Result<Vec<BubbleBox>, String> {
    let cleaned = post_process(sanitize_bubbles(bubbles));
    persist_bubbles(&state, &page_id, cleaned)
}

fn page_path_of(state: &State<'_, AppState>, page_id: &str) -> Result<String, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .projects
        .values()
        .flat_map(|p| p.pages.iter())
        .find(|pg| pg.id == page_id)
        .map(|pg| pg.path.clone())
        .ok_or_else(|| format!("page not found: {page_id}"))
}

fn persist_bubbles(
    state: &State<'_, AppState>,
    page_id: &str,
    cleaned: Vec<BubbleBox>,
) -> Result<Vec<BubbleBox>, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let page = store
        .projects
        .values_mut()
        .flat_map(|p| p.pages.iter_mut())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?;
    page.bubbles = cleaned.clone();
    page.status = if cleaned.is_empty() {
        crate::commands::project::PageStatus::NoBubbles
    } else {
        crate::commands::project::PageStatus::Detected
    };
    store.save_public()?;
    Ok(cleaned)
}

fn set_page_status(state: State<'_, AppState>, page_id: &str, status: &str) -> Result<(), String> {
    use crate::commands::project::PageStatus as S;
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let page = store
        .projects
        .values_mut()
        .flat_map(|p| p.pages.iter_mut())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?;
    page.status = match status {
        "detecting" => S::Detecting,
        "idle" => S::Idle,
        "translating" => S::Translating,
        "translated" => S::Translated,
        "failed" => S::Failed,
        other => return Err(format!("unknown status: {other}")),
    };
    store.save_public()
}
