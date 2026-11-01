use serde::{Deserialize, Serialize};
use tauri::State;

use crate::cache;
use crate::translation::{BubbleTranslation, PageTranslation};
use crate::AppState;

pub use crate::translate::{
    apply_cached, backoff_translate, bind_provider, cache_lookup, cancel_or_fail, claim_page,
    effective_glossary, fail_page, load_provider, load_snapshot, map_text,
    mark_page_completed_inner, persist_result, persist_translation, prepare_page, quality_of,
    restore_cancelled, save_translation_inner, store_result, translate_prep,
    translate_text_to_page, CancelState, Ctx, CANCELLED_MSG, CoreJob, PagePrep, PageSnapshot,
    PrepOpts, ProviderBinding,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslatePageInput {
    pub page_id: String,
    pub provider_id: String,
    #[serde(default = "default_lang")]
    pub target_lang: String,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(default)]
    pub skip_sfx: bool,
    #[serde(default = "default_quality")]
    pub mosaic_quality: String,
    #[serde(default = "default_dir")]
    pub reading_direction: String,
    pub glossary: Option<String>,
    #[serde(default)]
    pub custom_prompt: Option<String>,
    /// Override model per request; kosong/absen = pakai simpanan provider.
    #[serde(default)]
    pub model: Option<String>,
}

fn default_lang() -> String { "Indonesian".to_string() }
fn default_style() -> String { "standard".to_string() }
fn default_quality() -> String { "high".to_string() }
fn default_dir() -> String { "rtl".to_string() }

#[tauri::command(rename_all = "snake_case")]
pub async fn translate_page(
    state: State<'_, AppState>,
    cancel: State<'_, CancelState>,
    input: TranslatePageInput,
) -> Result<PageTranslation, String> {
    let ctx = Ctx { app: state.inner(), cancel: cancel.inner() };
    let (prep, binding) = prepare_page(
        &ctx,
        PrepOpts {
            page_id: &input.page_id,
            provider_id: &input.provider_id,
            target_lang: &input.target_lang,
            style: &input.style,
            skip_sfx: input.skip_sfx,
            mosaic_quality: &input.mosaic_quality,
            reading_direction: &input.reading_direction,
            glossary: input.glossary.as_deref(),
            custom_prompt: input.custom_prompt.as_deref(),
            model: input.model.as_deref(),
        },
    )
    .await?;
    Ok(translate_prep(&ctx, &prep, &binding).await?)
}

/// Batalkan translate berjalan untuk satu halaman (opsi B cancel): set flag di
/// `CancelState`; worker berhenti di cek berikut tanpa persist tanpa fail,
/// status dikembalikan ke pra-klaim. Idempotent; halaman tak dikenal = Ok.
#[tauri::command(rename_all = "snake_case")]
pub fn cancel_translate(
    cancel: State<'_, CancelState>,
    page_id: String,
) -> Result<(), String> {
    cancel.request_cancel(&page_id);
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_translation(
    state: State<'_, AppState>,
    cancel: State<'_, CancelState>,
    page_id: String,
    bubbles: Vec<BubbleTranslation>,
) -> Result<PageTranslation, String> {
    let ctx = Ctx { app: state.inner(), cancel: cancel.inner() };
    Ok(save_translation_inner(&ctx, &page_id, bubbles)?)
}

/// Tandai-selesai manual per halaman: paksa `translated` walau sebagian
/// bubble kosong (SFX yang memang sengaja dikosongkan). `completed=false`
/// mencabut flag dan menghitung ulang status dari data.
#[tauri::command(rename_all = "snake_case")]
pub fn mark_page_completed(
    state: State<'_, AppState>,
    page_id: String,
    completed: bool,
) -> Result<PageTranslation, String> {
    Ok(mark_page_completed_inner(state.inner(), &page_id, completed)?)
}

#[tauri::command(rename_all = "snake_case")]
pub fn clear_cache(state: State<'_, AppState>) -> Result<u64, String> {
    let db = crate::error::lock_or_err(&state.db)?;
    Ok(cache::clear_cache(&db)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_inputs_accept_optional_model() {
        // Override model per request: absen = None (fallback simpanan).
        let p: TranslatePageInput =
            serde_json::from_str(r#"{"pageId":"p","providerId":"v"}"#).unwrap();
        assert_eq!(p.model, None);
        let p: TranslatePageInput = serde_json::from_str(
            r#"{"pageId":"p","providerId":"v","model":"muse-spark-1.3"}"#,
        )
        .unwrap();
        assert_eq!(p.model.as_deref(), Some("muse-spark-1.3"));
        let b: crate::commands::batch::TranslateBatchInput =
            serde_json::from_str(r#"{"pageIds":[],"providerId":"v"}"#).unwrap();
        assert_eq!(b.model, None);
        let r: crate::commands::batch::RetryBubbleInput =
            serde_json::from_str(r#"{"pageId":"p","bubbleIndex":0,"providerId":"v"}"#).unwrap();
        assert_eq!(r.model, None);
    }
}
