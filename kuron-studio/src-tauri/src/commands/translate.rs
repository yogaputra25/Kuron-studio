use serde::{Deserialize, Serialize};
use tauri::State;

use crate::bubble::BubbleBox;
use crate::cache;
use crate::image_ops::{build_mosaic, compress_page, MosaicQuality};
use crate::parser::{parse_full_image_json, parse_mosaic_json, preview_200};
use crate::prompt::{append_glossary, build_mosaic_prompt, full_image_prompt, TranslateStyle};
use crate::provider;
use crate::provider::config::ProviderRecord;
use crate::translation::{
    map_full_image, map_mosaic, order_indices, preserve_user_edits, BubbleTranslation,
    PageTranslation,
};
use crate::AppState;

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
}

fn default_lang() -> String { "Indonesian".to_string() }
fn default_style() -> String { "standard".to_string() }
fn default_quality() -> String { "high".to_string() }
fn default_dir() -> String { "rtl".to_string() }

pub fn quality_of(s: &str) -> MosaicQuality {
    if s.eq_ignore_ascii_case("low") { MosaicQuality::Low } else { MosaicQuality::High }
}

/// Core testable: parse provider text into translations given geometry.
/// Satu struct arg agar lolos clippy too_many_arguments.
pub struct CoreJob<'a> {
    pub page_id: &'a str,
    pub target_lang: &'a str,
    pub style: &'a str,
    pub model: &'a str,
    pub bubbles: &'a [BubbleBox],
    pub ordered: &'a [usize],
    pub page_w: i32,
    pub page_h: i32,
    pub text: &'a str,
    pub prev: Option<&'a PageTranslation>,
}

pub fn translate_text_to_page(job: CoreJob<'_>) -> Result<PageTranslation, String> {
    let mapped = if job.bubbles.is_empty() {
        let items = parse_full_image_json(job.text)
            .map_err(|e| format!("parse full-image failed: {e} ({})", preview_200(job.text)))?;
        map_full_image(job.page_w, job.page_h, &items)
    } else {
        let results = parse_mosaic_json(job.text)
            .map_err(|e| format!("parse mosaic failed: {e} ({})", preview_200(job.text)))?;
        map_mosaic(job.bubbles, job.ordered, &results, job.page_w, job.page_h)
    };
    Ok(PageTranslation {
        page_id: job.page_id.to_string(),
        target_lang: job.target_lang.to_string(),
        style: job.style.to_string(),
        model: job.model.to_string(),
        bubbles: preserve_user_edits(mapped, job.prev),
    })
}

// --- M3: shared per-page pipeline (translate_page + translate_batch + retry) ---

/// Owned snapshot halaman; diambil sekali agar worker batch tak pinjam State.
#[derive(Debug, Clone)]
pub struct PageSnapshot {
    pub page_id: String,
    pub path: String,
    pub bubbles: Vec<BubbleBox>,
    pub page_w: i32,
    pub page_h: i32,
    pub prev: Option<PageTranslation>,
}

pub fn load_snapshot(
    state: &State<'_, AppState>,
    page_id: &str,
) -> Result<PageSnapshot, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let pg = store
        .projects
        .values()
        .flat_map(|p| p.pages.iter())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?
        .clone();
    Ok(PageSnapshot {
        page_id: pg.id,
        path: pg.path,
        bubbles: pg.bubbles,
        page_w: pg.width as i32,
        page_h: pg.height as i32,
        prev: pg.translation,
    })
}

/// Klaim atomik: halaman Translating ditolak ("busy") oleh penelepon kedua.
pub fn claim_page(state: &State<'_, AppState>, page_id: &str) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let page = store
        .projects
        .values_mut()
        .flat_map(|p| p.pages.iter_mut())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?;
    match page.status {
        crate::commands::project::PageStatus::Translating => {
            Err(format!("page busy (already translating): {page_id}"))
        }
        _ => {
            page.status = crate::commands::project::PageStatus::Translating;
            store.save_public()
        }
    }
}

pub fn load_provider(
    state: &State<'_, AppState>,
    provider_id: &str,
) -> Result<(reqwest::Client, ProviderRecord), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut rec = cache::get_provider(&db, provider_id)?
        .ok_or_else(|| format!("provider not found: {provider_id}"))?;
    // M4-1: keychain first, sqlite legacy fallback. Never logged.
    rec.api_key = crate::secrets::read_key(provider_id, &rec.api_key)?;
    Ok((state.http.clone(), rec))
}

/// Glossary efektif: override eksplisit menang; None → auto-context
/// (most-recent-5 bila tak ada teks relevan). Never throws: DB error → None.
pub fn effective_glossary(
    state: &State<'_, AppState>,
    explicit: Option<&str>,
    prev: Option<&PageTranslation>,
) -> Option<String> {
    match explicit {
        Some(g) if !g.trim().is_empty() => Some(g.trim().to_string()),
        _ => {
            let texts: Vec<String> = prev
                .map(|p| p.bubbles.iter().map(|b| b.original.clone()).collect())
                .unwrap_or_default();
            state
                .db
                .lock()
                .ok()
                .and_then(|db| crate::glossary::auto_context(&db, &texts))
        }
    }
}

/// Satu POST dengan backoff 2s/4s/8s khusus 429 (di atas retry internal
/// provider). Error non-429 langsung pulang.
pub async fn backoff_translate(
    client: &reqwest::Client,
    rec: &ProviderRecord,
    prompt: &str,
    jpeg: &[u8],
) -> Result<String, String> {
    let mut last = String::from("rate_limited: retry later");
    for (n, wait) in [0u64, 2, 4, 8].iter().enumerate() {
        if *wait > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(*wait)).await;
        }
        match provider::translate_image(client, rec, prompt, jpeg).await {
            Ok(t) => return Ok(t),
            Err(e) if e.contains("rate_limited") && n < 3 => {
                last = e;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last)
}

/// Opsi prepare satu halaman — satu struct demi clippy.
pub struct PrepOpts<'a> {
    pub page_id: &'a str,
    pub provider_id: &'a str,
    pub target_lang: &'a str,
    pub style: &'a str,
    pub skip_sfx: bool,
    pub mosaic_quality: &'a str,
    pub reading_direction: &'a str,
    pub glossary: Option<&'a str>,
}

/// Halaman siap-kirim: semua milik-owned agar bisa pindah ke worker batch.
pub struct PagePrep {
    pub page_id: String,
    pub target_lang: String,
    pub style: String,
    pub bubbles: Vec<BubbleBox>,
    pub ordered: Vec<usize>,
    pub page_w: i32,
    pub page_h: i32,
    pub prev: Option<PageTranslation>,
    pub prompt: String,
    pub jpeg: Vec<u8>,
    pub rec: ProviderRecord,
    pub client: reqwest::Client,
    pub cache_key: String,
}

/// Unit kerja batch: hanya yang 'static (client/rec/prompt/jpeg).
pub struct PageWork {
    pub page_id: String,
    pub prompt: String,
    pub jpeg: Vec<u8>,
    pub rec: ProviderRecord,
    pub client: reqwest::Client,
}

impl From<&PagePrep> for PageWork {
    fn from(p: &PagePrep) -> Self {
        Self {
            page_id: p.page_id.clone(),
            prompt: p.prompt.clone(),
            jpeg: p.jpeg.clone(),
            rec: p.rec.clone(),
            client: p.client.clone(),
        }
    }
}

/// Snapshot → claim → jpeg → prompt+glossary → provider → cache key.
pub async fn prepare_page(
    state: &State<'_, AppState>,
    opts: PrepOpts<'_>,
) -> Result<PagePrep, String> {
    let snap = load_snapshot(state, opts.page_id)?;
    claim_page(state, opts.page_id)?;

    let style = TranslateStyle::from_str(opts.style);
    let ordered = order_indices(&snap.bubbles, opts.reading_direction);
    let quality = quality_of(opts.mosaic_quality);
    let page_bytes =
        std::fs::read(&snap.path).map_err(|e| fail_page(state, opts.page_id, format!("read: {e}")))?;
    let ordered_bubbles: Vec<BubbleBox> =
        ordered.iter().map(|&i| snap.bubbles[i].clone()).collect();
    let jpeg = tauri::async_runtime::spawn_blocking(move || {
        if ordered_bubbles.is_empty() {
            compress_page(&page_bytes)
        } else {
            build_mosaic(&page_bytes, &ordered_bubbles, quality)
        }
    })
    .await
    .map_err(|e| format!("image task: {e}"))
    .and_then(|r| r)
    .map_err(|e| fail_page(state, opts.page_id, e))?;

    let gloss = effective_glossary(state, opts.glossary, snap.prev.as_ref());
    let prompt = if snap.bubbles.is_empty() {
        append_glossary(
            full_image_prompt(opts.target_lang, style, opts.skip_sfx),
            gloss.as_deref(),
        )
    } else {
        append_glossary(
            build_mosaic_prompt(opts.target_lang, style, opts.skip_sfx, ordered.len()),
            gloss.as_deref(),
        )
    };
    let bubbles_json = serde_json::to_string(&snap.bubbles).map_err(|e| e.to_string())?;
    let (client, rec) = load_provider(state, opts.provider_id)?;
    let key = cache::cache_key(cache::CacheKey {
        image: &jpeg,
        bubbles_json: &bubbles_json,
        target_lang: opts.target_lang,
        style: opts.style,
        model: &rec.model,
        skip_sfx: opts.skip_sfx,
        reading_dir: opts.reading_direction,
        glossary: gloss.as_deref(),
    });
    Ok(PagePrep {
        page_id: snap.page_id,
        target_lang: opts.target_lang.to_string(),
        style: opts.style.to_string(),
        bubbles: snap.bubbles,
        ordered,
        page_w: snap.page_w,
        page_h: snap.page_h,
        prev: snap.prev,
        prompt,
        jpeg,
        rec,
        client,
        cache_key: key,
    })
}

pub fn cache_lookup(state: &State<'_, AppState>, key: &str) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::cache_get(&db, key)
}

/// Terapkan payload cache: preservasi edit user + persist translated.
pub fn apply_cached(
    state: &State<'_, AppState>,
    page_id: &str,
    payload: &str,
    prev: Option<&PageTranslation>,
) -> Result<PageTranslation, String> {
    let mut page_t: PageTranslation =
        serde_json::from_str(payload).map_err(|e| format!("cache corrupt: {e}"))?;
    page_t.bubbles = preserve_user_edits(page_t.bubbles, prev);
    page_t.page_id = page_id.to_string();
    persist(state, page_id, &page_t, "translated")?;
    Ok(page_t)
}

/// Petakan teks provider → PageTranslation memakai geometri prep.
pub fn map_text(prep: &PagePrep, text: &str) -> Result<PageTranslation, String> {
    translate_text_to_page(CoreJob {
        page_id: &prep.page_id,
        target_lang: &prep.target_lang,
        style: &prep.style,
        model: &prep.rec.model,
        bubbles: &prep.bubbles,
        ordered: &prep.ordered,
        page_w: prep.page_w,
        page_h: prep.page_h,
        text,
        prev: prep.prev.as_ref(),
    })
}

/// Simpan hasil ke cache + persist translated.
pub fn store_result(
    state: &State<'_, AppState>,
    prep: &PagePrep,
    page_t: &PageTranslation,
) -> Result<(), String> {
    let payload = serde_json::to_string(page_t).map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::cache_set(&db, &prep.cache_key, &payload)?;
    persist(state, &prep.page_id, page_t, "translated")
}

/// POST (backoff) → map → retry POST sekali saat parse gagal → store.
pub async fn translate_prep(
    state: &State<'_, AppState>,
    prep: &PagePrep,
) -> Result<PageTranslation, String> {
    if let Some(payload) = cache_lookup(state, &prep.cache_key)? {
        return apply_cached(state, &prep.page_id, &payload, prep.prev.as_ref());
    }
    let text = backoff_translate(&prep.client, &prep.rec, &prep.prompt, &prep.jpeg)
        .await
        .map_err(|e| fail_page(state, &prep.page_id, e))?;
    let page_t = match map_text(prep, &text) {
        Ok(p) => p,
        Err(first) => {
            let text2 = backoff_translate(&prep.client, &prep.rec, &prep.prompt, &prep.jpeg)
                .await
                .map_err(|e| fail_page(state, &prep.page_id, e))?;
            map_text(prep, &text2).map_err(|_| fail_page(state, &prep.page_id, first))?
        }
    };
    store_result(state, prep, &page_t)?;
    Ok(page_t)
}

#[tauri::command]
pub async fn translate_page(
    state: State<'_, AppState>,
    input: TranslatePageInput,
) -> Result<PageTranslation, String> {
    let prep = prepare_page(
        &state,
        PrepOpts {
            page_id: &input.page_id,
            provider_id: &input.provider_id,
            target_lang: &input.target_lang,
            style: &input.style,
            skip_sfx: input.skip_sfx,
            mosaic_quality: &input.mosaic_quality,
            reading_direction: &input.reading_direction,
            glossary: input.glossary.as_deref(),
        },
    )
    .await?;
    translate_prep(&state, &prep).await
}

#[tauri::command]
pub fn save_translation(
    state: State<'_, AppState>,
    page_id: String,
    bubbles: Vec<BubbleTranslation>,
) -> Result<PageTranslation, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let page = store
        .projects
        .values_mut()
        .flat_map(|p| p.pages.iter_mut())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?;
    let mut page_t = page.translation.clone().unwrap_or(PageTranslation {
        page_id: page_id.clone(),
        target_lang: "Indonesian".to_string(),
        style: "standard".to_string(),
        model: String::new(),
        bubbles: Vec::new(),
    });
    page_t.page_id = page_id.clone();
    page_t.bubbles = bubbles;
    page.translation = Some(page_t.clone());
    page.status = crate::commands::project::PageStatus::Translated;
    store.save_public()?;
    Ok(page_t)
}

#[tauri::command]
pub fn clear_cache(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::clear_cache(&db)
}

fn persist(state: &State<'_, AppState>, page_id: &str, t: &PageTranslation, status: &str) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let page = store
        .projects
        .values_mut()
        .flat_map(|p| p.pages.iter_mut())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?;
    page.translation = Some(t.clone());
    page.status = match status {
        "translating" => crate::commands::project::PageStatus::Translating,
        "translated" => crate::commands::project::PageStatus::Translated,
        "failed" => crate::commands::project::PageStatus::Failed,
        _ => return Err(format!("unknown status: {status}")),
    };
    store.save_public()
}

/// Persist PageTranslation penuh sebagai translated (dipakai retry/export flow).
pub fn persist_translation(
    state: &State<'_, AppState>,
    page_id: &str,
    t: &PageTranslation,
) -> Result<(), String> {
    persist(state, page_id, t, "translated")
}

fn persist_status_only(state: &State<'_, AppState>, page_id: &str, status: &str) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let page = store
        .projects
        .values_mut()
        .flat_map(|p| p.pages.iter_mut())
        .find(|pg| pg.id == page_id)
        .ok_or_else(|| format!("page not found: {page_id}"))?;
    page.status = match status {
        "translating" => crate::commands::project::PageStatus::Translating,
        "translated" => crate::commands::project::PageStatus::Translated,
        "failed" => crate::commands::project::PageStatus::Failed,
        _ => return Err(format!("unknown status: {status}")),
    };
    store.save_public()
}

pub fn fail_page(state: &State<'_, AppState>, page_id: &str, e: String) -> String {
    let _ = persist_status_only(state, page_id, "failed");
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bubble::BubbleBox;

    #[test]
    fn core_mosaic_maps_geometry() {
        let b = BubbleBox { x: 10, y: 20, w: 60, h: 40, confidence: 1.0, shape: None, kind: None, tail: None };
        let p = translate_text_to_page(CoreJob { page_id: "p", target_lang: "Indonesian", style: "standard", model: "m",
            bubbles: &[b], ordered: &[0], page_w: 800, page_h: 1200,
            text: r#"{"1":{"original":"a","reading":"","translated":"A"}}"#, prev: None }).unwrap();
        assert_eq!(p.bubbles.len(), 1);
        assert_eq!((p.bubbles[0].x, p.bubbles[0].translated.as_str()), (10, "A"));
    }

    #[test]
    fn core_full_image_maps_pct() {
        let p = translate_text_to_page(CoreJob { page_id: "p", target_lang: "Indonesian", style: "standard", model: "m",
            bubbles: &[], ordered: &[], page_w: 1000, page_h: 2000,
            text: r#"[{"x":10,"y":20,"w":30,"h":40,"original":"a","reading":"","translated":"A"}]"#, prev: None }).unwrap();
        assert_eq!((p.bubbles[0].x, p.bubbles[0].y, p.bubbles[0].w, p.bubbles[0].h), (100, 400, 300, 800));
    }

    #[test]
    fn core_parse_error() {
        let bb = [BubbleBox { x: 0, y: 0, w: 10, h: 10, confidence: 1.0, shape: None, kind: None, tail: None }];
        let r = translate_text_to_page(CoreJob { page_id: "p", target_lang: "id", style: "s", model: "m",
            bubbles: &bb, ordered: &[0], page_w: 100, page_h: 100, text: "nope", prev: None });
        assert!(r.is_err());
    }
}
