pub mod repo;

// Re-ekspor agar path lama (`crate::glossary::list_entries`, …) tetap jalan:
// `commands::translate` (lane lain) dan `commands::glossary` tidak berubah.
pub use repo::{
    add_entry, auto_context, context_for, delete_entry, export_csv, import_csv,
    list_entries, update_entry,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlossaryEntry {
    pub id: String,
    pub source: String,
    pub target: String,
    pub created_at: i64,
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

pub fn context_for_entries(all: &[GlossaryEntry], texts: &[String]) -> Option<String> {
    let rel = select_relevant(all, texts);
    if rel.is_empty() {
        None
    } else {
        Some(build_block(&rel))
    }
}
