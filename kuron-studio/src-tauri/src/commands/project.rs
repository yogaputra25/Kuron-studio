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

fn is_pdf(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("pdf"),
        None => false,
    }
}

/// Filter stream gambar — mengikuti urutan array `/Filter` (urutan = urutan
/// decode). Hanya DCT (JPEG pass-through) + Flate (inflate manual, karena
/// `decompress()` lopdf skip stream berfilter-DCT). JPX/CCITT/JBIG2 ditolak
/// jujur (tanpa decoder murni-Rust yang matang di repo ini).
fn pdf_image_filter_kind(dict: &lopdf::Dictionary) -> Result<Vec<String>, String> {
    let f = dict
        .get(b"Filter")
        .map_err(|_| "stream gambar tanpa /Filter".to_string())?;
    let names: Vec<String> = match f {
        lopdf::Object::Name(n) => vec![String::from_utf8_lossy(n).into_owned()],
        lopdf::Object::Array(arr) => arr
            .iter()
            .filter_map(|o| o.as_name().ok().map(|n| String::from_utf8_lossy(n).into_owned()))
            .collect(),
        _ => return Err("tipe /Filter tak dikenal".to_string()),
    };
    if names.is_empty() {
        return Err("daftar /Filter kosong".to_string());
    }
    Ok(names)
}

/// Inflate manual satu lapis Flate (zlib). `decompress()` lopdf dilewati
/// untuk stream berfilter-DCT — ia skip stream itu sepenuhnya.
fn inflate_flate(raw: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut dec = flate2::read::ZlibDecoder::new(raw);
    let mut out = Vec::new();
    dec.read_to_end(&mut out)
        .map_err(|e| format!("inflate Flate gagal: {e}"))?;
    Ok(out)
}

/// Tulis pixel mentah Flate (non-JPEG: Gray/Indexed/RGB) jadi PNG via `image`.
/// Mendukung 8 bit/komponen tanpa predictor; predictor PNG per-baris
/// didukung via decode_frame lopdf bila tersedia (fallback: tolak halaman).
fn write_flate_png(
    inflated: &[u8],
    width: u32,
    height: u32,
    bpc: u8,
    colorspace: &str,
    out: &Path,
) -> Result<(), String> {
    if bpc != 8 {
        return Err(format!("BPC {bpc} tak didukung (hanya 8)"));
    }
    let (channels, palette): (u8, Option<Vec<[u8; 3]>>) = if colorspace.contains("DeviceRGB") {
        (3, None)
    } else if colorspace.contains("DeviceGray") {
        (1, None)
    } else if colorspace.contains("Indexed") {
        // Palet melekat di dict ColorSpace (stream/string) — Fase 1 hanya
        // dukung bila bisa dibaca; sisanya tolak halaman dengan jujur.
        return Err("Indexed tanpa palet terbaca — halaman di-skip".to_string());
    } else {
        return Err(format!("colorspace {colorspace} tak didukung"));
    };
    let stride = width as usize * channels as usize;
    // Coba tanpa predictor dulu (data mentah = stride*height).
    let pixels: Vec<u8> = if inflated.len() == stride * height as usize {
        inflated.to_vec()
    } else {
        return Err("predictor PNG per-baris tak didukung — halaman di-skip".to_string());
    };
    let img: image::RgbImage = match channels {
        3 => image::RgbImage::from_raw(width, height, pixels)
            .ok_or_else(|| "rakit RGB gagal".to_string())?,
        1 => {
            let gray = image::GrayImage::from_raw(width, height, pixels)
                .ok_or_else(|| "rakit Gray gagal".to_string())?;
            image::DynamicImage::ImageLuma8(gray).to_rgb8()
        }
        _ => unreachable!(),
    };
    let _ = palette;
    img.save(out).map_err(|e| format!("tulis PNG gagal: {e}"))?;
    // Hormati /Rotate nanti di level halaman (lihat extract_pdf_pages).
    Ok(())
}

/// Ekstrak gambar embedded per halaman PDF ke `{out_dir}/p001.jpg...`
/// (urutan page index). Mengembalikan (files, skipped, infos).
/// Pola `extract_archive` 1:1 — `safe_join` per output, sort, skipped jujur.
fn extract_pdf_pages(
    path: &Path,
    out_dir: &Path,
) -> Result<(Vec<PathBuf>, usize, Vec<String>), String> {
    let mut doc =
        lopdf::Document::load(path).map_err(|e| format!("buka PDF gagal: {e}"))?;
    if doc.is_encrypted() {
        doc.decrypt("")
            .map_err(|_| "PDF terkunci — buka kunci dulu".to_string())?;
    }
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let pages = doc.get_pages();
    let mut files = Vec::new();
    let mut skipped = 0;
    let mut infos: Vec<String> = Vec::new();
    // Urutan halaman = page index (BTreeMap terurut), bukan kemunculan objek.
    for (n, (_page_no, &page_id)) in pages.iter().enumerate() {
        let page_idx = n + 1;
        // Rotasi halaman (/Rotate kelipatan 90) — diterapkan saat tulis.
        let rotate_deg: i32 = doc
            .get_dictionary(page_id)
            .ok()
            .and_then(|d| d.get(b"Rotate").ok())
            .and_then(|o| o.as_i64().ok())
            .map(|r| r.rem_euclid(360) as i32)
            .unwrap_or(0);
        // Kumpulkan XObject gambar halaman ini via content stream (op Do).
        let mut candidates: Vec<(u64, Vec<u8>, lopdf::Dictionary)> = Vec::new();
        let content = doc
            .get_and_decode_page_content(page_id)
            .map_err(|e| format!("baca konten halaman {page_idx} gagal: {e}"))?;
        let resources = doc
            .get_page_resources(page_id)
            .map_err(|e| format!("baca resources halaman {page_idx} gagal: {e}"))?;
        // resources: (Option<&Dictionary>, Vec<ObjectId>) — XObject inline
        // di dict pertama bila ada.
        let mut xobj_dicts: Vec<lopdf::Dictionary> = Vec::new();
        // Dict inline (bila Resources dict langsung di halaman) ...
        if let Some(d) = resources.0 {
            // XObject bisa dict langsung atau referensi — get_deref tangani keduanya.
            if let Ok(xo) = d.get_deref(b"XObject", &doc) {
                if let Ok(xd) = xo.as_dict() {
                    xobj_dicts.push(xd.clone());
                }
            }
        }
        // ... plus dict via referensi Resources (kasus umum dunia-nyata:
        // halaman hanya pegang Reference, XObject hidup di objek lain).
        for rid in resources.1 {
            let Ok(res_obj) = doc.get_object(rid) else {
                continue;
            };
            let Ok(res_dict) = res_obj.as_dict() else {
                continue;
            };
            if let Ok(xo) = res_dict.get_deref(b"XObject", &doc) {
                if let Ok(xd) = xo.as_dict() {
                    xobj_dicts.push(xd.clone());
                }
            }
        }
        for op in &content.operations {
            if op.operator != "Do" {
                continue;
            }
            let Some(lopdf::Object::Name(name)) = op.operands.first() else {
                continue;
            };
            for xd in &xobj_dicts {
                let Ok(obj) = xd.get(name.as_slice()) else {
                    continue;
                };
                let Ok((_, resolved)) = doc.dereference(obj) else {
                    continue;
                };
                if let lopdf::Object::Stream(stm) = resolved {
                    let is_img = matches!(
                        stm.dict.get(b"Subtype"),
                        Ok(lopdf::Object::Name(n)) if n == b"Image"
                    );
                    if !is_img {
                        continue;
                    }
                    let w = stm
                        .dict
                        .get(b"Width")
                        .and_then(|o| o.as_i64())
                        .unwrap_or(0)
                        .max(0) as u64;
                    let h = stm
                        .dict
                        .get(b"Height")
                        .and_then(|o| o.as_i64())
                        .unwrap_or(0)
                        .max(0) as u64;
                    candidates.push((w * h, stm.content.clone(), stm.dict.clone()));
                }
            }
        }
        if candidates.is_empty() {
            skipped += 1;
            infos.push(format!("halaman {page_idx} vektor, ekspor manual"));
            continue;
        }
        // Ambil gambar TERBESAR (mask/SMask kecil diabaikan diam-diam).
        candidates.sort_by_key(|(area, _, _)| std::cmp::Reverse(*area));
        let (_, raw, dict) = &candidates[0];
        let filters = pdf_image_filter_kind(dict)
            .map_err(|e| format!("halaman {page_idx}: {e}"))?;
        // Tolak-jujur filter tanpa decoder murni-Rust di repo ini.
        if filters.iter().any(|f| {
            f.contains("JPXDecode")
                || f.contains("CCITTFaxDecode")
                || f.contains("JBIG2Decode")
        }) {
            skipped += 1;
            infos.push(format!(
                "halaman {page_idx} filter takdidukung ({}), konversi manual",
                filters.join("+")
            ));
            continue;
        }
        // Urutan array = urutan decode: Flate dulu bila ada, lalu DCT pass-through.
        let has_flate = filters.iter().any(|f| f.contains("FlateDecode"));
        let has_dct = filters.iter().any(|f| f.contains("DCTDecode"));
        let fname = format!("p{page_idx:03}.jpg");
        let Some(out) = safe_join(out_dir, &fname) else {
            skipped += 1;
            continue;
        };
        if has_dct {
            let mut jpeg = raw.clone();
            if has_flate {
                jpeg = inflate_flate(&jpeg).map_err(|e| format!("halaman {page_idx}: {e}"))?;
            }
            // Validasi magic JPEG sebelum tulis (data korup → skip, bukan rusak).
            if jpeg.len() < 4 || jpeg[0] != 0xFF || jpeg[1] != 0xD8 {
                skipped += 1;
                infos.push(format!("halaman {page_idx} data JPEG rusak"));
                continue;
            }
            fs::write(&out, &jpeg).map_err(|e| e.to_string())?;
            rotate_file_if_needed(&out, rotate_deg)?;
            files.push(out);
        } else if has_flate {
            // Flate murni: tulis PNG via pixel mentah (RGB/Gray 8-bit).
            let w = dict
                .get(b"Width")
                .and_then(|o| o.as_i64())
                .unwrap_or(0)
                .max(0) as u32;
            let h = dict
                .get(b"Height")
                .and_then(|o| o.as_i64())
                .unwrap_or(0)
                .max(0) as u32;
            let bpc = dict
                .get(b"BitsPerComponent")
                .and_then(|o| o.as_i64())
                .unwrap_or(8)
                .max(0) as u8;
            let cs = dict
                .get(b"ColorSpace")
                .map(|o| format!("{o:?}"))
                .unwrap_or_default();
            let png_out = out.with_extension("png");
            match write_flate_png(raw, w, h, bpc, &cs, &png_out) {
                Ok(()) => {
                    rotate_file_if_needed(&png_out, rotate_deg)?;
                    files.push(png_out);
                }
                Err(e) => {
                    skipped += 1;
                    infos.push(format!("halaman {page_idx}: {e}"));
                }
            }
        } else {
            skipped += 1;
            infos.push(format!(
                "halaman {page_idx} filter takdidukung ({}), konversi manual",
                filters.join("+")
            ));
        }
    }
    files.sort();
    Ok((files, skipped, infos))
}

/// Putar file gambar sesuai /Rotate halaman (kelipatan 90 derajat).
fn rotate_file_if_needed(path: &Path, deg: i32) -> Result<(), String> {
    match deg {
        0 => Ok(()),
        90 => {
            let img = image::open(path).map_err(|e| e.to_string())?;
            img.rotate90().save(path).map_err(|e| e.to_string())
        }
        180 => {
            let img = image::open(path).map_err(|e| e.to_string())?;
            img.rotate180().save(path).map_err(|e| e.to_string())
        }
        270 => {
            let img = image::open(path).map_err(|e| e.to_string())?;
            img.rotate270().save(path).map_err(|e| e.to_string())
        }
        _ => Ok(()),
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
        if is_pdf(path) {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("pdf");
            let safe_stem: String = stem
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect();
            let out_dir = extract_dir.join(format!("{safe_stem}_pdf"));
            let (files, skipped, _infos) = extract_pdf_pages(path, &out_dir)?;
            return Ok((files, skipped));
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

/// iOS document picker returns `file://` URLs; `Path::new` on the raw
/// string never resolves. Parse via plugin-fs `FilePath` (handles
/// percent-encoding), fall back to raw for plain desktop paths.
fn normalize_picker_path(raw: &str) -> PathBuf {
    use std::str::FromStr;
    tauri_plugin_fs::FilePath::from_str(raw)
        .ok()
        .and_then(|fp| fp.into_path().ok())
        .unwrap_or_else(|| PathBuf::from(raw))
}

/// iOS only: picker copies land in Caches (purgeable). Pin single image
/// files into the store dir so pages survive; archives/PDFs already
/// extract under `extract_dir`. Folder picker is unsupported on iOS, so
/// dir inputs never reach here from mobile.
#[cfg(target_os = "ios")]
fn stage_ios_pick(picked: &Path, store_dir: &Path, project_id: &str) -> Option<PathBuf> {
    if !picked.is_file() || !is_image_file(picked) {
        return None;
    }
    let dest_dir = store_dir.join(format!("{project_id}_imports"));
    fs::create_dir_all(&dest_dir).ok()?;
    let name = picked.file_name()?.to_str()?;
    let dest = dest_dir.join(format!("{}_{name}", &Uuid::new_v4().to_string()[..8]));
    fs::copy(picked, &dest).ok()?;
    Some(dest)
}

#[cfg(not(target_os = "ios"))]
fn stage_ios_pick(_picked: &Path, _store_dir: &Path, _project_id: &str) -> Option<PathBuf> {
    None
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_pages(
    project_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let store_dir = store.dir.clone();
    let extract_dir = store_dir.join(format!("{project_id}_extracted"));
    let mut all_files: Vec<PathBuf> = Vec::new();
    let mut skipped = 0;
    for raw in &paths {
        let picked = normalize_picker_path(raw);
        let input = stage_ios_pick(&picked, &store_dir, &project_id).unwrap_or(picked);
        let (mut files, skip) = expand_input(&input, &extract_dir)?;
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

    /// iOS document picker returns `file://` URLs (percent-encoded).
    /// Backend wajib resolve ke path fs — `Path::new` mentah tidak jalan.
    #[test]
    fn picker_url_normalizes_to_fs_path() {
        let dir = temp_dir("kuron-studio-picker-url");
        fs::create_dir_all(&dir).unwrap();
        let real = dir.join("my page.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            6,
            image::Rgb([1, 2, 3]),
        ))
        .save(&real)
        .unwrap();
        let url = format!("file://{}", real.to_string_lossy().replace(' ', "%20"));
        let norm = normalize_picker_path(&url);
        assert_eq!(norm, real);
        let norm_raw = normalize_picker_path(&real.to_string_lossy());
        assert_eq!(norm_raw, real);
        fs::remove_dir_all(&dir).ok();
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

#[cfg(test)]
mod pdf_tests {
    use super::*;
    use lopdf::{Dictionary, Object, Stream};

    fn temp_dir(tag: &str) -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{tag}-{}-{n}", std::process::id()))
    }

    /// Bangun PDF 1-halaman-JPEG sintetis murni di memori (tanpa biner
    /// commit): 1 XObject /Image DCT 4x4 merah sebagai halaman 1.
    fn synthetic_scan_pdf() -> Vec<u8> {
        // JPEG 4x4 minimal via crate image.
        let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            4,
            4,
            image::Rgb([200, 40, 90]),
        ));
        let mut jpg = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut jpg),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
        let mut doc = lopdf::Document::with_version("1.4");
        // XObject gambar: pass-through JPEG (Filter DCT saja, tanpa Flate).
        let mut img_dict = Dictionary::new();
        img_dict.set("Type", Object::Name(b"XObject".to_vec()));
        img_dict.set("Subtype", Object::Name(b"Image".to_vec()));
        img_dict.set("Width", Object::Integer(4));
        img_dict.set("Height", Object::Integer(4));
        img_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
        img_dict.set("BitsPerComponent", Object::Integer(8));
        img_dict.set(
            "Filter",
            Object::Name(b"DCTDecode".to_vec()),
        );
        let img_id = doc.add_object(Object::Stream(Stream::new(img_dict, jpg)));
        // Resources + content (Do /Im0) + page + pages + catalog.
        let mut xobj = Dictionary::new();
        xobj.set("Im0", Object::Reference(img_id));
        let mut res = Dictionary::new();
        res.set("XObject", Object::Dictionary(xobj));
        let res_id = doc.add_object(Object::Dictionary(res));
        let content = lopdf::content::Content {
            operations: vec![lopdf::content::Operation::new(
                "Do",
                vec![Object::Name(b"Im0".to_vec())],
            )],
        };
        let content_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            content.encode().unwrap(),
        )));
        let mut page_dict = Dictionary::new();
        page_dict.set("Type", Object::Name(b"Page".to_vec()));
        page_dict.set("MediaBox", Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(100),
            Object::Integer(100),
        ]));
        page_dict.set("Resources", Object::Reference(res_id));
        page_dict.set(
            "Contents",
            Object::Array(vec![Object::Reference(content_id)]),
        );
        let page_id = doc.add_object(Object::Dictionary(page_dict));
        let mut pages_dict = Dictionary::new();
        pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
        pages_dict.set(
            "Kids",
            Object::Array(vec![Object::Reference(page_id)]),
        );
        pages_dict.set("Count", Object::Integer(1));
        let pages_id = doc.add_object(Object::Dictionary(pages_dict));
        // Parent link + catalog.
        if let Ok(page_obj) = doc.get_object_mut(page_id) {
            if let Ok(d) = page_obj.as_dict_mut() {
                d.set("Parent", Object::Reference(pages_id));
            }
        }
        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));
        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        buf
    }

    #[test]
    fn pdf_scan_extracts_jpeg_passthrough() {
        let dir = temp_dir("pdf-extract");
        fs::create_dir_all(&dir).unwrap();
        let pdf = dir.join("scan.pdf");
        fs::write(&pdf, synthetic_scan_pdf()).unwrap();
        let out_dir = dir.join("out");
        let (files, skipped, _infos) = extract_pdf_pages(&pdf, &out_dir).unwrap();
        assert_eq!(skipped, 0, "halaman JPEG valid tak boleh di-skip");
        assert_eq!(files.len(), 1);
        // Pass-through: magic JPEG + dimensi 4x4 via page_from_file.
        let raw = fs::read(&files[0]).unwrap();
        assert!(raw.len() > 4 && raw[0] == 0xFF && raw[1] == 0xD8);
        let pg = page_from_file(&files[0]).expect("hasil ekstrak harus jadi Page");
        assert_eq!((pg.width, pg.height), (4, 4));
    }

    #[test]
    fn pdf_vector_page_skipped_honestly() {
        let dir = temp_dir("pdf-vector");
        fs::create_dir_all(&dir).unwrap();
        // PDF 1 halaman teks vektor tanpa XObject gambar.
        let mut doc = lopdf::Document::with_version("1.4");
        let content = lopdf::content::Content {
            operations: vec![lopdf::content::Operation::new(
                "Tj",
                vec![Object::string_literal("hello")],
            )],
        };
        let content_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            content.encode().unwrap(),
        )));
        let mut page_dict = Dictionary::new();
        page_dict.set("Type", Object::Name(b"Page".to_vec()));
        page_dict.set("MediaBox", Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(100),
            Object::Integer(100),
        ]));
        page_dict.set(
            "Contents",
            Object::Array(vec![Object::Reference(content_id)]),
        );
        let page_id = doc.add_object(Object::Dictionary(page_dict));
        let mut pages_dict = Dictionary::new();
        pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
        pages_dict.set("Kids", Object::Array(vec![Object::Reference(page_id)]));
        pages_dict.set("Count", Object::Integer(1));
        let pages_id = doc.add_object(Object::Dictionary(pages_dict));
        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));
        let pdf = dir.join("vector.pdf");
        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        fs::write(&pdf, buf).unwrap();
        let (files, skipped, infos) = extract_pdf_pages(&pdf, &dir.join("out")).unwrap();
        assert!(files.is_empty());
        assert_eq!(skipped, 1);
        assert!(infos.iter().any(|s| s.contains("vektor")));
    }

    #[test]
    fn pdf_recognized_but_not_image_or_archive() {
        assert!(is_pdf(Path::new("scan.PDF")));
        assert!(is_pdf(Path::new("a.pdf")));
        assert!(!is_pdf(Path::new("a.jpg")));
        assert!(!is_image_file(Path::new("a.pdf")));
    }
}
