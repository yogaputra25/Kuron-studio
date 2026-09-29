//! M5-1: Translation Memory — cari terjemahan sebelumnya lintas project.
//! Sumber: `projects.json` (store) — tanpa kolom DB baru, tanpa migrasi.
//! Match: substring case-insensitive pada `original`; skor = jumlah
//! kemunculan query (freq), tiebreak = target lebih pendek dulu.
//! Never throws: store lock gagal → vec kosong.

use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TmHit {
    pub project_name: String,
    pub page_file: String,
    pub bubble_index: usize,
    pub original: String,
    pub translated: String,
    pub score: usize,
}

fn count_occurs(hay: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    hay.match_indices(needle).count()
}

/// Cari `query` di semua bubble yang sudah diterjemahkan.
/// `limit` 0/negatif → default 10; hasil di-cap 50.
pub fn search_memory(state: &tauri::State<'_, AppState>, query: &str, limit: i32) -> Vec<TmHit> {
    let Ok(store) = state.store.lock() else {
        return Vec::new();
    };
    search_store(&store, query, limit)
}

/// Inti murni (tanpa State) agar unit-testable.
fn search_store(store: &crate::commands::project::ProjectStore, query: &str, limit: i32) -> Vec<TmHit> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let limit = (if limit <= 0 { 10 } else { limit.min(50) }) as usize;
    let mut hits: Vec<TmHit> = Vec::new();
    for proj in store.projects.values() {
        for pg in &proj.pages {
            let Some(t) = pg.translation.as_ref() else {
                continue;
            };
            let file = pg.path.split(['/', '\\']).next_back().unwrap_or(&pg.path);
            for b in &t.bubbles {
                if b.translated.trim().is_empty() {
                    continue;
                }
                let n = count_occurs(&b.original.to_lowercase(), &q);
                if n == 0 {
                    continue;
                }
                hits.push(TmHit {
                    project_name: proj.name.clone(),
                    page_file: file.to_string(),
                    bubble_index: b.index,
                    original: b.original.clone(),
                    translated: b.translated.clone(),
                    score: n,
                });
            }
        }
    }
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.translated.len().cmp(&b.translated.len()))
    });
    hits.truncate(limit);
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::project::{Page, PageStatus, Project, ProjectStore};
    use crate::translation::{BubbleTranslation, PageTranslation};

    fn bt(i: usize, o: &str, t: &str) -> BubbleTranslation {
        BubbleTranslation {
            index: i, x: 0, y: 0, w: 10, h: 10,
            original: o.into(), reading: "".into(), translated: t.into(),
            ai_original: o.into(), ai_reading: "".into(), ai_translated: t.into(),
            needs_white_patch: false, is_user_edited: false,
        }
    }

    fn store_with() -> ProjectStore {
        let dir = std::env::temp_dir().join(format!(
            "kuron-tm-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut store = ProjectStore::load(dir).unwrap();
        let t = PageTranslation {
            page_id: "p0".into(), target_lang: "id".into(), style: "s".into(),
            model: "m".into(),
            bubbles: vec![
                bt(0, "naga api besar naga", "naga api besar dan naga lagi"),
                bt(1, "naga kecil", "naga"),
                bt(2, "kosong", ""),
            ],
        };
        store.projects.insert(
            "p".into(),
            Project {
                id: "p".into(), name: "ch1".into(),
                pages: vec![Page {
                    id: "p0".into(), path: "ch/001.png".into(),
                    width: 800, height: 1200,
                    status: PageStatus::Translated,
                    bubbles: vec![], translation: Some(t),
                }],
            },
        );
        store
    }

    #[test]
    fn ranks_by_frequency_then_shorter_target() {
        let s = store_with();
        let hits = search_store(&s, "naga", 10);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].score, 2);
        assert_eq!(hits[0].translated, "naga api besar dan naga lagi");
        assert_eq!(hits[1].page_file, "001.png");
    }

    #[test]
    fn empty_and_case_insensitive() {
        let s = store_with();
        assert!(search_store(&s, "  ", 10).is_empty());
        assert_eq!(search_store(&s, "NAGA", 10).len(), 2);
        assert_eq!(search_store(&s, "NAGA", 1).len(), 1);
    }
}
