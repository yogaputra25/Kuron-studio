use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlossaryEntry {
    pub id: String,
    pub source: String,
    pub target: String,
    pub created_at: i64,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<GlossaryEntry> {
    Ok(GlossaryEntry {
        id: r.get(0)?,
        source: r.get(1)?,
        target: r.get(2)?,
        created_at: r.get(3)?,
    })
}

pub fn list_entries(conn: &Connection) -> Result<Vec<GlossaryEntry>, String> {
    let mut stmt = conn
        .prepare("SELECT id, source, target, created_at FROM glossary ORDER BY created_at DESC, rowid DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], row).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn add_entry(conn: &Connection, source: &str, target: &str) -> Result<GlossaryEntry, String> {
    let (source, target) = (source.trim().to_string(), target.trim().to_string());
    if source.is_empty() || target.is_empty() {
        return Err("glossary source/target kosong".to_string());
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
    )
    .map_err(|e| e.to_string())?;
    Ok(get_entry(conn, &e.id)?.unwrap_or(e))
}

fn get_entry(conn: &Connection, id: &str) -> Result<Option<GlossaryEntry>, String> {
    let mut stmt = conn
        .prepare("SELECT id, source, target, created_at FROM glossary WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
    match rows.next().map_err(|e| e.to_string())? {
        Some(r) => Ok(Some(row(r).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

pub fn update_entry(
    conn: &Connection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<GlossaryEntry, String> {
    let (source, target) = (source.trim().to_string(), target.trim().to_string());
    if source.is_empty() || target.is_empty() {
        return Err("glossary source/target kosong".to_string());
    }
    let n = conn
        .execute(
            "UPDATE glossary SET source = ?1, target = ?2 WHERE id = ?3",
            params![source, target, id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!("glossary not found: {id}"));
    }
    get_entry(conn, id)?.ok_or_else(|| format!("glossary not found: {id}"))
}

pub fn delete_entry(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM glossary WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
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

pub fn import_csv(conn: &Connection, csv: &str) -> Result<usize, String> {
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

pub fn export_csv(conn: &Connection) -> Result<String, String> {
    let mut out = String::from("source,target\n");
    // Export stabil abjad agar diff-friendly.
    let mut stmt = conn
        .prepare("SELECT source, target FROM glossary ORDER BY source, target")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let (s, t): (String, String) = (r.get(0)?, r.get(1)?);
            Ok((s, t))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (s, t) = r.map_err(|e| e.to_string())?;
        out.push_str(&format!("{},{}\n", quote_field(&s), quote_field(&t)));
    }
    Ok(out)
}

/// Relevan = source muncul sebagai substring case-insensitive di salah satu
/// teks bubble; urut created_at desc, max 5. texts kosong → 5 terbaru
/// (fallback agar auto-inject tetap berguna). Tak ada yang cocok → kosong.
pub fn select_relevant<'a>(
    all: &'a [GlossaryEntry],
    texts: &[String],
) -> Vec<&'a GlossaryEntry> {
    if texts.iter().all(|t| t.trim().is_empty()) {
        return all.iter().take(5).collect();
    }
    let lowered: Vec<String> = texts.iter().map(|t| t.to_lowercase()).collect();
    let mut hits: Vec<&GlossaryEntry> = all
        .iter()
        .filter(|e| {
            let s = e.source.to_lowercase();
            !s.is_empty() && lowered.iter().any(|t| t.contains(&s))
        })
        .collect();
    hits.sort_by_key(|e| std::cmp::Reverse(e.created_at));
    hits.truncate(5);
    hits
}

pub fn build_block(entries: &[&GlossaryEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}={}", e.source.trim(), e.target.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Blok glossary siap-append ke prompt, atau None bila tak ada yang relevan.
/// Menerima &Connection agar command tak perlu dua langkah (list lalu pilih).
pub fn context_for(conn: &Connection, texts: &[String]) -> Result<Option<String>, String> {
    let all = list_entries(conn)?;
    Ok(context_for_entries(&all, texts))
}

pub fn context_for_entries(all: &[GlossaryEntry], texts: &[String]) -> Option<String> {
    let rel = select_relevant(all, texts);
    if rel.is_empty() {
        None
    } else {
        Some(build_block(&rel))
    }
}

/// Auto-context untuk translate_page/batch: never throws — DB error atau
/// kosong → None (caller lanjut tanpa glossary).
pub fn auto_context(conn: &Connection, texts: &[String]) -> Option<String> {
    context_for(conn, texts).unwrap_or(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::init_db;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        init_db(&c).unwrap();
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
        let texts = vec!["S0 dan s1 dan S2 dan s3 dan s4 dan s5 dan s6!".to_string()];
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
