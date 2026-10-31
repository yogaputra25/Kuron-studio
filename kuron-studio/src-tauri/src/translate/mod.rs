//! Modul translate: pipeline terjemahan per-halaman, binding provider, dan snapshot.
//! Dipindah dari `commands/translate.rs` — tanpa perubahan perilaku.

pub mod pipeline;
pub mod provider_bind;
pub mod snapshot;

pub use pipeline::{
    apply_cached, backoff_translate, cache_lookup, cancel_or_fail, claim_page,
    effective_glossary, fail_page, map_text, mark_page_completed_inner, persist_result,
    persist_translation, prepare_page, quality_of, restore_cancelled, save_translation_inner,
    store_result, translate_prep, translate_text_to_page, CANCELLED_MSG, CoreJob, PagePrep,
    PrepOpts,
};
pub use provider_bind::{bind_provider, load_provider, ProviderBinding};
pub use snapshot::{load_snapshot, PageSnapshot};
