//! M5-1 TM + M5-2 QA + M5-6 share/zip commands.

use tauri::State;

use crate::memory::{search_memory, TmHit};
use crate::qa::{check_project, QaIssue};
use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub fn tm_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<TmHit>, String> {
    Ok(search_memory(&state, &query, limit.unwrap_or(10)))
}

#[tauri::command(rename_all = "snake_case")]
pub fn qa_check(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<QaIssue>, String> {
    check_project(&state, &project_id)
}

/// M5-6: export project (projects.json slice + daftar file gambar) ke zip
/// untuk share via git/zip. `path` = file zip tujuan.
#[tauri::command(rename_all = "snake_case")]
pub fn share_project(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
) -> Result<String, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let proj = store
        .projects
        .get(&project_id)
        .ok_or_else(|| format!("project not found: {project_id}"))?
        .clone();
    drop(store);
    let mut pages = proj.pages.clone();
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    let manifest = serde_json::to_vec_pretty(&proj).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file("project.json", zip::write::SimpleFileOptions::default())
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    zip.write_all(&manifest).map_err(|e| e.to_string())?;
    for (i, pg) in pages.iter().enumerate() {
        let data = std::fs::read(&pg.path)
            .map_err(|e| format!("read {}: {e}", pg.path))?;
        let ext = std::path::Path::new(&pg.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        zip.start_file(
            format!("pages/{:03}.{ext}", i + 1),
            zip::write::SimpleFileOptions::default(),
        )
        .map_err(|e| e.to_string())?;
        zip.write_all(&data).map_err(|e| e.to_string())?;
    }
    let bytes = zip.finish().map_err(|e| e.to_string())?.into_inner();
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
