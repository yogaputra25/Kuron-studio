use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use crate::provider::config::ProviderRecord;

pub fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS translation_cache (key TEXT PRIMARY KEY, payload TEXT NOT NULL, created_at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS providers (id TEXT PRIMARY KEY, provider_type TEXT NOT NULL, name TEXT NOT NULL, base_url TEXT NOT NULL DEFAULT '', api_key TEXT NOT NULL DEFAULT '', model TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS glossary (id TEXT PRIMARY KEY, source TEXT NOT NULL, target TEXT NOT NULL, created_at INTEGER NOT NULL);",
    )
    .map_err(|e| e.to_string())
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
    format!("{:x}", h.finalize())
}

pub fn cache_get(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT payload FROM translation_cache WHERE key = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;
    match rows.next().map_err(|e| e.to_string())? {
        Some(r) => Ok(Some(r.get(0).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

pub fn cache_set(conn: &Connection, key: &str, payload: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO translation_cache (key, payload, created_at) VALUES (?1, ?2, strftime('%s','now'))",
        params![key, payload],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn clear_cache(conn: &Connection) -> Result<u64, String> {
    conn.execute("DELETE FROM translation_cache", [])
        .map(|n| n as u64)
        .map_err(|e| e.to_string())
}

pub fn insert_provider(conn: &Connection, rec: &ProviderRecord) -> Result<(), String> {
    let t = serde_json::to_string(&rec.provider_type).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO providers (id, provider_type, name, base_url, api_key, model, created_at) VALUES (?1,?2,?3,?4,?5,?6,strftime('%s','now'))",
        params![rec.id, t, rec.name, rec.base_url, rec.api_key, rec.model],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn list_providers(conn: &Connection) -> Result<Vec<ProviderRecord>, String> {
    let mut stmt = conn
        .prepare("SELECT id, provider_type, name, base_url, api_key, model FROM providers ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let t: String = r.get(1)?;
            // Stored via serde_json::to_string so it already carries quotes.
            let pt: crate::provider::config::AiProviderType =
                serde_json::from_str(&t).unwrap_or(crate::provider::config::AiProviderType::Custom);
            Ok(ProviderRecord {
                id: r.get(0)?,
                provider_type: pt,
                name: r.get(2)?,
                base_url: r.get(3)?,
                api_key: r.get(4)?,
                model: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_provider(conn: &Connection, id: &str) -> Result<Option<ProviderRecord>, String> {
    Ok(list_providers(conn)?.into_iter().find(|r| r.id == id))
}

pub fn delete_provider(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM providers WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
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
    fn provider_crud() {
        let c = mem();
        let rec = ProviderRecord { id: "p1".into(), provider_type: crate::provider::config::AiProviderType::OpenAi, name: "n".into(), base_url: "".into(), api_key: "k".into(), model: "gpt-4o".into() };
        insert_provider(&c, &rec).unwrap();
        assert_eq!(list_providers(&c).unwrap().len(), 1);
        assert_eq!(get_provider(&c, "p1").unwrap().unwrap().model, "gpt-4o");
        delete_provider(&c, "p1").unwrap();
        assert!(list_providers(&c).unwrap().is_empty());
    }

    #[test]
    fn glossary_table_exists() {
        let c = mem();
        c.execute(
            "INSERT INTO glossary (id, source, target, created_at) VALUES ('g1','a','b',1)",
            [],
        )
        .unwrap();
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM glossary", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }
}
