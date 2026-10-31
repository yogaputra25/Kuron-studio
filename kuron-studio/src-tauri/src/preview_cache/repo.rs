//! Repository preview: satu-satunya pemilik akses tabel `page_previews`.
//!
//! Kunci/LRU murni tinggal di `super` (`preview_cache/mod.rs`); skema tabel
//! dimiliki `crate::db`. DDL `ensure_preview_table` lama sudah pindah ke
//! `crate::db::migrate` (dipanggil sekali saat startup) — modul ini murni query.

use rusqlite::{params, Connection};

use crate::error::AppError;

/// Hit persisten: kunci + max_side harus cocok (tier tak tertukar).
pub fn preview_get(conn: &Connection, key: &str, max_side: u32) -> Result<Option<Vec<u8>>, AppError> {
    let mut stmt = conn
        .prepare("SELECT jpeg FROM page_previews WHERE key = ?1 AND max_side = ?2")?;
    let mut rows = stmt
        .query(params![key, max_side])?;
    match rows.next()? {
        Some(r) => Ok(Some(r.get(0)?)),
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
) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO page_previews
         (key, max_side, path, mtime, size, jpeg, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%s','now'))",
        params![key, max_side, path, mtime_secs, size as i64, jpeg],
    )?;
    Ok(())
}

/// Buang entri halaman yang sudah tak ada (dipanggil `clean_pages`).
/// `keep` kosong = semua halaman hilang → bersihkan semua baris path terkait.
pub fn preview_purge(conn: &Connection, keep_paths: &[String]) -> Result<u64, AppError> {
    if keep_paths.is_empty() {
        return Ok(conn
            .execute("DELETE FROM page_previews", [])? as u64);
    }
    let holders = keep_paths.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM page_previews WHERE path NOT IN ({holders})");
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = keep_paths.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
    Ok(stmt.execute(params.as_slice())? as u64)
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::db::migrate(&c).unwrap();
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
