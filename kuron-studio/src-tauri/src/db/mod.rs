//! Satu-satunya pemilik skema SQLite: `open` + `migrate`.
//!
//! Semua `CREATE TABLE` tinggal di sini. Modul `repo::*` hanya query —
//! tidak ada DDL di tempat lain. Nama + kolom tabel TIDAK boleh diubah:
//! DB user existing di folder data OS tidak punya jalur migrasi versi,
//! jadi `migrate` harus idempoten (`IF NOT EXISTS`) dan non-destruktif.

use std::path::Path;

use rusqlite::Connection;

use crate::error::AppError;

/// Skema kanonis. Disalin verbatim dari `cache::init_db` (3 tabel) +
/// `preview_cache::ensure_preview_table` (1 tabel) — tanpa rename apa pun.
pub const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS translation_cache (key TEXT PRIMARY KEY, payload TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS providers (id TEXT PRIMARY KEY, provider_type TEXT NOT NULL, name TEXT NOT NULL, base_url TEXT NOT NULL DEFAULT '', api_key TEXT NOT NULL DEFAULT '', model TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS glossary (id TEXT PRIMARY KEY, source TEXT NOT NULL, target TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS page_previews (
    key TEXT NOT NULL,
    max_side INTEGER NOT NULL,
    path TEXT NOT NULL,
    mtime INTEGER NOT NULL,
    size INTEGER NOT NULL,
    jpeg BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (key, max_side)
);";

/// Buka file DB di `path`.
pub fn open(path: &Path) -> Result<Connection, AppError> {
    Ok(Connection::open(path)?)
}

/// Buat semua tabel yang belum ada. Idempoten — aman dipanggil tiap startup.
pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DDL versi lama: sebelum `page_previews` ada (hanya 3 tabel).
    /// Disalin dari `cache::init_db` versi lama — bukan dari SCHEMA_SQL —
    /// agar test benar-benar mensimulasikan DB user existing.
    const LEGACY_SQL: &str = "
CREATE TABLE translation_cache (key TEXT PRIMARY KEY, payload TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE providers (id TEXT PRIMARY KEY, provider_type TEXT NOT NULL, name TEXT NOT NULL, base_url TEXT NOT NULL DEFAULT '', api_key TEXT NOT NULL DEFAULT '', model TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL);
CREATE TABLE glossary (id TEXT PRIMARY KEY, source TEXT NOT NULL, target TEXT NOT NULL, created_at INTEGER NOT NULL);";

    fn table_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap()
    }

    #[test]
    fn migrate_legacy_db_adds_missing_tables_and_keeps_data() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(LEGACY_SQL).unwrap();
        // Data lama di tiap tabel versi lama.
        c.execute(
            "INSERT INTO translation_cache (key, payload, created_at) VALUES ('k1','{\"a\":1}',1)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO providers (id, provider_type, name, base_url, api_key, model, created_at) VALUES ('p1','\"openAi\"','n','','k','gpt-4o',1)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO glossary (id, source, target, created_at) VALUES ('g1','a','b',1)",
            [],
        )
        .unwrap();

        migrate(&c).unwrap();

        let names = table_names(&c);
        for t in ["translation_cache", "providers", "glossary", "page_previews"] {
            assert!(names.contains(&t.to_string()), "tabel {t} harus ada: {names:?}");
        }
        // Data lama utuh.
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM translation_cache", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM providers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM glossary", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let name: String = c
            .query_row("SELECT name FROM providers WHERE id = 'p1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "n");
    }

    #[test]
    fn migrate_idempotent_and_fresh_db_has_all_tables() {
        let c = Connection::open_in_memory().unwrap();
        migrate(&c).unwrap();
        migrate(&c).unwrap(); // jalan dua kali tanpa error
        let names = table_names(&c);
        assert_eq!(
            names,
            vec!["glossary", "page_previews", "providers", "translation_cache"]
        );
    }
}
