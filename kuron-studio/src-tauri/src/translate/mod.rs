//! Modul translate: pipeline terjemahan per-halaman, binding provider, snapshot,
//! dan status cancel terpisah (#38: keluar dari `AppState`).

use crate::AppState;

pub mod cancel;
pub mod pipeline;
pub mod provider_bind;
pub mod snapshot;

/// Konteks eksekusi pipeline: `app` untuk store/db/http,
/// `cancel` untuk flag batal + status pra-klaim.
pub struct Ctx<'a> {
    pub app: &'a AppState,
    pub cancel: &'a cancel::CancelState,
}
pub use cancel::CancelState;

pub use pipeline::{
    apply_cached, backoff_translate, cache_lookup, cancel_or_fail, claim_page,
    effective_glossary, fail_page, map_text, mark_page_completed_inner, persist_result,
    persist_translation, prepare_page, quality_of, restore_cancelled, save_translation_inner,
    store_result, translate_prep, translate_text_to_page, CANCELLED_MSG, CoreJob, PagePrep,
    PrepOpts,
};
pub use provider_bind::{bind_provider, load_provider, ProviderBinding};
pub use snapshot::{load_snapshot, PageSnapshot};
