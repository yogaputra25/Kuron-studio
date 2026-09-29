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
    map_full_image, map_mosaic, merge_ai_baseline, order_indices, preserve_user_edits,
    BubbleTranslation, PageTranslation,
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
    state: &AppState,
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
/// Stash status pra-klaim ke `AppState::prev_status` (path cancel restore ini,
/// bukan `fail_page`). `&AppState` (bukan `&State`) agar unit-test tanpa
/// runtime Tauri. Flag cancel basi SENGAJA tak dibuang di sini.
pub fn claim_page(state: &AppState, page_id: &str) -> Result<(), String> {
    let prev = {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        let page = store
            .projects
            .values_mut()
            .flat_map(|p| p.pages.iter_mut())
            .find(|pg| pg.id == page_id)
            .ok_or_else(|| format!("page not found: {page_id}"))?;
        match page.status {
            crate::commands::project::PageStatus::Translating => {
                return Err(format!("page busy (already translating): {page_id}"));
            }
            _ => {
                let prev = page.status.clone();
                page.status = crate::commands::project::PageStatus::Translating;
                store.save_public()?;
                prev
            }
        }
    };
    state.stash_prev(page_id, prev);
    // NOTE: flag cancel SENGAJA tak dibuang di sini — Batal bisa datang
    // sebelum claim (frontend busy=true duluan); backoff yang konsumsi.
    // Stale flag sekali self-healing (satu abort) lalu dibersihkan.
    Ok(())
}

/// Error cancel ("dibatalkan oleh user") — satu-satunya string yang memicu
/// path restore, bukan `fail_page`.
pub const CANCELLED_MSG: &str = "dibatalkan oleh user";

/// Path cancel: kembalikan status pra-klaim + buang flag. Bukan `fail_page`.
/// Tanpa stash (tak pernah klaim) = no-op status, flag tetap dibuang.
pub fn restore_cancelled(state: &AppState, page_id: &str) -> Result<(), String> {
    let prev = state.take_prev(page_id);
    state.clear_cancel(page_id);
    if let Some(st) = prev {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        if let Some(page) = store
            .projects
            .values_mut()
            .flat_map(|p| p.pages.iter_mut())
            .find(|pg| pg.id == page_id)
        {
            page.status = st;
            store.save_public()?;
        }
    }
    Ok(())
}

/// Error cancel → restore; error lain → `fail_page` (+bersihkan stash/flag).
pub fn cancel_or_fail(state: &AppState, page_id: &str, e: String) -> String {
    if e == CANCELLED_MSG {
        let _ = restore_cancelled(state, page_id);
        e
    } else {
        state.clear_cancel(page_id);
        state.take_prev(page_id);
        fail_page(state, page_id, e)
    }
}

pub fn load_provider(
    state: &AppState,
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
) -> Result<String, String> {
    let mut last = String::from("rate_limited: retry later");
    for (n, wait) in [0u64, 2, 4, 8].iter().enumerate() {
        if state.is_cancelled(page_id) {
            state.clear_cancel(page_id);
            return Err(CANCELLED_MSG.to_string());
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
                    return Err(CANCELLED_MSG.to_string());
                }
            }
        }
        match provider::translate_image(client, rec, prompt, jpeg).await {
            Ok(t) => {
                if state.is_cancelled(page_id) {
                    state.clear_cancel(page_id);
                    return Err(CANCELLED_MSG.to_string());
                }
                return Ok(t);
            }
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
    state: &AppState,
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

pub fn cache_lookup(state: &AppState, key: &str) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::cache_get(&db, key)
}

/// Terapkan payload cache: preservasi edit user + persist translated.
pub fn apply_cached(
    state: &AppState,
    page_id: &str,
    payload: &str,
    prev: Option<&PageTranslation>,
) -> Result<PageTranslation, String> {
    let mut page_t: PageTranslation =
        serde_json::from_str(payload).map_err(|e| format!("cache corrupt: {e}"))?;
    page_t.bubbles = preserve_user_edits(page_t.bubbles, prev);
    page_t.page_id = page_id.to_string();
    let r = persist(state, page_id, &page_t, "translated");
    state.clear_cancel(page_id);
    state.take_prev(page_id);
    r.map(|_| page_t)
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
    state: &AppState,
    prep: &PagePrep,
    page_t: &PageTranslation,
) -> Result<(), String> {
    let payload = serde_json::to_string(page_t).map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::cache_set(&db, &prep.cache_key, &payload)?;
    let r = persist(state, &prep.page_id, page_t, "translated");
    state.clear_cancel(&prep.page_id);
    state.take_prev(&prep.page_id);
    r
}

/// POST (backoff) → map → retry POST sekali saat parse gagal → store.
pub async fn translate_prep(
    state: &AppState,
    prep: &PagePrep,
) -> Result<PageTranslation, String> {
    if let Some(payload) = cache_lookup(state, &prep.cache_key)? {
        return apply_cached(state, &prep.page_id, &payload, prep.prev.as_ref());
    }
    let text = backoff_translate(state, &prep.page_id, &prep.client, &prep.rec, &prep.prompt, &prep.jpeg)
        .await
        .map_err(|e| cancel_or_fail(state, &prep.page_id, e))?;
    let page_t = match map_text(prep, &text) {
        Ok(p) => p,
        Err(first) => {
            let text2 = backoff_translate(state, &prep.page_id, &prep.client, &prep.rec, &prep.prompt, &prep.jpeg)
                .await
                .map_err(|e| cancel_or_fail(state, &prep.page_id, e))?;
            map_text(prep, &text2).map_err(|_| cancel_or_fail(state, &prep.page_id, first))?
        }
    };
    store_result(state, prep, &page_t)?;
    Ok(page_t)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn translate_page(
    state: State<'_, AppState>,
    input: TranslatePageInput,
) -> Result<PageTranslation, String> {
    let prep = prepare_page(
        state.inner(),
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
    translate_prep(state.inner(), &prep).await
}

/// Batalkan translate berjalan untuk satu halaman (opsi B cancel): set flag di
/// `AppState::cancel`; worker berhenti di cek berikut tanpa persist tanpa fail,
/// status dikembalikan ke pra-klaim. Idempotent; halaman tak dikenal = Ok.
#[tauri::command(rename_all = "snake_case")]
pub fn cancel_translate(
    state: State<'_, AppState>,
    page_id: String,
) -> Result<(), String> {
    state.request_cancel(&page_id);
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
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
    page_t.bubbles = merge_ai_baseline(&page_t.bubbles, bubbles);
    page.translation = Some(page_t.clone());
    page.status = crate::commands::project::PageStatus::Translated;
    store.save_public()?;
    Ok(page_t)
}

#[tauri::command(rename_all = "snake_case")]
pub fn clear_cache(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    cache::clear_cache(&db)
}

fn persist(state: &AppState, page_id: &str, t: &PageTranslation, status: &str) -> Result<(), String> {
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
    state: &AppState,
    page_id: &str,
    t: &PageTranslation,
) -> Result<(), String> {
    let r = persist(state, page_id, t, "translated");
    state.clear_cancel(page_id);
    state.take_prev(page_id);
    r
}

fn persist_status_only(state: &AppState, page_id: &str, status: &str) -> Result<(), String> {
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

pub fn fail_page(state: &AppState, page_id: &str, e: String) -> String {
    let _ = persist_status_only(state, page_id, "failed");
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bubble::BubbleBox;
    use crate::commands::detect::DetectState;
    use crate::commands::project::{Page, PageStatus, Project, ProjectStore};

    fn test_state_with_page(id: &str, status: PageStatus) -> AppState {
        let dir = std::env::temp_dir().join(format!(
            "kuron-cancel-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut store = ProjectStore::load(dir).unwrap();
        store.projects.insert(
            "p".into(),
            Project {
                id: "p".into(),
                name: "ch".into(),
                pages: vec![Page {
                    id: id.into(),
                    path: "ch/001.png".into(),
                    width: 800,
                    height: 1200,
                    status,
                    bubbles: vec![],
                    translation: None,
                }],
            },
        );
        AppState {
            store: std::sync::Mutex::new(store),
            detect: DetectState::default(),
            http: reqwest::Client::new(),
            db: std::sync::Mutex::new(rusqlite::Connection::open_in_memory().unwrap()),
            cancel: std::sync::Mutex::new(std::collections::HashSet::new()),
            prev_status: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn page_status_of(state: &AppState, id: &str) -> PageStatus {
        state
            .store
            .lock()
            .unwrap()
            .projects
            .values()
            .flat_map(|p| p.pages.iter())
            .find(|pg| pg.id == id)
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
        let msg = cancel_or_fail(&st, "p0", CANCELLED_MSG.to_string());
        assert_eq!(msg, CANCELLED_MSG);
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Translated);
        assert!(!st.is_cancelled("p0"));
        assert!(st.take_prev("p0").is_none());
    }

    #[test]
    fn cancel_restores_preclaim_status_detected() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        claim_page(&st, "p0").unwrap();
        st.request_cancel("p0");
        cancel_or_fail(&st, "p0", CANCELLED_MSG.to_string());
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Detected);
    }

    #[test]
    fn cancel_without_claim_is_noop_status_but_clears_flag() {
        let st = test_state_with_page("p0", PageStatus::Idle);
        st.request_cancel("p0");
        let msg = cancel_or_fail(&st, "p0", CANCELLED_MSG.to_string());
        assert_eq!(msg, CANCELLED_MSG);
        assert_eq!(page_status_of(&st, "p0"), PageStatus::Idle);
        assert!(!st.is_cancelled("p0"));
    }

    #[test]
    fn noncancel_error_still_fails_and_cleans_stash() {
        let st = test_state_with_page("p0", PageStatus::Detected);
        claim_page(&st, "p0").unwrap();
        let e = cancel_or_fail(&st, "p0", "boom".to_string());
        assert_eq!(e, "boom");
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
        assert_eq!(r, Err(CANCELLED_MSG.to_string()));
        assert!(!st.is_cancelled("p0"));
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
}
