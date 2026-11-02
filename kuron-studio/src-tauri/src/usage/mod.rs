pub mod repo;

// Re-ekspor agar path `crate::usage::{...}` tetap langsung dipakai
// pemanggil pipeline/commands.
pub use repo::{record_usage, summarize_for_pages};
use serde::{Deserialize, Serialize};

/// Satu baris pemakaian: token masuk/keluar per halaman per provider+model.
/// Keduanya NULL-able: provider yang tak melaporkan = "tak dilaporkan",
/// bukan nol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRecord {
    pub id: String,
    pub page_id: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub created_at: i64,
}

/// Agregat ringkasan per provider+model untuk akhir batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub provider: String,
    pub model: String,
    pub pages: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Benar bila ada baris tanpa angka (provider tak melaporkan).
    pub unreported: bool,
}
