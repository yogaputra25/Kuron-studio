//! Status halaman + normalisasi/derivasi terpusat.
//!
//! Aturan turunan status tinggal di sini supaya semua mirror (backend
//! `save_*`, frontend `lib/status.ts`) memakai logika yang sama.

use serde::{Deserialize, Serialize};

use super::page::Page;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PageStatus {
    #[default]
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "detecting")]
    Detecting,
    #[serde(rename = "detected")]
    Detected,
    #[serde(rename = "noBubbles")]
    NoBubbles,
    #[serde(rename = "translating")]
    Translating,
    #[serde(rename = "translated")]
    Translated,
    #[serde(rename = "failed")]
    Failed,
}

/// Normalisasi status simpanan saat load. Transient gantung
/// (`Translating`/`Detecting`) → `Detected`/`Idle` ikut ada-tidaknya bubble;
/// `Translated` yang terjemahannya sebagian tanpa flag manual turun mengikuti
/// aturan yang sama. Mengembalikan true bila status berubah.
pub(crate) fn normalize_page_status(pg: &mut Page) -> bool {
    let base = if pg.bubbles.is_empty() { PageStatus::Idle } else { PageStatus::Detected };
    match pg.status {
        PageStatus::Translating | PageStatus::Detecting => {
            pg.status = base;
            true
        }
        PageStatus::Translated => {
            let ok = pg
                .translation
                .as_ref()
                .is_some_and(crate::translation::translation_complete);
            if ok {
                false
            } else {
                pg.status = base;
                true
            }
        }
        _ => false,
    }
}

/// Terapkan status jujur sehabis simpan terjemahan: `Translated` hanya bila
/// lengkap (semua terisi ATAU flag manual); selain itu status lama
/// dipertahankan, transient `Translating` pulih ke pra-klaim (`prev`) atau
/// Detected/Idle. Menyatukan salinan identik di `save_translation_inner`
/// dan `persist_result` — JANGAN dipakai untuk `mark_page_completed`
/// (sengaja hitung ulang dari data, tanpa `prev`) maupun `persist_bubbles`
/// (konteks geometri, mengenal `NoBubbles`).
pub fn apply_translation_status(
    page: &mut Page,
    t: &crate::translation::PageTranslation,
    prev: Option<PageStatus>,
) {
    if crate::translation::translation_complete(t) {
        page.status = PageStatus::Translated;
    } else if page.status == PageStatus::Translating {
        page.status = prev.unwrap_or(if page.bubbles.is_empty() {
            PageStatus::Idle
        } else {
            PageStatus::Detected
        });
    }
}
