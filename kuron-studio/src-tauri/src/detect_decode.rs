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

/// Sisi grid prototypes `output1` (1,32,256,256) = input 1024 / 4.
pub const PROTO_SIDE: usize = 256;

 /// Decode `output0` flat (37*N_ANCHORS, row-major [row][anchor]) → boxes.
 /// Dipanggil dengan `conf_thres=0.25` (selaras `post_process`).
 pub fn decode(output0: &[f32], lb: &Letterbox, conf_thres: f32) -> Vec<BubbleBox> {
    decode_inner(output0, lb, conf_thres)
        .into_iter()
        .map(|(b, _, _)| b)
        .collect()
}

/// Seperti `decode`, tapi tiap box membawa score + index anchor lolos NMS —
/// anchor dibutuhkan untuk mengambil 32 koefisien mask (`mask_coeffs`).
pub fn decode_kept(output0: &[f32], lb: &Letterbox, conf_thres: f32) -> Vec<(BubbleBox, f32, usize)> {
    decode_inner(output0, lb, conf_thres)
}

fn decode_inner(output0: &[f32], lb: &Letterbox, conf_thres: f32) -> Vec<(BubbleBox, f32, usize)> {
     assert_eq!(output0.len(), 37 * N_ANCHORS);
     let at = |row: usize, a: usize| output0[row * N_ANCHORS + a];
    let mut out: Vec<(BubbleBox, f32, usize)> = Vec::new();
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
            out.push((b, score, a));
         }
     }
     nms(out)
 }
 
 /// NMS greedy per-class tunggal, IoU threshold 0.5.
fn nms(mut cands: Vec<(BubbleBox, f32, usize)>) -> Vec<(BubbleBox, f32, usize)> {
     cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut kept: Vec<(BubbleBox, f32, usize)> = Vec::with_capacity(cands.len());
    'outer: for (b, s, a) in cands {
         for k in &kept {
            if iou(&b, &k.0) > NMS_IOU {
                 continue 'outer;
             }
         }
        kept.push((b, s, a));
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

/// 32 koefisien mask untuk anchor `a`: baris 5..37 output0.
pub fn mask_coeffs(output0: &[f32], anchor: usize) -> [f32; N_COEFF] {
    core::array::from_fn(|k| output0[(5 + k) * N_ANCHORS + anchor])
}

/// Ambang logit mask: sigmoid(x) > 0.5 ⟺ x > 0.
pub const MASK_LOGIT_THRESH: f32 = 0.0;

/// Box px-original → crop dalam grid prototypes 256 (letterbox/4).
/// Return None bila crop ambruk (<3px) atau di luar grid.
pub fn proto_crop(b: &BubbleBox, lb: &Letterbox) -> Option<(usize, usize, usize, usize)> {
    let s = PROTO_SIDE as f32;
    let x0 = ((b.x as f32 * lb.scale + lb.pad_l as f32) / 4.0).floor().max(0.0);
    let y0 = ((b.y as f32 * lb.scale + lb.pad_t as f32) / 4.0).floor().max(0.0);
    let x1 = (((b.x + b.w) as f32 * lb.scale + lb.pad_l as f32) / 4.0).ceil().min(s);
    let y1 = (((b.y + b.h) as f32 * lb.scale + lb.pad_t as f32) / 4.0).ceil().min(s);
    let (x0, y0, x1, y1) = (x0 as usize, y0 as usize, x1 as usize, y1 as usize);
    if x1.saturating_sub(x0) < 3 || y1.saturating_sub(y0) < 3 {
        return None;
    }
    Some((x0, y0, x1, y1))
}

/// Mask model → polygon px-original: sigmoid(coeff·protos) di crop box,
/// threshold 0.5, tracing kontur Moore + RDP, petakan balik via inverse-letterbox.
/// Return None bila crop/protos tak valid atau mask kosong.
pub fn model_mask_polygon(
    protos: &[f32],
    coeffs: &[f32; N_COEFF],
    crop: (usize, usize, usize, usize),
    lb: &Letterbox,
) -> Option<Vec<[i32; 2]>> {
    const S: usize = PROTO_SIDE;
    if protos.len() != N_COEFF * S * S {
        return None;
    }
    let (x0, y0, x1, y1) = crop;
    if x1 > S || y1 > S || x1.saturating_sub(x0) < 3 || y1.saturating_sub(y0) < 3 {
        return None;
    }
    let (w, h) = (x1 - x0, y1 - y0);
    let mut mask = vec![vec![false; w]; h];
    let mut any = false;
    for (iy, y) in (y0..y1).enumerate() {
        for (ix, x) in (x0..x1).enumerate() {
            let mut logit = 0.0f32;
            for c in 0..N_COEFF {
                logit += coeffs[c] * protos[(c * S + y) * S + x];
            }
            if logit > MASK_LOGIT_THRESH {
                mask[iy][ix] = true;
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    let contour = crate::outline::trace_contour(&mask)?;
    let simp = crate::outline::simplify_rdp(
        &contour,
        crate::outline::RDP_EPSILON,
        crate::outline::MAX_POINTS,
    );
    if simp.len() < 3 {
        return None;
    }
    let (ow, oh) = (lb.orig_w as i32, lb.orig_h as i32);
    Some(
        simp.into_iter()
            .map(|(lx, ly)| {
                let ox = ((x0 as i32 + lx) as f32 * 4.0 - lb.pad_l as f32) / lb.scale;
                let oy = ((y0 as i32 + ly) as f32 * 4.0 - lb.pad_t as f32) / lb.scale;
                [
                    (ox.round() as i32).clamp(0, ow - 1),
                    (oy.round() as i32).clamp(0, oh - 1),
                ]
            })
            .collect(),
    )
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

    #[test]
    fn decode_kept_carries_winning_anchors() {
        let o = synth(&[
            (1, 100.0, 100.0, 60.0, 60.0, 0.9),
            (2, 800.0, 800.0, 60.0, 60.0, 0.8),
        ]);
        let kept = decode_kept(&o, &lb_square(), 0.25);
        assert_eq!(kept.len(), 2);
        let mut anchors: Vec<usize> = kept.iter().map(|(_, _, a)| *a).collect();
        anchors.sort_unstable();
        assert_eq!(anchors, vec![1, 2]);
        let plain = decode(&o, &lb_square(), 0.25);
        assert_eq!(kept.len(), plain.len());
    }

    #[test]
    fn mask_coeffs_reads_rows_5_to_36() {
        let n = N_ANCHORS;
        let mut o = vec![0.0f32; 37 * n];
        let mut want = 1.0f32;
        for row in o.chunks_exact_mut(n).skip(5).take(N_COEFF) {
            row[42] = want;
            want += 1.0;
        }
        let c = mask_coeffs(&o, 42);
        for (k, got) in c.iter().enumerate() {
            assert!((got - (k as f32 + 1.0)).abs() < 1e-6);
        }
    }

    fn bubble_at(x: i32, y: i32, w: i32, h: i32) -> BubbleBox {
        BubbleBox {
            x,
            y,
            w,
            h,
            confidence: 0.9,
            shape: None,
            kind: Some("rect".to_string()),
            tail: None,
        }
    }

    #[test]
    fn proto_crop_maps_box_to_quarter_grid() {
        let crop = proto_crop(&bubble_at(100, 100, 200, 160), &lb_square()).unwrap();
        assert_eq!(crop, (25, 25, 75, 65));
    }

    #[test]
    fn proto_crop_rejects_degenerate() {
        assert!(proto_crop(&bubble_at(0, 0, 4, 4), &lb_square()).is_none());
    }

    fn rect_protos(x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<f32> {
        const S: usize = PROTO_SIDE;
        let mut p = vec![0.0f32; N_COEFF * S * S];
        for y in y0..y1 {
            for x in x0..x1 {
                p[y * S + x] = 1.0;
            }
        }
        p
    }

    #[test]
    fn model_mask_polygon_recovers_rect() {
        let protos = rect_protos(25, 25, 75, 65);
        let mut coeffs = [0.0f32; N_COEFF];
        coeffs[0] = 8.0;
        let poly =
            model_mask_polygon(&protos, &coeffs, (25, 25, 75, 65), &lb_square()).unwrap();
        assert!(poly.len() >= 4, "rect harus jadi polygon, dapat {}", poly.len());
        for [px, py] in &poly {
            assert!((96..=304).contains(px), "x={px} di luar box");
            assert!((96..=264).contains(py), "y={py} di luar box");
        }
    }

    #[test]
    fn model_mask_polygon_rejects_bad_input() {
        let mut coeffs = [0.0f32; N_COEFF];
        coeffs[0] = 8.0;
        assert!(model_mask_polygon(&[0.0; 10], &coeffs, (0, 0, 64, 64), &lb_square()).is_none());
        let protos = rect_protos(10, 10, 50, 50);
        assert!(
            model_mask_polygon(&protos, &[0.0; N_COEFF], (10, 10, 50, 50), &lb_square()).is_none()
        );
        assert!(model_mask_polygon(&protos, &coeffs, (10, 10, 11, 11), &lb_square()).is_none());
    }
}
