use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use crate::error::AppError;

/// Kompat: `commands::translate` (lane lain, file beku) masih memanggil
/// `cache::get_provider`. Pemilik baru: [`crate::provider::repo`].
pub use crate::provider::repo::get_provider;

/// Kompat startup (`lib.rs` lane lain memanggil ini): skema kanonis tunggal
/// di [`crate::db::migrate`]. Jangan tambah DDL di sini.
pub fn init_db(conn: &Connection) -> Result<(), AppError> {
    crate::db::migrate(conn)
}

/// Satu struct arg agar lolos clippy too_many_arguments.
pub struct CacheKey<'a> {
    pub image: &'a [u8],
    pub bubbles_json: &'a str,
    pub target_lang: &'a str,
    pub style: &'a str,
    pub model: &'a str,
    pub skip_sfx: bool,
    pub reading_dir: &'a str,
    pub glossary: Option<&'a str>,
    pub custom_prompt: Option<&'a str>,
}

pub fn cache_key(job: CacheKey<'_>) -> String {
    let mut h = Sha256::new();
    h.update(job.image);
    h.update(job.bubbles_json.as_bytes());
    h.update(job.target_lang.as_bytes());
    h.update(job.style.as_bytes());
    h.update(job.model.as_bytes());
    h.update([u8::from(job.skip_sfx)]);
    h.update(job.reading_dir.as_bytes());
    // Normalize: None/""/whitespace all map to empty so key is stable.
    h.update(job.glossary.unwrap_or("").trim().as_bytes());
    h.update(job.custom_prompt.unwrap_or("").trim().as_bytes());
    format!("{:x}", h.finalize())
}

pub fn cache_get(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT payload FROM translation_cache WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    match rows.next()? {
        Some(r) => Ok(Some(r.get(0)?)),
        None => Ok(None),
    }
}

pub fn cache_set(conn: &Connection, key: &str, payload: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO translation_cache (key, payload, created_at) VALUES (?1, ?2, strftime('%s','now'))",
        params![key, payload],
    )?;
    Ok(())
}

pub fn clear_cache(conn: &Connection) -> Result<u64, AppError> {
    Ok(conn.execute("DELETE FROM translation_cache", [])? as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        init_db(&c).unwrap();
        c
    }

    fn key(skip_sfx: bool, reading_dir: &str, glossary: Option<&str>) -> String {
        cache_key(CacheKey {
            image: b"img",
            bubbles_json: "[]",
            target_lang: "id",
            style: "standard",
            model: "m",
            skip_sfx,
            reading_dir,
            glossary,
            custom_prompt: None,
        })
    }

    #[test]
    fn roundtrip() {
        let c = mem();
        let k = key(true, "rtl", None);
        assert!(cache_get(&c, &k).unwrap().is_none());
        cache_set(&c, &k, "{\"a\":1}").unwrap();
        assert_eq!(cache_get(&c, &k).unwrap().as_deref(), Some("{\"a\":1}"));
        assert_eq!(clear_cache(&c).unwrap(), 1);
        assert!(cache_get(&c, &k).unwrap().is_none());
    }

    #[test]
    fn key_covers_all_inputs() {
        let base = key(true, "rtl", None);
        assert_ne!(base, key(false, "rtl", None));
        assert_ne!(base, key(true, "ltr", None));
        assert_ne!(base, key(true, "rtl", Some("A=B")));
        // Glossary None vs blank is the same input.
        assert_eq!(base, key(true, "rtl", Some("  ")));
    }

    #[test]
    fn key_covers_custom_prompt() {
        let base = key(true, "rtl", None);
        let with = |c: Option<&str>| {
            cache_key(CacheKey {
                image: b"img",
                bubbles_json: "[]",
                target_lang: "id",
                style: "standard",
                model: "m",
                skip_sfx: true,
                reading_dir: "rtl",
                glossary: None,
                custom_prompt: c,
            })
        };
        assert_ne!(base, with(Some("bahasa santai")));
        assert_eq!(base, with(Some("  ")));
    }
}
