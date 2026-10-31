//! M5-2: QA Checks — untranslated, overflow, SFX leak.
//! Murni (tanpa State/lock) agar unit-testable; command `qa_check`
//! memanggilnya per project dari snapshot store.

use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::{lock_or_err, AppError};
use crate::translation::PageTranslation;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QaIssue {
    pub page_file: String,
    pub bubble_index: i32,
    /// "untranslated" | "overflow" | "sfxLeak" | "noBubbles".
    pub kind: String,
    pub detail: String,
}

fn file_of(path: &str) -> &str {
    path.split(['/', '\\']).next_back().unwrap_or(path)
}

/// Panjang wajar: metrik renderer bersama (render::page::text_overflows) —
/// bungkus kata + susut skala seperti paint_patch, bukan hitung char kasar.
/// SFX: original mengandung
/// pola SFX umum (katakana/ganda-konsonan panjang) tapi skipSfx=false
/// dan translated kosong → sfxLeak hanya bila bubble tanpa terjemahan
/// dan teks asli terlihat seperti SFX (non-kanji pendek / onomatope).
pub fn check_page(
    path: &str,
    bubble_count: usize,
    t: Option<&PageTranslation>,
    skip_sfx: bool,
) -> Vec<QaIssue> {
    let file = file_of(path).to_string();
    let Some(t) = t else {
        if bubble_count == 0 {
            return vec![QaIssue {
                page_file: file,
                bubble_index: -1,
                kind: "noBubbles".into(),
                detail: "belum ada bubble/terjemahan".into(),
            }];
        }
        return vec![QaIssue {
            page_file: file,
            bubble_index: -1,
            kind: "untranslated".into(),
            detail: "belum diterjemahkan".into(),
        }];
    };
    let mut out = Vec::new();
    for b in &t.bubbles {
        if b.translated.trim().is_empty() {
            // SFX leak: asli pendek onomatope + skipSfx mati → tandai khusus.
            let kind = if !skip_sfx && looks_sfx(&b.original) {
                "sfxLeak"
            } else {
                "untranslated"
            };
            out.push(QaIssue {
                page_file: file.clone(),
                bubble_index: b.index as i32,
                kind: kind.into(),
                detail: format!("bubble #{} kosong", b.index + 1),
            });
            continue;
        }
        // export-text-parity P7: kalau bubble punya layout dari kanvas, luber
        // dinilai dari layout itu (satu sumber, tidak dihitung ulang).
        // text-fit-polygon: shape absolut -> relatif bubble untuk chord.
        let rel: Option<Vec<[i32; 2]>> = b.shape.as_ref().filter(|p| p.len() >= 3).map(|p| {
            p.iter().map(|[px, py]| [px - b.x, py - b.y]).collect()
        });
        let rel_ref = rel.as_deref();
        let overflow = match &b.layout {
            Some(l) if !l.lines.is_empty() => {
                crate::render::page::layout_overflows(l, b.w, b.h, rel_ref)
            }
            _ => crate::render::page::text_overflows(&b.translated, b.w, b.h, rel_ref),
        };
        if overflow {
            out.push(QaIssue {
                page_file: file.clone(),
                bubble_index: b.index as i32,
                kind: "overflow".into(),
                detail: format!(
                    "teks overflow box {}x{}px",
                    b.w,
                    b.h
                ),
            });
        }
    }
    out
}

/// Heuristik SFX: ≤6 char tanpa spasi, atau katakana/halfwidth kana.
fn looks_sfx(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() || t.contains(' ') || t.contains('　') {
        return false;
    }
    if t.chars().count() <= 6 {
        return true;
    }
    t.chars().any(|c| ('\u{30A0}'..='\u{30FF}').contains(&c) || ('\u{FF61}'..='\u{FF9F}').contains(&c))
}

/// Seluruh project → issues diurut file, bubble.
pub fn check_project(state: &tauri::State<'_, AppState>, project_id: &str) -> Result<Vec<QaIssue>, AppError> {
    let store = lock_or_err(&state.store)?;
    let proj = store
        .projects
        .get(project_id)
        .ok_or_else(|| AppError::project_not_found(project_id))?;
    let pages = proj.sorted_all_pages();
    let mut out = Vec::new();
    for pg in &pages {
        // skipSfx tak tersimpan per-page; default false = ketat.
        out.extend(check_page(&pg.path, pg.bubbles.len(), pg.translation.as_ref(), false));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::BubbleTranslation;

    fn bt(i: usize, w: i32, o: &str, t: &str) -> BubbleTranslation {
        BubbleTranslation {
            index: i, x: 0, y: 0, w, h: 40, shape: None, layout: None, font_size: None,
            original: o.into(), reading: "".into(), translated: t.into(),
            ai_original: o.into(), ai_reading: "".into(), ai_translated: t.into(),
            needs_white_patch: false, is_user_edited: false,
        }
    }

    fn pt(bs: Vec<BubbleTranslation>) -> PageTranslation {
        PageTranslation {
            page_id: "p".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(), manual_done: false, bubbles: bs,
        }
    }

    #[test]
    fn flags_all_three_kinds() {
        let t = pt(vec![
            bt(0, 60, "こんにちは世界", "halo dunia yang sangat panjang melebihi lebar bubble ini jauh"),
            bt(1, 200, "selamat pagi semuanya", ""),
            bt(2, 200, "ドン", ""),
        ]);
        let issues = check_page("ch/001.png", 3, Some(&t), false);
        assert!(issues.iter().any(|i| i.kind == "overflow" && i.bubble_index == 0));
        assert!(issues.iter().any(|i| i.kind == "untranslated" && i.bubble_index == 1));
        assert!(issues.iter().any(|i| i.kind == "sfxLeak" && i.bubble_index == 2));
        assert_eq!(issues[0].page_file, "001.png");
    }

    #[test]
    fn sfx_skipped_when_flag_on() {
        let t = pt(vec![bt(0, 200, "ドン", "")]);
        let issues = check_page("a.png", 1, Some(&t), true);
        assert_eq!(issues[0].kind, "untranslated");
    }

    #[test]
    fn no_translation_page() {
        let issues = check_page("a.png", 2, None, false);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "untranslated");
        assert!(check_page("b.png", 0, None, false)[0].kind == "noBubbles");
    }
}
