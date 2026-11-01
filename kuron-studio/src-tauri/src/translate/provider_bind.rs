//! Binding dan pemuatan provider untuk pipeline translate.
//! Dipindah dari `commands/translate.rs` — tanpa perubahan perilaku.

use crate::cache;
use crate::error::AppError;
use crate::provider::config::ProviderRecord;
use crate::AppState;

use super::{pipeline::{restore_cancelled, PagePrep, PrepOpts}, Ctx};

pub fn load_provider(
    state: &AppState,
    provider_id: &str,
) -> Result<(reqwest::Client, ProviderRecord), AppError> {
    let db = crate::error::lock_or_err(&state.db)?;
    let mut rec = cache::get_provider(&db, provider_id)?
        .ok_or_else(|| crate::error::AppError::provider_not_found(provider_id))?;
    // File-encrypted: `enc1:` didekripsi lokal tanpa keychain; legacy
    // keychain/sqlite fallback. Never logged.
    rec.api_key = crate::secrets::read_key(&state.data_dir, provider_id, &rec.api_key)?;
    Ok((state.http.clone(), rec))
}

/// Bagian murah per provider: client + record (+model efektif) + cache key.
/// Satu `PagePrep` bisa di-bind ke beberapa provider tanpa prep ulang.
pub struct ProviderBinding {
    pub rec: ProviderRecord,
    pub client: reqwest::Client,
    pub cache_key: String,
}

/// Ikat prep agnostik ke satu provider: resolve record (+dekripsi key), model
/// efektif, cache key. Satu-satunya implementasi resolve provider (#59):
/// `commands::provider` memakai [`load_provider`] ini, bukan salinan.
/// Gagal di sini = halaman belum mulai kerja → status pulih pra-klaim
/// (bukan `fail_page`), klaim dilepas.
pub fn bind_provider(
    ctx: &Ctx<'_>,
    opts: &PrepOpts<'_>,
    prep: &PagePrep,
    gloss: Option<&str>,
) -> Result<ProviderBinding, AppError> {
    let bubbles_json = serde_json::to_string(&prep.bubbles)?;
    let (client, mut rec) = load_provider(ctx.app, opts.provider_id)
        .inspect_err(|_| {
            let _ = restore_cancelled(ctx, opts.page_id);
        })?;
    rec.model = crate::provider::effective_model(&rec, opts.model);
    let key = cache::cache_key(cache::CacheKey {
        image: &prep.jpeg,
        bubbles_json: &bubbles_json,
        target_lang: &prep.target_lang,
        style: &prep.style,
        model: &rec.model,
        skip_sfx: opts.skip_sfx,
        reading_dir: opts.reading_direction,
        glossary: gloss,
        custom_prompt: opts.custom_prompt,
    });
    Ok(ProviderBinding {
        rec,
        client,
        cache_key: key,
    })
}
