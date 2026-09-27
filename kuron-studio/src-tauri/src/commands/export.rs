use image::{Rgb, RgbImage};
use tauri::State;

use crate::translation::{BubbleTranslation, PageTranslation};
use crate::AppState;

// Font bitmap 5x7 latin (A-Z 0-9 + tanda baca umum). Non-latin dilewati diam-diam.
fn glyph(ch: char) -> Option<[u8; 7]> {
    let c = ch.to_ascii_uppercase();
    Some(match c {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x19, 0x15, 0x13, 0x13, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x0E, 0x10, 0x10, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x06, 0x04, 0x08],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        ':' => [0x00, 0x06, 0x06, 0x00, 0x06, 0x06, 0x00],
        '\'' => [0x04, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00],
        _ => return None,
    })
}

fn text_width(s: &str, scale: u32) -> u32 {
    (s.chars().filter(|c| glyph(*c).is_some()).count() as u32) * 6 * scale
}

fn draw_text(img: &mut RgbImage, s: &str, x0: i32, y0: i32, scale: u32) {
    let mut x = x0;
    for ch in s.chars() {
        let Some(rows) = glyph(ch) else {
            continue;
        };
        for (ry, row) in rows.iter().enumerate() {
            for bit in 0..5 {
                if row & (0x10 >> bit) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let (px, py) = (
                            x + bit * scale as i32 + dx as i32,
                            y0 + ry as i32 * scale as i32 + dy as i32,
                        );
                        if px >= 0 && py >= 0
                            && (px as u32) < img.width()
                            && (py as u32) < img.height()
                        {
                            img.put_pixel(px as u32, py as u32, Rgb([0, 0, 0]));
                        }
                    }
                }
            }
        }
        x += 6 * scale as i32;
    }
}

/// Bungkus kata agar muat dalam lebar patch (satuan px font).
fn wrap_words(text: &str, max_w: u32, scale: u32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in text.split_whitespace() {
        let cand = if cur.is_empty() {
            w.to_string()
        } else {
            format!("{cur} {w}")
        };
        if text_width(&cand, scale) <= max_w || cur.is_empty() {
            cur = cand;
        } else {
            lines.push(cur);
            cur = w.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn point_in_poly(px: i32, py: i32, poly: &[[i32; 2]]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n.saturating_sub(1);
    for i in 0..n {
        let (xi, yi) = (poly[i][0], poly[i][1]);
        let (xj, yj) = (poly[j][0], poly[j][1]);
        if ((yi > py) != (yj > py))
            && (px as f64) < (xj - xi) as f64 * (py - yi) as f64 / (yj - yi) as f64 + xi as f64
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Patch putih shape-aware: isi polygon bila shape ada, else rect bubble.
pub fn paint_patch(img: &mut RgbImage, b: &BubbleTranslation, shape: Option<&Vec<[i32; 2]>>) {
    let (x0, y0) = (b.x.max(0), b.y.max(0));
    let (x1, y1) = ((b.x + b.w).min(img.width() as i32), (b.y + b.h).min(img.height() as i32));
    match shape {
        Some(poly) if poly.len() >= 3 => {
            for y in y0..y1 {
                for x in x0..x1 {
                    if point_in_poly(x, y, poly) {
                        img.put_pixel(x as u32, y as u32, Rgb([255, 255, 255]));
                    }
                }
            }
        }
        _ => {
            for y in y0..y1 {
                for x in x0..x1 {
                    img.put_pixel(x as u32, y as u32, Rgb([255, 255, 255]));
                }
            }
        }
    }
    // Teks: skala dari lebar patch, bungkus kata, rata tengah.
    let inner_w = (x1 - x0 - 8).max(8) as u32;
    let mut scale = ((x1 - x0).max(16) as u32 / 90).clamp(1, 4);
    let mut lines = wrap_words(&b.translated, inner_w, scale);
    while lines.len() * 9 * scale as usize > (y1 - y0 - 6).max(9) as usize && scale > 1 {
        scale -= 1;
        lines = wrap_words(&b.translated, inner_w, scale);
    }
    let mut y = y0 + 4;
    for line in lines {
        let w = text_width(&line, scale);
        let x = x0 + ((x1 - x0) - w as i32).max(0) / 2;
        draw_text(img, &line, x, y, scale);
        y += 9 * scale as i32;
    }
}

/// Halaman + terjemahannya → PNG bytes (patch + teks per bubble).
/// `shapes[i]` = polygon bubble index i (None = rect), sejajar `t.bubbles`.
pub fn render_page_png(
    page_bytes: &[u8],
    t: &PageTranslation,
    shapes: &[Option<Vec<[i32; 2]>>],
) -> Result<Vec<u8>, String> {
    let mut img = image::load_from_memory(page_bytes)
        .map_err(|e| format!("decode: {e}"))?
        .to_rgb8();
    for b in &t.bubbles {
        let shape = shapes.get(b.index).and_then(|s| s.as_ref());
        paint_patch(&mut img, b, shape);
    }
    let mut out = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut out);
    use image::ImageEncoder;
    enc.write_image(img.as_raw(), img.width(), img.height(), image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// Halaman project dalam filename order → (nama_file, PNG bytes).
pub fn collect_pages(
    state: &State<'_, AppState>,
    project_id: &str,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let proj = store
        .projects
        .get(project_id)
        .ok_or_else(|| format!("project not found: {project_id}"))?
        .clone();
    drop(store);
    let mut pages = proj.pages;
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    for (i, pg) in pages.iter().enumerate() {
        let Some(t) = pg.translation.clone() else {
            continue;
        };
        let page_bytes =
            std::fs::read(&pg.path).map_err(|e| format!("read {}: {e}", pg.path))?;
        let shapes: Vec<Option<Vec<[i32; 2]>>> = {
            let max_idx = t.bubbles.iter().map(|b| b.index).max().unwrap_or(0);
            let mut v: Vec<Option<Vec<[i32; 2]>>> = vec![None; max_idx + 1];
            for (bi, bb) in pg.bubbles.iter().enumerate() {
                if bi < v.len() {
                    v[bi] = bb.shape.clone();
                }
            }
            v
        };
        let png = render_page_png(&page_bytes, &t, &shapes)?;
        out.push((format!("{:03}.png", i + 1), png));
    }
    if out.is_empty() {
        return Err("belum ada halaman terjemahan untuk export".to_string());
    }
    Ok(out)
}

pub fn export_json_bytes(
    state: &State<'_, AppState>,
    project_id: &str,
) -> Result<Vec<u8>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let proj = store
        .projects
        .get(project_id)
        .ok_or_else(|| format!("project not found: {project_id}"))?
        .clone();
    let mut pages = proj.pages;
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    let ts: Vec<&PageTranslation> = pages.iter().filter_map(|p| p.translation.as_ref()).collect();
    serde_json::to_vec_pretty(&ts).map_err(|e| e.to_string())
}

pub fn export_cbz_bytes(
    state: &State<'_, AppState>,
    project_id: &str,
) -> Result<Vec<u8>, String> {
    let pages = collect_pages(state, project_id)?;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, png) in &pages {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        zip.write_all(png).map_err(|e| e.to_string())?;
    }
    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

#[tauri::command]
pub fn export_project(
    state: State<'_, AppState>,
    project_id: String,
    format: String,
    path: String,
) -> Result<String, String> {
    match format.as_str() {
        "json" => {
            let bytes = export_json_bytes(&state, &project_id)?;
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
            Ok(path)
        }
        "png" => {
            let pages = collect_pages(&state, &project_id)?;
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            for (name, png) in &pages {
                let f = std::path::Path::new(&path).join(name);
                std::fs::write(f, png).map_err(|e| e.to_string())?;
            }
            Ok(path)
        }
        "cbz" => {
            let bytes = export_cbz_bytes(&state, &project_id)?;
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
            Ok(path)
        }
        other => Err(format!("format unknown: {other} (json|png|cbz)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn solid_png(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_pixel(w, h, Rgb([200, 200, 200]));
        let mut buf = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        use image::ImageEncoder;
        enc.write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
            .unwrap();
        buf
    }

    fn bt(i: usize, x: i32, y: i32, w: i32, h: i32, t: &str) -> BubbleTranslation {
        BubbleTranslation {
            index: i, x, y, w, h,
            original: "o".into(), reading: "".into(), translated: t.into(),
            needs_white_patch: true, is_user_edited: false,
        }
    }

    #[test]
    fn render_paints_white_and_text() {
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), bubbles: vec![bt(0, 50, 50, 200, 100, "HELLO")],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        // Patch putih menutupi area (tengah patch bukan abu-abu lagi).
        assert_ne!(img.get_pixel(150, 80).0, [200, 200, 200]);
        let dark = img.pixels().filter(|p| p.0 == [0, 0, 0]).count();
        assert!(dark > 20, "teks bitmap harus ada, got {dark}");
    }

    #[test]
    fn render_polygon_shape() {
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), bubbles: vec![bt(0, 50, 50, 200, 100, "HI")],
        };
        // Segitiga: pojok kiri-atas rect tetap abu-abu, tengah putih.
        let tri = vec![[50, 50], [250, 50], [150, 150]];
        let out = render_page_png(&png, &t, &[Some(tri)]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        assert_eq!(img.get_pixel(55, 145).0, [200, 200, 200]);
        assert_eq!(img.get_pixel(150, 70).0, [255, 255, 255]);
    }

    #[test]
    fn wrap_respects_width() {
        let lines = wrap_words("HELLO WORLD FOO BAR", 60, 1);
        assert!(lines.len() > 1);
        for l in &lines {
            assert!(text_width(l, 1) <= 90);
        }
    }
}
