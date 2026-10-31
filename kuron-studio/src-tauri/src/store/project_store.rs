//! `ProjectStore`: muat/simpan `projects.json`.
//! Dipindah verbatim dari `commands/project.rs` (#52); hanya `save`/`load`
//! yang berubah (atomik + backup + recovery). Bentuk JSON tidak berubah.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{normalize_page_status, Page, Project};
use crate::error::AppError;

use super::atomic;
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStore {
    #[serde(skip)]
    pub(crate) dir: PathBuf,
    pub projects: HashMap<String, Project>,
}

impl ProjectStore {
    pub fn load(dir: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&dir)?;
        let file = dir.join("projects.json");
        let bak = atomic::bak_file(&file);
        if !file.exists() && !bak.exists() {
            return Ok(Self {
                dir,
                projects: HashMap::new(),
            });
        }
        match Self::load_file(&dir, &file) {
            Ok(s) => Ok(s),
            Err(main_err) => match Self::load_file(&dir, &bak) {
                Ok(s) => {
                    crate::logging::warn(
                        "store.recovered_from_backup",
                        serde_json::json!({ "file": file.display().to_string() }),
                    );
                    // Sembuhkan main dari backup supaya crash berikut aman.
                    let _ = s.save();
                    Ok(s)
                }
                Err(_) => Err(main_err),
            },
        }
    }

    fn load_file(dir: &Path, file: &Path) -> Result<Self, AppError> {
        let raw = fs::read_to_string(file)?;
        let mut store: ProjectStore =
            serde_json::from_str(&raw).map_err(AppError::corrupt_store)?;
        store.dir = dir.to_path_buf();
        // Migrasi #7: flat lama { pages[] } -> folders [default]. Idempoten,
        // disimpan langsung supaya file di disk ikut bentuk baru.
        let mut migrated = false;
        for proj in store.projects.values_mut() {
            if proj.migrate_legacy() { migrated = true; }
            // page-status-truth: normalisasi status simpanan (transient
            // gantung + translated yang isinya sebagian tanpa flag manual).
            for folder in proj.folders.iter_mut() {
                for pg in folder.pages.iter_mut() {
                    if normalize_page_status(pg) { migrated = true; }
                }
            }
        }
        if migrated { let _ = store.save(); }
        Ok(store)
    }

    pub(crate) fn save(&self) -> Result<(), AppError> {
        let raw = serde_json::to_string_pretty(self)?;
        let file = self.dir.join("projects.json");
        // Putar backup dulu (hanya bila main valid), lalu tulis atomik.
        atomic::rotate_backup(&file)?;
        atomic::write_atomic(&file, raw.as_bytes())
    }

    pub(crate) fn save_public(&self) -> Result<(), AppError> {
        self.save()
    }

    pub(crate) fn get_mut(&mut self, project_id: &str) -> Result<&mut Project, AppError> {
        self.projects
            .get_mut(project_id)
            .ok_or_else(|| AppError::project_not_found(project_id))
    }

    /// Cari halaman di semua project/folder. Mengembalikan path saja (read-only).
    pub fn page_path(&self, page_id: &str) -> Option<String> {
        self.projects.values()
            .flat_map(|p| p.folders.iter())
            .flat_map(|f| f.pages.iter())
            .find(|pg| pg.id == page_id)
            .map(|pg| pg.path.clone())
    }

    /// Cari halaman mutable + touch project pemiliknya. Dipakai detect/translate.
    pub fn find_page_mut(&mut self, page_id: &str) -> Option<&mut Page> {
        for proj in self.projects.values_mut() {
            let found = proj.folders.iter().any(|f| f.pages.iter().any(|pg| pg.id == page_id));
            if found {
                proj.touch();
                for folder in proj.folders.iter_mut() {
                    if let Some(pos) = folder.pages.iter().position(|pg| pg.id == page_id) {
                        return Some(&mut folder.pages[pos]);
                    }
                }
            }
        }
        None
    }

    pub fn find_page(&self, page_id: &str) -> Option<Page> {
        self.projects.values()
            .flat_map(|p| p.folders.iter())
            .flat_map(|f| f.pages.iter())
            .find(|pg| pg.id == page_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PageStatus;

    /// Unik per test — `SystemTime` bisa kembalikan nanosecond yang sama.
    fn temp_dir(tag: &str) -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kuron-store-{tag}-{}-{n}", std::process::id()))
    }

    fn mk_page(id: &str) -> Page {
        Page {
            id: id.into(),
            path: format!("/{id}.jpg"),
            width: 10,
            height: 10,
            status: PageStatus::Detected,
            bubbles: vec![],
            translation: None,
        }
    }

    fn stocked(dir: &Path, n_pages: usize) -> ProjectStore {
        let mut s = ProjectStore::load(dir.to_path_buf()).unwrap();
        let mut p = Project::new("P".to_string());
        for i in 0..n_pages {
            p.folders[0].pages.push(mk_page(&format!("p{i}")));
        }
        s.projects.insert(p.id.clone(), p);
        s.save().unwrap();
        s
    }

    #[test]
    fn roundtrip_save_load() {
        let dir = temp_dir("roundtrip");
        let s = stocked(&dir, 3);
        let r = ProjectStore::load(dir).unwrap();
        assert_eq!(r.projects.len(), 1);
        assert_eq!(r.find_page("p1").unwrap().path, "/p1.jpg");
        assert_eq!(s.find_page("p2").unwrap().path, r.find_page("p2").unwrap().path);
    }

    #[test]
    fn corrupt_main_errors_jelas() {
        let dir = temp_dir("corrupt");
        stocked(&dir, 1);
        fs::write(dir.join("projects.json"), "{bukan json").unwrap();
        let err = ProjectStore::load(dir).unwrap_err();
        assert!(err.to_string().contains("corrupt"), "pesan tak jelas: {err}");
    }

    #[test]
    fn corrupt_main_pulih_dari_bak() {
        let dir = temp_dir("bak");
        // Gen A di main, lalu gen B (bak = A).
        stocked(&dir, 1);
        let mut s = ProjectStore::load(dir.clone()).unwrap();
        let pid = s.projects.keys().next().unwrap().clone();
        s.projects.get_mut(&pid).unwrap().folders[0].pages.push(mk_page("baru"));
        s.save().unwrap();
        assert!(atomic::bak_file(&dir.join("projects.json")).exists());
        // Crash: main korup.
        fs::write(dir.join("projects.json"), "{bukan json").unwrap();
        // Pulih ke gen A (last-known-good), B yang belum aman hilang.
        let r = ProjectStore::load(dir.clone()).unwrap();
        assert!(r.find_page("p0").is_some());
        assert!(r.find_page("baru").is_none());
        // Main sembuh: load berikut normal.
        assert!(ProjectStore::load(dir).is_ok());
    }

    #[test]
    fn tmp_leftover_diabaikan() {
        let dir = temp_dir("tmp");
        stocked(&dir, 2);
        fs::write(dir.join("projects.json.tmp"), "{sampah crash}").unwrap();
        let r = ProjectStore::load(dir).unwrap();
        assert_eq!(r.projects.values().next().unwrap().folders[0].pages.len(), 2);
    }

    #[test]
    fn tulis_seribu_kali_tanpa_parsial() {
        let dir = temp_dir("loop");
        let mut s = ProjectStore::load(dir.clone()).unwrap();
        let mut p = Project::new("P".to_string());
        s.projects.insert(p.id.clone(), p.clone());
        for i in 0..1000 {
            p = s.projects.values().next().unwrap().clone();
            p.name = format!("gen-{i}");
            s.projects.insert(p.id.clone(), p.clone());
            s.save().unwrap();
            if i % 100 == 0 {
                let r = ProjectStore::load(dir.clone()).unwrap();
                assert_eq!(r.projects.values().next().unwrap().name, format!("gen-{i}"));
            }
        }
        let r = ProjectStore::load(dir).unwrap();
        assert_eq!(r.projects.values().next().unwrap().name, "gen-999");
    }

    #[test]
    fn bubbles_survive_save_reload() {
        use crate::bubble::BubbleBox;
        let dir = temp_dir("test-bubbles");
        let mut store = ProjectStore::load(dir.clone()).unwrap();

        let mut project = Project::new("Test".to_string());
        let page_id = "page-1".to_string();
        project.folders[0].pages.push(Page {
            id: page_id.clone(),
            path: "/dummy/001.png".to_string(),
            width: 800,
            height: 1200,
            status: PageStatus::Idle,
            bubbles: vec![],
            translation: None,
        });
        store.projects.insert(project.id.clone(), project.clone());
        store.save().unwrap();

        let bubbles = vec![
            BubbleBox {
                x: 10,
                y: 20,
                w: 100,
                h: 80,
                confidence: 0.95,
                shape: None,
                kind: None,
                tail: None,
            },
            BubbleBox {
                x: 150,
                y: 200,
                w: 80,
                h: 60,
                confidence: 0.88,
                shape: None,
                kind: None,
                tail: None,
            },
        ];

        let p = store.get_mut(&project.id).unwrap();
        let page = p.folders[0].pages.iter_mut().find(|p| p.id == page_id).unwrap();
        page.bubbles = bubbles.clone();
        page.status = PageStatus::Detected;
        store.save().unwrap();

        let reloaded = ProjectStore::load(dir.clone()).unwrap();
        let loaded_proj = reloaded.projects.get(&project.id).unwrap();
        let loaded_page = loaded_proj.folders[0].pages.iter().find(|p| p.id == page_id).unwrap();

        assert_eq!(loaded_page.status, PageStatus::Detected);
        assert_eq!(loaded_page.bubbles.len(), 2);
        assert_eq!(loaded_page.bubbles[0].x, 10);
        assert_eq!(loaded_page.bubbles[0].w, 100);
        assert_eq!(loaded_page.bubbles[1].confidence, 0.88);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_normalizes_stale_status() {
        use crate::bubble::BubbleBox;
        use crate::translation::{BubbleTranslation, PageTranslation};
        let dir = temp_dir("normalize");
        let bb = BubbleBox { x: 0, y: 0, w: 10, h: 10, confidence: 1.0, shape: None, kind: None, tail: None };
        let tb = |t: &str| BubbleTranslation { index: 0, x: 0, y: 0, w: 10, h: 10, shape: None, layout: None, font_size: None, original: "o".into(), reading: "".into(), translated: t.into(), ai_original: "".into(), ai_reading: "".into(), ai_translated: "".into(), needs_white_patch: false, is_user_edited: false };
        let tr = |bubbles: Vec<BubbleTranslation>, manual_done: bool| PageTranslation { page_id: "x".into(), target_lang: "id".into(), style: "s".into(), model: "m".into(), manual_done, bubbles };
        let page = |id: &str, status: PageStatus, bubbles: Vec<BubbleBox>, translation: Option<PageTranslation>| Page {
            id: id.into(), path: format!("{id}.png"), width: 800, height: 1200,
            status, bubbles, translation,
        };
        let mut store = ProjectStore::load(dir.clone()).unwrap();
        let mut proj = Project::new("t".to_string());
        proj.id = "proj".to_string();
        proj.folders[0].pages = vec![
            page("gantung", PageStatus::Translating, vec![bb.clone()], None),
            page("over", PageStatus::Translated, vec![bb.clone()], Some(tr(vec![tb("isi"), tb("")], false))),
            page("manual", PageStatus::Translated, vec![bb.clone()], Some(tr(vec![tb("isi"), tb("")], true))),
            page("utang", PageStatus::Translated, vec![bb.clone()], Some(tr(vec![tb("a"), tb("b")], false))),
            page("kosong", PageStatus::Detecting, vec![], None),
        ];
        store.projects.insert("proj".to_string(), proj);
        store.save().unwrap();
        let reloaded = ProjectStore::load(dir.clone()).unwrap();
        let st = |id: &str| {
            reloaded.projects["proj"].folders[0]
                .pages.iter().find(|p| p.id == id).unwrap().status.clone()
        };
        assert_eq!(st("gantung"), PageStatus::Detected);
        assert_eq!(st("over"), PageStatus::Detected);
        assert_eq!(st("manual"), PageStatus::Translated);
        assert_eq!(st("utang"), PageStatus::Translated);
        assert_eq!(st("kosong"), PageStatus::Idle);
        fs::remove_dir_all(&dir).ok();
    }
}
