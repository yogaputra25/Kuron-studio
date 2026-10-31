//! Export packing: select_pages, collect_pages, export_json_bytes, export_cbz_bytes, count_cbz_entries.
//! Dipindah dari `commands/export.rs` — tanpa perubahan perilaku.

use tauri::State;

use crate::error::AppError;
use crate::render::page::render_page_png;
use crate::translation::PageTranslation;
use crate::AppState;

/// Hasil koleksi halaman + jumlah id terpilih yang hilang (skip, bukan gagal).
pub type CollectedPages = (Vec<(String, Vec<u8>)>, usize);

/// Satu jalur koleksi halaman untuk semua format (export-scope D2).
/// Prioritas: `page_ids` eksplisit > `folder_id` > semua. Id hilang → skip + hitung.
/// `skipped` = id diminta tapi tak ada di store (bukan halaman tanpa terjemahan —
/// itu tetap diskip diam-diam seperti sebelumnya oleh tiap jalur render).
pub fn select_pages(
    proj: &crate::domain::Project,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<(Vec<crate::domain::Page>, usize), AppError> {
    match page_ids.map(|ids| ids.iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>()) {
        Some(ids) if !ids.is_empty() => {
            let found = proj.pages_in_selection(&ids);
            let skipped = ids.len().saturating_sub(found.len());
            Ok((found, skipped))
        }
        _ => Ok((proj.pages_in_scope(folder_id)?, 0)),
    }
}

pub fn collect_pages(
    state: &State<'_, AppState>,
    project_id: &str,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<CollectedPages, AppError> {
    let store = crate::error::lock_or_err(&state.store)?;
    let proj = store
        .projects
        .get(project_id)
        .ok_or_else(|| AppError::project_not_found(project_id))?
        .clone();
    drop(store);
    let (pages, skipped) = select_pages(&proj, folder_id, page_ids)?;
    let mut out = Vec::new();
    for (i, pg) in pages.iter().enumerate() {
        let Some(t) = pg.translation.clone() else {
            continue;
        };
        let page_bytes =
            std::fs::read(&pg.path).map_err(|e| AppError::io(format!("read {}: {e}", pg.path)))?;
        let shapes = crate::render::page::shapes_for_export(&t, &pg.bubbles);
        let png = render_page_png(&page_bytes, &t, &shapes)?;
        out.push((format!("{:03}.png", i + 1), png));
    }
    if out.is_empty() {
        return Err(AppError::parse("belum ada halaman terjemahan untuk export"));
    }
    Ok((out, skipped))
}

pub fn export_json_bytes(
    state: &State<'_, AppState>,
    project_id: &str,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<(Vec<u8>, usize), String> {
    let store = crate::error::lock_or_err(&state.store)?;
    let proj = store
        .projects
        .get(project_id)
        .ok_or_else(|| crate::error::AppError::project_not_found(project_id))?
        .clone();
    let (pages, skipped) = select_pages(&proj, folder_id, page_ids)?;
    let ts: Vec<&PageTranslation> = pages.iter().filter_map(|p| p.translation.as_ref()).collect();
    Ok((serde_json::to_vec_pretty(&ts).map_err(|e| e.to_string())?, skipped))
}

pub fn export_cbz_bytes(
    state: &State<'_, AppState>,
    project_id: &str,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<(Vec<u8>, usize), String> {
    let (pages, skipped) = collect_pages(state, project_id, folder_id, page_ids)?;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, png) in &pages {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        zip.write_all(png).map_err(|e| e.to_string())?;
    }
    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok((cursor.into_inner(), skipped))
}

/// Jumlah entri file dalam arsip CBZ (tanpa mengekstrak).
pub fn count_cbz_entries(bytes: &[u8]) -> Result<usize, AppError> {
    let zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    Ok(zip.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// export-scope 2.3: seleksi subset per format — satu jalur untuk semua.
    /// Id hilang → skip + hitung; urutan = urutan input; folder scope tak berubah.
    #[test]
    fn select_pages_subset_skips_missing() {
        use crate::domain::{ChapterFolder, Page, PageStatus, Project};
        let mk = |id: &str, folder: &str| Page {
            id: id.into(),
            path: format!("{folder}/{id}.png"),
            width: 100,
            height: 100,
            status: PageStatus::Idle,
            bubbles: Vec::new(),
            translation: None,
        };
        let mut proj = Project::new("c".into());
        proj.folders = vec![
            ChapterFolder { id: "f1".into(), name: "A".into(), pages: vec![mk("b", "A"), mk("a", "A")], created_at: 0, parent_id: None },
            ChapterFolder { id: "f2".into(), name: "B".into(), pages: vec![mk("c", "B")], created_at: 0, parent_id: None },
        ];
        // Subset eksplisit: urutan input dipertahankan, yang hilang diskip + dihitung.
        let (got, skipped) = select_pages(&proj, None, Some(&["c".to_string(), "b".to_string(), "gone".to_string()])).unwrap();
        assert_eq!(got.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["c", "b"]);
        assert_eq!(skipped, 1);
        // Folder scope lama tetap jalan (None = semua, tanpa skipped).
        let (all, skipped) = select_pages(&proj, None, None).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(skipped, 0);
        let (sub, skipped) = select_pages(&proj, Some("f2"), None).unwrap();
        assert_eq!(sub.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["c"]);
        assert_eq!(skipped, 0);
    }
}
