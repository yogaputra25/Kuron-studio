use image::{GenericImage, RgbImage};

use crate::bubble::BubbleBox;

/// Kualitas mosaic — mirror `MosaicQuality` Kuron (cap + JPEG quality).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MosaicQuality {
    Low,
    High,
}

impl MosaicQuality {
    fn jpeg_quality(&self) -> u8 {
        match self {
            MosaicQuality::Low => 75,
            MosaicQuality::High => 85,
        }
    }

    fn byte_cap(&self) -> usize {
        match self {
            MosaicQuality::Low => 1_000_000,
            MosaicQuality::High => 2_000_000,
        }
    }
}

const LABEL_H: u32 = 40;
const ROW_GAP: u32 = 10;

// Font bitmap 3x5 untuk digit label merah — tanpa dep font eksternal.
// ponytail: ganti ab_glyph + DejaVu saat butuh label >99 atau huruf.
const DIGITS: [[[u8; 3]; 5]; 10] = [
    [[1, 1, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 1, 1]],
    [[0, 1, 0], [1, 1, 0], [0, 1, 0], [0, 1, 0], [1, 1, 1]],
    [[1, 1, 1], [0, 0, 1], [1, 1, 1], [1, 0, 0], [1, 1, 1]],
    [[1, 1, 1], [0, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]],
    [[1, 0, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [0, 0, 1]],
    [[1, 1, 1], [1, 0, 0], [1, 1, 1], [0, 0, 1], [1, 1, 1]],
    [[1, 1, 1], [1, 0, 0], [1, 1, 1], [1, 0, 1], [1, 1, 1]],
    [[1, 1, 1], [0, 0, 1], [0, 0, 1], [0, 1, 0], [0, 1, 0]],
    [[1, 1, 1], [1, 0, 1], [1, 1, 1], [1, 0, 1], [1, 1, 1]],
    [[1, 1, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]],
];

fn draw_digit(canvas: &mut RgbImage, digit: u8, x0: u32, y0: u32, scale: u32) {
    let glyph = DIGITS[(digit.min(9)) as usize];
    for (gy, row) in glyph.iter().enumerate() {
        for (gx, &on) in row.iter().enumerate() {
            if on == 0 {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let (px, py) = (x0 + gx as u32 * scale + dx, y0 + gy as u32 * scale + dy);
                    if px < canvas.width() && py < canvas.height() {
                        canvas.put_pixel(px, py, image::Rgb([255, 255, 255]));
                    }
                }
            }
        }
    }
}

fn draw_label(canvas: &mut RgbImage, index: usize, y0: u32, width: u32) {
    for y in y0..y0 + LABEL_H {
        for x in 0..width {
            canvas.put_pixel(x, y, image::Rgb([220, 20, 20]));
        }
    }
    // Nomor 1-based, max 2 digit.
    let n = (index + 1).min(99) as u8;
    if n >= 10 {
        draw_digit(canvas, n / 10, 8, y0 + 10, 4);
        draw_digit(canvas, n % 10, 8 + 3 * 4 + 4, y0 + 10, 4);
    } else {
        draw_digit(canvas, n, 8, y0 + 10, 4);
    }
}

fn encode_jpeg_rgb(rgb: &RgbImage, quality: u8) -> Result<Vec<u8>, String> {
    let (w, h) = rgb.dimensions();
    let mut out = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
    enc.encode(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// Mosaic Kuron 1:1 — crop 20% pad + clamp, resize 2x, stack vertikal +
/// gap 10 + label merah, cap 2MB/1MB dengan downscale 0.75x loop.
pub fn build_mosaic(
    page_bytes: &[u8],
    bubbles: &[BubbleBox],
    quality: MosaicQuality,
) -> Result<Vec<u8>, String> {
    if bubbles.is_empty() {
        return Err("no bubbles — pakai fallback compress_page".to_string());
    }
    let page = image::load_from_memory(page_bytes)
        .map_err(|e| format!("decode: {e}"))?
        .to_rgb8();
    let (pw, ph) = page.dimensions();

    let mut chips: Vec<RgbImage> = Vec::with_capacity(bubbles.len());
    for b in bubbles {
        let pad_x = (b.w as f32 * 0.2) as i32;
        let pad_y = (b.h as f32 * 0.2) as i32;
        let x0 = (b.x - pad_x).clamp(0, pw as i32 - 1).max(0) as u32;
        let y0 = (b.y - pad_y).clamp(0, ph as i32 - 1).max(0) as u32;
        let x1 = (b.x + b.w + pad_x).clamp(1, pw as i32) as u32;
        let y1 = (b.y + b.h + pad_y).clamp(1, ph as i32) as u32;
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        let crop = image::imageops::crop_imm(&page, x0, y0, x1 - x0, y1 - y0).to_image();
        let (cw, ch) = (crop.width().max(1), crop.height().max(1));
        chips.push(image::imageops::resize(
            &crop,
            cw * 2,
            ch * 2,
            image::imageops::FilterType::Triangle,
        ));
    }
    if chips.is_empty() {
        return Err("semua bubble di luar bounds".to_string());
    }

    let total_w = chips.iter().map(|c| c.width()).max().unwrap_or(64);
    let mut total_h: u32 = 0;
    for c in &chips {
        total_h += LABEL_H + c.height() + ROW_GAP;
    }
    total_h -= ROW_GAP;

    let mut mosaic = RgbImage::from_pixel(total_w, total_h, image::Rgb([255, 255, 255]));
    let mut y = 0;
    for (i, chip) in chips.iter().enumerate() {
        draw_label(&mut mosaic, i, y, total_w);
        y += LABEL_H;
        let x_off = (total_w - chip.width()) / 2;
        mosaic
            .copy_from(chip, x_off, y)
            .map_err(|e| e.to_string())?;
        y += chip.height() + ROW_GAP;
    }

    // Cap + downscale loop.
    let mut jpeg = encode_jpeg_rgb(&mosaic, quality.jpeg_quality())?;
    while jpeg.len() > quality.byte_cap() && mosaic.width() > 64 {
        let (nw, nh) = (
            ((mosaic.width() as f32 * 0.75) as u32).max(64),
            ((mosaic.height() as f32 * 0.75) as u32).max(64),
        );
        mosaic = image::imageops::resize(&mosaic, nw, nh, image::imageops::FilterType::Triangle);
        jpeg = encode_jpeg_rgb(&mosaic, quality.jpeg_quality())?;
    }
    Ok(jpeg)
}

/// Fallback Kuron 1:1 — longest side 1280, JPEG85.
pub fn compress_page(page_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let page = image::load_from_memory(page_bytes)
        .map_err(|e| format!("decode: {e}"))?
        .to_rgb8();
    let (w, h) = page.dimensions();
    let longest = w.max(h).max(1);
    let out = if longest > 1280 {
        let s = 1280.0 / longest as f32;
        image::imageops::resize(
            &page,
            ((w as f32 * s) as u32).max(1),
            ((h as f32 * s) as u32).max(1),
            image::imageops::FilterType::Triangle,
        )
    } else {
        page
    };
    encode_jpeg_rgb(&out, 85)
}

/// Long-strip split — potong tiap max_chunk_h px, JPEG90 per chunk.
pub fn chunk_webtoon(page_bytes: &[u8], max_chunk_h: u32) -> Result<Vec<Vec<u8>>, String> {
    let page = image::load_from_memory(page_bytes)
        .map_err(|e| format!("decode: {e}"))?
        .to_rgb8();
    let (w, h) = page.dimensions();
    if max_chunk_h == 0 {
        return Err("max_chunk_h nol".to_string());
    }
    let mut out = Vec::new();
    let mut y = 0;
    while y < h {
        let ch = (h - y).min(max_chunk_h);
        let crop = image::imageops::crop_imm(&page, 0, y, w, ch).to_image();
        out.push(encode_jpeg_rgb(&crop, 90)?);
        y += ch;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bubble::BubbleBox;
    use image::DynamicImage;
    use std::io::Cursor;

    fn solid_png(w: u32, h: u32, v: [u8; 3]) -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, image::Rgb(v)));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    fn noise_png(w: u32, h: u32) -> Vec<u8> {
        let mut img = RgbImage::new(w, h);
        let mut s: u32 = 0x1234_5678;
        for p in img.pixels_mut() {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            p.0 = [(s & 0xff) as u8, ((s >> 8) & 0xff) as u8, ((s >> 16) & 0xff) as u8];
        }
        let mut buf = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    fn b(x: i32, y: i32, w: i32, h: i32) -> BubbleBox {
        BubbleBox {
            x, y, w, h,
            confidence: 0.9,
            shape: None,
            kind: None,
            tail: None,
        }
    }

    #[test]
    fn mosaic_layout_dims() {
        // 200x200. Bubble1 50x50 @ (50,50): crop 70x70 (20% pad) -> 2x = 140x140.
        // Bubble2 40x40 @ (120,120): crop 56x56 (pad 8) -> 2x = 112x112.
        // total 140 x (40+140 +10+ 40+112=342).
        let png = solid_png(200, 200, [200, 30, 30]);
        let out = build_mosaic(&png, &[b(50, 50, 50, 50), b(120, 120, 40, 40)], MosaicQuality::High).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!((img.width(), img.height()), (140, 342));
    }

    #[test]
    fn mosaic_cap_enforced() {
        let png = noise_png(1200, 1200);
        let bubbles = vec![
            b(0, 0, 500, 500),
            b(600, 0, 500, 500),
            b(0, 600, 500, 500),
            b(600, 600, 500, 500),
        ];
        let out = build_mosaic(&png, &bubbles, MosaicQuality::Low).unwrap();
        assert!(out.len() <= 1_000_000, "cap 1MB, got {}", out.len());
    }

    #[test]
    fn mosaic_empty_errors() {
        let png = solid_png(100, 100, [0, 0, 0]);
        assert!(build_mosaic(&png, &[], MosaicQuality::High).is_err());
    }

    #[test]
    fn compress_resizes_longest() {
        let png = solid_png(2000, 1000, [30, 30, 200]);
        let out = compress_page(&png).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!((img.width(), img.height()), (1280, 640));
    }

    #[test]
    fn chunk_splits_rows() {
        let png = solid_png(100, 3000, [30, 200, 30]);
        let chunks = chunk_webtoon(&png, 1000).unwrap();
        assert_eq!(chunks.len(), 3);
        for c in &chunks {
            let img = image::load_from_memory(c).unwrap();
            assert_eq!(img.height(), 1000);
        }
    }
}
