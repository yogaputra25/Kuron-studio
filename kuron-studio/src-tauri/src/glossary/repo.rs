//! Repository glossary: satu-satunya pemilik akses tabel `glossary`.
//!
//! Logika murni (CSV, seleksi relevan) tinggal di `super` (`glossary/mod.rs`).
//! Skema tabel dimiliki `crate::db`; di sini hanya query.

use rusqlite::{params, Connection};

use crate::error::AppError;

use super::GlossaryEntry;

fn row(r: &rusqlite::Row) -> rusqlite::Result<GlossaryEntry> {
    Ok(GlossaryEntry {
        id: r.get(0)?,
        source: r.get(1)?,
        target: r.get(2)?,
        created_at: r.get(3)?,
    })
}

pub fn list_entries(conn: &Connection) -> Result<Vec<GlossaryEntry>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, source, target, created_at FROM glossary ORDER BY created_at DESC, rowid DESC")?;
    let rows = stmt.query_map([], row)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn add_entry(conn: &Connection, source: &str, target: &str) -> Result<GlossaryEntry, AppError> {
    let (source, target) = (source.trim().to_string(), target.trim().to_string());
    if source.is_empty() || target.is_empty() {
        return Err(AppError::parse("glossary source/target kosong"));
    }
    let e = GlossaryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        source,
        target,
        created_at: 0,
    };
    conn.execute(
        "INSERT INTO glossary (id, source, target, created_at) VALUES (?1,?2,?3,strftime('%s','now'))",
        params![e.id, e.source, e.target],
    )?;
    Ok(get_entry(conn, &e.id)?.unwrap_or(e))
}

fn get_entry(conn: &Connection, id: &str) -> Result<Option<GlossaryEntry>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, source, target, created_at FROM glossary WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(r) => Ok(Some(row(r)?)),
        None => Ok(None),
    }
}

pub fn update_entry(
    conn: &Connection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<GlossaryEntry, AppError> {
    let (source, target) = (source.trim().to_string(), target.trim().to_string());
    if source.is_empty() || target.is_empty() {
        return Err(AppError::parse("glossary source/target kosong"));
    }
    let n = conn
        .execute(
            "UPDATE glossary SET source = ?1, target = ?2 WHERE id = ?3",
            params![source, target, id],
        )?;
    if n == 0 {
        return Err(AppError::glossary_not_found(id));
    }
    get_entry(conn, id)?.ok_or_else(|| AppError::glossary_not_found(id))
}

pub fn delete_entry(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM glossary WHERE id = ?1", params![id])?;
    Ok(())
}

/// Parse satu baris CSV: `source,target` dengan quote minimal ("a,b", "" escape).
fn parse_line(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }
    let (source, target) = if let Some(body) = t.strip_prefix('"') {
        let mut src = String::new();
        let mut chars = body.chars().peekable();
        let mut closed = false;
        while let Some(c) = chars.next() {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    src.push('"');
                } else {
                    closed = true;
                    break;
                }
            } else {
                src.push(c);
            }
        }
        if !closed {
            return None;
        }
        let rest: String = chars.collect();
        let rest = rest.trim_start();
        let rest = rest.strip_prefix(',')?;
        (src, rest.trim().to_string())
    } else {
        let (a, b) = t.split_once(',')?;
        (a.trim().to_string(), b.trim().to_string())
    };
    if source.trim().is_empty() || target.trim().is_empty() {
        return None;
    }
    Some((source, target))
}

fn quote_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn import_csv(conn: &Connection, csv: &str) -> Result<usize, AppError> {
    let mut n = 0;
    let mut lines = csv.lines();
    // Lewati header opsional `source,target` (case-insensitive).
    if let Some(first) = lines.clone().next() {
        let h: Vec<String> = first.split(',').map(|s| s.trim().to_lowercase()).collect();
        if h == ["source", "target"] {
            lines.next();
        }
    }
    for line in lines {
        if let Some((s, t)) = parse_line(line) {
            add_entry(conn, &s, &t)?;
            n += 1;
        }
    }
    Ok(n)
}

pub fn export_csv(conn: &Connection) -> Result<String, AppError> {
    let mut out = String::from("source,target\n");
    // Export stabil abjad agar diff-friendly.
    let mut stmt = conn
        .prepare("SELECT source, target FROM glossary ORDER BY source, target")?;
    let rows = stmt
        .query_map([], |r| {
            let (s, t): (String, String) = (r.get(0)?, r.get(1)?);
            Ok((s, t))
        })?;
    for r in rows {
        let (s, t) = r?;
        out.push_str(&format!("{},{}\n", quote_field(&s), quote_field(&t)));
    }
    Ok(out)
}

/// Blok glossary siap-append ke prompt, atau None bila tak ada yang relevan.
/// Menerima &Connection agar command tak perlu dua langkah (list lalu pilih).
pub fn context_for(conn: &Connection, texts: &[String]) -> Result<Option<String>, AppError> {
    let all = list_entries(conn)?;
    Ok(super::context_for_entries(&all, texts))
}

/// Auto-context untuk translate_page/batch: never throws — DB error atau
/// kosong → None (caller lanjut tanpa glossary).
pub fn auto_context(conn: &Connection, texts: &[String]) -> Option<String> {
    context_for(conn, texts).unwrap_or(None)
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

    fn seed(c: &Connection, n: usize) {
        for i in 0..n {
            add_entry(c, &format!("s{i}"), &format!("t{i}")).unwrap();
        }
    }

    #[test]
    fn crud() {
        let c = mem();
        let e = add_entry(&c, " Naruto ", " Naruto ").unwrap();
        assert!(!e.id.is_empty());
        let u = update_entry(&c, &e.id, "Sasuke", "Sasuke").unwrap();
        assert_eq!(u.source, "Sasuke");
        assert_eq!(list_entries(&c).unwrap().len(), 1);
        delete_entry(&c, &e.id).unwrap();
        assert!(list_entries(&c).unwrap().is_empty());
    }

    #[test]
    fn crud_rejects_blank() {
        let c = mem();
        assert!(add_entry(&c, " ", "x").is_err());
        assert!(add_entry(&c, "x", "").is_err());
    }

    #[test]
    fn relevant_limit5_case_insensitive_desc() {
        let c = mem();
        seed(&c, 7);
        let texts = vec!["S0 dan s1 dan S2 dan s3 dan S4 dan s5 dan S6!".to_string()];
        let all = list_entries(&c).unwrap();
        let rel = select_relevant(&all, &texts);
        assert_eq!(rel.len(), 5);
        // created_at desc: entri termuda (s6) dulu.
        assert_eq!(rel[0].source, "s6");
    }

    #[test]
    fn empty_texts_falls_back_to_recent5() {
        let c = mem();
        seed(&c, 7);
        let all = list_entries(&c).unwrap();
        let rel = select_relevant(&all, &[]);
        assert_eq!(rel.len(), 5);
        assert_eq!(rel[0].source, "s6");
    }

    #[test]
    fn true_no_match_is_none() {
        let c = mem();
        seed(&c, 3);
        let all = list_entries(&c).unwrap();
        assert!(select_relevant(&all, &["zzz qqq".to_string()]).is_empty());
        assert!(context_for_entries(&all, &["zzz".to_string()]).is_none());
    }

    #[test]
    fn block_format() {
        let e = GlossaryEntry { id: "1".into(), source: "a".into(), target: "b".into(), created_at: 1 };
        assert_eq!(build_block(&[&e]), "a=b");
    }

    #[test]
    fn csv_roundtrip() {
        let c = mem();
        let n = import_csv(&c, "source,target\n\"a,b\",c\nd,e\n\nbadline\n").unwrap();
        assert_eq!(n, 2);
        let out = export_csv(&c).unwrap();
        assert!(out.contains("\"a,b\",c"));
        assert!(out.contains("d,e"));
    }
}
