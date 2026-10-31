//! Project + folder: struktur data dan konstruktor/migrasi.
//! Operasi tree (subtree, ancestor, selektor halaman) tinggal di `tree.rs`.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::page::Page;

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
    pub(crate) legacy_pages: Vec<Page>,
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

    pub fn touch(&mut self) {
        self.updated_at = now_epoch();
    }

    pub fn all_pages(&self) -> Vec<Page> {
        self.folders.iter().flat_map(|f| f.pages.clone()).collect()
    }
}
