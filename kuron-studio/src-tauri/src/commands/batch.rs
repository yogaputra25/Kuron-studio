use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::translate::{
    backoff_translate, cancel_or_fail, claim_page, effective_glossary, fail_page,
    load_provider, load_snapshot, persist_translation, prepare_page, restore_cancelled,
    translate_prep, CancelState, Ctx, PagePrep, PrepOpts, ProviderBinding,
};
use crate::error::AppError;
use crate::jobs::{DEFAULT_BATCH_CONCURRENCY, JobRegistry};
use crate::parser::preview_200;
use crate::prompt::{append_custom, append_glossary, build_mosaic_prompt, TranslateStyle};
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
    #[serde(default)]
    pub custom_prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    /// Budget per halaman (detik). None/absen = default; None/<=0 = 900 (defense in depth).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

fn d_lang() -> String { "Indonesian".to_string() }
fn d_style() -> String { "standard".to_string() }
fn d_quality() -> String { "high".to_string() }
fn d_dir() -> String { "rtl".to_string() }

/// Default budget per halaman: 900 detik (15 menit).
pub const DEFAULT_BATCH_PAGE_TIMEOUT_SECS: u64 = 900;

/// Normalisasi budget: None/<=0 = default 900 (defense in depth;
/// frontend juga menormalisasi invalid ke default sebelum mengirim).
pub fn normalize_batch_timeout_secs(timeout_secs: Option<u64>) -> u64 {
    match timeout_secs {
        Some(s) if s > 0 => s,
        _ => DEFAULT_BATCH_PAGE_TIMEOUT_SECS,
    }
}

/// Pesan timeout eksplisit; N = menit bulat (budget/60).
pub fn batch_timeout_message(budget_secs: u64) -> String {
    format!(
        "timeout {} mnt, dilewati — coba Retry Failed",
        budget_secs / 60
    )
}

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
    #[serde(default)]
    pub custom_prompt: Option<String>,
    /// Override model per request; kosong/absen = pakai simpanan provider.
    #[serde(default)]
    pub model: Option<String>,
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

/// Hasil satu tugas prepare: siap POST, atau outcome gagal-yang-dilaporkan.
struct PageOutcome {
    page_id: String,
    result: Result<PageTranslation, AppError>,
}

/// Slot hasil fase-1 berindeks: Work = siap POST; Skip = masuk antrean gagal.
/// Work di-box (PagePrep+Binding jauh lebih besar dari Skip; clippy large_enum_variant).
enum PrepSlot {
    Work(Box<(PagePrep, ProviderBinding)>),
    Skip(PageOutcome),
}

#[tauri::command(rename_all = "snake_case")]
pub async fn translate_batch(
    app: AppHandle,
    _state: State<'_, AppState>,
    _cancel: State<'_, CancelState>,
    input: TranslateBatchInput,
) -> Result<Vec<PageTranslation>, String> {

    // Budget dibaca SEKALI per call; None/<=0 = 900.
    let budget_secs = normalize_batch_timeout_secs(input.timeout_secs);
    let total = input.page_ids.len();
    if total == 0 {
        return Ok(vec![]);
    }
    // Fase 1 (paralel bounded): dedup registry + snapshot+claim+prepare.
    // Duplikat dalam batch langsung jadi progress gagal ("busy", tanpa claim);
    // yang gagal prepare juga + klaim registry dilepas.
    // N = core CPU (clamp 2-8): kerja CPU-lokal tanpa rate-limit, memori bounded.
    // Hasil dikumpulkan berindeks lalu di-replay BERURUTAN supaya urutan
    // works/pending/emit identik dengan mode sekuensial.
    let prep_parallelism = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 8);
    let registry = Arc::new(JobRegistry::new(DEFAULT_BATCH_CONCURRENCY));
    let prep_permits = Arc::new(tokio::sync::Semaphore::new(prep_parallelism));
    let mut prep_set = tokio::task::JoinSet::new();
    for (n, id) in input.page_ids.iter().enumerate() {
        let reg = Arc::clone(&registry);
        let permits = Arc::clone(&prep_permits);
        let handle = app.clone();
        // Clone input per tugas (input dipakai lagi di fase 3 untuk urutan).
        let page_id = id.clone();
        let provider_id = input.provider_id.clone();
        let target_lang = input.target_lang.clone();
        let style = input.style.clone();
        let skip_sfx = input.skip_sfx;
        let mosaic_quality = input.mosaic_quality.clone();
        let reading_direction = input.reading_direction.clone();
        let glossary = input.glossary.clone();
        let custom_prompt = input.custom_prompt.clone();
        let model = input.model.clone();
        prep_set.spawn(async move {
            let _permit = permits.acquire_owned().await;
            let state_ref: State<'_, AppState> = handle.state();
            let cancel_ref: State<'_, CancelState> = handle.state();
            let ctx = Ctx {
                app: state_ref.inner(),
                cancel: cancel_ref.inner(),
            };
            if !reg.admit(&page_id) {
                return (
                    n,
                    PrepSlot::Skip(PageOutcome {
                        page_id: page_id.clone(),
                        result: Err(AppError::busy(&page_id)),
                    }),
                );
            }
            let prepared = tokio::time::timeout(
                std::time::Duration::from_secs(budget_secs),
                prepare_page(
                    &ctx,
                    PrepOpts {
                        page_id: &page_id,
                        provider_id: &provider_id,
                        target_lang: &target_lang,
                        style: &style,
                        skip_sfx,
                        mosaic_quality: &mosaic_quality,
                        reading_direction: &reading_direction,
                        glossary: glossary.as_deref(),
                        custom_prompt: custom_prompt.as_deref(),
                        model: model.as_deref(),
                    },
                ),
            )
            .await;
            match prepared {
                Ok(Ok((prep, binding))) => (n, PrepSlot::Work(Box::new((prep, binding)))),
                Ok(Err(e)) => {
                    reg.release(&page_id);
                    (
                        n,
                        PrepSlot::Skip(PageOutcome {
                            page_id,
                            result: Err(e),
                        }),
                    )
                }
                Err(_) => {
                    // Budget habis: klaim basi di-restore (bukan fail) supaya
                    // translate manual tak tertolak "page busy".
                    let _ = restore_cancelled(&ctx, &page_id);
                    reg.release(&page_id);
                    (
                        n,
                        PrepSlot::Skip(PageOutcome {
                            page_id,
                            result: Err(AppError::provider(batch_timeout_message(budget_secs))),
                        }),
                    )
                }
            }
        });
    }
    // Kumpulkan slot + emit LANGSUNG per worker selesai (done tetap 0 —
    // belum ada outcome; pesan "Menyiapkan N/M" jalan real-time).
    // Urutan hasil akhir tetap input-order via replay_prepared di bawah.
    let mut slots: Vec<(usize, PrepSlot)> = Vec::with_capacity(total);
    let mut completed = 0usize;
    while let Some(res) = prep_set.join_next().await {
        let Ok((n, slot)) = res else { continue };
        completed += 1;
        let (page_id, message) = match &slot {
            PrepSlot::Work(boxed) => (
                boxed.0.page_id.clone(),
                prepare_progress_message(completed, total),
            ),
            PrepSlot::Skip(out) => (
                out.page_id.clone(),
                prepare_progress_message(completed, total),
            ),
        };
        emit_progress(&app, &page_id, 0, total, true, message);
        slots.push((n, slot));
    }
    // Replay BERURUTAN input: urutan works/pending identik mode sekuensial
    // (teruji unit via replay_prepared_urut_deterministik_walau_acak).
    let (works, pending) = replay_prepared(slots);
    // Fase 2: mpsc + semaphore registry untuk POST LLM. State dipindah
    // per-worker via clone AppHandle + try_state (Mutex di dalam AppState
    // aman di-share). Pre-flight cancel di translate_prep: job antre yang
    // dibatalkan restore tanpa POST.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PageOutcome>(total.max(1));
    let permits = registry.semaphore();
    for (prep, binding) in works {
        let tx = tx.clone();
        let reg = Arc::clone(&registry);
        let permits = Arc::clone(&permits);
        let handle = app.clone();
        tokio::spawn(async move {
            let _permit = permits.acquire_owned().await;
            let state_ref: State<'_, AppState> = handle.state();
            let cancel_ref: State<'_, CancelState> = handle.state();
            let page_id = prep.page_id.clone();
            let result = match tokio::time::timeout(
                std::time::Duration::from_secs(budget_secs),
                translate_prep(
                    &Ctx { app: state_ref.inner(), cancel: cancel_ref.inner() },
                    &prep,
                    &binding,
                ),
            )
            .await
            {
                Ok(r) => r,
                Err(_) => Err(AppError::provider(batch_timeout_message(budget_secs))),
            };
            reg.release(&page_id);
            let _ = tx.send(PageOutcome { page_id, result }).await;
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

/// Pesan progres fase-1 ("Menyiapkan i/M..."). Fungsi terpisah agar formatnya
/// teruji unit; emit-nya sendiri butuh AppHandle (diuji integrasi/manual).
fn prepare_progress_message(n: usize, total: usize) -> String {
    format!("Menyiapkan {n}/{total}...")
}

/// Replay hasil fase-1 berurutan input: urutan works/pending identik dengan
/// mode sekuensial walau task selesai acak. Murni (tanpa AppHandle) → teruji.
fn replay_prepared(
    mut slots: Vec<(usize, PrepSlot)>,
) -> (Vec<(PagePrep, ProviderBinding)>, Vec<PageOutcome>) {
    slots.sort_by_key(|(n, _)| *n);
    let mut works = Vec::with_capacity(slots.len());
    let mut pending = Vec::new();
    for (_, slot) in slots {
        match slot {
            PrepSlot::Work(boxed) => works.push(*boxed),
            PrepSlot::Skip(out) => pending.push(out),
        }
    }
    (works, pending)
}

fn outcome_msg(r: &Result<PageTranslation, AppError>) -> String {
    match r {
        Ok(t) => format!("{} bubble ({})", t.bubbles.len(), t.model),
        Err(e) => e.to_string(),
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
    cancel: State<'_, CancelState>,
    input: RetryBubbleInput,
) -> Result<PageTranslation, String> {
    let ctx = Ctx { app: state.inner(), cancel: cancel.inner() };
    let snap = load_snapshot(ctx.app, &input.page_id)?;
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
    claim_page(&ctx, &input.page_id)?;

    let style = TranslateStyle::from_str(&input.style);
    let target = &snap.bubbles[input.bubble_index];
    let page_bytes = std::fs::read(&snap.path)
        .map_err(|e| fail_page(ctx.app, &input.page_id, AppError::io(format!("read: {e}"))))?;
    let chip = target.clone();
    let jpeg = tauri::async_runtime::spawn_blocking(move || {
        crate::render::image_ops::build_mosaic(&page_bytes, &[chip], crate::commands::translate::quality_of(&input.mosaic_quality))
    })
    .await
    .map_err(|e| AppError::io(format!("image task: {e}")))
    .and_then(|r| r)
    .map_err(|e| fail_page(ctx.app, &input.page_id, e))?;

    let gloss = effective_glossary(ctx.app, input.glossary.as_deref(), snap.prev.as_ref());
    let prompt = append_custom(
        append_glossary(
            build_mosaic_prompt(&input.target_lang, style, input.skip_sfx, 1),
            gloss.as_deref(),
        ),
        input.custom_prompt.as_deref(),
    );
    let (client, mut rec) = load_provider(ctx.app, &input.provider_id)?;
    rec.model = crate::provider::effective_model(&rec, input.model.as_deref());
    let text = backoff_translate(ctx.cancel, &input.page_id, &client, &rec, &prompt, &jpeg)
        .await
        .map_err(|e| cancel_or_fail(&ctx, &input.page_id, e))?;
    let results = crate::parser::parse_mosaic_json(&text)
        .map_err(|e| cancel_or_fail(&ctx, &input.page_id, AppError::parse(format!("parse retry failed: {e} ({})", preview_200(&text)))))?;
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
            shape: target.shape.clone(),
            layout: None,
            font_size: None,
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
    if ctx.cancel.is_cancelled(&input.page_id) {
        let _ = restore_cancelled(&ctx, &input.page_id);
        return Err(AppError::cancelled().into());
    }
    persist_translation(&ctx, &input.page_id, &merged)?;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_timeout_is_900_when_none() {
        assert_eq!(normalize_batch_timeout_secs(None), 900);
        assert_eq!(
            normalize_batch_timeout_secs(Some(0)),
            DEFAULT_BATCH_PAGE_TIMEOUT_SECS
        );
    }

    #[test]
    fn timeout_message_uses_whole_minutes() {
        assert_eq!(
            batch_timeout_message(900),
            "timeout 15 mnt, dilewati — coba Retry Failed"
        );
    }

    #[test]
    fn legacy_wire_without_timeout_parses() {
        let v: TranslateBatchInput = serde_json::from_value(serde_json::json!({
            "pageIds": ["p1"],
            "providerId": "prov"
        }))
        .expect("wire lama tanpa timeout_secs tetap parse");
        assert_eq!(v.timeout_secs, None);
        assert_eq!(normalize_batch_timeout_secs(v.timeout_secs), 900);
    }

    /// Worker kedaluwarsa tercatat failed (pesan timeout eksplisit) + antrean lanjut.
    #[tokio::test]
    async fn expired_worker_fails_and_queue_continues() {
        let budget_secs = 1u64; // super-kecil: sleep melebihi budget
        let slow: Result<(), AppError> = match tokio::time::timeout(
            std::time::Duration::from_secs(budget_secs),
            async {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            },
        )
        .await
        {
            Ok(()) => Ok(()),
            Err(_) => Err(AppError::provider(batch_timeout_message(budget_secs))),
        };
        let err = slow.expect_err("pekerja lambat harus kedaluwarsa");
        assert_eq!(
            err.to_string(),
            "timeout 0 mnt, dilewati — coba Retry Failed"
        );
        // Antrean lanjut: halaman berikutnya tetap diproses.
        let fast: Result<&str, AppError> = match tokio::time::timeout(
            std::time::Duration::from_secs(budget_secs),
            async { "ok" },
        )
        .await
        {
            Ok(v) => Ok(v),
            Err(_) => Err(AppError::provider(batch_timeout_message(budget_secs))),
        };
        assert_eq!(fast.unwrap(), "ok");
    }

    /// Prepare kedaluwarsa: klaim di-restore (bukan fail) supaya single
    /// translate tidak tertolak "page busy", + pesan timeout eksplisit.
    #[tokio::test]
    async fn expired_prepare_restores_claim_and_reports_timeout() {
        use crate::commands::detect::DetectState;
        use crate::domain::{Page, PageStatus, Project};
        use crate::store::ProjectStore;
        let dir = std::env::temp_dir().join(format!(
            "kuron-prep-timeout-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut store = ProjectStore::load(dir.clone()).unwrap();
        let mut proj = Project::new("ch".into());
        proj.id = "p".into();
        proj.folders[0].pages = vec![Page {
            id: "p0".into(),
            path: "ch/001.png".into(),
            width: 800,
            height: 1200,
            status: PageStatus::Detected,
            bubbles: vec![],
            translation: None,
        }];
        store.projects.insert("p".into(), proj);
        let st = AppState {
            store: std::sync::Mutex::new(store),
            detect: DetectState::default(),
            http: reqwest::Client::new(),
            db: std::sync::Mutex::new(rusqlite::Connection::open_in_memory().unwrap()),
            previews: std::sync::Mutex::new(crate::preview_cache::PreviewCaches::new()),
            data_dir: dir,
        };
        let cs = CancelState::default();
        let ctx = Ctx { app: &st, cancel: &cs };
        // Klaim seperti prepare_page, lalu timeout seperti cabang Elapsed fase 1.
        claim_page(&ctx, "p0").unwrap();
        let slow: Result<(), AppError> = match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            async {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            },
        )
        .await
        {
            Ok(()) => Ok(()),
            Err(_) => {
                let _ = restore_cancelled(&ctx, "p0");
                Err(AppError::provider(batch_timeout_message(900)))
            }
        };
        let err = slow.expect_err("prepare lambat harus kedaluwarsa");
        assert_eq!(
            err.to_string(),
            "timeout 15 mnt, dilewati — coba Retry Failed"
        );
        let status = st
            .store
            .lock()
            .unwrap()
            .find_page("p0")
            .unwrap()
            .status
            .clone();
        assert_eq!(status, PageStatus::Detected);
    }

    #[test]
    fn prepare_progress_message_menyiapkan_counter() {
        assert_eq!(prepare_progress_message(3, 454), "Menyiapkan 3/454...");
    }

    fn dummy_outcome(id: &str) -> PageOutcome {
        PageOutcome {
            page_id: id.into(),
            result: Err(AppError::busy(id)),
        }
    }

    #[test]
    fn replay_prepared_urut_deterministik_walau_selesai_acak() {
        // Simulasi task selesai acak (2, 0, 1): replay harus urut 0, 1, 2.
        let slots = vec![
            (
                2usize,
                PrepSlot::Skip(dummy_outcome("p2")),
            ),
            (
                0usize,
                PrepSlot::Skip(dummy_outcome("p0")),
            ),
            (
                1usize,
                PrepSlot::Skip(dummy_outcome("p1")),
            ),
        ];
        let (works, pending) = replay_prepared(slots);
        assert!(works.is_empty());
        let ids: Vec<&str> = pending.iter().map(|p| p.page_id.as_str()).collect();
        assert_eq!(ids, vec!["p0", "p1", "p2"]);
    }
}
