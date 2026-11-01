use serde::Serialize;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::bubble::{model_search_paths, post_process, sanitize_bubbles, BubbleBox};
use crate::error::AppError;
use crate::render::detector::DetectorState;
use crate::AppState;

/// Status ONNX untuk banner frontend — model belum ada = mode manual penuh.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectStatus {
    pub engine: String,
    pub model_found: bool,
    pub model_path: Option<String>,
}

#[derive(Debug)]
pub struct DetectState {
    resource_dir: Option<PathBuf>,
    data_dir: PathBuf,
    detector: Arc<DetectorState>,
}

impl Default for DetectState {
    fn default() -> Self {
        Self {
            resource_dir: None,
            data_dir: std::env::temp_dir(),
            detector: Arc::new(DetectorState::new()),
        }
    }
}

impl DetectState {
    pub fn new(resource_dir: Option<PathBuf>, data_dir: PathBuf) -> Self {
        Self {
            resource_dir,
            data_dir,
            detector: Arc::new(DetectorState::new()),
        }
    }

    fn model_path(&self) -> Option<PathBuf> {
        model_search_paths(self.resource_dir.clone(), &self.data_dir)
            .into_iter()
            .find(|p| p.exists())
    }

    pub fn status(&self) -> DetectStatus {
        let found = self.model_path();
        DetectStatus {
            // "ort" = session nyata; "manual" = tanpa model (canvas manual).
            engine: if found.is_some() { "ort".to_string() } else { "manual".to_string() },
            model_found: found.is_some(),
            model_path: found.map(|p| p.to_string_lossy().to_string()),
        }
    }
}

impl Clone for DetectState {
    fn clone(&self) -> Self {
        Self {
            resource_dir: self.resource_dir.clone(),
            data_dir: self.data_dir.clone(),
            detector: Arc::clone(&self.detector),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectProgress {
    pub page_id: String,
    pub done: usize,
    pub total: usize,
    pub ok: bool,
    pub message: String,
}

#[tauri::command(rename_all = "snake_case")]
pub fn detect_status(state: State<'_, AppState>) -> Result<DetectStatus, String> {
    Ok(state.detect.status())
}

/// Detect nyata (ort) bila model ada; tanpa model → Ok([]) seperti dulu.
#[tauri::command(rename_all = "snake_case")]
pub async fn detect_bubbles(
    state: State<'_, AppState>,
    page_id: String,
) -> Result<Vec<BubbleBox>, String> {
    detect_bubbles_on(state, 0, page_id).await
}

/// Inti detect satu halaman dengan pin slot pool (batch paralel).
/// Slot 0 = jalur single/editor (perilaku lama tak berubah).
async fn detect_bubbles_on(
    state: State<'_, AppState>,
    slot: usize,
    page_id: String,
) -> Result<Vec<BubbleBox>, String> {
    set_page_status(state.clone(), &page_id, "detecting")?;
    let status = state.detect.status();
    let Some(model_path) = status.model_path.map(PathBuf::from) else {
        set_page_status(state, &page_id, "idle")?;
        return Ok(vec![]);
    };
    let page_path = page_path_of(&state, &page_id)?;
    let detector = state.detect.detector.clone();
    let path_for_infer = page_path.clone();
    let boxes = tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path_for_infer).map_err(|e| AppError::io(format!("read: {e}")))?;
        detector.detect_bytes_on(slot, &model_path, &bytes)
    })
    .await
    .map_err(|e| AppError::io(format!("infer task: {e}")))??;
    // bubble-polygon: isi shape dari outline gambar bila kotak polos.
    // Gagal/tak-cocok = diam, fallback kotak (tak pernah error di sini).
    let mut cleaned = post_process(sanitize_bubbles(boxes));
    if cleaned.iter().any(|b| b.shape.is_none()) {
        if let Ok(bytes) = std::fs::read(&page_path) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgb = img.to_rgb8();
                for b in cleaned.iter_mut().filter(|b| b.shape.is_none()) {
                    if let Some(poly) = crate::render::outline::refine_outline(&rgb, b.x, b.y, b.w, b.h) {
                        b.shape = Some(poly);
                    }
                }
            }
        }
    }
    set_page_status(state.clone(), &page_id, "idle")?;
    persist_bubbles(&state, &page_id, cleaned.clone())?;
    Ok(cleaned)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn detect_bubbles_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    page_ids: Vec<String>,
) -> Result<Vec<Vec<BubbleBox>>, String> {
    // detect-ux-pack 2.1: satu gagal → catat + lanjut, bukan abort (`?`).
    // Paralel bounded: semaphore = slot pool session (tanpa kontensi Mutex).
    let slots = state.detect.detector.pool_len().max(1);
    let budget = std::time::Duration::from_secs(
        crate::commands::batch::DEFAULT_BATCH_PAGE_TIMEOUT_SECS,
    );
    let out = collect_batch(
        &page_ids,
        |n: usize, id: String| {
            // Hanya tangkap yang 'static (AppHandle); State dibuat fresh di dalam.
            let handle = app.clone();
            let slot = n % slots;
            async move {
                let state: State<'_, AppState> = handle.state();
                match detect_bubbles_on(state, slot, id.clone()).await {
                    Ok(b) => (b, true, String::new()),
                    Err(e) => {
                        let state2: State<'_, AppState> = handle.state();
                        let _ = set_page_status(state2, &id, "failed");
                        (Vec::new(), false, e)
                    }
                }
            }
        },
        |ev| {
            let _ = app.emit("detect_progress", ev);
        },
        budget,
        slots,
    )
    .await;
    Ok(out)
}

/// Pesan timeout detect (menit bulat, selaras pesan timeout batch).
fn detect_timeout_message(budget_secs: u64) -> String {
    format!("timeout {} mnt, dilewati", budget_secs / 60)
}

/// Satu slot hasil worker: indeks input + id + outcome (boxes, ok, pesan).
type DetectSlot = (usize, String, (Vec<BubbleBox>, bool, String));

/// Inti loop batch tanpa AppHandle/State: urutan = input, gagal lanjut.
/// Murni sisi orkestrasi (fetch di-inject) → unit-test tanpa Tauri runtime.
/// Paralel bounded: event di-emit LANGSUNG tiap worker selesai (done monoton =
/// hitungan selesai) supaya UI bergerak real-time; hasil return tetap
/// dirakit berurutan input via slot berindeks + sort.
async fn collect_batch<Fut>(
    page_ids: &[String],
    mut fetch: impl FnMut(usize, String) -> Fut,
    mut emit: impl FnMut(DetectProgress),
    budget: std::time::Duration,
    max_concurrent: usize,
) -> Vec<Vec<BubbleBox>>
where
    Fut: Future<Output = (Vec<BubbleBox>, bool, String)> + Send + 'static,
{
    let total = page_ids.len();
    let permits = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent.max(1)));
    let mut set = tokio::task::JoinSet::new();
    for (n, id) in page_ids.iter().enumerate() {
        let permits = std::sync::Arc::clone(&permits);
        let id = id.clone();
        // Future dibuat di task utama (fetch FnMut, tak perlu Send);
        // yang dikirim ke worker hanya future-nya (harus Send + 'static).
        let fut = fetch(n, id.clone());
        set.spawn(async move {
            let _permit = permits.acquire_owned().await;
            let out = match tokio::time::timeout(budget, fut).await {
                Ok(out) => out,
                Err(_) => (Vec::new(), false, detect_timeout_message(budget.as_secs())),
            };
            (n, id, out)
        });
    }
    let mut slots: Vec<DetectSlot> = Vec::with_capacity(total);
    // Emit LANGSUNG tiap worker selesai (done = hitungan selesai, monoton):
    // UI bergerak real-time walau batch ratusan halaman. Urutan hasil akhir
    // tetap input-order via sort di bawah (replay hanya untuk return value).
    let mut completed = 0usize;
    while let Some(res) = set.join_next().await {
        if let Ok((n, id, (boxes, ok, message))) = res {
            completed += 1;
            emit(DetectProgress {
                page_id: id.clone(),
                done: completed,
                total,
                ok,
                message: message.clone(),
            });
            slots.push((n, id, (boxes, ok, message)));
        }
    }
    slots.sort_by_key(|(n, _, _)| *n);
    let mut all = Vec::with_capacity(total);
    for (_, _, (boxes, _, _)) in slots {
        all.push(boxes);
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bubble::BubbleBox;

    fn box_at(x: i32) -> BubbleBox {
        BubbleBox {
            x,
            y: 0,
            w: 10,
            h: 10,
            confidence: 1.0,
            shape: None,
            kind: None,
            tail: None,
        }
    }

    /// detect-ux-pack 2.2: gagal di tengah → sisa tetap jalan + ringkasan benar.
    #[tokio::test]
    async fn gagal_tengah_lanjut_dan_urutan_utuh() {
        let ids = ["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let mut events = Vec::new();
        let out = collect_batch(
            &ids,
            |_n, id| async move {
                if id == "p2" {
                    (Vec::new(), false, "decode failed".to_string())
                } else {
                    (vec![box_at(1)], true, String::new())
                }
            },
            |ev| events.push(ev),
            std::time::Duration::from_secs(30),
            2,
        )
        .await;
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].len(), 1);
        assert!(out[1].is_empty());
        assert_eq!(out[2].len(), 1);
        assert_eq!(events.len(), 3);
        assert_eq!(events.iter().map(|e| e.done).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert!(events.iter().all(|e| e.total == 3));
        // Urutan event = urutan selesai (acak); hasil return tetap input-order.
        let mut ids: Vec<&str> = events.iter().map(|e| e.page_id.as_str()).collect();
        ids.sort_unstable();
        assert_eq!(ids, vec!["p1", "p2", "p3"]);
        let failed = events.iter().find(|e| !e.ok).expect("satu gagal");
        assert_eq!(failed.page_id, "p2");
        assert_eq!(failed.message, "decode failed");
    }

    /// Paralel: selesai acak → hasil + event tetap berurutan input.
    #[tokio::test]
    async fn selesai_acak_tetap_urut() {
        let ids = ["p0".to_string(), "p1".to_string(), "p2".to_string()];
        let mut events = Vec::new();
        let out = collect_batch(
            &ids,
            |_n, id| async move {
                // p0 paling lambat, p2 tercepat: tanpa reorder, urutan rusak.
                let ms = match id.as_str() {
                    "p0" => 60,
                    "p1" => 30,
                    _ => 0,
                };
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                (vec![box_at(1)], true, String::new())
            },
            |ev| events.push(ev),
            std::time::Duration::from_secs(30),
            3,
        )
        .await;
        assert_eq!(out.len(), 3);
        assert_eq!(events.iter().map(|e| e.done).collect::<Vec<_>>(), vec![1, 2, 3]);
        // Event mengikuti urutan SELESAI (p2 tercepat): hasil tetap input-order.
        assert_eq!(
            events.iter().map(|e| e.page_id.as_str()).collect::<Vec<_>>(),
            vec!["p2", "p1", "p0"]
        );
    }

    /// Halaman macet (melebihi budget) → failed berpesan timeout, antrean lanjut.
    #[tokio::test]
    async fn timeout_macet_dilewati_antrean_lanjut() {
        let ids = ["p0".to_string(), "p1".to_string()];
        let mut events = Vec::new();
        let out = collect_batch(
            &ids,
            |_n, id| async move {
                if id == "p0" {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
                (vec![box_at(1)], true, String::new())
            },
            |ev| events.push(ev),
            std::time::Duration::from_secs(1),
            2,
        )
        .await;
        assert_eq!(out.len(), 2);
        assert!(out[0].is_empty(), "p0 kedaluwarsa → kosong");
        assert_eq!(out[1].len(), 1, "p1 tetap jalan");
        // p1 (cepat) selesai dulu, p0 menyusul via timeout 1 dtk.
        assert_eq!(events.iter().map(|e| e.done).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(events[0].page_id, "p1");
        assert!(events[0].ok);
        assert_eq!(events[1].page_id, "p0");
        assert!(!events[1].ok);
        assert!(events[1].message.contains("timeout"), "pesan: {}", events[1].message);
    }
    use crate::store::ProjectStore;
    use crate::domain::{Page, PageStatus, Project};
    use crate::translation::PageTranslation;

    fn status_state(status: PageStatus, tr: Option<PageTranslation>) -> AppState {
        let dir = std::env::temp_dir().join(format!(
            "kuron-detect-status-{}-{}",
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
            status,
            bubbles: vec![],
            translation: tr,
        }];
        store.projects.insert("p".into(), proj);
        AppState {
            store: std::sync::Mutex::new(store),
            detect: DetectState::default(),
            http: reqwest::Client::new(),
            db: std::sync::Mutex::new(rusqlite::Connection::open_in_memory().unwrap()),
            previews: std::sync::Mutex::new(crate::preview_cache::PreviewCaches::new()),
            data_dir: dir,
        }
    }

    fn status_of(st: &AppState) -> PageStatus {
        st.store.lock().unwrap().find_page("p0").unwrap().status.clone()
    }

    fn some_tr() -> PageTranslation {
        PageTranslation {
            page_id: "p0".into(),
            target_lang: "id".into(),
            style: "s".into(),
            model: "m".into(),
            manual_done: false,
            bubbles: vec![],
        }
    }

    #[test]
    fn translated_survives_bubble_save() {
        let st = status_state(PageStatus::Translated, Some(some_tr()));
        persist_bubbles(&st, "p0", vec![box_at(1)]).unwrap();
        assert_eq!(status_of(&st), PageStatus::Translated);
    }

    #[test]
    fn untranslated_bubble_save_sets_detected() {
        let st = status_state(PageStatus::Idle, None);
        persist_bubbles(&st, "p0", vec![box_at(1)]).unwrap();
        assert_eq!(status_of(&st), PageStatus::Detected);
    }

    #[test]
    fn empty_bubble_save_sets_no_bubbles() {
        let st = status_state(PageStatus::Idle, None);
        persist_bubbles(&st, "p0", vec![]).unwrap();
        assert_eq!(status_of(&st), PageStatus::NoBubbles);
    }

}

/// Simpan bubble hasil edit canvas + update status page:
/// >=1 bubble = detected, 0 = noBubbles — kecuali halaman berterjemahan
/// (status translated/translating dipertahankan, data terjemahan utuh).
#[tauri::command(rename_all = "snake_case")]
pub fn save_bubbles(
    state: State<'_, AppState>,
    page_id: String,
    bubbles: Vec<BubbleBox>,
) -> Result<Vec<BubbleBox>, String> {
    let cleaned = post_process(sanitize_bubbles(bubbles));
    Ok(persist_bubbles(&state, &page_id, cleaned)?)
}

fn page_path_of(state: &State<'_, AppState>, page_id: &str) -> Result<String, AppError> {
    let store = crate::error::lock_or_err(&state.store)?;
    store.page_path(page_id).ok_or_else(|| AppError::page_not_found(page_id))
}

fn persist_bubbles(
    state: &AppState,
    page_id: &str,
    cleaned: Vec<BubbleBox>,
) -> Result<Vec<BubbleBox>, AppError> {
    use crate::domain::PageStatus as S;
    let mut store = crate::error::lock_or_err(&state.store)?;
    let page = store.find_page_mut(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
    page.bubbles = cleaned.clone();
    // Status jujur: halaman berterjemahan tak didemosikan oleh edit geometri;
    // in-flight Translating juga tak disentuh.
    match page.status {
        S::Translated if page.translation.is_some() => {}
        S::Translating => {}
        _ => {
            page.status = if cleaned.is_empty() { S::NoBubbles } else { S::Detected };
        }
    }
    store.save_public()?;
    Ok(cleaned)
}

fn set_page_status(state: State<'_, AppState>, page_id: &str, status: &str) -> Result<(), AppError> {
    use crate::domain::PageStatus as S;
    let mut store = crate::error::lock_or_err(&state.store)?;
    let page = store.find_page_mut(page_id).ok_or_else(|| AppError::page_not_found(page_id))?;
    page.status = match status {
        "detecting" => S::Detecting,
        "idle" => S::Idle,
        "translating" => S::Translating,
        "translated" => S::Translated,
        "failed" => S::Failed,
        other => return Err(AppError::parse(format!("unknown status: {other}"))),
    };
    store.save_public()?;
    Ok(())
}
