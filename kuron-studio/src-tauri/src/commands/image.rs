use std::time::{Instant, UNIX_EPOCH};

use base64::Engine;
use image::ExtendedColorType;
use tauri::State;

use crate::logging;
use crate::preview_cache::{
    THUMB_TIER_MAX_SIDE, ensure_preview_table, preview_get, preview_key, preview_set,
};
use crate::AppState;

/// 512px thumbnail as data URL for the project grid.
/// Reads full file via Rust command — no webview fs permission needed (M0).
///
/// Mutu per ukuran (sharp-preview D1): editor (>768) = Lanczos3 + q90
/// agar garis/furigana/screentone tajam; grid (<=768) = Triangle + q80
/// tetap murah. Batas 768 dinamai agar niat terbaca, bukan angka ajaib.
pub(crate) const EDITOR_PREVIEW_MIN_SIDE: u32 = 768;
const EDITOR_JPEG_QUALITY: u8 = 90;
const GRID_JPEG_QUALITY: u8 = 80;

/// Tier mutu dari ukuran yang diminta — fungsi murni agar bisa di-test
/// tanpa I/O: editor (>768) tajam, grid murah.
fn preview_tier(max_side: u32) -> (image::imageops::FilterType, u8) {
    if max_side > EDITOR_PREVIEW_MIN_SIDE {
        (image::imageops::FilterType::Lanczos3, EDITOR_JPEG_QUALITY)
    } else {
        (image::imageops::FilterType::Triangle, GRID_JPEG_QUALITY)
    }
}

/// Pipeline murni file → JPEG bytes (tanpa cache, tanpa base64): dipakai
/// command + benchmark ukur (grid-preview-perf 1.1).
pub(crate) fn render_preview_jpeg(
    path: &str,
    max_side: u32,
    filter: image::imageops::FilterType,
    quality: u8,
) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read failed: {e}"))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("decode failed: {e}"))?;
    let (w, h) = (img.width(), img.height());
    let thumb = if w.max(h).max(1) > max_side {
        let scale = max_side as f32 / w.max(h) as f32;
        img.resize(
            ((w as f32 * scale) as u32).max(1),
            ((h as f32 * scale) as u32).max(1),
            filter,
        )
    } else {
        img
    };
    let rgb = thumb.to_rgb8();
    let (tw, th) = (rgb.width(), rgb.height());
    let mut jpeg = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality);
    enc.encode(rgb.as_raw(), tw, th, ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(jpeg)
}

/// Versi file untuk kunci cache: (mtime detik, ukuran byte).
fn file_version(path: &str) -> Result<(i64, u64), String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("read failed: {e}"))?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    Ok((mtime, meta.len()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_image_preview(
    path: String,
    max_side: Option<u32>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let max_side = max_side.unwrap_or(512).clamp(64, 2048);
    // Editor (>768) = tajam; grid = murah. Wire key {path, max_side} tak berubah.
    let (filter, quality) = preview_tier(max_side);
    let persist_tier = max_side <= THUMB_TIER_MAX_SIDE;
    let (mtime, size) = file_version(&path)?;
    let key = preview_key(&path, mtime, size, max_side);
    // 1. LRU sesi (tanpa I/O).
    if let Ok(mut pc) = state.previews.lock() {
        if let Some(jpeg) = pc.get(persist_tier, &key) {
            logging::debug(
                "preview_cache_hit",
                serde_json::json!({ "tier": if persist_tier { "thumb" } else { "full" }, "bytes": jpeg.len() }),
            );
            return Ok(format!(
                "data:image/jpeg;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&jpeg)
            ));
        }
    }
    // 2. SQLite persisten (hanya tier grid; full 1600 sesi-saja).
    if persist_tier {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        ensure_preview_table(&db)?;
        if let Some(jpeg) = preview_get(&db, &key, max_side)? {
            drop(db);
            if let Ok(mut pc) = state.previews.lock() {
                pc.put(persist_tier, key.clone(), jpeg.clone());
            }
            logging::debug(
                "preview_cache_hit",
                serde_json::json!({ "tier": "thumb-persist", "bytes": jpeg.len() }),
            );
            return Ok(format!(
                "data:image/jpeg;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&jpeg)
            ));
        }
    }
    // 3. Decode penuh (diukur untuk baseline 1.1).
    let t0 = Instant::now();
    let path_for_store = path.clone();
    let jpeg = tauri::async_runtime::spawn_blocking(move || {
        render_preview_jpeg(&path, max_side, filter, quality)
    })
    .await
    .map_err(|e| format!("preview task: {e}"))??;
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    logging::debug(
        "preview_cache_miss",
        serde_json::json!({ "tier": if persist_tier { "thumb" } else { "full" }, "ms": (ms * 10.0).round() / 10.0, "bytes": jpeg.len() }),
    );
    if let Ok(mut pc) = state.previews.lock() {
        pc.put(persist_tier, key.clone(), jpeg.clone());
    }
    if persist_tier {
        if let Ok(db) = state.db.lock() {
            // Cache turunan boleh gagal diam-diam — decode sukses tetap dikembalikan.
            let _ = preview_set(&db, &key, max_side, &path_for_store, mtime, size, &jpeg);
        }
    }
    Ok(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&jpeg)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_errors_without_state() {
        // Tanpa State Tauri: jalur gagal = versi file + render langsung.
        assert!(file_version("definitely-not-here.png").is_err());
        let (f, q) = preview_tier(512);
        assert!(render_preview_jpeg("definitely-not-here.png", 512, f, q).is_err());
    }

    #[test]
    fn preview_tier_splits_editor_and_grid() {
        // Perilaku tier (bukan nilai konstanta — clippy assertions_on_constants):
        // grid-512 murah, editor-1600 tajam, batas di antaranya.
        let (f_grid, q_grid) = preview_tier(512);
        let (f_ed, q_ed) = preview_tier(1600);
        assert!(matches!(f_grid, image::imageops::FilterType::Triangle));
        assert!(matches!(f_ed, image::imageops::FilterType::Lanczos3));
        assert!(q_ed > q_grid);
        // Di batas: sisi grid (<=) vs sisi editor (>).
        assert!(matches!(
            preview_tier(EDITOR_PREVIEW_MIN_SIDE).0,
            image::imageops::FilterType::Triangle
        ));
        assert!(matches!(
            preview_tier(EDITOR_PREVIEW_MIN_SIDE + 1).0,
            image::imageops::FilterType::Lanczos3
        ));
    }
}
