//! Cache preview gambar (grid-preview-perf D4): kunci `(path, mtime, size,
//! max_side)`; LRU memori sesi + SQLite persisten untuk tier grid (<=768).
//! JPEG disimpan mentah (bytes); base64 hanya di wire agar kontrak invoke
//! (`get_image_preview: [path, max_side]`) tak berubah.
use std::collections::{HashMap, VecDeque};

use rusqlite::{params, Connection};
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

pub fn ensure_preview_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS page_previews (
            key TEXT NOT NULL,
            max_side INTEGER NOT NULL,
            path TEXT NOT NULL,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL,
            jpeg BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (key, max_side)
        );",
    )
    .map_err(|e| e.to_string())
}

/// Hit persisten: kunci + max_side harus cocok (tier tak tertukar).
pub fn preview_get(conn: &Connection, key: &str, max_side: u32) -> Result<Option<Vec<u8>>, String> {
    let mut stmt = conn
        .prepare("SELECT jpeg FROM page_previews WHERE key = ?1 AND max_side = ?2")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query(params![key, max_side])
        .map_err(|e| e.to_string())?;
    match rows.next().map_err(|e| e.to_string())? {
        Some(r) => Ok(Some(r.get(0).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

pub fn preview_set(
    conn: &Connection,
    key: &str,
    max_side: u32,
    path: &str,
    mtime_secs: i64,
    size: u64,
    jpeg: &[u8],
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO page_previews
         (key, max_side, path, mtime, size, jpeg, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%s','now'))",
        params![key, max_side, path, mtime_secs, size as i64, jpeg],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Buang entri halaman yang sudah tak ada (dipanggil `clean_pages`).
/// `keep` kosong = semua halaman hilang → bersihkan semua baris path terkait.
pub fn preview_purge(conn: &Connection, keep_paths: &[String]) -> Result<u64, String> {
    if keep_paths.is_empty() {
        return conn
            .execute("DELETE FROM page_previews", [])
            .map(|n| n as u64)
            .map_err(|e| e.to_string());
    }
    let holders = keep_paths.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM page_previews WHERE path NOT IN ({holders})");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params: Vec<&dyn rusqlite::ToSql> = keep_paths.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
    stmt.execute(params.as_slice())
        .map(|n| n as u64)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        ensure_preview_table(&c).unwrap();
        c
    }

    #[test]
    fn key_stabil_untuk_file_sama_beda_untuk_versi_beda() {
        let a = preview_key("ch/001.png", 1000, 500, 512);
        assert_eq!(a, preview_key("ch/001.png", 1000, 500, 512));
        assert_ne!(a, preview_key("ch/001.png", 1001, 500, 512)); // mtime beda
        assert_ne!(a, preview_key("ch/001.png", 1000, 501, 512)); // size beda
        assert_ne!(a, preview_key("ch/001.png", 1000, 500, 1600)); // tier beda
        assert_ne!(a, preview_key("ch/002.png", 1000, 500, 512)); // path beda
    }

    #[test]
    fn lru_mengusir_yang_terlama_dan_refresh_saat_hit() {
        let mut lru = MemoryLru::new(2);
        lru.put("a".into(), vec![1]);
        lru.put("b".into(), vec![2]);
        assert_eq!(lru.get("a"), Some(vec![1])); // a segar lagi
        lru.put("c".into(), vec![3]); // usir b (terlama)
        assert_eq!(lru.len(), 2);
        assert_eq!(lru.get("b"), None);
        assert_eq!(lru.get("a"), Some(vec![1]));
        assert_eq!(lru.get("c"), Some(vec![3]));
    }

    #[test]
    fn sqlite_roundtrip_dan_purge_yatim() {
        let c = mem();
        assert_eq!(preview_get(&c, "k1", 512).unwrap(), None);
        preview_set(&c, "k1", 512, "ch/001.png", 100, 50, &[1, 2, 3]).unwrap();
        // max_side beda = miss (tier tak tertukar).
        assert_eq!(preview_get(&c, "k1", 1600).unwrap(), None);
        assert_eq!(preview_get(&c, "k1", 512).unwrap(), Some(vec![1, 2, 3]));
        // Halaman terhapus → purge.
        let n = preview_purge(&c, &["ch/002.png".to_string()]).unwrap();
        assert_eq!(n, 1);
        assert_eq!(preview_get(&c, "k1", 512).unwrap(), None);
    }

    /// Benchmark baseline (grid-preview-perf 1.1/D5): decode dingin vs hit
    /// cache pada gambar sintetis. Angka dicetak (nocapture) lalu dicatat di
    /// design D5 — bukan assert waktu (flaky di CI), assert perilaku saja.
    #[test]
    fn bench_cold_vs_hit_mencatat_angka() {
        use crate::commands::image::render_preview_jpeg;
        let dir = std::env::temp_dir().join(format!(
            "kuron-preview-bench-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("page.png");
        // Gradien deterministik 1200x1600 ≈ halaman manga.
        let img = image::RgbImage::from_fn(1200, 1600, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
        });
        img.save(&path).unwrap();
        let path_s = path.to_string_lossy().to_string();
        let meta = std::fs::metadata(&path).unwrap();
        let mtime = meta
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let (f, q) = (image::imageops::FilterType::Triangle, 80u8);
        // Dingin: 5x decode penuh.
        let t0 = std::time::Instant::now();
        let mut jpeg = Vec::new();
        for _ in 0..5 {
            jpeg = render_preview_jpeg(&path_s, 512, f, q).unwrap();
        }
        let cold_ms = t0.elapsed().as_secs_f64() * 1000.0 / 5.0;
        // Hit: set + get SQLite memori (tanpa decode).
        let c = mem();
        let key = preview_key(&path_s, mtime, meta.len(), 512);
        let t1 = std::time::Instant::now();
        for _ in 0..50 {
            preview_set(&c, &key, 512, &path_s, mtime, meta.len(), &jpeg).unwrap();
            let hit = preview_get(&c, &key, 512).unwrap().unwrap();
            assert_eq!(hit.len(), jpeg.len());
        }
        let hit_ms = t1.elapsed().as_secs_f64() * 1000.0 / 50.0;
        eprintln!(
            "[preview-bench] cold_decode_ms={:.1} cache_hit_ms={:.2} jpeg_bytes={} speedup={:.0}x",
            cold_ms,
            hit_ms,
            jpeg.len(),
            cold_ms / hit_ms.max(0.01)
        );
        assert!(cold_ms > hit_ms); // hit harus orde lebih cepat dari decode
        std::fs::remove_dir_all(&dir).ok();
    }
}
