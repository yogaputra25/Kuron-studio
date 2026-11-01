//! Render halaman: patch putih shape-aware + teks (TTF sistem atau bitmap
//! 5x7), layout bungkus-kata, dan PNG akhir. Pindahan verbatim dari
//! `commands::export` — tanpa perubahan perilaku.

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::{Rgb, RgbImage};
use std::sync::OnceLock;

use crate::error::AppError;
use crate::translation::{BubbleTranslation, PageTranslation};
/// Repair R4: shape diambil dari `translation` dulu (sumber pasca-save sidebar),
/// fallback `page.bubbles` by posisi (file lama tanpa shape), fallback None = bbox.
/// `shapes[i]` = polygon bubble index i, sejajar `t.bubbles`.
pub fn shapes_for_export(
    t: &PageTranslation,
    page_bubbles: &[crate::bubble::BubbleBox],
) -> Vec<Option<Vec<[i32; 2]>>> {
    let max_idx = t.bubbles.iter().map(|b| b.index).max().unwrap_or(0);
    let mut v: Vec<Option<Vec<[i32; 2]>>> = vec![None; max_idx + 1];
    for b in &t.bubbles {
        if b.index >= v.len() {
            continue;
        }
        if let Some(s) = b.shape.clone() {
            v[b.index] = Some(s);
            continue;
        }
        if let Some(bb) = page_bubbles.get(b.index) {
            v[b.index] = bb.shape.clone();
        }
    }
    v
}

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

fn bitmap_width(s: &str, scale: u32) -> u32 {
    (s.chars().filter(|c| glyph(*c).is_some()).count() as u32) * 6 * scale
}

fn bitmap_draw(img: &mut RgbImage, s: &str, x0: i32, y0: i32, scale: u32) {
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

// M5-4: system TTF via ab_glyph (no shaping/harfbuzz). Bitmap stays as fallback.
static FONTS: OnceLock<Vec<FontVec>> = OnceLock::new();

/// Font yang dicoba, berurutan — font pertama yang punya glyph itulah yang
/// dipakai. Urutan ini menentukan metrik teks, jadi overlay kanvas WAJIB
/// memakai urutan yang sama: lihat `OVERLAY_FONT_STACK` di
/// `src/lib/editor/text/textMeasure.ts` (daftar dari `layout.json` yang sama;
/// lintas berkas — jangan ubah satu sisi saja.
fn candidate_font_paths() -> Vec<String> {
    let l = layout();
    if cfg!(target_os = "windows") {
        return l.windows_fonts.clone();
    } else if cfg!(target_os = "macos") {
        return l.macos_fonts.clone();
    }
    l.linux_fonts.clone()
}

fn load_system_fonts() -> &'static Vec<FontVec> {
    FONTS.get_or_init(|| {
        let paths = candidate_font_paths();
        let mut fonts = Vec::new();
        for p in &paths {
            if fonts.len() >= 3 {
                break;
            }
            let Ok(bytes) = std::fs::read(p) else {
                continue;
            };
            if p.ends_with(".ttc") {
                for idx in 0..4u32 {
                    if fonts.len() >= 3 {
                        break;
                    }
                    if let Ok(f) = FontVec::try_from_vec_and_index(bytes.clone(), idx) {
                        fonts.push(f);
                    }
                }
            } else if let Ok(f) = FontVec::try_from_vec(bytes) {
                fonts.push(f);
            }
        }
        if fonts.is_empty() {
            eprintln!("kuron-studio: no system TTF found (tried {} paths), bitmap fallback", paths.len());
        }
        fonts
    })
}

#[cfg(test)]
pub(crate) fn font_for(ch: char) -> Option<&'static FontVec> {
    load_system_fonts().iter().find(|f| f.glyph_id(ch).0 != 0)
}

/// Bitmap 5x7 hanya punya ukuran bulat; ambil pendekatan terdekat dari `px`.
/// (Jalur bitmap = fallback saat tak ada TTF yang punya glyph-nya.)
fn bitmap_scale_for(px: f32) -> u32 {
    (px / 7.0).round().max(1.0) as u32
}

/// Lebar teks pada ukuran `px` (px halaman, pecahan — bukan kelipatan 7).
fn ttf_width(s: &str, px: f32) -> Option<u32> {
    let fonts = load_system_fonts();
    if fonts.is_empty() {
        return None;
    }
    let mut w = 0u32;
    let bitmap_scale = bitmap_scale_for(px);
    for ch in s.chars() {
        let Some(f) = fonts.iter().find(|f| f.glyph_id(ch).0 != 0) else {
            w += glyph(ch).map(|_| 6 * bitmap_scale).unwrap_or(0);
            continue;
        };
        let gid = f.glyph_id(ch);
        w += f.as_scaled(PxScale::from(px)).h_advance(gid).ceil() as u32;
    }
    Some(w)
}

fn ttf_draw(img: &mut RgbImage, s: &str, x0: i32, y0: i32, px: f32) -> bool {
    let fonts = load_system_fonts();
    if fonts.is_empty() {
        return false;
    }
    let bitmap_scale = bitmap_scale_for(px);
    let mut x = x0 as f32;
    let mut drew_any = false;
    for ch in s.chars() {
        let Some(f) = fonts.iter().find(|f| f.glyph_id(ch).0 != 0) else {
            if glyph(ch).is_some() {
                let single = ch.to_string();
                bitmap_draw(img, &single, x.round() as i32, y0, bitmap_scale);
                x += 6.0 * bitmap_scale as f32;
                drew_any = true;
            }
            continue;
        };
        let scaled = f.as_scaled(PxScale::from(px));
        let gid = f.glyph_id(ch);
        let advance = scaled.h_advance(gid).ceil();
        let mut glyph = scaled.scaled_glyph(ch);
        glyph.position = ab_glyph::point(x, y0 as f32 + scaled.ascent());
        if let Some(out) = scaled.outline_glyph(glyph) {
            // draw() memberi (dx,dy) relatif bounds glyph — offset dengan
            // px_bounds().min yang sudah absolut-halaman (position dilipat
            // via offset raster; terverifikasi source ab_glyph 0.2).
            let bb = out.px_bounds();
            let (ox, oy) = (bb.min.x as i32, bb.min.y as i32);
            out.draw(|dx, dy, c| {
                if c > 0.5 {
                    let (pxi, pyi) = (ox + dx as i32, oy + dy as i32);
                    if pxi >= 0 && pyi >= 0
                        && (pxi as u32) < img.width()
                        && (pyi as u32) < img.height()
                    {
                        img.put_pixel(pxi as u32, pyi as u32, Rgb([0, 0, 0]));
                    }
                }
            });
            drew_any = true;
        } else {
            // Space (no outline) tetap maju kursor.
            drew_any = true;
        }
        x += advance;
    }
    drew_any
}

fn text_width(s: &str, px: f32) -> u32 {
    ttf_width(s, px).unwrap_or_else(|| bitmap_width(s, bitmap_scale_for(px)))
}

fn draw_text(img: &mut RgbImage, s: &str, x0: i32, y0: i32, px: f32) {
    if ttf_draw(img, s, x0, y0, px) {
        return;
    }
    bitmap_draw(img, s, x0, y0, bitmap_scale_for(px))
}

/// Salinan `src/lib/contracts/layout.json` — SATU sumber kebenaran untuk
/// konstanta layout. Kanvas (`textLayout.ts`/`textMeasure.ts`) impor file yang
/// sama; jangan ubah angka satu sisi saja.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayoutContracts {
    line_height: f32,
    padding: i32,
    fill_factor: f32,
    min_font: f32,
    max_font: f32,
    font_step: f32,
    body_min_frac: f32,
    body_scan_step: f32,
    windows_fonts: Vec<String>,
    macos_fonts: Vec<String>,
    linux_fonts: Vec<String>,
}

static LAYOUT: OnceLock<LayoutContracts> = OnceLock::new();

fn layout() -> &'static LayoutContracts {
    LAYOUT.get_or_init(|| {
        serde_json::from_str(include_str!("../../../src/lib/contracts/layout.json"))
            .expect("contracts/layout.json harus valid")
    })
}

/// Port `computeTextLayout` (`src/lib/textLayout.ts`) untuk jalur fallback:
/// ukuran huruf TERBESAR yang blok teksnya masih muat `fill_factor` × kotak.
/// Auto-fit membungkus + menyusut terhadap area virtual itu, jadi hasil akhir
/// menyisakan margin di semua sisi (tanpa ini teks mepet outline).
///
/// Aturannya harus sama dengan kanvas; yang berbeda hanya sumber metrik
/// (TTF sistem di sini, Konva di kanvas). Kalau satu sisi mengubah rumusnya,
/// ubah keduanya — atau test kontrak konstanta akan gagal lebih dulu.
fn layout_from_text(text: &str, w: i32, h: i32, shape: Option<&[[i32; 2]]>) -> (f32, Vec<String>) {
    let fill = layout().fill_factor;
    let inner_w = (((w - layout().padding * 2).max(1) as f32) * fill).max(1.0) as u32;
    let inner_h = (((h - layout().padding * 2).max(1) as f32) * fill).max(1.0);
    let valid_shape = shape.filter(|s| s.len() >= 3);
    if valid_shape.is_none() {
        let mut font = layout().max_font;
        let mut lines = wrap_words(text, inner_w, font);
        while font > layout().min_font && lines.len() as f32 * font * layout().line_height > inner_h {
            font = (font - layout().font_step).max(layout().min_font);
            lines = wrap_words(text, inner_w, font);
        }
        return (font, lines);
    }
    // Jalur polygon: cermin `computeTextLayout` TS (D1). Shape relatif bubble.
    let s = valid_shape.expect("difilter di atas");
    let pad = layout().padding as f32;
    // Badan saja (ekor keluar dari area teks); fallback = kotak dalam.
    let (lo, hi) = body_span(s, w, h, layout().padding).unwrap_or((pad, h as f32 - pad));
    let span_h = (hi - lo).max(1.0);
    let height_limit = inner_h.min(span_h);
    let cx = w as f32 / 2.0;
    let band_w = |y: f32| -> u32 {
        match chord_at(s, y, cx) {
            Some((x0, x1)) => ((x1 - x0 - pad * 2.0).min(inner_w as f32).max(1.0)) as u32,
            None => 1,
        }
    };
    let fits = |lines: &[String], font: f32| -> bool {
        let line_px = font * layout().line_height;
        if lines.len() as f32 * line_px > height_limit + 1e-6 {
            return false;
        }
        let top = lo + (span_h - lines.len() as f32 * line_px) / 2.0;
        lines.iter().enumerate().all(|(j, ln)| {
            text_width(ln, font) as f32 <= band_w(top + (j as f32 + 0.5) * line_px) as f32 + 1e-6
        })
    };
    let try_font = |font: f32| -> Vec<String> {
        let line_px = font * layout().line_height;
        let n0 = wrap_words(text, inner_w, font).len().max(1);
        let top0 = lo + (span_h - n0 as f32 * line_px) / 2.0;
        let widths: Vec<u32> = (0..n0).map(|i| band_w(top0 + (i as f32 + 0.5) * line_px)).collect();
        wrap_words_var(text, &widths, font)
    };
    let mut font = layout().max_font;
    let mut lines = try_font(font);
    while font > layout().min_font && !fits(&lines, font) {
        font = (font - layout().font_step).max(layout().min_font);
        lines = try_font(font);
    }
    (font, lines)
}

/// Port `chordAt` (`src/lib/textLayout.ts`): tali-busur polygon (relatif
/// bubble) pada garis horizontal `y`. Pilih interval yang memuat `cx`,
/// kalau tak ada yang terlebar. `None` = garis tak memotong.
fn chord_at(shape: &[[i32; 2]], y: f32, cx: f32) -> Option<(f32, f32)> {
    let mut xs: Vec<f32> = Vec::new();
    let n = shape.len();
    for i in 0..n {
        let (x1, y1) = (shape[i][0] as f32, shape[i][1] as f32);
        let (x2, y2) = (shape[(i + 1) % n][0] as f32, shape[(i + 1) % n][1] as f32);
        if (y1 > y) != (y2 > y) {
            xs.push(x1 + (x2 - x1) * (y - y1) / (y2 - y1));
        }
    }
    if xs.len() < 2 {
        return None;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut best: Option<(f32, f32)> = None;
    let mut i = 0;
    while i + 1 < xs.len() {
        let (a, b) = (xs[i], xs[i + 1]);
        i += 2;
        if b > a {
            if cx >= a && cx <= b {
                return Some((a, b));
            }
            if best.map(|(x0, x1)| b - a > x1 - x0).unwrap_or(true) {
                best = Some((a, b));
            }
        }
    }
    best
}

/// Fraksi chord minimum agar pita dianggap badan (port TS `bodySpan`).
/// Nilai dari `layout()` di atas (sumber: `layout.json`).
fn body_min_frac() -> f32 {
    layout().body_min_frac
}
/// Port langkah sampling vertikal (px). Sama di dua sisi.
fn body_scan_step() -> f32 {
    layout().body_scan_step
}

/// Port `bodySpan`: lajur menyambung di sekitar chord terlebar dengan lebar
/// >= `body_min_frac()` × terlebar. Ekor sempit keluar dari area teks.
fn body_span(shape: &[[i32; 2]], w: i32, h: i32, padding: i32) -> Option<(f32, f32)> {
    let cx = w as f32 / 2.0;
    let mut widths: Vec<f32> = Vec::new();
    let mut y = 0.0;
    while y < h as f32 {
        match chord_at(shape, y + body_scan_step() / 2.0, cx) {
            Some((x0, x1)) => widths.push(x1 - x0),
            None => widths.push(0.0),
        }
        y += body_scan_step();
    }
    if widths.is_empty() {
        return None;
    }
    let mut max_w = 0.0f32;
    let mut arg = 0usize;
    for (i, &wd) in widths.iter().enumerate() {
        if wd > max_w {
            max_w = wd;
            arg = i;
        }
    }
    if max_w <= 0.0 {
        return None;
    }
    let th = max_w * body_min_frac();
    let mut lo = arg;
    while lo > 0 && widths[lo - 1] >= th {
        lo -= 1;
    }
    let mut hi = arg;
    while hi + 1 < widths.len() && widths[hi + 1] >= th {
        hi += 1;
    }
    Some((
        (padding as f32).max(lo as f32 * body_scan_step()),
        (h as f32 - padding as f32).min((hi + 1) as f32 * body_scan_step()),
    ))
}

/// Port `LineBand`: pita + chord tiap baris (relatif bubble).
struct LineBand {
    top: f32,
    x0: f32,
    x1: f32,
}

/// Port `lineBands`: blok tengah dalam rentang polygon (dijepit padding).
/// `None` = tanpa shape (pemanggil pakai jalur kotak lama).
fn line_bands(
    shape: Option<&[[i32; 2]]>,
    w: i32,
    h: i32,
    padding: i32,
    line_count: usize,
    font_size: f32,
    line_height: f32,
) -> Option<Vec<LineBand>> {
    let s = shape.filter(|s| s.len() >= 3)?;
    if line_count == 0 {
        return None;
    }
    let (lo, hi) = body_span(s, w, h, padding).unwrap_or((padding as f32, h as f32 - padding as f32));
    let span_h = (hi - lo).max(1.0);
    let line_px = font_size * line_height;
    let top = lo + (span_h - line_count as f32 * line_px) / 2.0;
    let cx = w as f32 / 2.0;
    let mut bands = Vec::with_capacity(line_count);
    for j in 0..line_count {
        let yc = top + (j as f32 + 0.5) * line_px;
        match chord_at(s, yc, cx) {
            Some((x0, x1)) => bands.push(LineBand { top: top + j as f32 * line_px, x0, x1 }),
            None => bands.push(LineBand { top: top + j as f32 * line_px, x0: cx, x1: cx }),
        }
    }
    Some(bands)
}

/// Bungkus kata agar muat `max_w`, dan PECAH kata yang lebih lebar dari kotak
/// (Konva juga begitu).
///
/// Tanpa pemecahan, kata panjang luber permanen: mengecilkan font tak akan
/// pernah menolong, karena pada ukuran minimum pun kata itu masih lebih lebar
/// dari kotak. Paragraf kosong dipertahankan sebagai baris kosong — sama
/// seperti `wrapLines` di sisi TypeScript.
fn wrap_words(text: &str, max_w: u32, px: f32) -> Vec<String> {
    wrap_words_core(text, &|_| max_w, px)
}

/// Bungkus dengan batas lebar per-baris (untuk polygon: tali-busur tiap pita).
fn wrap_words_var(text: &str, widths: &[u32], px: f32) -> Vec<String> {
    let last = widths.last().copied().unwrap_or(1);
    wrap_words_core(text, &|i| widths.get(i).copied().unwrap_or(last), px)
}

fn wrap_words_core(text: &str, width_for_line: &dyn Fn(usize) -> u32, px: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for para in text.split('\n') {
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut cur = String::new();
        for w in words {
            let cand = if cur.is_empty() {
                w.to_string()
            } else {
                format!("{cur} {w}")
            };
            if text_width(&cand, px) <= width_for_line(lines.len()) {
                cur = cand;
                continue;
            }
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
            }
            if text_width(w, px) <= width_for_line(lines.len()) {
                cur = w.to_string();
                continue;
            }
            let mut piece = String::new();
            for ch in w.chars() {
                if !piece.is_empty() && text_width(&format!("{piece}{ch}"), px) > width_for_line(lines.len()) {
                    lines.push(std::mem::take(&mut piece));
                }
                piece.push(ch);
            }
            cur = piece;
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
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
    // Shape absolut -> relatif bubble (frame yang dipakai layout/chord).
    let rel: Option<Vec<[i32; 2]>> = shape
        .filter(|p| p.len() >= 3)
        .map(|p| p.iter().map(|[px, py]| [px - x0, py - y0]).collect());
    let rel_ref = rel.as_deref();
    match &b.layout {
        Some(l) if !l.lines.is_empty() => draw_layout(img, l, x0, y0, x1, y1, rel_ref),
        _ => {
            let (font_size, lines) = layout_from_text(&b.translated, x1 - x0, y1 - y0, rel_ref);
            if !lines.is_empty() {
                draw_layout(
                    img,
                    &crate::translation::TextLayout {
                        font_size,
                        line_height: layout().line_height,
                        padding: layout().padding,
                        lines,
                    },
                    x0,
                    y0,
                    x1,
                    y1,
                    rel_ref,
                );
            }
        }
    }
}

/// Gambar layout final dari kanvas (px halaman) apa adanya.
///
/// Aturan penempatan WAJIB sama dengan `CanvasEditor`: padding di dalam kotak,
/// tiap baris rata tengah (pada chord bila shape ada), blok di tengah vertikal,
/// jarak baris `fontSize * lineHeight`. Kalau salah satu sisi mengubah rumusnya,
/// paritas hilang — ubah keduanya atau jangan ubah sama sekali.
fn draw_layout(
    img: &mut RgbImage,
    l: &crate::translation::TextLayout,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    shape: Option<&[[i32; 2]]>,
) {
    let (box_w, box_h) = (x1 - x0, y1 - y0);
    let line_px = l.font_size * l.line_height;
    let block_h = l.lines.len() as f32 * line_px;
    if let Some(bands) = line_bands(shape, box_w, box_h, l.padding, l.lines.len(), l.font_size, l.line_height) {
        // Tiap baris tengah pada chord-nya (cermin kanvas per-baris).
        for (line, band) in l.lines.iter().zip(bands.iter()) {
            let w = text_width(line, l.font_size);
            let center = (band.x0 + band.x1) / 2.0;
            let x = x0 + (center - w as f32 / 2.0).round() as i32;
            draw_text(img, line, x, (y0 as f32 + band.top).round() as i32, l.font_size);
        }
        return;
    }
    let mut y = y0 as f32 + (box_h as f32 - block_h) / 2.0;
    for line in &l.lines {
        let w = text_width(line, l.font_size);
        let x = x0 + (box_w - w as i32).max(0) / 2;
        draw_text(img, line, x, y.round() as i32, l.font_size);
        y += line_px;
    }
}

/// Luber menurut layout final dari kanvas (export-text-parity P7).
///
/// Dinilai dari `layout` — bukan dihitung ulang dengan algoritma lama, yang
/// justru sudah berbeda dari yang dipakai export sekarang. Bila shape ada,
/// tiap baris dinilai terhadap chord pitanya (text-fit-polygon).
pub fn layout_overflows(
    l: &crate::translation::TextLayout,
    w: i32,
    h: i32,
    shape: Option<&[[i32; 2]]>,
) -> bool {
    let inner_h = (h - l.padding * 2).max(1) as f32;
    if shape.is_some_and(|s| s.len() >= 3) {
        let bands = line_bands(shape, w, h, l.padding, l.lines.len(), l.font_size, l.line_height);
        if let Some(bands) = bands {
            let block_h = l.lines.len() as f32 * l.font_size * l.line_height;
            let s = shape.expect("bands ada berarti shape valid");
            let (lo, hi) = body_span(s, w, h, l.padding).unwrap_or((l.padding as f32, h as f32 - l.padding as f32));
            let span_h = (hi - lo).max(1.0);
            if block_h > span_h.min(inner_h) + 1e-6 {
                return true;
            }
            return l.lines.iter().zip(bands.iter()).any(|(line, band)| {
                text_width(line, l.font_size) as f32 > (band.x1 - band.x0 - l.padding as f32 * 2.0).max(1.0) + 1e-6
            });
        }
    }
    let inner_w = (w - l.padding * 2).max(1) as u32;
    if l.lines.len() as f32 * l.font_size * l.line_height > inner_h {
        return true;
    }
    l.lines.iter().any(|line| text_width(line, l.font_size) > inner_w)
}

/// Cek overflow memakai metrik renderer yang sama (TTF/bitmap dispatch).
/// Mirip layout paint_patch: bungkus kata + loop susut skala.
/// ponytail: tanpa shaping/ligatur/RTL — upgrade ke harfbuzz bila perlu.
pub fn text_overflows(text: &str, w: i32, h: i32, shape: Option<&[[i32; 2]]>) -> bool {
    if text.split_whitespace().next().is_none() {
        return false;
    }
    // Nilai dari port layout yang sama dengan export memakai — bukan rumus
    // terpisah. Dihitung ulang di sini karena QA berjalan tanpa `layout`
    // tersimpan.
    let (font_size, lines) = layout_from_text(text, w, h, shape);
    layout_overflows(
        &crate::translation::TextLayout {
            font_size,
            line_height: layout().line_height,
            padding: layout().padding,
            lines,
        },
        w,
        h,
        shape,
    )
}

pub fn render_page_png(
    page_bytes: &[u8],
    t: &PageTranslation,
    shapes: &[Option<Vec<[i32; 2]>>],
) -> Result<Vec<u8>, AppError> {
    let mut img = image::load_from_memory(page_bytes)
        .map_err(|e| AppError::parse(format!("decode: {e}")))?
        .to_rgb8();
    for b in &t.bubbles {
        let shape = shapes.get(b.index).and_then(|s| s.as_ref());
        paint_patch(&mut img, b, shape);
    }
    let mut out = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut out);
    use image::ImageEncoder;
    enc.write_image(img.as_raw(), img.width(), img.height(), image::ExtendedColorType::Rgb8)
        .map_err(|e| AppError::parse(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    pub(super) fn solid_png(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_pixel(w, h, Rgb([200, 200, 200]));
        let mut buf = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        use image::ImageEncoder;
        enc.write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
            .unwrap();
        buf
    }
    pub(super) fn bt(i: usize, x: i32, y: i32, w: i32, h: i32, t: &str) -> BubbleTranslation {
        BubbleTranslation {
            index: i, x, y, w, h, shape: None, layout: None, font_size: None,
            original: "o".into(), reading: "".into(), translated: t.into(),
            ai_original: "o".into(), ai_reading: "".into(), ai_translated: t.into(),
            needs_white_patch: true, is_user_edited: false,
        }
    }

    #[test]
    fn render_paints_white_and_text() {
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![bt(0, 50, 50, 200, 100, "HELLO")],
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
            model: "m".into(), manual_done: false, bubbles: vec![bt(0, 50, 50, 200, 100, "HI")],
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
        let lines = wrap_words("HELLO WORLD FOO BAR", 60, 7.0);
        assert!(lines.len() > 1);
        for l in &lines {
            assert!(text_width(l, 7.0) <= 90);
        }
    }

    /// Kontrak lintas bahasa: nilai runtime == `layout.json` apa adanya
    /// (termasuk float — `as f32` di sini menempuh jalur yang sama dengan
    /// parse struct, jadi ketidaksesuaian mapping/rename langsung gagal).
    #[test]
    fn layout_matches_json_contract() {
        let l = layout();
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../../../src/lib/contracts/layout.json"
        ))
        .unwrap();
        let f = |k: &str| v[k].as_f64().unwrap() as f32;
        assert_eq!(l.line_height, f("lineHeight"));
        assert_eq!(l.padding, v["padding"].as_i64().unwrap() as i32);
        assert_eq!(l.fill_factor, f("fillFactor"));
        assert_eq!(l.min_font, f("minFont"));
        assert_eq!(l.max_font, f("maxFont"));
        assert_eq!(l.font_step, f("fontStep"));
        assert_eq!(l.body_min_frac, f("bodyMinFrac"));
        assert_eq!(l.body_scan_step, f("bodyScanStep"));
        let strs = |k: &str| {
            v[k].as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(l.windows_fonts, strs("windowsFonts"));
        assert_eq!(l.macos_fonts, strs("macosFonts"));
        assert_eq!(l.linux_fonts, strs("linuxFonts"));
        assert!(!l.windows_fonts.is_empty() && !l.macos_fonts.is_empty() && !l.linux_fonts.is_empty());
    }

    #[test]
    fn render_ttf_latin_dark_pixels() {
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![bt(0, 50, 50, 200, 100, "HELLO WORLD")],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        let dark = img.pixels().filter(|p| p.0 == [0, 0, 0]).count();
        assert!(dark > 20, "latin harus render, got {dark}");
    }

    #[test]
    fn render_cjk_graceful() {
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![bt(0, 50, 50, 200, 100, "日本語テスト")],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        if font_for('日').is_some() {
            let dark = img.pixels().filter(|p| p.0 == [0, 0, 0]).count();
            assert!(dark > 20, "CJK font ada tapi tak render, got {dark}");
        } else {
            assert_eq!(img.get_pixel(150, 80).0, [255, 255, 255]);
        }
    }

    #[test]
    fn text_width_cjk_counts_when_font_present() {
        if font_for('日').is_some() {
            assert!(text_width("日本語", 7.0) > 0);
        } else {
            assert_eq!(text_width("日本語", 7.0), 0);
        }
    }

    #[test]
    fn text_overflows_mirrors_layout() {
        assert!(text_overflows("kata ".repeat(50).trim(), 60, 30, None));
        assert!(!text_overflows("HI", 200, 100, None));
        assert!(!text_overflows("", 60, 30, None));
        assert!(!text_overflows("   ", 60, 30, None));
    }

}

#[cfg(test)]
mod text_position_tests {
    use super::tests::{bt, solid_png};
    use super::*;
    use image::RgbImage;

    fn dark_in_rect(img: &RgbImage, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
        let mut n = 0;
        for y in y0..y1.min(img.height()) {
            for x in x0..x1.min(img.width()) {
                if img.get_pixel(x, y).0 == [0, 0, 0] {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn ttf_text_inside_bubble_corner_clean() {
        // TTF path hanya bila font sistem ada; bila tidak, bitmap path
        // (posisi sudah benar) yang diuji — keduanya wajib di-dalam-bubble.
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![bt(0, 50, 50, 200, 100, "HELLO")],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        let inside = dark_in_rect(&img, 50, 50, 250, 150);
        assert!(inside > 20, "teks harus di dalam bubble, got {inside}");
        let corner = dark_in_rect(&img, 0, 0, 30, 30);
        assert_eq!(corner, 0, "pojok halaman harus bersih, got {corner}");
    }

    #[test]
    fn ttf_multiline_inside_bubble() {
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false,
            bubbles: vec![bt(0, 50, 50, 220, 160, "HELLO WORLD FOO BAR BAZ")],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        let inside = dark_in_rect(&img, 50, 50, 270, 210);
        assert!(inside > 20, "multiline harus di dalam bubble, got {inside}");
        let corner = dark_in_rect(&img, 0, 0, 30, 30);
        assert_eq!(corner, 0, "pojok halaman harus bersih, got {corner}");
    }

    #[test]
    fn draws_layout_verbatim() {
        // Layout dari kanvas harus digambar apa adanya, bukan dihitung ulang.
        // Satu baris, font 14px: blok mendarat di TENGAH vertikal kotak —
        // jalur lama menaruhnya di tepi atas (y0 + 4).
        let png = solid_png(400, 400);
        let mut b = bt(0, 50, 50, 200, 100, "HI");
        b.layout = Some(crate::translation::TextLayout {
            font_size: 14.0,
            line_height: 1.25,
            padding: 4,
            lines: vec!["HI".into()],
        });
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![b],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        let dark = img.pixels().filter(|p| p.0 == [0, 0, 0]).count();
        assert!(dark > 0, "layout harus menggambar teks, got {dark}");
        // Barisnya di tengah, bukan di baris paling atas kotak (y0+4 = 54).
        let atas = dark_in_rect(&img, 50, 51, 250, 60);
        let tengah = dark_in_rect(&img, 50, 85, 250, 115);
        assert!(tengah > 0, "teks layout harus ada di tengah kotak");
        assert!(tengah >= atas, "blok harus di tengah, bukan menempel atas");
    }

    #[test]
    fn legacy_path_still_draws_without_layout() {
        // Proyek lama: `layout` absen → jalur hitung-sendiri tetap keluar.
        let png = solid_png(400, 400);
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![bt(0, 50, 50, 200, 100, "HI")],
        };
        let out = render_page_png(&png, &t, &[None]).unwrap();
        let img = image::load_from_memory(&out).unwrap().to_rgb8();
        let dark = img.pixels().filter(|p| p.0 == [0, 0, 0]).count();
        assert!(dark > 0, "jalur lama harus tetap menggambar, got {dark}");
    }

    #[test]
    fn layout_overflow_uses_layout_not_recompute() {
        let banyak = crate::translation::TextLayout {
            font_size: 20.0, line_height: 1.25, padding: 4,
            lines: vec!["x".into(); 10],
        };
        // Blok 10 * 20 * 1.25 = 250 px > 100 - 8 → luber.
        assert!(layout_overflows(&banyak, 200, 100, None));
        let sedikit = crate::translation::TextLayout {
            font_size: 10.0, line_height: 1.25, padding: 4,
            lines: vec!["x".into()],
        };
        assert!(!layout_overflows(&sedikit, 200, 100, None));
    }

    #[test]
    fn fallback_font_grows_with_box() {
        // Regresi jalur lama: ukuran datang dari `clamp(lebar/90,1,4)` → 7 px
        // untuk hampir semua bubble. Sekarang ukuran dicari sampai blok teks
        // muat TINGGI kotak, jadi kotak yang lebih tinggi mendapat font lebih
        // besar.
        let (kecil, _) = layout_from_text("HI", 100, 40, None);
        let (besar, _) = layout_from_text("HI", 400, 400, None);
        assert!(
            besar > kecil,
            "kotak lebih tinggi harus dapat font lebih besar: {kecil} vs {besar}"
        );
        assert!(kecil >= layout().min_font, "tak boleh di bawah minimum");
        assert!(besar <= layout().max_font, "tak boleh di atas maksimum");
    }

    #[test]
    fn fallback_breaks_long_words() {
        // Satu kata tanpa spasi yang lebih lebar dari kotak harus dipecah —
        // kalau dibiarkan, mengecilkan font tak akan pernah menolong.
        let lines = wrap_words("AAAAAAAAAAAAAAAAAAAA", 40, 20.0);
        assert!(lines.len() > 1, "kata panjang harus dipecah, got {lines:?}");
        for l in &lines {
            assert!(text_width(l, 20.0) <= 40, "baris luber: {l:?}");
        }
    }

    #[test]
    fn fallback_lines_fit_inner_width() {
        let (_, lines) = layout_from_text("kata ".repeat(30).trim(), 160, 200, None);
        let inner = (160 - layout().padding * 2) as u32;
        assert!(!lines.is_empty());
        for l in &lines {
            assert!(
                text_width(l, layout().min_font) <= inner,
                "baris harus muat lebar dalam pada ukuran minimum: {l:?}"
            );
        }
    }

    /// Elips kasar 200x200 (cermin vektor TS di `textLayout.test.ts`).
    fn ellipse_shape() -> Vec<[i32; 2]> {
        vec![
            [100, 0], [150, 14], [180, 50], [190, 100],
            [180, 150], [150, 186], [100, 200], [50, 186],
            [20, 150], [10, 100], [20, 50], [50, 14],
        ]
    }

    #[test]
    fn chord_narrow_at_tip_wide_at_middle() {
        let s = ellipse_shape();
        let top = chord_at(&s, 5.0, 100.0).expect("garis y=5 harus memotong");
        let mid = chord_at(&s, 100.0, 100.0).expect("garis y=100 harus memotong");
        assert!(mid.1 - mid.0 > top.1 - top.0, "tengah harus lebih lebar dari ujung");
        assert!(mid.1 - mid.0 > 150.0, "tengah elips harus hampir penuh");
        assert!(chord_at(&s, -10.0, 100.0).is_none());
        assert!(chord_at(&s, 300.0, 100.0).is_none());
    }

    #[test]
    fn rect_shape_matches_no_shape() {
        let rect = vec![[0, 0], [200, 0], [200, 100], [0, 100]];
        let (f1, l1) = layout_from_text("halo dunia yang panjang", 200, 100, None);
        let (f2, l2) = layout_from_text("halo dunia yang panjang", 200, 100, Some(&rect));
        assert!((f1 - f2).abs() < 1e-6, "rect harus identik dengan tanpa-shape");
        assert_eq!(l1, l2);
    }

    #[test]
    fn polygon_lines_fit_their_chords() {
        let s = ellipse_shape();
        let text = "kata ".repeat(30).trim().to_string();
        let (font, lines) = layout_from_text(&text, 200, 200, Some(&s));
        let bands = line_bands(Some(&s), 200, 200, layout().padding, lines.len(), font, layout().line_height)
            .expect("bands harus ada");
        assert_eq!(bands.len(), lines.len());
        for (ln, band) in lines.iter().zip(bands.iter()) {
            let avail = (band.x1 - band.x0 - layout().padding as f32 * 2.0).max(1.0);
            assert!(
                text_width(ln, font) as f32 <= avail + 1e-6,
                "baris keluar chord: {ln:?}"
            );
        }
    }

    #[test]
    fn overflow_flags_line_outside_chord() {
        let s = ellipse_shape();
        // Satu baris selebar kotak di pita sempit elips -> luber.
        let l = crate::translation::TextLayout {
            font_size: 20.0,
            line_height: layout().line_height,
            padding: 0,
            lines: vec!["abcdefghijklmnopqrstuvwxyz0123456789".into()],
        };
        assert!(layout_overflows(&l, 200, 200, Some(&s)));
        let ok = crate::translation::TextLayout {
            font_size: 10.0,
            line_height: layout().line_height,
            padding: 0,
            lines: vec!["hi".into()],
        };
        assert!(!layout_overflows(&ok, 200, 200, Some(&s)));
    }

    fn tailed_shape() -> Vec<[i32; 2]> {
        vec![[0, 0], [200, 0], [200, 150], [110, 150], [100, 300], [90, 150], [0, 150]]
    }

    #[test]
    fn tail_excluded_from_text_area() {
        let s = tailed_shape();
        let (lo, hi) = body_span(&s, 200, 300, layout().padding).expect("badan harus ketemu");
        assert!(hi <= 152.0, "ekor (mulai y=150) harus di luar span: hi={hi}");
        assert!(lo <= layout().padding as f32, "badan mulai dari atas");
        let text = "kata ".repeat(30).trim().to_string();
        let (font, lines) = layout_from_text(&text, 200, 300, Some(&s));
        assert!(font > layout().min_font, "font tidak terjepit selebar ekor");
        let bands = line_bands(Some(&s), 200, 300, layout().padding, lines.len(), font, layout().line_height)
            .expect("bands harus ada");
        for b in &bands {
            assert!(
                b.top + font * layout().line_height <= 152.0,
                "pita masuk ekor: top={}",
                b.top
            );
        }
        assert!(!layout_overflows(
            &crate::translation::TextLayout {
                font_size: font,
                line_height: layout().line_height,
                padding: layout().padding,
                lines,
            },
            200,
            300,
            Some(&s),
        ));
    }

    #[test]
    fn shapes_prefers_translation_over_page() {
        let mut b = BubbleTranslation {
            index: 0, x: 50, y: 50, w: 200, h: 100, shape: None, layout: None, font_size: None,
            original: "o".into(), reading: "".into(), translated: "HI".into(),
            ai_original: "o".into(), ai_reading: "".into(), ai_translated: "HI".into(),
            needs_white_patch: true, is_user_edited: false,
        };
        let fresh = vec![[10, 10], [60, 10], [60, 60]];
        b.shape = Some(fresh.clone());
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![b],
        };
        let stale = crate::bubble::BubbleBox {
            x: 50, y: 50, w: 200, h: 100, confidence: 1.0,
            shape: Some(vec![[50, 50], [250, 50], [150, 150]]),
            kind: None, tail: None,
        };
        let shapes = shapes_for_export(&t, &[stale]);
        assert_eq!(shapes[0], Some(fresh));
    }

    #[test]
    fn shapes_falls_back_to_page_then_bbox() {
        let b = BubbleTranslation {
            index: 0, x: 50, y: 50, w: 200, h: 100, shape: None, layout: None, font_size: None,
            original: "o".into(), reading: "".into(), translated: "HI".into(),
            ai_original: "o".into(), ai_reading: "".into(), ai_translated: "HI".into(),
            needs_white_patch: true, is_user_edited: false,
        };
        let t = PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: vec![b],
        };
        let with = crate::bubble::BubbleBox {
            x: 50, y: 50, w: 200, h: 100, confidence: 1.0,
            shape: Some(vec![[50, 50], [250, 50], [150, 150]]),
            kind: None, tail: None,
        };
        let shapes = shapes_for_export(&t, &[with]);
        assert!(shapes[0].is_some());
        let plain = crate::bubble::BubbleBox {
            x: 50, y: 50, w: 200, h: 100, confidence: 1.0,
            shape: None, kind: None, tail: None,
        };
        let shapes2 = shapes_for_export(&t, &[plain]);
        assert_eq!(shapes2[0], None);
    }
}
