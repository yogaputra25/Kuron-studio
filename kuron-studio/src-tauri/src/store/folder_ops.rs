//! Folder operations: create_folder, rename_folder, delete_folder, clean_pages, move_pages.
//! Dipindah dari `commands/project.rs` — tanpa perubahan perilaku.

use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::domain::{ChapterFolder, Page};
use crate::error::AppError;
use crate::store::ProjectStore;

/// Halaman bisa dibaca = file ada dan decoder bisa ambil dimensi.
fn is_page_readable(path: &str) -> bool {
    crate::commands::import::page_from_file(Path::new(path)).is_some()
}

/// Guard hapus file fisik: hanya file di bawah store dir (hasil ekstraksi
/// CBZ milik app). File asli user di luar store dir TIDAK pernah dihapus.
fn is_inside_store(store_dir: &Path, path: &Path) -> bool {
    match (path.canonicalize(), store_dir.canonicalize()) {
        (Ok(p), Ok(base)) => p.starts_with(base),
        _ => false,
    }
}

pub fn create_folder_inner(
    store: &mut ProjectStore,
    project_id: &str,
    name: &str,
    parent_id: Option<&str>,
) -> Result<ChapterFolder, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::parse("folder name is empty"));
    }
    let project = store.get_mut(project_id)?;
    let parent = parent_id.filter(|s| !s.is_empty());
    if let Some(p) = parent {
        if project.folder(p).is_none() {
            return Err(AppError::parent_folder_not_found(p));
        }
    }
    // Unik di antara saudara kandung (parent sama), bukan global.
    if project.folders.iter().any(|f| f.name == name && f.parent_id.as_deref() == parent) {
        return Err(AppError::parse(format!("folder sudah ada: {name}")));
    }
    let folder = match parent {
        Some(p) => ChapterFolder::new_child(name, p.to_string()),
        None => ChapterFolder::new(name),
    };
    let out = folder.clone();
    project.folders.push(folder);
    project.folders.sort_by(|a, b| a.name.cmp(&b.name));
    project.touch();
    store.save()?;
    Ok(out)
}

pub fn rename_folder_inner(
    store: &mut ProjectStore,
    project_id: &str,
    folder_id: &str,
    name: &str,
) -> Result<ChapterFolder, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::parse("folder name is empty"));
    }
    let project = store.get_mut(project_id)?;
    let parent_of_target = project.folder(folder_id).and_then(|f| f.parent_id.clone());
    if project.folders.iter().any(|f| f.name == name && f.id != folder_id && f.parent_id == parent_of_target) {
        return Err(AppError::parse(format!("folder sudah ada: {name}")));
    }
    let folder = project.folder_mut(folder_id).ok_or_else(|| AppError::folder_not_found(folder_id))?;
    folder.name = name;
    let out = folder.clone();
    project.folders.sort_by(|a, b| a.name.cmp(&b.name));
    project.touch();
    store.save()?;
    Ok(out)
}

pub fn delete_folder_inner(
    store: &mut ProjectStore,
    project_id: &str,
    folder_id: &str,
) -> Result<(), AppError> {
    let store_dir = store.dir.clone();
    let doomed: Vec<String>;
    {
        let project = store.get_mut(project_id)?;
        if project.folders.len() <= 1 {
            return Err(AppError::parse("tidak bisa hapus folder terakhir; hapus project saja"));
        }
        if project.folder(folder_id).is_none() {
            return Err(AppError::folder_not_found(folder_id));
        }
        // Hapus subtree: folder + semua keturunan + halaman mereka.
        let kill: Vec<String> = project.subtree_ids(folder_id);
        let mut gone = Vec::new();
        project.folders.retain(|f| {
            if kill.contains(&f.id) {
                gone.extend(f.pages.iter().map(|p| p.path.clone()));
                false
            } else { true }
        });
        doomed = gone;
        project.touch();
    }
    let mut removed_files = 0;
    for p in &doomed {
        let path = Path::new(p);
        if is_inside_store(&store_dir, path) && path.is_file() && fs::remove_file(path).is_ok() {
            removed_files += 1;
        }
    }
    let _ = removed_files;
    store.save()?;
    Ok(())
}

/// Buang halaman yang file-nya hilang / tidak bisa di-decode — mis. sidecar
/// `._*.jpg` dari arsip macOS yang terlanjur masuk projects.json.
/// File fisik ikut dihapus HANYA kalau ada di dalam store dir.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub removed: usize,
    pub kept: usize,
    pub removed_files: usize,
}

pub fn clean_pages_inner(store: &mut ProjectStore, project_id: &str) -> Result<CleanResult, AppError> {
    clean_pages_inner_scoped(store, project_id, None)
}

pub fn clean_pages_inner_scoped(store: &mut ProjectStore, project_id: &str, folder_id: Option<&str>) -> Result<CleanResult, AppError> {
    let store_dir = store.dir.clone();
    let mut doomed = Vec::new();
    let kept: usize;
    {
        let project = store.get_mut(project_id)?;
        if let Some(want) = folder_id.filter(|s| !s.is_empty()) {
            let folder = project.folder_mut(want).ok_or_else(|| AppError::folder_not_found(want))?;
            folder.pages.retain(|pg| {
                if is_page_readable(&pg.path) { true } else { doomed.push(pg.path.clone()); false }
            });
            kept = folder.pages.len();
        } else {
            for folder in project.folders.iter_mut() {
                folder.pages.retain(|pg| {
                    if is_page_readable(&pg.path) { true } else { doomed.push(pg.path.clone()); false }
                });
            }
            kept = project.folders.iter().map(|f| f.pages.len()).sum();
        }
        project.touch();
    }
    let removed = doomed.len();
    let mut removed_files = 0;
    for p in &doomed {
        let path = Path::new(p);
        if is_inside_store(&store_dir, path) && path.is_file() && fs::remove_file(path).is_ok() {
            removed_files += 1;
        }
    }
    store.save()?;
    Ok(CleanResult { removed, kept, removed_files })
}

/// Pindah halaman antar folder dalam project yang sama (organize Mode A -> B).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveResult {
    pub moved: usize,
    pub skipped: usize,
}

pub fn move_pages_inner(
    store: &mut ProjectStore,
    project_id: &str,
    page_ids: &[String],
    folder_id: &str,
) -> Result<MoveResult, AppError> {
    let project = store.get_mut(project_id)?;
    if !project.folders.iter().any(|f| f.id == folder_id) {
        return Err(AppError::folder_not_found(folder_id));
    }
    let mut moving: Vec<Page> = Vec::new();
    for folder in project.folders.iter_mut() {
        if folder.id == folder_id { continue; }
        let mut keep = Vec::new();
        for pg in folder.pages.drain(..) {
            if page_ids.contains(&pg.id) { moving.push(pg); } else { keep.push(pg); }
        }
        folder.pages = keep;
    }
    // Id yang tidak ketemu di folder mana pun = unknown; yang sudah di tujuan
    // tidak ikut drain, jadi skipped = diminta − benar-benar pindah.
    let skipped = page_ids.len().saturating_sub(moving.len());
    if moving.is_empty() {
        return Err(AppError::parse(if skipped > 0 {
            "semua halaman sudah di folder tujuan"
        } else {
            "tidak ada halaman yang dipindah (id salah / sudah di folder tujuan)"
        }));
    }
    let moved = moving.len();
    let dest = project.folder_mut(folder_id).expect("checked");
    dest.pages.extend(moving);
    dest.pages.sort_by(|a, b| a.path.cmp(&b.path));
    project.touch();
    store.save()?;
    Ok(MoveResult { moved, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::domain::{Page, PageStatus, Project};

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

    fn tree_proj() -> Project {
        let mut proj = Project::new("t".to_string());
        proj.id = "proj".to_string();
        let root = proj.folders[0].id.clone();
        let mut ch = ChapterFolder::new_child("Ch 12".to_string(), root.clone());
        ch.id = "ch12".to_string();
        let mut b = ChapterFolder::new_child("Batch 1".to_string(), "ch12".to_string());
        b.id = "b1".to_string();
        proj.folders.push(ch);
        proj.folders.push(b);
        proj
    }

    #[test]
    fn clean_pages_purges_junk_only() {
        let dir = temp_dir("kuron-studio-clean");
        let mut store = ProjectStore::load(dir.clone()).unwrap();

        let good = dir.join("001.png");
        image::RgbImage::new(10, 10).save(&good).unwrap();
        let junk_a = dir.join("._001.jpg");
        fs::write(&junk_a, b"Mac OS X resource fork").unwrap();
        let junk_b = dir.join("corrupt.png");
        fs::write(&junk_b, b"bukan gambar valid sama sekali").unwrap();

        let mut proj = Project::new("test".to_string());
        proj.id = "proj".to_string();
        let f = &mut proj.folders[0];
        f.pages = vec![
            stub_page("p1", &good),
            stub_page("p2", &junk_a),
            stub_page("p3", &junk_b),
        ];
        store.projects.insert("proj".to_string(), proj);

        let res = clean_pages_inner(&mut store, "proj").unwrap();
        assert_eq!((res.removed, res.kept, res.removed_files), (2, 1, 2));
        assert!(good.exists(), "gambar valid tidak boleh terhapus");
        assert!(!junk_a.exists() && !junk_b.exists());

        let reloaded = ProjectStore::load(dir.clone()).unwrap();
        let pages = &reloaded.projects.get("proj").unwrap().folders[0].pages;
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].id, "p1");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clean_pages_keeps_files_outside_store() {
        let outside_dir = temp_dir("kuron-studio-outside");
        fs::create_dir_all(&outside_dir).unwrap();
        let outside = outside_dir.join("mine.jpg");
        fs::write(&outside, b"bukan gambar, tapi file user").unwrap();

        let dir = temp_dir("kuron-studio-store");
        let mut store = ProjectStore::load(dir.clone()).unwrap();

        let mut proj = Project::new("test".to_string());
        proj.id = "proj".to_string();
        proj.folders[0].pages = vec![stub_page("p1", &outside)];
        store.projects.insert("proj".to_string(), proj);
        let res = clean_pages_inner(&mut store, "proj").unwrap();
        assert_eq!((res.removed, res.removed_files), (1, 0));
        assert!(outside.exists(), "file user di luar store dir harus aman");
        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&outside_dir).ok();
    }

    #[test]
    fn move_pages_reports_moved_and_skipped() {
        let dir = temp_dir("kuron-studio-move");
        let mut store = ProjectStore::load(dir).unwrap();
        let mut proj = tree_proj();
        proj.id = "proj".to_string();
        proj.folders.iter_mut().find(|f| f.id == "ch12").unwrap().pages =
            vec![stub_page("c1", Path::new("c1.jpg")), stub_page("c2", Path::new("c2.jpg"))];
        proj.folders.iter_mut().find(|f| f.id == "b1").unwrap().pages =
            vec![stub_page("b1p", Path::new("b1p.jpg"))];
        store.projects.insert("proj".to_string(), proj);

        // Parsial: 1 pindah, 1 sudah di tujuan, 1 id asing.
        let r = move_pages_inner(
            &mut store,
            "proj",
            &["c1".to_string(), "b1p".to_string(), "ghost".to_string()],
            "b1",
        )
        .unwrap();
        assert_eq!((r.moved, r.skipped), (1, 2));
        let dest = store.projects["proj"].folder("b1").unwrap();
        assert!(dest.pages.iter().any(|p| p.id == "c1"));

        // Full overlap = tolak sebelum tulis.
        let err = move_pages_inner(&mut store, "proj", &["c1".to_string()], "b1").unwrap_err();
        assert!(err.to_string().contains("sudah di folder tujuan"));
    }
}
