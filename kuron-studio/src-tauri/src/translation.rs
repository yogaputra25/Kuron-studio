use serde::{Deserialize, Serialize};

use crate::bubble::BubbleBox;
use crate::parser::{BubbleResult, FullBubble};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BubbleTranslation {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default)]
    pub original: String,
    #[serde(default)]
    pub reading: String,
    #[serde(default)]
    pub translated: String,
    /// Baseline AI terakhir (opsi A reset-translation): diisi jalur AI saja,
    /// TAK disentuh manual. `save_translation` merge dari stored by index.
    #[serde(default)]
    pub ai_original: String,
    #[serde(default)]
    pub ai_reading: String,
    #[serde(default)]
    pub ai_translated: String,
    #[serde(default)]
    pub needs_white_patch: bool,
    #[serde(default)]
    pub is_user_edited: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageTranslation {
    pub page_id: String,
    pub target_lang: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub bubbles: Vec<BubbleTranslation>,
}

/// Wide box or >25% page area needs white patch before typeset.
pub fn needs_white_patch(w: i32, h: i32, page_w: i32, page_h: i32) -> bool {
    let (w, h) = (w.max(0) as f64, h.max(0) as f64);
    if h <= 0.0 {
        return false;
    }
    let aspect = w / h;
    let frac = if page_w > 0 && page_h > 0 {
        (w * h) / (page_w as f64 * page_h as f64)
    } else {
        0.0
    };
    aspect > 2.5 || frac > 0.25
}

/// Mirror frontend bubble.ts: same-row (center within 0.5*rowH) grouped,
/// rtl sorts x desc else asc; rows top-to-bottom.
pub fn order_indices(bubbles: &[BubbleBox], reading_dir: &str) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..bubbles.len()).collect();
    idx.sort_by(|&a, &b| {
        let (ba, bb) = (&bubbles[a], &bubbles[b]);
        let row_a = ba.y as f64 + ba.h as f64 / 2.0;
        let row_b = bb.y as f64 + bb.h as f64 / 2.0;
        let row_h = ba.h.max(bb.h).max(1) as f64;
        if (row_a - row_b).abs() > row_h * 0.5 {
            row_a.partial_cmp(&row_b).unwrap_or(std::cmp::Ordering::Equal)
        } else if reading_dir == "ltr" {
            ba.x.cmp(&bb.x)
        } else {
            bb.x.cmp(&ba.x)
        }
    });
    idx
}

fn patch_for(b: &BubbleBox, page_w: i32, page_h: i32) -> bool {
    needs_white_patch(b.w, b.h, page_w, page_h)
}

/// Reattach geometry from stored bubbles by index (results keyed 1-based in reading order).
pub fn map_mosaic(
    bubbles: &[BubbleBox],
    ordered: &[usize],
    results: &[(usize, BubbleResult)],
    page_w: i32,
    page_h: i32,
) -> Vec<BubbleTranslation> {
    let by_key: std::collections::HashMap<usize, &BubbleResult> =
        results.iter().map(|(n, r)| (*n, r)).collect();
    ordered
        .iter()
        .enumerate()
        .map(|(rank, &bi)| {
            let b = &bubbles[bi];
            let r = by_key.get(&(rank + 1));
            BubbleTranslation {
                index: bi,
                x: b.x,
                y: b.y,
                w: b.w,
                h: b.h,
                original: r.map(|x| x.original.clone()).unwrap_or_default(),
                reading: r.map(|x| x.reading.clone()).unwrap_or_default(),
                translated: r.map(|x| x.translated.clone()).unwrap_or_default(),
                ai_original: r.map(|x| x.original.clone()).unwrap_or_default(),
                ai_reading: r.map(|x| x.reading.clone()).unwrap_or_default(),
                ai_translated: r.map(|x| x.translated.clone()).unwrap_or_default(),
                needs_white_patch: patch_for(b, page_w, page_h),
                is_user_edited: false,
            }
        })
        .collect()
}

/// Map percent coords to px.
pub fn map_full_image(page_w: i32, page_h: i32, items: &[FullBubble]) -> Vec<BubbleTranslation> {
    items
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let (x, y, w, h) = (
                (f.x / 100.0 * page_w as f32) as i32,
                (f.y / 100.0 * page_h as f32) as i32,
                (f.w / 100.0 * page_w as f32) as i32,
                (f.h / 100.0 * page_h as f32) as i32,
            );
            BubbleTranslation {
                index: i,
                x,
                y,
                w,
                h,
                original: f.original.clone(),
                reading: f.reading.clone(),
                translated: f.translated.clone(),
                ai_original: f.original.clone(),
                ai_reading: f.reading.clone(),
                ai_translated: f.translated.clone(),
                needs_white_patch: needs_white_patch(w, h, page_w, page_h),
                is_user_edited: false,
            }
        })
        .collect()
}

/// Keep user-edited bubbles (match by index) across re-translates.
pub fn preserve_user_edits(
    new: Vec<BubbleTranslation>,
    prev: Option<&PageTranslation>,
) -> Vec<BubbleTranslation> {
    preserve_user_edits_except_vec(new, prev, None)
}

/// Seperti preserve_user_edits tapi bubble `except_index` selalu diambil dari
/// hasil baru (dipakai retry_bubble: target refresh, lainnya dipertahankan).
pub fn preserve_user_edits_except(
    merged: PageTranslation,
    prev: &PageTranslation,
    except_index: usize,
) -> PageTranslation {
    let bubbles = preserve_user_edits_except_vec(merged.bubbles, Some(prev), Some(except_index));
    PageTranslation { bubbles, ..merged }
}

fn preserve_user_edits_except_vec(
    new: Vec<BubbleTranslation>,
    prev: Option<&PageTranslation>,
    except_index: Option<usize>,
) -> Vec<BubbleTranslation> {
    match prev {
        None => new,
        Some(p) => {
            let edited: std::collections::HashMap<usize, &BubbleTranslation> = p
                .bubbles
                .iter()
                .filter(|b| b.is_user_edited)
                .map(|b| (b.index, b))
                .collect();
            new.into_iter()
                .map(|mut b| {
                    if Some(b.index) != except_index {
                        if let Some(old) = edited.get(&b.index) {
                            b.original = old.original.clone();
                            b.reading = old.reading.clone();
                            b.translated = old.translated.clone();
                            // Baseline lama ikut: AI baru untuk bubble ini memang
                            // ditolak user, jadi "AI terakhir yg dilihat" = yg lama.
                            b.ai_original = old.ai_original.clone();
                            b.ai_reading = old.ai_reading.clone();
                            b.ai_translated = old.ai_translated.clone();
                            b.is_user_edited = true;
                        }
                    }
                    b
                })
                .collect()
        }
    }
}

/// Merge baseline AI dari stored ke kiriman `save_translation` (match by index).
/// ANTI-TAMPER: frontend TAK dipercaya mengeset `ai*` — kiriman bisa kosong/stale
/// (race debounce); hanya stored yang jadi sumber baseline. Jangan "perbaiki"
/// jadi pass-through.
pub fn merge_ai_baseline(
    stored: &[BubbleTranslation],
    incoming: Vec<BubbleTranslation>,
) -> Vec<BubbleTranslation> {
    let base: std::collections::HashMap<usize, &BubbleTranslation> =
        stored.iter().map(|b| (b.index, b)).collect();
    incoming
        .into_iter()
        .map(|mut b| {
            if let Some(s) = base.get(&b.index) {
                b.ai_original = s.ai_original.clone();
                b.ai_reading = s.ai_reading.clone();
                b.ai_translated = s.ai_translated.clone();
            }
            b
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bubble::BubbleBox;

    fn b(x: i32, y: i32, w: i32, h: i32) -> BubbleBox {
        BubbleBox { x, y, w, h, confidence: 1.0, shape: None, kind: None, tail: None }
    }

    #[test]
    fn rtl_row_order() {
        let bs = vec![b(0, 0, 50, 50), b(200, 0, 50, 50), b(0, 200, 50, 50)];
        assert_eq!(order_indices(&bs, "rtl"), vec![1, 0, 2]);
        assert_eq!(order_indices(&bs, "ltr"), vec![0, 1, 2]);
    }

    #[test]
    fn patch_rules() {
        assert!(needs_white_patch(300, 50, 800, 1200));
        assert!(needs_white_patch(500, 500, 800, 1200));
        assert!(!needs_white_patch(100, 80, 800, 1200));
    }

    #[test]
    fn edits_preserved() {
        let mk = |o: &str, r: &str, t: &str, e: bool| BubbleTranslation { index: 0, x: 0, y: 0, w: 10, h: 10, original: o.into(), reading: r.into(), translated: t.into(), ai_original: "ai-O".into(), ai_reading: "ai-R".into(), ai_translated: "ai".into(), needs_white_patch: false, is_user_edited: e };
        let prev = PageTranslation { page_id: "p".into(), target_lang: "id".into(), style: "".into(), model: "".into(), bubbles: vec![mk("manual-O", "manual-R", "user", true)] };
        let out = preserve_user_edits(vec![mk("ai-O", "ai-R", "ai", false)], Some(&prev));
        assert_eq!(out[0].original, "manual-O");
        assert_eq!(out[0].reading, "manual-R");
        assert_eq!(out[0].translated, "user");
        assert!(out[0].is_user_edited);
    }

    #[test]
    fn unedited_bubbles_still_update() {
        let mk = |i: usize, t: &str, e: bool| BubbleTranslation { index: i, x: 0, y: 0, w: 10, h: 10, original: format!("o{i}"), reading: "".into(), translated: t.into(), ai_original: format!("o{i}"), ai_reading: "".into(), ai_translated: t.into(), needs_white_patch: false, is_user_edited: e };
        let prev = PageTranslation { page_id: "p".into(), target_lang: "id".into(), style: "".into(), model: "".into(), bubbles: vec![mk(0, "keep", true), mk(1, "old", false)] };
        let out = preserve_user_edits(vec![mk(0, "ai0", false), mk(1, "ai1", false)], Some(&prev));
        assert_eq!(out[0].translated, "keep");
        assert!(out[0].is_user_edited);
        assert_eq!(out[1].translated, "ai1");
        assert!(!out[1].is_user_edited);
    }

    #[test]
    fn baseline_survives_preserve() {
        // Edited bubble: ai* lama dipertahankan, bukan AI baru yg ditolak.
        let old = BubbleTranslation { index: 0, x: 0, y: 0, w: 10, h: 10, original: "manual".into(), reading: "mr".into(), translated: "user".into(), ai_original: "BASE-O".into(), ai_reading: "BASE-R".into(), ai_translated: "BASE".into(), needs_white_patch: false, is_user_edited: true };
        let fresh = BubbleTranslation { index: 0, x: 0, y: 0, w: 10, h: 10, original: "new-O".into(), reading: "new-R".into(), translated: "new".into(), ai_original: "new-O".into(), ai_reading: "new-R".into(), ai_translated: "new".into(), needs_white_patch: false, is_user_edited: false };
        let prev = PageTranslation { page_id: "p".into(), target_lang: "id".into(), style: "".into(), model: "".into(), bubbles: vec![old] };
        let out = preserve_user_edits(vec![fresh], Some(&prev));
        assert_eq!(out[0].translated, "user");
        assert_eq!(out[0].ai_translated, "BASE");
        assert_eq!(out[0].ai_original, "BASE-O");
    }

    #[test]
    fn save_merge_keeps_stored_baseline() {
        // Frontend kirim manual + ai* kosong/stale → stored yg menang.
        let stored = vec![BubbleTranslation { index: 0, x: 0, y: 0, w: 10, h: 10, original: "x".into(), reading: "".into(), translated: "x".into(), ai_original: "AI-O".into(), ai_reading: "AI-R".into(), ai_translated: "AI".into(), needs_white_patch: false, is_user_edited: false }];
        let incoming = vec![BubbleTranslation { index: 0, x: 0, y: 0, w: 10, h: 10, original: "m".into(), reading: "mr".into(), translated: "user".into(), ai_original: "".into(), ai_reading: "".into(), ai_translated: "".into(), needs_white_patch: false, is_user_edited: true }];
        let out = merge_ai_baseline(&stored, incoming);
        assert_eq!(out[0].translated, "user");
        assert!(out[0].is_user_edited);
        assert_eq!(out[0].ai_translated, "AI");
        assert_eq!(out[0].ai_original, "AI-O");
    }

    #[test]
    fn old_json_without_baseline_loads() {
        let raw = r#"{"index":0,"x":0,"y":0,"w":10,"h":10,"original":"o","reading":"","translated":"t","needsWhitePatch":false,"isUserEdited":false}"#;
        let b: BubbleTranslation = serde_json::from_str(raw).unwrap();
        assert_eq!(b.ai_original, "");
        assert_eq!(b.ai_translated, "");
    }
}
