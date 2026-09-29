use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::translate::{
    CANCELLED_MSG, apply_cached, backoff_translate, cache_lookup, cancel_or_fail, claim_page, effective_glossary,
    fail_page, load_provider, load_snapshot, map_text, persist_translation, store_result, PagePrep,
    PageWork, PrepOpts,
};
use crate::parser::preview_200;
use crate::prompt::{append_glossary, build_mosaic_prompt, TranslateStyle};
use crate::translation::PageTranslation;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateBatchInput {
    pub page_ids: Vec<String>,
    pub provider_id: String,
    #[serde(default = "d_lang")]
    pub target_lang: String,
    #[serde(default = "d_style")]
    pub style: String,
    #[serde(default)]
    pub skip_sfx: bool,
    #[serde(default = "d_quality")]
    pub mosaic_quality: String,
    #[serde(default = "d_dir")]
    pub reading_direction: String,
    #[serde(default)]
    pub glossary: Option<String>,
}

fn d_lang() -> String { "Indonesian".to_string() }
fn d_style() -> String { "standard".to_string() }
fn d_quality() -> String { "high".to_string() }
fn d_dir() -> String { "rtl".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryBubbleInput {
    pub page_id: String,
    pub bubble_index: usize,
    pub provider_id: String,
    #[serde(default = "d_lang")]
    pub target_lang: String,
    #[serde(default = "d_style")]
    pub style: String,
    #[serde(default)]
    pub skip_sfx: bool,
    #[serde(default = "d_quality")]
    pub mosaic_quality: String,
    #[serde(default = "d_dir")]
    pub reading_direction: String,
    #[serde(default)]
    pub glossary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateProgress {
    pub page_id: String,
    pub done: usize,
    pub total: usize,
    pub ok: bool,
    pub message: String,
}

/// Hasil satu worker: dibawa lewat mpsc, di-emit + dipersist di sisi penerima.
struct PageOutcome {
    page_id: String,
    result: Result<PageTranslation, String>,
}

/// Terjemahkan satu halaman penuh dari PageWork (dipakai worker batch).
/// Never panics: semua Err jadi PageOutcome::Err oleh caller.
async fn run_work(
    state: &AppState,
    prep: &PagePrep,
    work: &PageWork,
) -> Result<PageTranslation, String> {
    if let Some(payload) = cache_lookup(state, &prep.cache_key)? {
        return apply_cached(state, &prep.page_id, &payload, prep.prev.as_ref());
    }
    let text = backoff_translate(state, &prep.page_id, &work.client, &work.rec, &work.prompt, &work.jpeg)
        .await
        .map_err(|e| cancel_or_fail(state, &prep.page_id, e))?;
    let page_t = match map_text(prep, &text) {
        Ok(p) => p,
        Err(first) => {
            let text2 = backoff_translate(state, &prep.page_id, &work.client, &work.rec, &work.prompt, &work.jpeg)
                .await
                .map_err(|e| cancel_or_fail(state, &prep.page_id, e))?;
            map_text(prep, &text2).map_err(|_| cancel_or_fail(state, &prep.page_id, first))?
        }
    };
    store_result(state, prep, &page_t)?;
    Ok(page_t)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn translate_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    input: TranslateBatchInput,
) -> Result<Vec<PageTranslation>, String> {
    let total = input.page_ids.len();
    if total == 0 {
        return Ok(vec![]);
    }
    // Fase 1 (sekuensial, murah): snapshot+claim+prepare tiap halaman.
    // Halaman yang gagal prepare langsung jadi progress gagal.
    let mut works: Vec<(PagePrep, PageWork)> = Vec::new();
    let mut pending: Vec<PageOutcome> = Vec::new();
    for id in &input.page_ids {
        match crate::commands::translate::prepare_page(
            state.inner(),
            PrepOpts {
                page_id: id,
                provider_id: &input.provider_id,
                target_lang: &input.target_lang,
                style: &input.style,
                skip_sfx: input.skip_sfx,
                mosaic_quality: &input.mosaic_quality,
                reading_direction: &input.reading_direction,
                glossary: input.glossary.as_deref(),
            },
        )
        .await
        {
            Ok(prep) => {
                let work = PageWork::from(&prep);
                works.push((prep, work));
            }
            Err(e) => pending.push(PageOutcome {
                page_id: id.clone(),
                result: Err(e),
            }),
        }
    }

    // Fase 2: mpsc + semaphore 3 untuk POST LLM. State dipindah per-worker
    // via clone AppHandle + try_state (Mutex di dalam AppState aman di-share).
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PageOutcome>(total.max(1));
    let sem = Arc::new(tokio::sync::Semaphore::new(3));
    for (prep, work) in works {
        let tx = tx.clone();
        let sem = sem.clone();
        let handle = app.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire_owned().await;
            let state_ref: State<'_, AppState> = handle.state();
            let outcome = PageOutcome {
                page_id: prep.page_id.clone(),
                result: run_work(state_ref.inner(), &prep, &work).await,
            };
            let _ = tx.send(outcome).await;
        });
    }
    drop(tx);

    // Fase 3: kumpulkan + emit translate_progress per halaman selesai.
    let mut ok_pages: Vec<PageTranslation> = Vec::new();
    let mut done = 0usize;
    // Progress untuk yang gagal prepare dulu (tetap berurutan done).
    for p in pending {
        done += 1;
        emit_progress(&app, &p.page_id, done, total, false, outcome_msg(&p.result));
    }
    while let Some(out) = rx.recv().await {
        done += 1;
        let ok = out.result.is_ok();
        let msg = outcome_msg(&out.result);
        if let Ok(t) = out.result {
            ok_pages.push(t);
        }
        emit_progress(&app, &out.page_id, done, total, ok, msg);
    }
    // Kembalikan sesuai urutan input.
    ok_pages.sort_by_key(|t| {
        input
            .page_ids
            .iter()
            .position(|id| id == &t.page_id)
            .unwrap_or(usize::MAX)
    });
    Ok(ok_pages)
}

fn outcome_msg(r: &Result<PageTranslation, String>) -> String {
    match r {
        Ok(t) => format!("{} bubble ({})", t.bubbles.len(), t.model),
        Err(e) => e.clone(),
    }
}

fn emit_progress(
    app: &AppHandle,
    page_id: &str,
    done: usize,
    total: usize,
    ok: bool,
    message: String,
) {
    let _ = app.emit(
        "translate_progress",
        TranslateProgress {
            page_id: page_id.to_string(),
            done,
            total,
            ok,
            message,
        },
    );
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retry_bubble(
    state: State<'_, AppState>,
    input: RetryBubbleInput,
) -> Result<PageTranslation, String> {
    let snap = load_snapshot(state.inner(), &input.page_id)?;
    if input.bubble_index >= snap.bubbles.len() {
        return Err(format!(
            "bubble index {} di luar {} bubble",
            input.bubble_index,
            snap.bubbles.len()
        ));
    }
    let prev = snap.prev.clone().ok_or_else(|| {
        format!(
            "belum ada terjemahan untuk retry: {}",
            input.page_id
        )
    })?;
    claim_page(state.inner(), &input.page_id)?;

    let style = TranslateStyle::from_str(&input.style);
    let target = &snap.bubbles[input.bubble_index];
    let page_bytes = std::fs::read(&snap.path)
        .map_err(|e| fail_page(state.inner(), &input.page_id, format!("read: {e}")))?;
    let chip = target.clone();
    let jpeg = tauri::async_runtime::spawn_blocking(move || {
        crate::image_ops::build_mosaic(&page_bytes, &[chip], crate::commands::translate::quality_of(&input.mosaic_quality))
    })
    .await
    .map_err(|e| format!("image task: {e}"))
    .and_then(|r| r)
    .map_err(|e| fail_page(state.inner(), &input.page_id, e))?;

    let gloss = effective_glossary(state.inner(), input.glossary.as_deref(), snap.prev.as_ref());
    let prompt = append_glossary(
        build_mosaic_prompt(&input.target_lang, style, input.skip_sfx, 1),
        gloss.as_deref(),
    );
    let (client, rec) = load_provider(state.inner(), &input.provider_id)?;
    let text = backoff_translate(state.inner(), &input.page_id, &client, &rec, &prompt, &jpeg)
        .await
        .map_err(|e| cancel_or_fail(state.inner(), &input.page_id, e))?;
    let results = crate::parser::parse_mosaic_json(&text)
        .map_err(|e| cancel_or_fail(state.inner(), &input.page_id, format!("parse retry failed: {e} ({})", preview_200(&text))))?;
    let r = results
        .first()
        .map(|(_, r)| r.clone())
        .unwrap_or(crate::parser::BubbleResult {
            original: String::new(),
            reading: String::new(),
            translated: String::new(),
        });

    // Gabung: hanya bubble target diganti, edit user lain dipertahankan.
    let mut merged = prev.clone();
    merged.page_id = input.page_id.clone();
    merged.target_lang = input.target_lang.clone();
    merged.style = input.style.clone();
    merged.model = rec.model.clone();
    if let Some(b) = merged.bubbles.iter_mut().find(|b| b.index == input.bubble_index) {
        b.original = r.original.clone();
        b.reading = r.reading.clone();
        b.translated = r.translated.clone();
        // Retry = user minta AI baru → hasilnya jadi baseline baru + normal.
        b.ai_original = r.original.clone();
        b.ai_reading = r.reading.clone();
        b.ai_translated = r.translated.clone();
        b.is_user_edited = false;
    } else {
        merged.bubbles.push(crate::translation::BubbleTranslation {
            index: input.bubble_index,
            x: target.x,
            y: target.y,
            w: target.w,
            h: target.h,
            original: r.original.clone(),
            reading: r.reading.clone(),
            translated: r.translated.clone(),
            ai_original: r.original.clone(),
            ai_reading: r.reading.clone(),
            ai_translated: r.translated.clone(),
            needs_white_patch: crate::translation::needs_white_patch(
                target.w, target.h, snap.page_w, snap.page_h,
            ),
            is_user_edited: false,
        });
        merged.bubbles.sort_by_key(|b| b.index);
    }
    let merged = crate::translation::preserve_user_edits_except(merged, &prev, input.bubble_index);
    // Cancel pasca-POST: hasil retry yg keburu jadi DIBUANG (sadar 1 token),
    // status pulih pra-klaim, tanpa persist.
    if state.inner().is_cancelled(&input.page_id) {
        let _ = crate::commands::translate::restore_cancelled(state.inner(), &input.page_id);
        return Err(CANCELLED_MSG.to_string());
    }
    persist_translation(state.inner(), &input.page_id, &merged)?;
    Ok(merged)
}
