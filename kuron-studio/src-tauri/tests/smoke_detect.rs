//! Smoke test M1-3/M1-4: load bubble.onnx nyata + infer satu halaman sintetis.
//! Dijalankan via `cargo test --test smoke_detect`. Bukan unit test lib
//! (butuh file model 11MB); gagal bila model hilang atau output shape berubah.

use std::path::PathBuf;

use image::{Rgb, RgbImage};
use kuron_studio_lib::detector::DetectorState;

fn model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/models/bubble.onnx")
}

/// Halaman sintetis 800x1200: latar putih + elips border hitam (mirip bubble).
fn synthetic_page_png() -> Vec<u8> {
    let (w, h) = (800u32, 1200u32);
    let mut img = RgbImage::from_pixel(w, h, Rgb([255, 255, 255]));
    let (cx, cy, rx, ry) = (400.0f32, 400.0f32, 250.0f32, 150.0f32);
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx) / rx;
            let dy = (y as f32 - cy) / ry;
            let d = dx * dx + dy * dy;
            if (d - 1.0).abs() < 0.03 {
                img.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }
    }
    let mut buf = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut buf);
    use image::ImageEncoder;
    enc.write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .expect("encode png");
    buf
}

#[test]
fn smoke_detect_runs_on_real_model() {
    let path = model_path();
    assert!(path.exists(), "model hilang: {}", path.display());
    let bytes = synthetic_page_png();
    let det = DetectorState::new();
    let t0 = std::time::Instant::now();
    let boxes = det
        .detect_bytes(&path, &bytes)
        .expect("infer bubble.onnx gagal");
    let dt = t0.elapsed();
    println!("smoke: {} boxes dalam {dt:.1?}", boxes.len());
    // Halaman sintetis boleh 0 box; yang penting infer jalan + koordinat valid.
    for b in &boxes {
        assert!(b.x >= 0 && b.y >= 0 && b.w >= 8 && b.h >= 8);
        assert!((0.0..=1.0).contains(&b.confidence));
        assert!(b.x + b.w <= 800 && b.y + b.h <= 1200);
    }
}
