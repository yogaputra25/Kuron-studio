use std::fs;
use std::sync::OnceLock;
use tauri::State;

use crate::AppState;
use crate::error::{lock_or_err, AppError};

// Re-ekspor untuk modul lain (import, lib, FE contracts).
pub use crate::domain::{ChapterFolder, Page, PageStatus, Project, DEFAULT_FOLDER_NAME};
pub use crate::store::folder_ops::{
    clean_pages_inner, clean_pages_inner_scoped, create_folder_inner, delete_folder_inner,
    move_pages_inner, rename_folder_inner, CleanResult, MoveResult,
};
pub use crate::store::ProjectStore;
pub use crate::render::sample::create_sample_project_inner;

/// Salinan `src/lib/contracts/extensions.json` — SATU sumber kebenaran untuk
/// ekstensi import. Frontend (`types.ts`) impor file yang sama; jangan tambah
/// ekstensi di satu sisi saja.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ExtensionContracts {
    pub(crate) image: Vec<String>,
    pub(crate) archive: Vec<String>,
    pub(crate) pdf: Vec<String>,
}

static EXTENSIONS: OnceLock<ExtensionContracts> = OnceLock::new();

pub(crate) fn extensions() -> &'static ExtensionContracts {
    EXTENSIONS.get_or_init(|| {
        serde_json::from_str(include_str!("../../../src/lib/contracts/extensions.json"))
            .expect("contracts/extensions.json harus valid")
    })
}

/// Ekstensi yang diterima import — dibaca dari `extensions.json` (lihat
/// `extensions()` di atas). Dipakai bareng `IMAGE_FORMATS`: tiap entri di
/// sini WAJIB ada di sana dan wajib lolos round-trip test
/// `image_format_support_is_real` — kalau dekoder ternyata tidak bisa
/// baca formatnya, import diam-diam jadi halaman rusak.
pub(crate) fn image_exts() -> &'static [String] {
    &extensions().image
}

/// Dipakai test untuk membuktikan tiap `IMAGE_EXTS` benar-benar bisa
/// di-encode lalu di-decode oleh crate `image` yang aktif di build ini.
#[cfg(test)]
const IMAGE_FORMATS: [(&str, image::ImageFormat); 10] = [
    ("jpg", image::ImageFormat::Jpeg),
    ("jpeg", image::ImageFormat::Jpeg),
    ("jpe", image::ImageFormat::Jpeg),
    ("jfif", image::ImageFormat::Jpeg),
    ("png", image::ImageFormat::Png),
    ("webp", image::ImageFormat::WebP),
    ("bmp", image::ImageFormat::Bmp),
    ("gif", image::ImageFormat::Gif),
    ("tiff", image::ImageFormat::Tiff),
    ("tif", image::ImageFormat::Tiff),
];

#[tauri::command(rename_all = "snake_case")]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let store = lock_or_err(&state.store)?;
    let mut projects: Vec<Project> = store.projects.values().cloned().collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_project(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("project name is empty".to_string());
    }
    let mut store = lock_or_err(&state.store)?;
    let project = Project::new(name);
    store.projects.insert(project.id.clone(), project.clone());
    store.save()?;
    Ok(project)
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_sample_project(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    let mut store = lock_or_err(&state.store)?;
    Ok(create_sample_project_inner(&mut store, &name)?)
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_folder(
    project_id: String,
    name: String,
    state: State<'_, AppState>,
    parent_id: Option<String>,
) -> Result<ChapterFolder, String> {
    let mut store = lock_or_err(&state.store)?;
    Ok(create_folder_inner(&mut store, &project_id, &name, parent_id.as_deref())?)
}

#[tauri::command(rename_all = "snake_case")]
pub fn rename_folder(
    project_id: String,
    folder_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<ChapterFolder, String> {
    let mut store = lock_or_err(&state.store)?;
    Ok(rename_folder_inner(&mut store, &project_id, &folder_id, &name)?)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_folder(project_id: String, folder_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = lock_or_err(&state.store)?;
    Ok(delete_folder_inner(&mut store, &project_id, &folder_id)?)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_project(project_id: String, state: State<'_, AppState>) -> Result<Project, String> {
    let store = lock_or_err(&state.store)?;
    store
        .projects
        .get(&project_id)
        .cloned()
        .ok_or_else(|| AppError::project_not_found(&project_id).into())
}

#[tauri::command(rename_all = "snake_case")]
pub fn clean_pages(project_id: String, state: State<'_, AppState>, folder_id: Option<String>) -> Result<CleanResult, String> {
    let mut store = lock_or_err(&state.store)?;
    let out = clean_pages_inner_scoped(&mut store, &project_id, folder_id.as_deref())?;
    // grid-preview-perf 3.3: buang cache preview halaman yang sudah tak ada.
    let keep: Vec<String> = store
        .projects
        .get(&project_id)
        .map(|p| p.all_pages().iter().map(|pg| pg.path.clone()).collect())
        .unwrap_or_default();
    drop(store);
    if let Ok(db) = state.db.lock() {
        // Cache turunan boleh gagal diam-diam — clean utama sudah sukses.
        let _ = crate::preview_cache::preview_purge(&db, &keep);
    }
    Ok(out)
}

#[tauri::command(rename_all = "snake_case")]
pub fn move_pages(project_id: String, page_ids: Vec<String>, folder_id: String, state: State<'_, AppState>) -> Result<MoveResult, String> {
    let mut store = lock_or_err(&state.store)?;
    Ok(move_pages_inner(&mut store, &project_id, &page_ids, &folder_id)?)
}

/// Hapus project + folder ekstraksi miliknya. File gambar asli user di luar
/// store dir tidak disentuh; cache terjemahan tetap (bersih manual).
#[tauri::command(rename_all = "snake_case")]
pub fn delete_project(project_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = lock_or_err(&state.store)?;
    let removed = store.projects.remove(&project_id);
    if removed.is_none() {
        return Err(AppError::project_not_found(&project_id).into());
    }
    store.save()?;
    let extract = store.dir.join(format!("{project_id}_extracted"));
    if extract.is_dir() {
        fs::remove_dir_all(&extract).map_err(|e| format!("hapus folder ekstraksi: {e}"))?;
    }
    let sample = store.dir.join(format!("{project_id}_sample"));
    if sample.is_dir() {
        fs::remove_dir_all(&sample).map_err(|e| format!("hapus folder sample: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn temp_dir(tag: &str) -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("{tag}-{n}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn stub_page(id: &str, path: &Path) -> Page {
        Page {
            id: id.to_string(),
            path: path.to_string_lossy().to_string(),
            width: 100,
            height: 100,
            status: PageStatus::Idle,
            bubbles: Vec::new(),
            translation: None,
        }
    }

    fn stub_project() -> Project {
        let dir = temp_dir("kuron-select");
        let mk = |id: &str| stub_page(id, &dir.join(format!("{id}.png")));
        Project {
            id: "p".into(),
            name: "c".into(),
            folders: vec![
                ChapterFolder {
                    id: "f1".into(),
                    name: "Ch1".into(),
                    pages: vec![mk("b"), mk("a")],
                    created_at: 0,
                    parent_id: None,
                },
                ChapterFolder {
                    id: "f2".into(),
                    name: "Ch2".into(),
                    pages: vec![mk("c")],
                    created_at: 0,
                    parent_id: None,
                },
            ],
            updated_at: 0,
            legacy_pages: Vec::new(),
        }
    }

    #[test]
    fn pages_in_selection_order_and_skips_missing() {
        let proj = stub_project();
        let got = proj.pages_in_selection(&["c".to_string(), "b".to_string(), "gone".to_string()]);
        assert_eq!(got.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["c", "b"]);
        let none = proj.pages_in_selection(&[]);
        assert!(none.is_empty());
    }

    #[test]
    fn image_format_support_is_real() {
        assert_eq!(
            image_exts().len(),
            IMAGE_FORMATS.len(),
            "IMAGE_FORMATS test harus punya pasangan 1:1 dengan image_exts()",
        );

        let img = image::RgbImage::new(4, 4);
        for &(ext, format) in &IMAGE_FORMATS {
            assert!(
                image_exts().contains(&ext.to_string()),
                "ekstensi {ext} tidak ada di image_exts()",
            );

            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, format)
                .unwrap_or_else(|e| panic!("gagal encode format {ext}: {e}"));
            let bytes = buf.into_inner();
            assert!(!bytes.is_empty(), "output encode {ext} kosong");

            let decoded = image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("gagal decode balik format {ext}: {e}"));
            assert_eq!(
                (decoded.width(), decoded.height()),
                (4, 4),
                "dimensi decode {ext} tidak cocok",
            );
        }
    }

    #[test]
    fn extensions_match_json_contract() {
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../../../src/lib/contracts/extensions.json"
        ))
        .expect("extensions.json harus valid JSON");
        let ext = extensions();
        let json_images: Vec<String> = v["image"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(ext.image, json_images);
        assert!(ext.image.contains(&"jpg".to_string()));
        assert!(ext.image.contains(&"png".to_string()));
    }

}
