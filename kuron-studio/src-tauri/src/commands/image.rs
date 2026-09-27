use base64::Engine;
use image::ExtendedColorType;

/// 512px thumbnail as data URL for the project grid.
/// Reads full file via Rust command — no webview fs permission needed (M0).
#[tauri::command(rename_all = "snake_case")]
pub async fn get_image_preview(path: String, max_side: Option<u32>) -> Result<String, String> {
    let max_side = max_side.unwrap_or(512).clamp(64, 2048);
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| format!("read failed: {e}"))?;
        let img = image::load_from_memory(&bytes).map_err(|e| format!("decode failed: {e}"))?;
        let (w, h) = (img.width(), img.height());
        let thumb = if w.max(h).max(1) > max_side {
            let scale = max_side as f32 / w.max(h) as f32;
            img.resize(
                ((w as f32 * scale) as u32).max(1),
                ((h as f32 * scale) as u32).max(1),
                image::imageops::FilterType::Triangle,
            )
        } else {
            img
        };
        let rgb = thumb.to_rgb8();
        let (tw, th) = (rgb.width(), rgb.height());
        let mut jpeg = Vec::new();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 80);
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
}
