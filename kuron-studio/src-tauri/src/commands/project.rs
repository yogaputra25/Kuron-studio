use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use uuid::Uuid;

use crate::AppState;
use crate::bubble::BubbleBox;

/// Ekstensi yang diterima import. Dipakai bareng `IMAGE_FORMATS`: tiap
/// entri di sini WAJIB ada di sana dan wajib lolos round-trip test
/// `image_format_support_is_real` — kalau dekoder ternyata tidak bisa
/// baca formatnya, import diam-diam jadi halaman rusak.
///
/// TGA sengaja tidak ada: tidak punya magic bytes, jadi `image_dimensions`
/// tidak bisa menentukan format dari header. AVIF juga tidak — crate
/// `image` 0.25 hanya punya *encoder* AVIF, decoder-nya di feature
/// terpisah `avif-native` yang tidak aktif.
const IMAGE_EXTS: [&str; 10] = [
    "jpg", "jpeg", "jpe", "jfif", "png", "webp", "bmp", "gif", "tiff", "tif",
];

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub status: PageStatus,
    /// Bubble hasil detect/edit canvas (original px). Kosong = belum detect.
    #[serde(default)]
    pub bubbles: Vec<BubbleBox>,
    #[serde(default)]
    pub translation: Option<crate::translation::PageTranslation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PageStatus {
    #[default]
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "detecting")]
    Detecting,
    #[serde(rename = "detected")]
    Detected,
    #[serde(rename = "noBubbles")]
    NoBubbles,
    #[serde(rename = "translating")]
    Translating,
    #[serde(rename = "translated")]
    Translated,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub pages: Vec<Page>,
    pub skipped: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStore {
    #[serde(skip)]
    dir: PathBuf,
    pub projects: HashMap<String, Project>,
}

impl ProjectStore {
    pub fn load(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join("projects.json");
        if !file.exists() {
            return Ok(Self {
                dir,
                projects: HashMap::new(),
            });
        }
        let raw = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let mut store: ProjectStore =
            serde_json::from_str(&raw).map_err(|e| format!("projects.json corrupt: {e}"))?;
        store.dir = dir;
        Ok(store)
    }

    fn save(&self) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(self.dir.join("projects.json"), raw).map_err(|e| e.to_string())
    }

    pub(crate) fn save_public(&self) -> Result<(), String> {
        self.save()
    }

    fn get_mut(&mut self, project_id: &str) -> Result<&mut Project, String> {
        self.projects
            .get_mut(project_id)
            .ok_or_else(|| format!("project not found: {project_id}"))
    }
}

/// Sidecar AppleDouble (`._0001.jpg`) — resource fork yang dibikin macOS saat
/// zip/cbz. Namanya berakhir `.jpg` tapi isinya metadata, bukan gambar, jadi
/// `image::load_from_memory` gagal dengan "format could not be determined".
fn is_appledouble(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("._"))
}

/// `true` untuk entry zip yang cuma metadata (`__MACOSX/…`, `._*`).
fn is_archive_metadata(name: &str) -> bool {
    name.split(['/', '\\'])
        .any(|seg| seg.eq_ignore_ascii_case("__MACOSX") || seg.starts_with("._"))
}

/// Guard zip-slip: reject nama entry yang absolut atau keluar dari `base`.
fn safe_join(base: &Path, name: &str) -> Option<PathBuf> {
    let mut out = base.to_path_buf();
    for comp in Path::new(name).components() {
        match comp {
            std::path::Component::Normal(c) => out.push(c),
            std::path::Component::CurDir => {}
            // ParentDir / RootDir / Prefix = path traversal atau absolut.
            _ => return None,
        }
    }
    out.starts_with(base).then_some(out)
}

pub fn is_image_file(path: &Path) -> bool {
    if is_appledouble(path) {
        return false;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => IMAGE_EXTS.contains(&ext.to_ascii_lowercase().as_str()),
        None => false,
    }
}

fn is_archive(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_ascii_lowercase().as_str(), "zip" | "cbz"),
        None => false,
    }
}

/// `None` = file bukan gambar yang bisa di-decode (sidecar `._*`, korup,
/// 0 byte). Import menghitungnya sebagai skipped supaya projects.json tidak
/// pernah menerima halaman rusak.
fn page_from_file(path: &Path) -> Option<Page> {
    let (width, height) = image::image_dimensions(path).ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some(Page {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        width,
        height,
        status: PageStatus::Idle,
        bubbles: Vec::new(),
        translation: None,
    })
}

/// Expand one dropped/selected path into image files.
/// Dir -> sorted image children. zip/cbz -> extracted under store dir, then sorted.
/// Single image -> itself. Anything else counts as skipped.
fn expand_input(path: &Path, extract_dir: &Path) -> Result<(Vec<PathBuf>, usize), String> {
    if path.is_dir() {
        let mut files: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect();
        let total = files.len();
        files.retain(|p| is_image_file(p));
        let skipped = total - files.len();
        files.sort();
        return Ok((files, skipped));
    }
    if path.is_file() {
        if is_archive(path) {
            return extract_archive(path, extract_dir);
        }
        if is_image_file(path) {
            return Ok((vec![path.to_path_buf()], 0));
        }
        return Ok((vec![], 1));
    }
    Err(format!("path not found: {}", path.display()))
}

fn extract_archive(path: &Path, extract_dir: &Path) -> Result<(Vec<PathBuf>, usize), String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    fs::create_dir_all(extract_dir).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    let mut skipped = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if entry.is_dir() {
            continue;
        }
        // `__MACOSX/…` + `._*` = metadata, bukan halaman. Count sebagai skipped
        // supaya user tahu arsipnya membawa file junk.
        if is_archive_metadata(&name) {
            skipped += 1;
            continue;
        }
        let Some(out) = safe_join(extract_dir, &name) else {
            skipped += 1;
            continue;
        };
        if !is_image_file(Path::new(&name)) {
            skipped += 1;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out_file = fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
        files.push(out);
    }
    files.sort();
    Ok((files, skipped))
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
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
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        pages: Vec::new(),
    };
    store.projects.insert(project.id.clone(), project.clone());
    store.save()?;
    Ok(project)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_project(project_id: String, state: State<'_, AppState>) -> Result<Project, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .projects
        .get(&project_id)
        .cloned()
        .ok_or_else(|| format!("project not found: {project_id}"))
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_pages(
    project_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let extract_dir = store.dir.join(format!("{project_id}_extracted"));
    let mut all_files: Vec<PathBuf> = Vec::new();
    let mut skipped = 0;
    for raw in &paths {
        let (mut files, skip) = expand_input(Path::new(raw), &extract_dir)?;
        all_files.append(&mut files);
        skipped += skip;
    }
    let mut pages: Vec<Page> = Vec::with_capacity(all_files.len());
    for f in &all_files {
        match page_from_file(f) {
            Some(pg) => pages.push(pg),
            None => skipped += 1,
        }
    }
    let project = store.get_mut(&project_id)?;
    project.pages.extend(pages.clone());
    project.pages.sort_by(|a, b| a.path.cmp(&b.path));
    store.save()?;
    Ok(ImportResult { pages, skipped })
}

/// Halaman bisa dibaca = file ada dan decoder bisa ambil dimensi.
fn is_page_readable(path: &str) -> bool {
    page_from_file(Path::new(path)).is_some()
}

/// Guard hapus file fisik: hanya file di bawah store dir (hasil ekstraksi
/// CBZ milik app). File asli user di luar store dir TIDAK pernah dihapus.
fn is_inside_store(store_dir: &Path, path: &Path) -> bool {
    match (path.canonicalize(), store_dir.canonicalize()) {
        (Ok(p), Ok(base)) => p.starts_with(base),
        _ => false,
    }
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

pub fn clean_pages_inner(store: &mut ProjectStore, project_id: &str) -> Result<CleanResult, String> {
    let store_dir = store.dir.clone();
    let mut doomed = Vec::new();
    {
        let project = store.get_mut(project_id)?;
        project.pages.retain(|pg| {
            if is_page_readable(&pg.path) {
                true
            } else {
                doomed.push(pg.path.clone());
                false
            }
        });
    }
    let removed = doomed.len();
    let kept = store.projects[project_id].pages.len();
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

#[tauri::command(rename_all = "snake_case")]
pub fn clean_pages(project_id: String, state: State<'_, AppState>) -> Result<CleanResult, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    clean_pages_inner(&mut store, &project_id)
}

/// Hapus project + folder ekstraksi miliknya. File gambar asli user di luar
/// store dir tidak disentuh; cache terjemahan tetap (bersih manual).
#[tauri::command(rename_all = "snake_case")]
pub fn delete_project(project_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let removed = store.projects.remove(&project_id);
    if removed.is_none() {
        return Err(format!("project not found: {project_id}"));
    }
    store.save()?;
    let extract = store.dir.join(format!("{project_id}_extracted"));
    if extract.is_dir() {
        fs::remove_dir_all(&extract).map_err(|e| format!("hapus folder ekstraksi: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unik per test — `SystemTime` bisa kembalikan nanosecond yang sama.
    fn temp_dir(tag: &str) -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{tag}-{}-{n}", std::process::id()))
    }

    fn stub_page(id: &str, path: &Path) -> Page {
        Page {
            id: id.to_string(),
            path: path.to_string_lossy().to_string(),
            width: 0,
            height: 0,
            status: PageStatus::Idle,
            bubbles: Vec::new(),
            translation: None,
        }
    }

    #[test]
    fn image_ext_case_insensitive() {
        assert!(is_image_file(Path::new("p.JPG")));
        assert!(is_image_file(Path::new("p.webp")));
        assert!(!is_image_file(Path::new("p.txt")));
        assert!(!is_image_file(Path::new("noext")));
    }

    /// Guard utama: daftar import tidak boleh lebih lebar dari kemampuan
    /// dekoder. Kalau crate `image` kehilangan feature, test ini gagal —
    /// bukan user yang menemukan lewat "image format could not be determined"
    /// di tengah project.
    #[test]
    fn image_format_support_is_real() {
        assert_eq!(
            IMAGE_EXTS.len(),
            IMAGE_FORMATS.len(),
            "IMAGE_EXTS / IMAGE_FORMATS harus sinkron"
        );
        for (ext, format) in IMAGE_FORMATS {
            assert!(
                IMAGE_EXTS.contains(&ext),
                "ext `{ext}` ada di IMAGE_FORMATS tapi hilang dari IMAGE_EXTS"
            );
            let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
                32,
                24,
                image::Rgb([200, 40, 90]),
            ));
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), format)
                .unwrap_or_else(|e| panic!("encode `{ext}` gagal: {e}"));
            let dec = image::load_from_memory(&buf)
                .unwrap_or_else(|e| panic!("decode `{ext}` gagal: {e}"));
            assert_eq!(
                (dec.width(), dec.height()),
                (32, 24),
                "dimensi `{ext}` salah setelah round-trip"
            );
            assert!(
                is_image_file(Path::new(&format!("page.{ext}"))),
                "ext `{ext}` tidak lolos is_image_file"
            );
        }
        for ext in IMAGE_EXTS {
            assert!(
                IMAGE_FORMATS.iter().any(|(e, _)| *e == ext),
                "ext `{ext}` ada di IMAGE_EXTS tapi tidak ada di IMAGE_FORMATS"
            );
        }
    }

    /// Format yang sengaja DITOLAK import, dengan alasannya.
    #[test]
    fn rejected_formats_stay_rejected() {
        // TGA: tanpa magic bytes, `image_dimensions` buta header.
        assert!(!is_image_file(Path::new("p.tga")));
        // HEIC/HEIF/JXL: tidak ada di crate image 0.25 sama sekali.
        assert!(!is_image_file(Path::new("p.heic")));
        assert!(!is_image_file(Path::new("p.heif")));
        assert!(!is_image_file(Path::new("p.jxl")));
        // AVIF: encoder saja, decoder butuh feature `avif-native`.
        assert!(!is_image_file(Path::new("p.avif")));
    }

    /// Regresi: `._0001.jpg` dari `__MACOSX/` lolos filter ext sebelum fix.
    #[test]
    fn appledouble_rejected() {
        assert!(!is_image_file(Path::new("ch/__MACOSX/ch/._0001.jpg")));
        assert!(!is_image_file(Path::new("._0001.jpg")));
        assert!(!is_image_file(Path::new("._cover.png")));
    }

    #[test]
    fn archive_metadata_detected() {
        assert!(is_archive_metadata("__MACOSX/ch/._0001.jpg"));
        assert!(is_archive_metadata("__MACOSX/"));
        assert!(is_archive_metadata("._0001.jpg"));
        assert!(is_archive_metadata("ch\\__MACOSX\\ch\\._1.jpg"));
        assert!(!is_archive_metadata("ch/0001.jpg"));
        assert!(!is_archive_metadata("page01.jpg"));
    }

    /// Zip-slip: `../` atau path absolut tidak boleh keluar dari base.
    #[test]
    fn safe_join_blocks_traversal() {
        let base = Path::new("/tmp/extract");
        assert_eq!(
            safe_join(base, "ch/001.jpg"),
            Some(PathBuf::from("/tmp/extract/ch/001.jpg"))
        );
        assert_eq!(
            safe_join(base, "./ch/001.jpg"),
            Some(PathBuf::from("/tmp/extract/ch/001.jpg"))
        );
        assert_eq!(safe_join(base, "../../etc/passwd"), None);
        assert_eq!(safe_join(base, "/etc/passwd"), None);
    }

    /// Header AppleDouble bukan JPEG → `image_dimensions` gagal → di-skip.
    #[test]
    fn page_from_file_rejects_non_image() {
        let dir = temp_dir("kuron-studio-badimg");
        fs::create_dir_all(&dir).unwrap();
        let junk = dir.join("._0001.jpg");
        fs::write(&junk, b"\x00\x05\x16\x07\x00\x02\x00\x00Mac ").unwrap();
        assert!(page_from_file(&junk).is_none());

        let empty = dir.join("empty.jpg");
        fs::write(&empty, b"").unwrap();
        assert!(page_from_file(&empty).is_none());

        let real = dir.join("ok.png");
        let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            6,
            image::Rgb([1, 2, 3]),
        ));
        img.save(&real).unwrap();
        let pg = page_from_file(&real).expect("png valid");
        assert_eq!((pg.width, pg.height), (8, 6));
        fs::remove_dir_all(&dir).ok();
    }

    /// End-to-end: project berisi 1 gambar valid + 2 sampah → clean_pages
    /// buang 2 sisanya, dan file fisik di store dir ikut terhapus.
    #[test]
    fn clean_pages_purges_junk_only() {
        let dir = temp_dir("kuron-studio-clean");
        let mut store = ProjectStore::load(dir.clone()).unwrap();

        let good = dir.join("good.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            6,
            image::Rgb([1, 2, 3]),
        ))
        .save(&good)
        .unwrap();
        let junk_a = dir.join("._0001.jpg");
        let junk_b = dir.join("._0002.jpg");
        fs::write(&junk_a, b"\x00\x05\x16\x07\x00\x02\x00\x00Mac ").unwrap();
        fs::write(&junk_b, b"\x00\x05\x16\x07\x00\x02\x00\x00Mac ").unwrap();

        store.projects.insert(
            "proj".to_string(),
            Project {
                id: "proj".to_string(),
                name: "t".to_string(),
                pages: vec![stub_page("g", &good), stub_page("a", &junk_a), stub_page("b", &junk_b)],
            },
        );

        let res = clean_pages_inner(&mut store, "proj").unwrap();
        assert_eq!((res.removed, res.kept, res.removed_files), (2, 1, 2));
        assert!(good.exists(), "gambar valid tidak boleh terhapus");
        assert!(!junk_a.exists() && !junk_b.exists());
        let reloaded = ProjectStore::load(dir.clone()).unwrap();
        assert_eq!(reloaded.projects["proj"].pages.len(), 1);
        assert_eq!(reloaded.projects["proj"].pages[0].id, "g");
        fs::remove_dir_all(&dir).ok();
    }

    /// File asli user di luar store dir tidak boleh ikut terhapus.
    #[test]
    fn clean_pages_keeps_files_outside_store() {
        let outside_dir = temp_dir("kuron-studio-outside");
        fs::create_dir_all(&outside_dir).unwrap();
        let outside = outside_dir.join("mine.jpg");
        fs::write(&outside, b"\x00\x05\x16\x07Mac ").unwrap();
        let dir = temp_dir("kuron-studio-keep");
        let mut store = ProjectStore::load(dir.clone()).unwrap();
        store.projects.insert(
            "proj".to_string(),
            Project {
                id: "proj".to_string(),
                name: "t".to_string(),
                pages: vec![stub_page("x", &outside)],
            },
        );
        let res = clean_pages_inner(&mut store, "proj").unwrap();
        assert_eq!((res.removed, res.removed_files), (1, 0));
        assert!(outside.exists(), "file user di luar store dir harus aman");
        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&outside_dir).ok();
    }

    #[test]
    fn archive_detected() {
        assert!(is_archive(Path::new("ch.zip")));
        assert!(is_archive(Path::new("ch.CBZ")));
        assert!(!is_archive(Path::new("p.png")));
    }

    #[test]
    fn missing_path_errors() {
        let dir = std::env::temp_dir().join("kuron-studio-missing-probe");
        let res = expand_input(&dir.join("nope"), &dir);
        assert!(res.is_err());
    }

    #[test]
    fn bubbles_survive_save_reload() {
        // M1-12: save_bubbles → projects.json → load() → bubbles + status utuh.
        use crate::bubble::BubbleBox;
        let dir = std::env::temp_dir().join(format!(
            "kuron-studio-roundtrip-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut store = ProjectStore::load(dir.clone()).unwrap();
        let page = Page {
            id: "p1".to_string(),
            path: "p1.png".to_string(),
            width: 800,
            height: 1200,
            status: PageStatus::Detected,
            bubbles: vec![BubbleBox {
                x: 10,
                y: 20,
                w: 60,
                h: 40,
                confidence: 1.0,
                shape: Some(vec![[10, 20], [70, 20], [70, 60]]),
                kind: Some("freeform".to_string()),
                tail: Some(vec![[40, 60], [50, 90]]),
            }],
            translation: None,
        };
        store.projects.insert(
            "proj".to_string(),
            Project {
                id: "proj".to_string(),
                name: "t".to_string(),
                pages: vec![page],
            },
        );
        store.save().unwrap();
        let reloaded = ProjectStore::load(dir).unwrap();
        let pg = &reloaded.projects["proj"].pages[0];
        assert_eq!(pg.bubbles.len(), 1);
        assert_eq!(pg.bubbles[0].shape.as_ref().unwrap().len(), 3);
        assert_eq!(pg.bubbles[0].tail.as_ref().unwrap().len(), 2);
        assert_eq!(pg.status, PageStatus::Detected);
    }
}
