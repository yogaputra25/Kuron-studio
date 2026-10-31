//! Core translation pipeline: preparation, backoff, cache, execution, and persistence.
//! Dipindah dari `commands/translate.rs` — tanpa perubahan perilaku.

use crate::bubble::BubbleBox;
use crate::cache;
use crate::error::AppError;
use crate::parser::{parse_full_image_json, parse_mosaic_json, preview_200};
use crate::prompt::{append_custom, append_glossary, build_mosaic_prompt, full_image_prompt, TranslateStyle};
use crate::provider;
use crate::provider::config::ProviderRecord;
use crate::render::image_ops::{build_mosaic, compress_page, MosaicQuality};
use crate::translation::{
    map_full_image, map_mosaic, merge_ai_baseline, order_indices, preserve_user_edits,
    BubbleTranslation, PageTranslation,
};
use crate::AppState;

use super::provider_bind::{bind_provider, ProviderBinding};
use super::snapshot::load_snapshot;

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

pub fn translate_text_to_page(job: CoreJob<'_>) -> Result<PageTranslation, AppError> {
    let mapped = if job.bubbles.is_empty() {
        let items = parse_full_image_json(job.text)
            .map_err(|e| AppError::parse(format!("parse full-image failed: {e} ({})", preview_200(job.text))))?;
        map_full_image(job.page_w, job.page_h, &items)
    } else {
        let results = parse_mosaic_json(job.text)
            .map_err(|e| AppError::parse(format!("parse mosaic failed: {e} ({})", preview_200(job.text))))?;
        map_mosaic(job.bubbles, job.ordered, &results, job.page_w, job.page_h)
    };
    Ok(PageTranslation {
        page_id: job.page_id.to_string(),
        target_lang: job.target_lang.to_string(),
        style: job.style.to_string(),
        model: job.model.to_string(),
        bubbles: preserve_user_edits(mapped, job.prev),
        // Flag manual menyatu di persist_result (nempel ke simpanan),
        // bukan di hasil segar — re-translate tak mencabut override user.
        manual_done: false,
    })
}

/// Klaim atomik: halaman Translating ditolak ("busy") oleh penelepon kedua.
/// Stash status pra-klaim ke `AppState::prev_status` (path cancel restore ini,
/// bukan `fail_page`). `&AppState` (bukan `&State`) agar unit-test tanpa
/// runtime Tauri. Flag cancel basi SENGAJA tak dibuang di sini.
pub fn claim_page(state: &AppState, page_id: &str) -> Result<(), AppError> {
    let prev = {
        let mut store = crate::error::lock_or_err(&state.store)?;
        let page = store.find_page_mut(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
        if page.status == crate::domain::PageStatus::Translating {
            return Err(crate::error::AppError::busy("halaman sedang diterjemahkan"));
        }
        let prev = page.status.clone();
        page.status = crate::domain::PageStatus::Translating;
        store.save_public()?;
        prev
    };
    state.stash_prev(page_id, prev);
    Ok(())
}

/// Satu-satunya string pemicu path restore (bukan `fail_page`).
/// Sinkron dengan `crate::error::AppError::Cancelled`.
/// Didefinisikan di sini (#56) agar lib/error/commands punya SATU sumber.
///
/// Unit test `error::tests::cancelled_message_sync` menjaga konstanta ini
/// dan display `AppError::Cancelled` tidak pecah; modul lain memakai
/// konstanta ini atau `AppError::cancelled()` — JANGAN ketik string literal
/// di tempat lain. Pembeda penting: error ini adalah SATU-SATUNYA yang memicu
/// path restore, bukan `fail_page`.
pub const CANCELLED_MSG: &str = "dibatalkan oleh user";

/// Path cancel: kembalikan status pra-klaim + buang flag. Bukan `fail_page`.
/// Tanpa stash (tak pernah klaim) = no-op status, flag tetap dibuang.
pub fn restore_cancelled(state: &AppState, page_id: &str) -> Result<(), AppError> {
    let prev = state.take_prev(page_id);
    state.clear_cancel(page_id);
    if let Some(st) = prev {
        let mut store = crate::error::lock_or_err(&state.store)?;
        if let Some(page) = store.find_page_mut(page_id) {
            page.status = st;
            store.save_public()?;
        }
    }
    Ok(())
}

/// Helper sentral cancel-atau-fail (#56):
/// Error `Cancelled` -> panggil `restore_cancelled` (status kembali pra-klaim).
/// Error lainnya -> catat status `failed` via `fail_page`.
/// Perbandingan cancel via `matches!` atas varian, bukan string.
pub fn cancel_or_fail(state: &AppState, page_id: &str, e: AppError) -> AppError {
    if matches!(e, AppError::Cancelled) {
        let _ = restore_cancelled(state, page_id);
        e
    } else {
        state.clear_cancel(page_id);
        state.take_prev(page_id);
        fail_page(state, page_id, e)
    }
}

/// Glossary efektif: override eksplisit menang; None → auto-context
/// (most-recent-5 bila tak ada teks relevan). Never throws: DB error → None.
pub fn effective_glossary(
    state: &AppState,
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
/// provider). Error non-429 langsung pulang. `page_id` kosong = tanpa cek
/// cancel (kompat); non-kosong = cek flag tiap iterasi + `select!` saat sleep.
/// Cek ulang pasca-parse: POST yg keburu selesai saat cancel DIBUANG (satu
/// call token tetap kepakai — sadar, bukan refund).
pub async fn backoff_translate(
    state: &AppState,
    page_id: &str,
    client: &reqwest::Client,
    rec: &ProviderRecord,
    prompt: &str,
    jpeg: &[u8],
) -> Result<String, crate::error::AppError> {
    let mut last = crate::error::AppError::provider("rate_limited: retry later");
    for (n, wait) in [0u64, 2, 4, 8].iter().enumerate() {
        if state.is_cancelled(page_id) {
            state.clear_cancel(page_id);
            return Err(crate::error::AppError::cancelled());
        }
        if *wait > 0 {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(*wait)) => {}
                _ = async {
                    loop {
                        if state.is_cancelled(page_id) { break; }
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                } => {
                    state.clear_cancel(page_id);
                    return Err(crate::error::AppError::cancelled());
                }
            }
        }
        match provider::translate_image(client, rec, prompt, jpeg).await {
            Ok(t) => {
                if state.is_cancelled(page_id) {
                    state.clear_cancel(page_id);
                    return Err(crate::error::AppError::cancelled());
                }
                return Ok(t);
            }
            Err(e) if e.to_string().contains("rate_limited") && n < 3 => {
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
    pub custom_prompt: Option<&'a str>,
    pub model: Option<&'a str>,
}

/// Halaman siap-kirim yang provider-agnostik (#55, persiapan #66 failover):
/// baca gambar + mosaic/compress + prompt + glossary. `ProviderBinding`
/// yang murah-diulang dipisah supaya ganti provider tak bangun ulang ini.
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
}

/// Snapshot → claim → jpeg → prompt+glossary → bind provider.
/// Mengembalikan prep agnostik + binding; gagal-bind = halaman belum mulai
/// kerja → status pulih pra-klaim (bukan `fail_page`), klaim dilepas.
pub async fn prepare_page(
    state: &AppState,
    opts: PrepOpts<'_>,
) -> Result<(PagePrep, ProviderBinding), AppError> {
    let snap = load_snapshot(state, opts.page_id)?;
    claim_page(state, opts.page_id)?;

    let style = TranslateStyle::from_str(opts.style);
    let ordered = order_indices(&snap.bubbles, opts.reading_direction);
    let quality = quality_of(opts.mosaic_quality);
    let page_bytes =
        std::fs::read(&snap.path).map_err(|e| fail_page(state, opts.page_id, AppError::io(format!("read: {e}"))))?;
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
    .map_err(|e| AppError::io(format!("image task: {e}")))
    .and_then(|r| r)
    .map_err(|e| fail_page(state, opts.page_id, e))?;

    let gloss = effective_glossary(state, opts.glossary, snap.prev.as_ref());
    let prompt = if snap.bubbles.is_empty() {
        append_custom(
            append_glossary(
                full_image_prompt(opts.target_lang, style, opts.skip_sfx),
                gloss.as_deref(),
            ),
            opts.custom_prompt,
        )
    } else {
        append_custom(
            append_glossary(
                build_mosaic_prompt(opts.target_lang, style, opts.skip_sfx, ordered.len()),
                gloss.as_deref(),
            ),
            opts.custom_prompt,
        )
    };
    let prep = PagePrep {
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
    };
    let binding = bind_provider(state, &opts, &prep, gloss.as_deref())?;
    Ok((prep, binding))
}

pub fn cache_lookup(state: &AppState, key: &str) -> Result<Option<String>, AppError> {
    let db = crate::error::lock_or_err(&state.db)?;
    cache::cache_get(&db, key)
}

/// Terapkan payload cache: preservasi edit user + persist translated.
pub fn apply_cached(
    state: &AppState,
    page_id: &str,
    payload: &str,
    prev: Option<&PageTranslation>,
) -> Result<PageTranslation, AppError> {
    let mut page_t: PageTranslation = serde_json::from_str(payload)?;
    page_t.page_id = page_id.to_string();
    page_t.bubbles = preserve_user_edits(page_t.bubbles, prev);
    persist_result(state, page_id, &page_t)?;
    state.clear_cancel(page_id);
    state.take_prev(page_id);
    Ok(page_t)
}

/// Petakan teks provider → PageTranslation memakai geometri prep + model binding.
pub fn map_text(prep: &PagePrep, model: &str, text: &str) -> Result<PageTranslation, AppError> {
    translate_text_to_page(CoreJob {
        page_id: &prep.page_id,
        target_lang: &prep.target_lang,
        style: &prep.style,
        model,
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
    state: &AppState,
    prep: &PagePrep,
    binding: &ProviderBinding,
    page_t: &PageTranslation,
) -> Result<(), AppError> {
    let payload = serde_json::to_string(page_t)?;
    let db = crate::error::lock_or_err(&state.db)?;
    cache::cache_set(&db, &binding.cache_key, &payload)?;
    let r = persist_result(state, &prep.page_id, page_t);
    state.clear_cancel(&prep.page_id);
    state.take_prev(&prep.page_id);
    r
}

/// Orkestrasi tunggal halaman siap-kirim (#55): dipakai `translate_page`
/// dan worker `translate_batch` — JANGAN bikin salinan kedua.
/// Pre-flight cancel: job antre yang keburu dibatalkan langsung restore
/// tanpa sentuh cache/POST. Lalu cache → POST (backoff) → map → retry POST
/// sekali saat parse gagal → store.
pub async fn translate_prep(
    state: &AppState,
    prep: &PagePrep,
    binding: &ProviderBinding,
) -> Result<PageTranslation, AppError> {
    if state.is_cancelled(&prep.page_id) {
        return Err(cancel_or_fail(state, &prep.page_id, crate::error::AppError::cancelled()));
    }
    if let Some(payload) = cache_lookup(state, &binding.cache_key)? {
        return apply_cached(state, &prep.page_id, &payload, prep.prev.as_ref());
    }
    let text = backoff_translate(state, &prep.page_id, &binding.client, &binding.rec, &prep.prompt, &prep.jpeg)
        .await
        .map_err(|e| cancel_or_fail(state, &prep.page_id, e))?;
    let page_t = match map_text(prep, &binding.rec.model, &text) {
        Ok(p) => p,
        Err(first) => {
            let text2 = backoff_translate(state, &prep.page_id, &binding.client, &binding.rec, &prep.prompt, &prep.jpeg)
                .await
                .map_err(|e| cancel_or_fail(state, &prep.page_id, e))?;
            map_text(prep, &binding.rec.model, &text2).map_err(|_| cancel_or_fail(state, &prep.page_id, first))?
        }
    };
    store_result(state, prep, binding, &page_t)?;
    Ok(page_t)
}

/// Inti `save_translation` tanpa runtime Tauri (bisa di-unit-test).
pub fn save_translation_inner(
    state: &AppState,
    page_id: &str,
    bubbles: Vec<BubbleTranslation>,
) -> Result<PageTranslation, AppError> {
    let prev = state.take_prev(page_id);
    let mut store = crate::error::lock_or_err(&state.store)?;
    let page = store.find_page_mut(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
    let mut page_t = page.translation.clone().unwrap_or(PageTranslation {
        page_id: page_id.to_string(),
        target_lang: "Indonesian".to_string(),
        style: "standard".to_string(),
        model: String::new(),
        manual_done: false,
        bubbles: Vec::new(),
    });
    page_t.page_id = page_id.to_string();
    page_t.bubbles = merge_ai_baseline(&page_t.bubbles, bubbles);
    page.translation = Some(page_t.clone());
    crate::domain::apply_translation_status(page, &page_t, prev);
    store.save_public()?;
    Ok(page_t)
}

/// Inti `mark_page_completed` tanpa runtime Tauri (bisa di-unit-test).
pub fn mark_page_completed_inner(
    state: &AppState,
    page_id: &str,
    completed: bool,
) -> Result<PageTranslation, AppError> {
    use crate::domain::PageStatus as S;
    let mut store = crate::error::lock_or_err(&state.store)?;
    let page = store.find_page_mut(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
    let mut page_t = page
        .translation
        .clone()
        .ok_or_else(|| crate::error::AppError::no_translation(page_id))?;
    page_t.manual_done = completed;
    page.translation = Some(page_t.clone());
    page.status = if crate::translation::translation_complete(&page_t) {
        S::Translated
    } else if page.bubbles.is_empty() {
        S::Idle
    } else {
        S::Detected
    };
    store.save_public()?;
    Ok(page_t)
}

/// Persist hasil terjemahan + status jujur: `Translated` hanya bila lengkap
/// (semua terisi ATAU flag manual); selain itu status lama dipertahankan,
/// transient `Translating` pulih ke pra-klaim (stash) atau Detected/Idle.
pub fn persist_result(
    state: &AppState,
    page_id: &str,
    t: &PageTranslation,
) -> Result<(), AppError> {
    let prev = state.take_prev(page_id);
    let mut store = crate::error::lock_or_err(&state.store)?;
    let page = store.find_page_mut(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
    // Override manual nempel: re-translate tak mencabut tandai-selesai user.
    let mut merged = t.clone();
    if page.translation.as_ref().is_some_and(|p| p.manual_done) {
        merged.manual_done = true;
    }
    page.translation = Some(merged.clone());
    crate::domain::apply_translation_status(page, &merged, prev);
    store.save_public()
}

/// Persist PageTranslation penuh + status jujur (dipakai retry/export flow).
pub fn persist_translation(
    state: &AppState,
    page_id: &str,
    t: &PageTranslation,
) -> Result<(), AppError> {
    let r = persist_result(state, page_id, t);
    state.clear_cancel(page_id);
    state.take_prev(page_id);
    r
}

fn persist_status_only(state: &AppState, page_id: &str, status: &str) -> Result<(), AppError> {
    let mut store = crate::error::lock_or_err(&state.store)?;
    let page = store.find_page_mut(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
    page.status = match status {
        "translating" => crate::domain::PageStatus::Translating,
        "translated" => crate::domain::PageStatus::Translated,
        "failed" => crate::domain::PageStatus::Failed,
        _ => return Err(AppError::parse(format!("unknown status: {status}"))),
    };
    store.save_public()
}

pub fn fail_page(state: &AppState, page_id: &str, e: AppError) -> AppError {
    let _ = persist_status_only(state, page_id, "failed");
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bubble::BubbleBox;
    use crate::commands::detect::DetectState;
    use crate::store::ProjectStore;
    use crate::domain::{Page, PageStatus, Project};

    fn test_state_with_page(id: &str, status: PageStatus) -> AppState {
        let dir = std::env::temp_dir().join(format!(
            "kuron-cancel-{}-{}",
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
            id: id.into(),
            path: "ch/001.png".into(),
            width: 800,
            height: 1200,
            status,
            bubbles: vec![],
            translation: None,
        }];
        store.projects.insert("p".into(), proj);
        AppState {
            store: std::sync::Mutex::new(store),
            detect: DetectState::default(),
            http: reqwest::Client::new(),
            db: std::sync::Mutex::new(rusqlite::Connection::open_in_memory().unwrap()),
            previews: std::sync::Mutex::new(crate::preview_cache::PreviewCaches::new()),
            cancel: std::sync::Mutex::new(std::collections::HashSet::new()),
            prev_status: std::sync::Mutex::new(std::collections::HashMap::new()),
            data_dir: dir,
        }
    }

    fn page_status_of(state: &AppState, id: &str) -> PageStatus {
        state
            .store
            .lock()
            .unwrap()
            .find_page(id)
            .unwrap()
            .status
            .clone()
    }

    #[test]
    fn cancel_restores_preclaim_status_translated() {
        let st = test_state_with_page("p0", PageStatus::Translated);
        claim_page(&st, "p0").unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Translating);
        st.request_cancel("p0");
        let msg = cancel_or_fail(&st, "p0", AppError::cancelled());
        assert_eq!(msg.to_string(), CANCELLED_MSG);
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Translated);
        assert!(!st.is_cancelled("p0"));
        assert!(st.take_prev("p0").is_none());
    }

    #[test]
    fn cancel_restores_preclaim_status_detected() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        claim_page(&st, "p0").unwrap();
        st.request_cancel("p0");
        cancel_or_fail(&st, "p0", AppError::cancelled());
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Detected);
    }

    #[test]
    fn cancel_without_claim_is_noop_status_but_clears_flag() {
        let st = test_state_with_page("p0", PageStatus::Idle);
        st.request_cancel("p0");
        let msg = cancel_or_fail(&st, "p0", AppError::cancelled());
        assert_eq!(msg.to_string(), CANCELLED_MSG);
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Idle);
        assert!(!st.is_cancelled("p0"));
    }

    #[test]
    fn noncancel_error_still_fails_and_cleans_stash() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        claim_page(&st, "p0").unwrap();
        let e = cancel_or_fail(&st, "p0", AppError::parse("boom"));
        assert_eq!(e.to_string(), "boom");
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Failed);
        assert!(st.take_prev("p0").is_none());
        assert!(!st.is_cancelled("p0"));
    }

    #[tokio::test]
    async fn backoff_aborts_on_preset_flag_without_provider_call() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        st.request_cancel("p0");
        // base_url tak-routable: bila provider sempat dipanggil, Err non-cancel.
        let rec = ProviderRecord {
            id: "x".into(),
            provider_type: crate::provider::config::AiProviderType::OpenAi,
            name: "x".into(),
            base_url: "http://127.0.0.1:1".into(),
            api_key: "k".into(),
            model: "m".into(),
        };
        let client = reqwest::Client::new();
        let r = backoff_translate(&st, "p0", &client, &rec, "p", &[]).await;
        assert_eq!(r, Err(AppError::cancelled()));
        assert!(!st.is_cancelled("p0"));
    }

    #[tokio::test]
    async fn translate_prep_preflight_cancel_tanpa_post_atau_cache() {
        // Job antre batch yang keburu dibatalkan: restore pra-klaim tanpa
        // sentuh cache (db in-memory tanpa tabel) maupun POST (tak-routable).
        let st = test_state_with_page("p0", PageStatus::Detected);
        claim_page(&st, "p0").unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Translating);
        st.request_cancel("p0");
        let prep = PagePrep {
            page_id: "p0".into(),
            target_lang: "id".into(),
            style: "s".into(),
            bubbles: vec![],
            ordered: vec![],
            page_w: 800,
            page_h: 1200,
            prev: None,
            prompt: "p".into(),
            jpeg: vec![],
        };
        let binding = ProviderBinding {
            rec: ProviderRecord {
                id: "x".into(),
                provider_type: crate::provider::config::AiProviderType::OpenAi,
                name: "x".into(),
                base_url: "http://127.0.0.1:1".into(),
                api_key: "k".into(),
                model: "m".into(),
            },
            client: reqwest::Client::new(),
            cache_key: "tak-ada".into(),
        };
        let r = translate_prep(&st, &prep, &binding).await;
        assert_eq!(r, Err(AppError::cancelled()));
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Detected);
        assert!(!st.is_cancelled("p0"));
        assert!(st.take_prev("p0").is_none());
    }

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

    fn tr_bubble(i: usize, t: &str) -> BubbleTranslation {
        BubbleTranslation {
            index: i, x: 0, y: 0, w: 10, h: 10, shape: None, layout: None,
            font_size: None, original: "o".into(), reading: "".into(),
            translated: t.into(), ai_original: "".into(), ai_reading: "".into(),
            ai_translated: "".into(), needs_white_patch: false, is_user_edited: false,
        }
    }

    fn tr_full(tr: &[&str]) -> Vec<BubbleTranslation> {
        tr.iter().enumerate().map(|(i, t)| tr_bubble(i, t)).collect()
    }

    /// Seed translation + geometri langsung di store test.
    fn seed_tr(st: &AppState, status: PageStatus, tr: Vec<BubbleTranslation>) {
        let mut store = st.store.lock().unwrap();
        let page = store.find_page_mut("p0").unwrap();
        page.status = status;
        page.bubbles = tr
            .iter()
            .map(|b| BubbleBox {
                x: b.x, y: b.y, w: b.w, h: b.h, confidence: 1.0,
                shape: None, kind: None, tail: None,
            })
            .collect();
        page.translation = Some(PageTranslation {
            page_id: "p0".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: tr,
        });
    }

    #[test]
    fn partial_save_keeps_previous_status() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        seed_tr(&st, PageStatus::Detected, tr_full(&["isi", ""]));
        save_translation_inner(&st, "p0", tr_full(&["isi", ""])).unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Detected);
    }

    #[test]
    fn complete_save_marks_translated() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        seed_tr(&st, PageStatus::Detected, tr_full(&["isi", ""]));
        save_translation_inner(&st, "p0", tr_full(&["isi", "lengkap"])).unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Translated);
    }

    #[test]
    fn empty_manual_save_is_not_translated() {
        let st = test_state_with_page("p0", PageStatus::Idle);
        save_translation_inner(&st, "p0", vec![]).unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Idle);
    }

    #[test]
    fn mark_done_forces_translated_even_partial() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        seed_tr(&st, PageStatus::Detected, tr_full(&["isi", ""]));
        let out = mark_page_completed_inner(&st, "p0", true).unwrap();
        assert!(out.manual_done);
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Translated);
    }

    #[test]
    fn unmark_recomputes_from_data() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        seed_tr(&st, PageStatus::Detected, tr_full(&["isi", ""]));
        mark_page_completed_inner(&st, "p0", true).unwrap();
        mark_page_completed_inner(&st, "p0", false).unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Detected);
    }

    #[test]
    fn persist_partial_translating_restores_preclaim() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        seed_tr(&st, PageStatus::Detected, tr_full(&["isi", ""]));
        claim_page(&st, "p0").unwrap();
        let t = st.store.lock().unwrap().find_page("p0").unwrap().translation.clone().unwrap();
        persist_result(&st, "p0", &t).unwrap();
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Detected);
    }

    #[test]
    fn persist_sticky_manual_survives_retranslate() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        seed_tr(&st, PageStatus::Detected, tr_full(&["isi", ""]));
        mark_page_completed_inner(&st, "p0", true).unwrap();
        claim_page(&st, "p0").unwrap();
        let fresh = PageTranslation {
            page_id: "p0".into(), target_lang: "id".into(), style: "s".into(),
            model: "m2".into(), manual_done: false, bubbles: tr_full(&["baru", ""]),
        };
        persist_result(&st, "p0", &fresh).unwrap();
        let pg = st.store.lock().unwrap().find_page("p0").unwrap();
        assert!(pg.translation.as_ref().unwrap().manual_done);
        assert_eq!(pg.status, PageStatus::Translated);
    }
}
