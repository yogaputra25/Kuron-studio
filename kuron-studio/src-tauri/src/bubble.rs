use serde::{Deserialize, Serialize};

/// Mirror Kuron `BubbleBox` — satu-satunya source of truth bubble di backend.
/// Frontend (`src/lib/bubble.ts`) wajib field-for-field kompatibel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BubbleBox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default)]
    pub confidence: f32,
    /// Polygon outline dalam px original (freeform). None = rect/ellipse biasa.
    #[serde(default)]
    pub shape: Option<Vec<[i32; 2]>>,
    /// "rect" | "ellipse" | "freeform". None = rect (legacy).
    #[serde(default)]
    pub kind: Option<String>,
    /// Ekor bubble: [base, tip] dalam px original.
    #[serde(default)]
    pub tail: Option<Vec<[i32; 2]>>,
}

impl BubbleBox {
    pub fn area(&self) -> i64 {
        (self.w.max(0) as i64) * (self.h.max(0) as i64)
    }

    /// True jika `self` sepenuhnya menelan rect `other`.
    pub fn engulfs(&self, other: &BubbleBox) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.w >= other.x + other.w
            && self.y + self.h >= other.y + other.h
    }
}

/// Postprocess hasil ONNX / input manual — mirror `postProcessBoxes` Kuron:
/// filter confidence >= 0.25, buang box raksasa false-positive yang menelan
/// box 2.5x lebih kecil.
pub fn post_process(boxes: Vec<BubbleBox>) -> Vec<BubbleBox> {
    let kept: Vec<BubbleBox> = boxes
        .into_iter()
        .filter(|b| b.confidence >= 0.25 && b.w > 8 && b.h > 8)
        .collect();
    let n = kept.len();
    let mut giant = vec![false; n];
    for i in 0..n {
        for j in 0..n {
            if i == j || giant[i] {
                continue;
            }
            if kept[i].engulfs(&kept[j])
                && kept[i].area() as f32 >= 2.5 * kept[j].area().max(1) as f32
            {
                giant[i] = true;
                break;
            }
        }
    }
    kept
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !giant[*i])
        .map(|(_, b)| b)
        .collect()
}

/// Sanitasi bubble dari canvas sebelum persist: buang yang nol-area,
/// batasi 500/page agar projects.json tidak bengkak.
pub fn sanitize_bubbles(boxes: Vec<BubbleBox>) -> Vec<BubbleBox> {
    boxes
        .into_iter()
        .filter(|b| b.w > 0 && b.h > 0)
        .take(500)
        .collect()
}

// ponytail: `ort` (ONNX Runtime) dipasang saat file bubble.onnx tersedia.
// Upgrade path: tambah `ort = "2"` di Cargo.toml, `BubbleDetector::new`
// load model + `detect()` preprocess→infer→decode lalu `post_process`.
// Stub di bawah bikin M1 (canvas manual) shippable tanpa model.

/// Lokasi model yang dicari, berurutan: bundled resource, lalu app-data.
/// User cukup drop `bubble.onnx` ke salah satunya tanpa rebuild.
pub fn model_search_paths(resource_dir: Option<std::path::PathBuf>, data_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut v = Vec::new();
    if let Some(res) = resource_dir {
        // Layout bundel: prefix `resources/` di-strip → `<res>/models/bubble.onnx`.
        v.push(res.join("models").join("bubble.onnx"));
        // Layout dev (`tauri dev` copy mentah): `<target/debug>/resources/models/bubble.onnx`.
        v.push(res.join("resources").join("models").join("bubble.onnx"));
    }
    v.push(data_dir.join("models").join("bubble.onnx"));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(x: i32, y: i32, w: i32, h: i32, conf: f32) -> BubbleBox {
        BubbleBox {
            x,
            y,
            w,
            h,
            confidence: conf,
            shape: None,
            kind: None,
            tail: None,
        }
    }

    #[test]
    fn drops_low_confidence() {
        let out = post_process(vec![b(0, 0, 50, 50, 0.9), b(60, 0, 50, 50, 0.1)]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].x, 0);
    }

    #[test]
    fn drops_giant_engulfing_box() {
        let giant = b(0, 0, 500, 500, 0.9);
        let small = b(10, 10, 50, 50, 0.8);
        let out = post_process(vec![giant, small.clone()]);
        assert_eq!(out, vec![small]);
    }

    #[test]
    fn keeps_adjacent_boxes() {
        let out = post_process(vec![b(0, 0, 100, 100, 0.9), b(110, 0, 100, 100, 0.9)]);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn sanitize_drops_zero_area() {
        let out = sanitize_bubbles(vec![b(0, 0, 0, 50, 1.0), b(0, 0, 50, 50, 1.0)]);
        assert_eq!(out.len(), 1);
    }
}
