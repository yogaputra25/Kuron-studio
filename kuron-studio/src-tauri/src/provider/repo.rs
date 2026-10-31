//! Repository provider: satu-satunya pemilik akses tabel `providers`.
//!
//! Dipindah dari `cache.rs` (issue #58): modul cache tidak lagi menyentuh
//! tabel ini. Skema tabel dimiliki `crate::db`; di sini hanya query.

use rusqlite::{params, Connection};

use crate::error::AppError;

use super::config::{AiProviderType, ProviderRecord};

pub fn insert_provider(conn: &Connection, rec: &ProviderRecord) -> Result<(), AppError> {
    insert_provider_with_key(conn, rec, &rec.api_key)
}

/// M4-1: store sqlite row with an explicit key value. Keychain holds the
/// real secret; callers pass "" on keychain success, legacy key on failure.
pub fn insert_provider_with_key(
    conn: &Connection,
    rec: &ProviderRecord,
    api_key: &str,
) -> Result<(), AppError> {
    let t = serde_json::to_string(&rec.provider_type)?;
    conn.execute(
        "INSERT OR REPLACE INTO providers (id, provider_type, name, base_url, api_key, model, created_at) VALUES (?1,?2,?3,?4,?5,?6,strftime('%s','now'))",
        params![rec.id, t, rec.name, rec.base_url, api_key, rec.model],
    )?;
    Ok(())
}

pub fn list_providers(conn: &Connection) -> Result<Vec<ProviderRecord>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, provider_type, name, base_url, api_key, model FROM providers ORDER BY name")?;
    let rows = stmt
        .query_map([], |r| {
            let t: String = r.get(1)?;
            // Stored via serde_json::to_string so it already carries quotes.
            let pt: AiProviderType =
                serde_json::from_str(&t).unwrap_or(AiProviderType::Custom);
            Ok(ProviderRecord {
                id: r.get(0)?,
                provider_type: pt,
                name: r.get(2)?,
                base_url: r.get(3)?,
                api_key: r.get(4)?,
                model: r.get(5)?,
            })
        })?;
    let list = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(list)
}

pub fn get_provider(conn: &Connection, id: &str) -> Result<Option<ProviderRecord>, AppError> {
    Ok(list_providers(conn)?.into_iter().find(|r| r.id == id))
}

pub fn delete_provider(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM providers WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::db::migrate(&c).unwrap();
        c
    }

    #[test]
    fn provider_crud() {
        let c = mem();
        let rec = ProviderRecord { id: "p1".into(), provider_type: AiProviderType::OpenAi, name: "n".into(), base_url: "".into(), api_key: "k".into(), model: "gpt-4o".into() };
        insert_provider(&c, &rec).unwrap();
        assert_eq!(list_providers(&c).unwrap().len(), 1);
        assert_eq!(get_provider(&c, "p1").unwrap().unwrap().model, "gpt-4o");
        delete_provider(&c, "p1").unwrap();
        assert!(list_providers(&c).unwrap().is_empty());
    }

    #[test]
    fn insert_with_explicit_key_overrides_record_key() {
        let c = mem();
        let rec = ProviderRecord { id: "p1".into(), provider_type: AiProviderType::Gemini, name: "n".into(), base_url: "".into(), api_key: "real".into(), model: "m".into() };
        // Keychain sukses → sqlite disimpan kosong.
        insert_provider_with_key(&c, &rec, "").unwrap();
        assert_eq!(get_provider(&c, "p1").unwrap().unwrap().api_key, "");
    }
}
