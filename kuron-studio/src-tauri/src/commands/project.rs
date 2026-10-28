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
pub struct ChapterFolder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub pages: Vec<Page>,
    /// Epoch seconds; 0 = unknown (old data).
    #[serde(default)]
    pub created_at: i64,
    /// Parent folder; None = root. File lama tanpa field ini jadi root semua.
    #[serde(default)]
    pub parent_id: Option<String>,
}

impl ChapterFolder {
    pub fn new(name: String) -> Self {
        Self { id: Uuid::new_v4().to_string(), name, pages: Vec::new(), created_at: now_epoch(), parent_id: None }
    }

    pub fn new_child(name: String, parent_id: String) -> Self {
        Self { id: Uuid::new_v4().to_string(), name, pages: Vec::new(), created_at: now_epoch(), parent_id: Some(parent_id) }
    }
}

pub(crate) fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub const DEFAULT_FOLDER_NAME: &str = "default";

 #[derive(Debug, Clone, Serialize, Deserialize)]
 #[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    /// Folder/chapter dalam project. Mode A (simple) = 1 folder `default`.
    #[serde(default)]
    pub folders: Vec<ChapterFolder>,
    /// Epoch seconds terakhir dimodifikasi (import/detect/translate/folder).
    #[serde(default)]
    pub updated_at: i64,
    /// Kompat baca: projects.json lama `{ pages[] }` tanpa `folders`.
    /// Tidak diserialisasi — sekali load langsung dimigrasi ke folders.
    #[serde(default, rename = "pages", skip_serializing)]
    legacy_pages: Vec<Page>,
 }

impl Project {
    pub fn new(name: String) -> Self {
        let now = now_epoch();
        let mut default = ChapterFolder::new(DEFAULT_FOLDER_NAME.to_string());
        default.created_at = now;
        Self { id: Uuid::new_v4().to_string(), name, folders: vec![default], updated_at: now, legacy_pages: Vec::new() }
    }

    /// Migrasi in-memory: flat lama -> 1 folder `default`. Idempoten.
    pub fn migrate_legacy(&mut self) -> bool {
        if !self.legacy_pages.is_empty() {
            if self.folders.is_empty() {
                let mut f = ChapterFolder::new(DEFAULT_FOLDER_NAME.to_string());
                f.pages = std::mem::take(&mut self.legacy_pages);
                self.folders.push(f);
            } else {
                // Data campuran (seharusnya tak terjadi): tempel ke folder pertama.
                let extra = std::mem::take(&mut self.legacy_pages);
                if let Some(first) = self.folders.first_mut() {
                    first.pages.extend(extra);
                }
            }
            if self.updated_at == 0 { self.updated_at = now_epoch(); }
            return true;
        }
        if self.folders.is_empty() {
            self.folders.push(ChapterFolder::new(DEFAULT_FOLDER_NAME.to_string()));
            return true;
        }
        false
    }

    pub fn all_pages(&self) -> Vec<&Page> {
        self.folders.iter().flat_map(|f| f.pages.iter()).collect()
    }

    pub fn all_pages_mut(&mut self) -> Vec<&mut Page> {
        self.folders.iter_mut().flat_map(|f| f.pages.iter_mut()).collect()
    }

    pub fn sorted_all_pages(&self) -> Vec<Page> {
        let mut out: Vec<Page> = self.folders.iter().flat_map(|f| f.pages.clone()).collect();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }

    /// Halaman satu subtree (folder + semua keturunan), urut filename.
    pub fn sorted_subtree_pages(&self, folder_id: &str) -> Result<Vec<Page>, String> {
        if self.folder(folder_id).is_none() {
            return Err(format!("folder not found: {folder_id}"));
        }
        let ids = self.subtree_ids(folder_id);
        let mut out: Vec<Page> = self
            .folders
            .iter()
            .filter(|f| ids.contains(&f.id))
            .flat_map(|f| f.pages.clone())
            .collect();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }

    /// Halaman untuk scope export/clean: None/kosong = semua folder.
    pub fn pages_in_scope(&self, folder_id: Option<&str>) -> Result<Vec<Page>, String> {
        match folder_id.filter(|s| !s.is_empty()) {
            Some(fid) => self.sorted_subtree_pages(fid),
            None => Ok(self.sorted_all_pages()),
        }
    }

    pub fn folder(&self, folder_id: &str) -> Option<&ChapterFolder> {
        self.folders.iter().find(|f| f.id == folder_id)
    }

    pub fn folder_mut(&mut self, folder_id: &str) -> Option<&mut ChapterFolder> {
        self.folders.iter_mut().find(|f| f.id == folder_id)
    }

    /// Id semua folder dalam subtree (termasuk root-nya sendiri).
    pub fn subtree_ids(&self, folder_id: &str) -> Vec<String> {
        let mut out = vec![folder_id.to_string()];
        let mut i = 0;
        while i < out.len() {
            let cur = out[i].clone();
            for f in self.folders.iter().filter(|f| f.parent_id.as_deref() == Some(cur.as_str())) {
                out.push(f.id.clone());
            }
            i += 1;
        }
        out.retain(|id| self.folders.iter().any(|f| &f.id == id));
        out
    }

    /// Rantai ancestor dari folder (parent, grandparent, …). Dipakai guard siklus.
    pub fn ancestors_of(&self, folder_id: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = self.folder(folder_id).and_then(|f| f.parent_id.clone());
        while let Some(id) = cur {
            if out.contains(&id) { break; } // data korup: putus rantai
            out.push(id.clone());
            cur = self.folder(&id).and_then(|f| f.parent_id.clone());
        }
        out
    }

    /// Pindah parent folder. Menolak: folder/parent tak dikenal, parent = diri
    /// sendiri, parent = keturunan sendiri (siklus). `None` = jadikan root.
    pub fn set_parent(&mut self, folder_id: &str, new_parent: Option<&str>) -> Result<(), String> {
        if self.folder(folder_id).is_none() {
            return Err(format!("folder not found: {folder_id}"));
        }
        if let Some(p) = new_parent.filter(|s| !s.is_empty()) {
            if p == folder_id {
                return Err("folder tidak bisa menjadi parent dirinya sendiri".to_string());
            }
            if self.folder(p).is_none() {
                return Err(format!("parent folder not found: {p}"));
            }
            if self.subtree_ids(folder_id).contains(&p.to_string()) {
                return Err("parent adalah keturunan folder ini (siklus)".to_string());
            }
            // Nama harus unik di antara saudara kandung yang baru.
            let name = self.folder(folder_id).expect("checked").name.clone();
            if self.folders.iter().any(|f| f.id != folder_id && f.parent_id.as_deref() == Some(p) && f.name == name) {
                return Err(format!("folder sudah ada: {name}"));
            }
            self.folder_mut(folder_id).expect("checked").parent_id = Some(p.to_string());
        } else {
            self.folder_mut(folder_id).expect("checked").parent_id = None;
        }
        self.touch();
        Ok(())
    }

    /// Folder tujuan import: id eksplisit > folder `default` > folder pertama > buat baru.
    pub fn resolve_folder_mut(&mut self, folder_id: Option<&str>) -> &mut ChapterFolder {
        if let Some(want) = folder_id.filter(|s| !s.is_empty()) {
            if self.folders.iter().any(|f| f.id == want) {
                return self.folders.iter_mut().find(|f| f.id == want).expect("checked");
            }
        }
        if let Some(i) = self.folders.iter().position(|f| f.name == DEFAULT_FOLDER_NAME) {
            return &mut self.folders[i];
        }
        if self.folders.is_empty() {
            self.folders.push(ChapterFolder::new(DEFAULT_FOLDER_NAME.to_string()));
        }
        &mut self.folders[0]
    }

    pub fn touch(&mut self) {
        self.updated_at = now_epoch();
    }
}
 
 #[derive(Debug, Clone, Serialize)]
 #[serde(rename_all = "camelCase")]
 pub struct ImportResult {
     pub pages: Vec<Page>,
     pub skipped: usize,
    pub folder_id: String,
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
        // Migrasi #7: flat lama { pages[] } -> folders [default]. Idempoten,
        // disimpan langsung supaya file di disk ikut bentuk baru.
        let mut migrated = false;
        for proj in store.projects.values_mut() {
            if proj.migrate_legacy() { migrated = true; }
        }
        if migrated { let _ = store.save(); }
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

    /// Cari halaman di semua project/folder. Mengembalikan path saja (read-only).
    pub fn page_path(&self, page_id: &str) -> Option<String> {
        self.projects.values()
            .flat_map(|p| p.folders.iter())
            .flat_map(|f| f.pages.iter())
            .find(|pg| pg.id == page_id)
            .map(|pg| pg.path.clone())
    }

    /// Cari halaman mutable + touch project pemiliknya. Dipakai detect/translate.
    pub fn find_page_mut(&mut self, page_id: &str) -> Option<&mut Page> {
        for proj in self.projects.values_mut() {
            let found = proj.folders.iter().any(|f| f.pages.iter().any(|pg| pg.id == page_id));
            if found {
                proj.touch();
                for folder in proj.folders.iter_mut() {
                    if let Some(pos) = folder.pages.iter().position(|pg| pg.id == page_id) {
                        return Some(&mut folder.pages[pos]);
                    }
                }
            }
        }
        None
    }

    pub fn find_page(&self, page_id: &str) -> Option<Page> {
        self.projects.values()
            .flat_map(|p| p.folders.iter())
            .flat_map(|f| f.pages.iter())
            .find(|pg| pg.id == page_id)
            .cloned()
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
    let project = Project::new(name);
    store.projects.insert(project.id.clone(), project.clone());
    store.save()?;
    Ok(project)
}

/// Project contoh untuk onboarding: 3 halaman PNG generik + bubble seed.
/// File tinggal di store dir (`{id}_sample`) → ikut terhapus saat project
/// dihapus, seperti folder ekstraksi CBZ.
pub fn create_sample_project_inner(store: &mut ProjectStore, name: &str) -> Result<Project, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("project name is empty".to_string());
    }
    let mut project = Project::new(name);
    let dir = store.dir.join(format!("{}_sample", project.id));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let folder = project.folders.first_mut().expect("Project::new selalu punya folder default");
    for (i, slug) in ["p001", "p002", "p003"].iter().enumerate() {
        let (img, bubbles) = sample_page(i);
        let path = dir.join(format!("{slug}.png"));
        img.save(&path).map_err(|e| e.to_string())?;
        folder.pages.push(Page {
            id: Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            width: img.width(),
            height: img.height(),
            status: PageStatus::Detected,
            bubbles,
            translation: None,
        });
    }
    folder.pages.sort_by(|a, b| a.path.cmp(&b.path));
    project.touch();
    store.projects.insert(project.id.clone(), project.clone());
    store.save()?;
    Ok(project)
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_sample_project(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    create_sample_project_inner(&mut store, &name)
}

fn px(img: &mut image::RgbImage, x: i32, y: i32, c: image::Rgb<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, c);
    }
}

fn fill_rect(img: &mut image::RgbImage, x: u32, y: u32, w: u32, h: u32, c: image::Rgb<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            px(img, (x + dx) as i32, (y + dy) as i32, c);
        }
    }
}

/// Panel manga: fill + bingkai hitam tebal.
fn panel(img: &mut image::RgbImage, x: u32, y: u32, w: u32, h: u32, fill: image::Rgb<u8>) {
    let ink = image::Rgb([24, 22, 20]);
    fill_rect(img, x, y, w, h, fill);
    let t = 6;
    fill_rect(img, x, y, w, t, ink);
    fill_rect(img, x, y + h - t, w, t, ink);
    fill_rect(img, x, y, t, h, ink);
    fill_rect(img, x + w - t, y, t, h, ink);
}

/// Bubble ellipse: fill terang + tepi hitam; kembalikan BubbleBox-nya.
fn ellipse_bubble(
    img: &mut image::RgbImage,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
) -> BubbleBox {
    let fill = image::Rgb([255, 253, 248]);
    let edge = image::Rgb([24, 22, 20]);
    let t = 5;
    for dy in -ry - t..=ry + t {
        for dx in -rx - t..=rx + t {
            let inner = (dx * dx) as f32 / ((rx * rx) as f32) + (dy * dy) as f32 / ((ry * ry) as f32);
            if inner <= 1.0 {
                px(img, cx + dx, cy + dy, fill);
            } else {
                let outer = (dx * dx) as f32 / (((rx + t) * (rx + t)) as f32)
                    + (dy * dy) as f32 / (((ry + t) * (ry + t)) as f32);
                if outer <= 1.0 {
                    px(img, cx + dx, cy + dy, edge);
                }
            }
        }
    }
    BubbleBox { x: cx - rx, y: cy - ry, w: rx * 2, h: ry * 2, confidence: 1.0, shape: None, kind: Some("ellipse".to_string()), tail: None }
}

/// Satu halaman contoh (800×1152): panel + 2–3 bubble. Layout beda per indeks.
fn sample_page(i: usize) -> (image::RgbImage, Vec<BubbleBox>) {
    let mut img = image::RgbImage::new(800, 1152);
    let paper = image::Rgb([250, 249, 246]);
    let tone = image::Rgb([226, 221, 213]);
    let tone2 = image::Rgb([206, 199, 189]);
    fill_rect(&mut img, 0, 0, 800, 1152, paper);
    let mut bubbles = Vec::new();
    match i % 3 {
        0 => {
            panel(&mut img, 48, 48, 704, 560, tone);
            panel(&mut img, 48, 640, 704, 464, paper);
            bubbles.push(ellipse_bubble(&mut img, 220, 230, 92, 64));
            bubbles.push(ellipse_bubble(&mut img, 580, 430, 80, 58));
            bubbles.push(ellipse_bubble(&mut img, 320, 820, 110, 72));
        }
        1 => {
            panel(&mut img, 48, 48, 704, 400, paper);
            panel(&mut img, 48, 480, 340, 624, tone2);
            panel(&mut img, 408, 480, 344, 624, tone);
            bubbles.push(ellipse_bubble(&mut img, 600, 210, 86, 60));
            bubbles.push(ellipse_bubble(&mut img, 200, 710, 96, 66));
            bubbles.push(ellipse_bubble(&mut img, 570, 920, 80, 56));
        }
        _ => {
            panel(&mut img, 48, 48, 704, 1056, tone);
            bubbles.push(ellipse_bubble(&mut img, 260, 310, 100, 70));
            bubbles.push(ellipse_bubble(&mut img, 530, 760, 90, 62));
        }
    }
    (img, bubbles)
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_folder(project_id: String, name: String, state: State<'_, AppState>, parent_id: Option<String>) -> Result<ChapterFolder, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("folder name is empty".to_string());
    }
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let project = store.get_mut(&project_id)?;
    let parent = parent_id.as_deref().filter(|s| !s.is_empty());
    if let Some(p) = parent {
        if project.folder(p).is_none() {
            return Err(format!("parent folder not found: {p}"));
        }
    }
    // Unik di antara saudara kandung (parent sama), bukan global.
    if project.folders.iter().any(|f| f.name == name && f.parent_id.as_deref() == parent) {
        return Err(format!("folder sudah ada: {name}"));
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

#[tauri::command(rename_all = "snake_case")]
pub fn rename_folder(project_id: String, folder_id: String, name: String, state: State<'_, AppState>) -> Result<ChapterFolder, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("folder name is empty".to_string());
    }
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let project = store.get_mut(&project_id)?;
    let parent_of_target = project.folder(&folder_id).and_then(|f| f.parent_id.clone());
    if project.folders.iter().any(|f| f.name == name && f.id != folder_id && f.parent_id == parent_of_target) {
        return Err(format!("folder sudah ada: {name}"));
    }
    let folder = project.folder_mut(&folder_id).ok_or_else(|| format!("folder not found: {folder_id}"))?;
    folder.name = name;
    let out = folder.clone();
    project.folders.sort_by(|a, b| a.name.cmp(&b.name));
    project.touch();
    store.save()?;
    Ok(out)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_folder(project_id: String, folder_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let store_dir = store.dir.clone();
    let doomed: Vec<String>;
    {
        let project = store.get_mut(&project_id)?;
        if project.folders.len() <= 1 {
            return Err("tidak bisa hapus folder terakhir; hapus project saja".to_string());
        }
        if project.folder(&folder_id).is_none() {
            return Err(format!("folder not found: {folder_id}"));
        }
        // Hapus subtree: folder + semua keturunan + halaman mereka.
        let kill: Vec<String> = project.subtree_ids(&folder_id);
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
    folder_id: Option<String>,
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
    let folder = project.resolve_folder_mut(folder_id.as_deref());
    let fid = folder.id.clone();
    folder.pages.extend(pages.clone());
    folder.pages.sort_by(|a, b| a.path.cmp(&b.path));
    project.touch();
    store.save()?;
    Ok(ImportResult { pages, skipped, folder_id: fid })
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
    clean_pages_inner_scoped(store, project_id, None)
}

pub fn clean_pages_inner_scoped(store: &mut ProjectStore, project_id: &str, folder_id: Option<&str>) -> Result<CleanResult, String> {
    let store_dir = store.dir.clone();
    let mut doomed = Vec::new();
    let kept: usize;
    {
        let project = store.get_mut(project_id)?;
        if let Some(want) = folder_id.filter(|s| !s.is_empty()) {
            let folder = project.folder_mut(want).ok_or_else(|| format!("folder not found: {want}"))?;
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

#[tauri::command(rename_all = "snake_case")]
pub fn clean_pages(project_id: String, state: State<'_, AppState>, folder_id: Option<String>) -> Result<CleanResult, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
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
) -> Result<MoveResult, String> {
    let project = store.get_mut(project_id)?;
    if !project.folders.iter().any(|f| f.id == folder_id) {
        return Err(format!("folder not found: {folder_id}"));
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
        return Err(if skipped > 0 {
            "semua halaman sudah di folder tujuan".to_string()
        } else {
            "tidak ada halaman yang dipindah (id salah / sudah di folder tujuan)".to_string()
        });
    }
    let moved = moving.len();
    let dest = project.folder_mut(folder_id).expect("checked");
    dest.pages.extend(moving);
    dest.pages.sort_by(|a, b| a.path.cmp(&b.path));
    project.touch();
    store.save()?;
    Ok(MoveResult { moved, skipped })
}

#[tauri::command(rename_all = "snake_case")]
pub fn move_pages(project_id: String, page_ids: Vec<String>, folder_id: String, state: State<'_, AppState>) -> Result<MoveResult, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    move_pages_inner(&mut store, &project_id, &page_ids, &folder_id)
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
    let sample = store.dir.join(format!("{project_id}_sample"));
    if sample.is_dir() {
        fs::remove_dir_all(&sample).map_err(|e| format!("hapus folder sample: {e}"))?;
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

        let mut proj = Project::new("t".to_string());
        proj.id = "proj".to_string();
        proj.folders[0].pages = vec![stub_page("g", &good), stub_page("a", &junk_a), stub_page("b", &junk_b)];
        store.projects.insert("proj".to_string(), proj);

        let res = clean_pages_inner(&mut store, "proj").unwrap();
        assert_eq!((res.removed, res.kept, res.removed_files), (2, 1, 2));
        assert!(good.exists(), "gambar valid tidak boleh terhapus");
        assert!(!junk_a.exists() && !junk_b.exists());
        let reloaded = ProjectStore::load(dir.clone()).unwrap();
        assert_eq!(reloaded.projects["proj"].sorted_all_pages().len(), 1);
        assert_eq!(reloaded.projects["proj"].sorted_all_pages()[0].id, "g");
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
        let mut proj = Project::new("t".to_string());
        proj.id = "proj".to_string();
        proj.folders[0].pages = vec![stub_page("x", &outside)];
        store.projects.insert("proj".to_string(), proj);
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
        let mut proj = Project::new("t".to_string());
        proj.id = "proj".to_string();
        proj.folders[0].pages = vec![page];
        store.projects.insert("proj".to_string(), proj);
        store.save().unwrap();
        let reloaded = ProjectStore::load(dir).unwrap();
        let all_pages = reloaded.projects["proj"].sorted_all_pages();
        let pg = &all_pages[0];
        assert_eq!(pg.bubbles.len(), 1);
        assert_eq!(pg.bubbles[0].shape.as_ref().unwrap().len(), 3);
        assert_eq!(pg.bubbles[0].tail.as_ref().unwrap().len(), 2);
        assert_eq!(pg.status, PageStatus::Detected);
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
    fn set_parent_rejects_self_and_cycles() {
        let mut proj = tree_proj();
        assert!(proj.set_parent("ch12", Some("ch12")).is_err());
        assert!(proj.set_parent("ch12", Some("b1")).is_err());
        assert!(proj.set_parent("ch12", Some("nope")).is_err());
        assert!(proj.set_parent("ch12", Some(proj.folders[0].id.clone().as_str())).is_ok());
        assert!(proj.set_parent("b1", None).is_ok());
        assert_eq!(proj.folder("b1").unwrap().parent_id, None);
    }

    #[test]
    fn subtree_ids_cover_descendants() {
        let proj = tree_proj();
        let mut ids = proj.subtree_ids("ch12");
        ids.sort();
        assert_eq!(ids, vec!["b1".to_string(), "ch12".to_string()]);
    }

    #[test]
    fn sibling_names_may_repeat_across_parents() {
        let proj = tree_proj();
        let root = proj.folders[0].id.clone();
        // "Batch 1" sudah ada di bawah ch12 — tolak duplikat sesaudara.
        assert!(proj.folders.iter().any(|f| f.name == "Batch 1"
            && f.parent_id.as_deref() == Some("ch12")));
        // Nama sama di bawah root = saudara beda → boleh (cek aturan manual).
        let clash = proj.folders.iter().any(|f| f.name == "Batch 1"
            && f.id != "b1" && f.parent_id.as_deref() == Some(root.as_str()));
        assert!(!clash);
    }

    #[test]
    fn subtree_scope_aggregates_descendants_sorted() {
        let mut proj = tree_proj();
        proj.folders[0].pages = vec![stub_page("r", Path::new("r.jpg"))];
        proj.folders.iter_mut().find(|f| f.id == "ch12").unwrap().pages =
            vec![stub_page("c", Path::new("c.jpg"))];
        proj.folders.iter_mut().find(|f| f.id == "b1").unwrap().pages =
            vec![stub_page("b", Path::new("b.jpg"))];
        // Scope ch12 = dirinya + Batch 1, urut filename.
        let got = proj.pages_in_scope(Some("ch12")).unwrap();
        assert_eq!(got.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["b", "c"]);
        // Scope kosong = semua folder termasuk root.
        let all = proj.pages_in_scope(None).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, "b");
        // Folder tak dikenal = error jujur.
        assert!(proj.pages_in_scope(Some("nope")).is_err());
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
        assert!(err.contains("sudah di folder tujuan"));
    }
    #[test]
    fn sample_project_has_pages_with_seeded_bubbles() {
        let dir = temp_dir("kuron-studio-sample");
        let mut store = ProjectStore::load(dir.clone()).unwrap();
        let proj = create_sample_project_inner(&mut store, "Contoh").unwrap();
        let pages = &proj.folders[0].pages;
        assert_eq!(proj.folders.len(), 1);
        assert_eq!(pages.len(), 3);
        for pg in pages {
            assert!(pg.path.ends_with(".png"));
            assert!(std::path::Path::new(&pg.path).is_file());
            assert_eq!(pg.status, PageStatus::Detected);
            assert!(pg.bubbles.len() >= 2);
            for b in &pg.bubbles {
                assert!(b.w >= 8 && b.h >= 8);
                assert_eq!(b.kind.as_deref(), Some("ellipse"));
                assert!(b.x >= 0 && b.y >= 0);
                assert!(b.x + b.w <= pg.width as i32 && b.y + b.h <= pg.height as i32);
            }
        }
        // File di store dir → ikut terhapus saat project dihapus.
        assert!(dir.join(format!("{}_sample", proj.id)).is_dir());
        assert!(create_sample_project_inner(&mut store, "  ").is_err());
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
