//! M5-1 TM + M5-2 QA + M5-6 share/zip commands.

use tauri::State;

use crate::memory::{search_memory, TmHit};
use crate::qa::{check_project, QaIssue};
use crate::AppState;

/// Info environment untuk bug report: versi, OS, lokasi file log.
/// Tidak memuat secret apa pun (lihat `logging::diagnostics`).
#[tauri::command(rename_all = "snake_case")]
pub fn diagnostics(state: State<'_, AppState>) -> serde_json::Value {
    let mut v = crate::logging::diagnostics();
    // Path app data ikut dilaporkan supaya panel bisa baca log-nya sendiri
    // tanpa menebak konvensi folder di frontend.
    if let Some(obj) = v.as_object_mut() {
        obj.insert("data_dir".into(), serde_json::Value::String(state.data_dir()));
    }
    v
}

/// N baris terakhir file log, sebagai JSON Lines mentah.
///
/// Dipakai panel Diagnostics di UI supaya user bisa menyalin log lalu
/// menempelkannya ke issue tanpa harus mencari file di Finder.
#[tauri::command(rename_all = "snake_case")]
pub fn read_log(state: State<'_, AppState>, lines: Option<usize>) -> Result<String, String> {
    let n = lines.unwrap_or(200).clamp(1, 5000);
    let path = state.data_dir_path().join("kuron-studio.log");
    let raw = match std::fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let all: Vec<&str> = raw.lines().collect();
    let start = all.len().saturating_sub(n);
    Ok(all[start..].join("\n"))
}

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
