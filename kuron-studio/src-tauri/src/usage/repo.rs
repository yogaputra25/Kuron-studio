//! Repository usage: satu-satunya pemilik akses tabel `usage_records`.
//!
//! Skema tabel dimiliki `crate::db`; di sini hanya query.

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::provider::http::TokenUsage;

use super::UsageSummary;

/// Catat pemakaian satu halaman. `usage = None` tetap dicatat sebagai baris
/// (angka NULL) agar halaman ber-POST tanpa laporan terbedakan dari cache-hit
/// yang memang tanpa baris. Idempotent per halaman: INSERT OR REPLACE berdasar
/// kunci (`page_id`, provider, model) — kirim ulang menimpa, bukan ganda.
pub fn record_usage(
    conn: &Connection,
    page_id: &str,
    provider: &str,
    model: &str,
    usage: Option<TokenUsage>,
) -> Result<(), AppError> {
    let key = format!("{page_id}\u{1F}{provider}\u{1F}{model}");
    let (input, output) = usage.map(|u| (Some(u.input as i64), Some(u.output as i64))).unwrap_or((None, None));
    conn.execute(
        "INSERT INTO usage_records (id, page_id, provider, model, input_tokens, output_tokens, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,strftime('%s','now'))
         ON CONFLICT(id) DO UPDATE SET input_tokens=excluded.input_tokens, output_tokens=excluded.output_tokens, created_at=excluded.created_at",
        params![key, page_id, provider, model, input, output],
    )?;
    Ok(())
}

/// Ringkasan per provider+model untuk halaman-halaman satu run. Halaman tanpa
/// baris (cache-hit / dilewati) tidak ikut hitung — ringkasan hanya atas POST
/// nyata. `unreported` benar bila ada baris berangka NULL di grup itu.
pub fn summarize_for_pages(
    conn: &Connection,
    page_ids: &[String],
) -> Result<Vec<UsageSummary>, AppError> {
    if page_ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = page_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT provider, model,
                COUNT(*),
                COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0),
                SUM(CASE WHEN input_tokens IS NULL OR output_tokens IS NULL THEN 1 ELSE 0 END)
         FROM usage_records WHERE page_id IN ({placeholders}) GROUP BY provider, model ORDER BY provider, model"
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = page_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let rows = stmt.query_map(params.as_slice(), |r| {
        let unreported_n: i64 = r.get(5)?;
        Ok(UsageSummary {
            provider: r.get(0)?,
            model: r.get(1)?,
            pages: r.get::<_, i64>(2)? as u64,
            input_tokens: r.get::<_, i64>(3)? as u64,
            output_tokens: r.get::<_, i64>(4)? as u64,
            unreported: unreported_n > 0,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
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
    fn record_lalu_ringkas_per_provider_model() {
        let c = mem();
        record_usage(&c, "p1", "prov", "m", Some(TokenUsage { input: 10, output: 20 })).unwrap();
        record_usage(&c, "p2", "prov", "m", Some(TokenUsage { input: 5, output: 5 })).unwrap();
        // Tanpa angka = unreported, bukan nol.
        record_usage(&c, "p3", "prov", "m", None).unwrap();
        let s = summarize_for_pages(&c, &["p1".into(), "p2".into(), "p3".into()]).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].pages, 3);
        assert_eq!(s[0].input_tokens, 15);
        assert_eq!(s[0].output_tokens, 25);
        assert!(s[0].unreported);
    }

    #[test]
    fn kirim_ulang_menimpa_bukan_ganda() {
        let c = mem();
        record_usage(&c, "p1", "prov", "m", Some(TokenUsage { input: 10, output: 20 })).unwrap();
        record_usage(&c, "p1", "prov", "m", Some(TokenUsage { input: 1, output: 2 })).unwrap();
        let s = summarize_for_pages(&c, &["p1".into()]).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].pages, s[0].input_tokens), (1, 1));
    }

    #[test]
    fn halaman_tanpa_baris_tak_ikut_hitung() {
        let c = mem();
        record_usage(&c, "p1", "prov", "m", Some(TokenUsage { input: 1, output: 1 })).unwrap();
        let s = summarize_for_pages(&c, &["p1".into(), "p-cache".into()]).unwrap();
        assert_eq!(s[0].pages, 1);
    }
}
