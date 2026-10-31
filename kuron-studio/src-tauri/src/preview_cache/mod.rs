//! Cache preview gambar (grid-preview-perf D4): kunci `(path, mtime, size,
//! max_side)`; LRU memori sesi + SQLite persisten untuk tier grid (<=768).
//! JPEG disimpan mentah (bytes); base64 hanya di wire agar kontrak invoke
//! (`get_image_preview: [path, max_side]`) tak berubah.
//!
//! Akses SQLite (`preview_get/set/purge`) tinggal di `repo`; skema tabel
//! dimiliki `crate::db`.

pub mod repo;

// Re-ekspor agar path lama (`crate::preview_cache::preview_get`, …) tetap
// jalan: `commands::project` (lane lain) dan `commands::image` tidak berubah.
pub use repo::{preview_get, preview_purge, preview_set};

use std::collections::{HashMap, VecDeque};

use sha2::{Digest, Sha256};

/// Tier grid (murah, persisten) vs editor (tajam, sesi-saja).
pub const THUMB_TIER_MAX_SIDE: u32 = 768;
/// ~300 thumb grid x ~30KB ≈ 9MB sesi — di bawah plafon memori eksplisit.
pub const THUMB_LRU_CAP: usize = 300;
/// Full 1600px x ~300KB — hanya beberapa halaman terakhir.
pub const FULL_LRU_CAP: usize = 4;

/// Kunci cache stabil untuk satu (file, versi file, ukuran minta).
pub fn preview_key(path: &str, mtime_secs: i64, size: u64, max_side: u32) -> String {
    let mut h = Sha256::new();
    h.update(path.as_bytes());
    h.update([0]);
    h.update(mtime_secs.to_le_bytes());
    h.update(size.to_le_bytes());
    h.update(max_side.to_le_bytes());
    format!("{:x}", h.finalize())
}

/// LRU sederhana tanpa dep baru: HashMap + urutan pemakaian (depan = terlama).
pub struct MemoryLru {
    cap: usize,
    map: HashMap<String, Vec<u8>>,
    order: VecDeque<String>,
}

impl MemoryLru {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Hit: segarkan urutan, kembalikan clone (lock dilepas cepat oleh pemanggil).
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        let v = self.map.get(key)?.clone();
        self.order.retain(|k| k != key);
        self.order.push_back(key.to_string());
        Some(v)
    }

    pub fn put(&mut self, key: String, jpeg: Vec<u8>) {
        if self.cap == 0 {
            return;
        }
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else {
            while self.map.len() >= self.cap {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                } else {
                    break;
                }
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, jpeg);
    }
}

/// Dua tier terpisah agar full 1600px tak mengusir thumb grid (D4).
pub struct PreviewCaches {
    pub thumb: MemoryLru,
    pub full: MemoryLru,
}

impl PreviewCaches {
    pub fn new() -> Self {
        Self {
            thumb: MemoryLru::new(THUMB_LRU_CAP),
            full: MemoryLru::new(FULL_LRU_CAP),
        }
    }

    pub fn get(&mut self, persist_tier: bool, key: &str) -> Option<Vec<u8>> {
        if persist_tier {
            self.thumb.get(key)
        } else {
            self.full.get(key)
        }
    }

    pub fn put(&mut self, persist_tier: bool, key: String, jpeg: Vec<u8>) {
        if persist_tier {
            self.thumb.put(key, jpeg);
        } else {
            self.full.put(key, jpeg);
        }
    }
}

impl Default for PreviewCaches {
    fn default() -> Self {
        Self::new()
    }
}
