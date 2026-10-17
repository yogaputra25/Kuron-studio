use base64::Engine;
use image::ExtendedColorType;

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
#[tauri::command(rename_all = "snake_case")]
pub async fn get_image_preview(path: String, max_side: Option<u32>) -> Result<String, String> {
    let max_side = max_side.unwrap_or(512).clamp(64, 2048);
    // Editor (>768) = tajam; grid = murah. Wire key {path, max_side} tak berubah.
    let (filter, quality) = preview_tier(max_side);
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| format!("read failed: {e}"))?;
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
        Ok(format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&jpeg)
        ))
    })
    .await
    .map_err(|e| format!("preview task: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_file_errors() {
        let res = get_image_preview("definitely-not-here.png".to_string(), Some(512)).await;
        assert!(res.is_err());
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
