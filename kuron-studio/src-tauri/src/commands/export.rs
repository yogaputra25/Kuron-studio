use tauri::State;

use crate::AppState;
pub use crate::render::export::{
    collect_pages, count_cbz_entries, describe_skipped, export_cbz_bytes, export_json_bytes, file_name, select_pages,
};
use crate::render::page::shapes_for_export;

#[tauri::command(rename_all = "snake_case")]
pub fn export_project(
    state: State<'_, AppState>,
    project_id: String,
    format: String,
    path: String,
    folder_id: Option<String>,
    page_ids: Option<Vec<String>>,
) -> Result<String, String> {
    let fid = folder_id.as_deref();
    let pids = page_ids.as_deref();
    // `done` = file tertulis; `skipped` = id diminta tapi hilang;
    // `skipped_names` = halaman ada tapi belum diterjemahkan (nama file).
    let finish = |done: usize, skipped: usize, skipped_names: &[String]| {
        if skipped == 0 && skipped_names.is_empty() {
            return Ok(path.clone());
        }
        let mut msg = format!("{path} ({done} halaman");
        if skipped > 0 {
            msg.push_str(&format!(", {skipped} dilewati"));
        }
        if !skipped_names.is_empty() {
            msg.push_str(", belum diterjemahkan: ");
            msg.push_str(&describe_skipped(skipped_names));
        }
        msg.push(')');
        Ok(msg)
    };
    match format.as_str() {
        "json" => {
            let (bytes, skipped, skipped_names) = export_json_bytes(&state, &project_id, fid, pids)?;
            let done: usize = serde_json::from_slice::<Vec<serde_json::Value>>(&bytes)
                .map_err(|e| e.to_string())?
                .len();
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
            finish(done, skipped, &skipped_names)
        }
        "png" => {
            let (pages, skipped, skipped_names) = collect_pages(&state, &project_id, fid, pids)?;
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            for (name, png) in &pages {
                let f = std::path::Path::new(&path).join(name);
                std::fs::write(f, png).map_err(|e| e.to_string())?;
            }
            finish(pages.len(), skipped, &skipped_names)
        }
        "cbz" => {
            let (bytes, skipped, skipped_names) = export_cbz_bytes(&state, &project_id, fid, pids)?;
            std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
            // Jumlah entri = jumlah halaman ter-render (baca kembali dari zip).
            finish(count_cbz_entries(&bytes).map_err(|e| e.to_string())?, skipped, &skipped_names)
        }
        // M5-3: PSD layer-per-bubble via psd::render_page_psd.
        "psd" => {
            let store = crate::error::lock_or_err(&state.store)?;
            let proj = store
                .projects
                .get(&project_id)
                .ok_or_else(|| crate::error::AppError::project_not_found(&project_id))?
                .clone();
            drop(store);
            let (pages, skipped) = select_pages(&proj, fid, pids).map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            let mut n = 0usize;
            let mut skipped_names = Vec::new();
            for pg in pages.iter() {
                let Some(t) = pg.translation.clone() else {
                    skipped_names.push(file_name(&pg.path));
                    continue;
                };
                n += 1;
                let page_bytes =
                    std::fs::read(&pg.path).map_err(|e| format!("read {}: {e}", pg.path))?;
                let shapes = shapes_for_export(&t, &pg.bubbles);
                let psd = crate::render::psd::render_page_psd(&page_bytes, &t, &shapes)?;
                std::fs::write(
                    std::path::Path::new(&path).join(format!("{:03}.psd", n)),
                    psd,
                )
                .map_err(|e| e.to_string())?;
            }
            if n == 0 {
                return Err("belum ada halaman terjemahan untuk export".to_string());
            }
            finish(n, skipped, &skipped_names)
        }
        other => Err(format!("format unknown: {other} (json|png|cbz|psd)")),
    }
}
