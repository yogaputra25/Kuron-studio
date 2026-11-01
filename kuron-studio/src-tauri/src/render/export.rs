//! Export packing: select_pages, collect_pages, export_json_bytes, export_cbz_bytes, count_cbz_entries.
//! Dipindah dari `commands/export.rs` — tanpa perubahan perilaku.

use tauri::State;

use crate::error::AppError;
use crate::render::page::render_page_png;
use crate::AppState;

/// Hasil koleksi halaman + jumlah id terpilih yang hilang + nama file yang
/// dilewati karena belum diterjemahkan (lapor, bukan gagal).
pub type CollectedPages = (Vec<(String, Vec<u8>)>, usize, Vec<String>);

/// Satu jalur koleksi halaman untuk semua format (export-scope D2).
/// Prioritas: `page_ids` eksplisit > `folder_id` > semua. Id hilang → skip + hitung.
/// Urutan SELALU urutan cerita (sort by path) — termasuk seleksi eksplisit:
/// urutan klik tidak dibawa ke file export agar CBZ/PNG berurutan dibaca.
pub fn select_pages(
    proj: &crate::domain::Project,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<(Vec<crate::domain::Page>, usize), AppError> {
    let (mut pages, skipped) = match page_ids.map(|ids| ids.iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>()) {
        Some(ids) if !ids.is_empty() => {
            let found = proj.pages_in_selection(&ids);
            let skipped = ids.len().saturating_sub(found.len());
            (found, skipped)
        }
        _ => (proj.pages_in_scope(folder_id)?, 0),
    };
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((pages, skipped))
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
    let mut skipped_names = Vec::new();
    let mut n = 0usize;
    for pg in &pages {
        let Some(t) = pg.translation.clone() else {
            skipped_names.push(file_name(&pg.path));
            continue;
        };
        n += 1;
        let page_bytes =
            std::fs::read(&pg.path).map_err(|e| AppError::io(format!("read {}: {e}", pg.path)))?;
        let shapes = crate::render::page::shapes_for_export(&t, &pg.bubbles);
        let png = render_page_png(&page_bytes, &t, &shapes)?;
        out.push((format!("{:03}.png", n), png));
    }
    if out.is_empty() {
        return Err(AppError::parse("belum ada halaman terjemahan untuk export".to_string()));
    }
    Ok((out, skipped, skipped_names))
}

/// Nama file dari path (`a/b/c.jpg` → `c.jpg`), untuk pesan skip.
pub fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Ringkasan nama halaman dilewati: penuh bila ≤5, `dan N lainnya` bila lebih.
pub fn describe_skipped(names: &[String]) -> String {
    const MAX: usize = 5;
    if names.len() <= MAX {
        return names.join(", ");
    }
    format!("{}, dan {} lainnya", names[..MAX].join(", "), names.len() - MAX)
}

pub fn export_json_bytes(
    state: &State<'_, AppState>,
    project_id: &str,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<(Vec<u8>, usize, Vec<String>), String> {
    let store = crate::error::lock_or_err(&state.store)?;
    let proj = store
        .projects
        .get(project_id)
        .ok_or_else(|| crate::error::AppError::project_not_found(project_id))?
        .clone();
    let (pages, skipped) = select_pages(&proj, folder_id, page_ids)?;
    let mut skipped_names = Vec::new();
    let mut ts = Vec::new();
    for p in &pages {
        match p.translation.as_ref() {
            Some(t) => ts.push(t),
            None => skipped_names.push(file_name(&p.path)),
        }
    }
    Ok((serde_json::to_vec_pretty(&ts).map_err(|e| e.to_string())?, skipped, skipped_names))
}

pub fn export_cbz_bytes(
    state: &State<'_, AppState>,
    project_id: &str,
    folder_id: Option<&str>,
    page_ids: Option<&[String]>,
) -> Result<(Vec<u8>, usize, Vec<String>), String> {
    let (pages, skipped, skipped_names) = collect_pages(state, project_id, folder_id, page_ids)?;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, png) in &pages {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        zip.write_all(png).map_err(|e| e.to_string())?;
    }
    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok((cursor.into_inner(), skipped, skipped_names))
}

/// Jumlah entri file dalam arsip CBZ (tanpa mengekstrak).
pub fn count_cbz_entries(bytes: &[u8]) -> Result<usize, AppError> {
    let zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    Ok(zip.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// export-scope: seleksi subset per format — satu jalur untuk semua.
    /// Id hilang → skip + hitung; urutan SELALU urutan cerita (sort by path),
    /// termasuk seleksi eksplisit (urutan klik tidak dibawa ke file).
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
        // Subset eksplisit acak ["c","b"]: dinormalisasi ke urutan path (cerita).
        let (got, skipped) = select_pages(&proj, None, Some(&["c".to_string(), "b".to_string(), "gone".to_string()])).unwrap();
        assert_eq!(got.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["b", "c"]);
        assert_eq!(skipped, 1);
        // Folder scope lama tetap jalan (None = semua, tanpa skipped).
        let (all, skipped) = select_pages(&proj, None, None).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(skipped, 0);
        let (sub, skipped) = select_pages(&proj, Some("f2"), None).unwrap();
        assert_eq!(sub.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["c"]);
        assert_eq!(skipped, 0);
    }

    #[test]
    fn file_name_mengambil_segmen_terakhir() {
        assert_eq!(file_name("a/b/c.jpg"), "c.jpg");
        assert_eq!(file_name("solo.png"), "solo.png");
    }

    #[test]
    fn describe_skipped_penuh_sampai_lima_lalu_ringkas() {
        let v = |n: usize| (0..n).map(|i| format!("p{i:02}.jpg")).collect::<Vec<_>>();
        assert_eq!(describe_skipped(&[]), "");
        assert_eq!(describe_skipped(&v(2)), "p00.jpg, p01.jpg");
        assert_eq!(
            describe_skipped(&v(7)),
            "p00.jpg, p01.jpg, p02.jpg, p03.jpg, p04.jpg, dan 2 lainnya"
        );
    }
}
