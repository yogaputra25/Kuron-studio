//! YOLO11n-seg decode untuk `bubble.onnx` (manga109 balloon, 11.3MB, opset 17).
//!
//! Model: letterbox 1024 (fill 114) → `output0` (1,37,21504) = 4 box
//! (cx,cy,w,h dalam px letterbox) + 1 score mentah + 32 mask-coeff — Sigmoid
//! sudah di dalam graph, jadi score = probabilitas langsung — dan `output1`
//! (1,32,256,256) prototypes (belum dipakai: opsi B mask→polygon).
//! Box dipetakan balik ke px original via inverse-letterbox, lalu NMS IoU 0.5
//! + `post_process` (conf>=0.25, giant 2.5x) di `bubble.rs`.
//!
//! ponytail: opsi B (mask→polygon via sigmoid(coeff@proto) crop-box +
//! marching-squares) saat overlay M2 butuh shape presisi; API `decode`
//! mengembalikan rect dan `shape: None` kompatibel maju.

use crate::bubble::BubbleBox;

pub const MODEL_SIDE: u32 = 1024;
const PAD: f32 = 114.0;
const N_ANCHORS: usize = 21504;
const N_COEFF: usize = 32;
const NMS_IOU: f32 = 0.5;

/// Letterbox RGB (u8) ke tensor CHW float32 [0,1] + info inverse-mapping.
pub fn letterbox(rgb: &image::RgbImage) -> (Vec<f32>, Letterbox) {
    let (w, h) = rgb.dimensions();
    let s = MODEL_SIDE as f32 / w.max(h).max(1) as f32;
    let (nw, nh) = ((w as f32 * s) as u32, (h as f32 * s) as u32);
    let small = image::imageops::resize(rgb, nw.max(1), nh.max(1), image::imageops::FilterType::Triangle);
    let (pad_l, pad_t) = (
        (MODEL_SIDE - small.width()) / 2,
        (MODEL_SIDE - small.height()) / 2,
    );
    let mut canvas = vec![PAD / 255.0; (MODEL_SIDE * MODEL_SIDE * 3) as usize];
    for y in 0..small.height() {
        for x in 0..small.width() {
            let p = small.get_pixel(x, y).0;
            let (dx, dy) = (x + pad_l, y + pad_t);
            let base = (dy * MODEL_SIDE + dx) as usize;
            canvas[base] = p[0] as f32 / 255.0;
            canvas[(MODEL_SIDE * MODEL_SIDE) as usize + base] = p[1] as f32 / 255.0;
            canvas[2 * (MODEL_SIDE * MODEL_SIDE) as usize + base] = p[2] as f32 / 255.0;
        }
    }
    (
        canvas,
        Letterbox {
            scale: s,
            pad_l,
            pad_t,
            orig_w: w,
            orig_h: h,
        },
    )
}

#[derive(Debug, Clone, Copy)]
pub struct Letterbox {
    pub scale: f32,
    pub pad_l: u32,
    pub pad_t: u32,
    pub orig_w: u32,
    pub orig_h: u32,
}

impl Letterbox {
    /// Petakan box (cx,cy,w,h px letterbox) → BubbleBox px original.
    pub fn unmap(&self, cx: f32, cy: f32, w: f32, h: f32, score: f32) -> Option<BubbleBox> {
        let x0 = (cx - w / 2.0 - self.pad_l as f32) / self.scale;
        let y0 = (cy - h / 2.0 - self.pad_t as f32) / self.scale;
        let (bw, bh) = (w / self.scale, h / self.scale);
        if bw < 8.0 || bh < 8.0 {
            return None;
        }
        let x = x0.round() as i32;
        let y = y0.round() as i32;
        Some(BubbleBox {
            x: x.clamp(0, self.orig_w as i32 - 1).max(0),
            y: y.clamp(0, self.orig_h as i32 - 1).max(0),
            w: (bw.round() as i32)
                .max(1)
                .min(self.orig_w as i32 - x.clamp(0, self.orig_w as i32 - 1).max(0)),
            h: (bh.round() as i32)
                .max(1)
                .min(self.orig_h as i32 - y.clamp(0, self.orig_h as i32 - 1).max(0)),
            confidence: score,
            shape: None,
            kind: Some("rect".to_string()),
            tail: None,
        })
    }
}

/// Decode `output0` flat (37*N_ANCHORS, row-major [row][anchor]) → boxes.
/// Dipanggil dengan `conf_thres=0.25` (selaras `post_process`).
pub fn decode(output0: &[f32], lb: &Letterbox, conf_thres: f32) -> Vec<BubbleBox> {
    assert_eq!(output0.len(), 37 * N_ANCHORS);
    let at = |row: usize, a: usize| output0[row * N_ANCHORS + a];
    let mut out: Vec<(BubbleBox, f32)> = Vec::new();
    for a in 0..N_ANCHORS {
        let score = at(4, a);
        if score < conf_thres {
            continue;
        }
        let (cx, cy, w, h) = (at(0, a), at(1, a), at(2, a), at(3, a));
        if !cx.is_finite() || !cy.is_finite() || w <= 0.0 || h <= 0.0 {
            continue;
        }
        if let Some(b) = lb.unmap(cx, cy, w, h, score) {
            out.push((b, score));
        }
    }
    nms(out)
}

/// NMS greedy per-class tunggal, IoU threshold 0.5.
fn nms(mut cands: Vec<(BubbleBox, f32)>) -> Vec<BubbleBox> {
    cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut kept: Vec<BubbleBox> = Vec::with_capacity(cands.len());
    'outer: for (b, _) in cands {
        for k in &kept {
            if iou(&b, k) > NMS_IOU {
                continue 'outer;
            }
        }
        kept.push(b);
    }
    kept
}

fn iou(a: &BubbleBox, b: &BubbleBox) -> f32 {
    let (ax1, ay1, ax2, ay2) = (a.x, a.y, a.x + a.w, a.y + a.h);
    let (bx1, by1, bx2, by2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let (ix1, iy1) = (ax1.max(bx1), ay1.max(by1));
    let (ix2, iy2) = (ax2.min(bx2), ay2.min(by2));
    let inter = ((ix2 - ix1).max(0) as f32) * ((iy2 - iy1).max(0) as f32);
    if inter <= 0.0 {
        return 0.0;
    }
    let union = (a.area().max(1) + b.area().max(1)) as f32 - inter;
    inter / union.max(1.0)
}

#[allow(dead_code)]
fn mask_coeffs(_output0: &[f32], _anchor: usize) -> [f32; N_COEFF] {
    // ponytail opsi B: output0[(5+k)*N_ANCHORS + a], lalu
    // sigmoid(coeff @ prototypes[32][256][256]) crop-box → polygon.
    [0.0; N_COEFF]
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    fn lb_square() -> Letterbox {
        Letterbox {
            scale: 1.0,
            pad_l: 0,
            pad_t: 0,
            orig_w: 1024,
            orig_h: 1024,
        }
    }

    /// Bangun output0 sintetik: row r, anchor `idx` bernilai `vals`.
    fn synth(patches: &[(usize, f32, f32, f32, f32, f32)]) -> Vec<f32> {
        let stride = N_ANCHORS;
        let mut o = vec![0.0f32; 37 * stride];
        for &(idx, cx, cy, w, h, score) in patches {
            o[idx] = cx;
            o[stride + idx] = cy;
            o[2 * stride + idx] = w;
            o[3 * stride + idx] = h;
            o[4 * stride + idx] = score;
        }
        o
    }

    #[test]
    fn letterbox_identity_for_1024_square() {
        let img = RgbImage::from_pixel(1024, 1024, image::Rgb([200, 30, 30]));
        let (t, lb) = letterbox(&img);
        assert_eq!(t.len(), 1024 * 1024 * 3);
        assert!((lb.scale - 1.0).abs() < 1e-6);
        assert_eq!((lb.pad_l, lb.pad_t), (0, 0));
        assert!((t[0] - 200.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn letterbox_tall_page_centers_and_fills_114() {
        // 512x1024 → scale 1.0, pad_l=(1024-512)/2=256, pad_t=0.
        let img = RgbImage::from_pixel(512, 1024, image::Rgb([10, 20, 30]));
        let (t, lb) = letterbox(&img);
        assert_eq!((lb.pad_l, lb.pad_t), (256, 0));
        // Pojok kiri-atas kanvas = fill 114.
        assert!((t[0] - 114.0 / 255.0).abs() < 1e-6);
        // Pixel gambar pertama di x=256.
        assert!((t[256] - 10.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn decode_single_box_maps_identity() {
        let o = synth(&[(7, 512.0, 512.0, 100.0, 80.0, 0.9)]);
        let out = decode(&o, &lb_square(), 0.25);
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].x, out[0].y, out[0].w, out[0].h), (462, 472, 100, 80));
        assert_eq!(out[0].kind.as_deref(), Some("rect"));
    }

    #[test]
    fn decode_drops_low_score() {
        let o = synth(&[(3, 100.0, 100.0, 50.0, 50.0, 0.1)]);
        assert!(decode(&o, &lb_square(), 0.25).is_empty());
    }

    #[test]
    fn decode_nms_keeps_winner() {
        // Dua box hampir identik, score beda → 1 selamat.
        let o = synth(&[
            (1, 200.0, 200.0, 100.0, 100.0, 0.9),
            (2, 205.0, 205.0, 100.0, 100.0, 0.7),
        ]);
        let out = decode(&o, &lb_square(), 0.25);
        assert_eq!(out.len(), 1);
        assert!((out[0].confidence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn decode_keeps_distant_boxes() {
        let o = synth(&[
            (1, 100.0, 100.0, 60.0, 60.0, 0.9),
            (2, 800.0, 800.0, 60.0, 60.0, 0.8),
        ]);
        assert_eq!(decode(&o, &lb_square(), 0.25).len(), 2);
    }
}
