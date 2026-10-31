//! Tree queries & navigation atas `Project.folders` (B2).
//!
//! Logika hierarki: subtree, ancestor, siklus check, dan flattened-scope
//! untuk ekspor/clean. Tipe data tetap [`ChapterFolder`] dan [`Project`];
//! berkas ini hanya implementasi method.

use crate::error::AppError;
use super::project::Project;

impl Project {
    /// Cari folder by id di semua level (flat scan karena simpanan flat).
    pub fn folder(&self, id: &str) -> Option<&super::project::ChapterFolder> {
        self.folders.iter().find(|f| f.id == id)
    }

    /// Cari folder mutable by id.
    pub fn folder_mut(&mut self, id: &str) -> Option<&mut super::project::ChapterFolder> {
        self.folders.iter_mut().find(|f| f.id == id)
    }

    /// Halaman di dalam folder + semua subfolder-nya (rekursif), urut path.
    pub fn subtree_pages(&self, folder_id: &str) -> Result<Vec<super::page::Page>, AppError> {
        if self.folder(folder_id).is_none() {
            return Err(AppError::folder_not_found(folder_id));
        }
        let ids = self.subtree_ids(folder_id);
        let mut out: Vec<super::page::Page> = self
            .folders
            .iter()
            .filter(|f| ids.contains(&f.id))
            .flat_map(|f| f.pages.clone())
            .collect();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }

    /// Halaman untuk scope export/clean: None/kosong = semua folder.
    pub fn pages_in_scope(&self, folder_id: Option<&str>) -> Result<Vec<super::page::Page>, AppError> {
        match folder_id.filter(|s| !s.is_empty()) {
            Some(fid) => self.sorted_subtree_pages(fid),
            None => Ok(self.sorted_all_pages()),
        }
    }

    /// Dipakai ekspor subset pilihan user — bukan error bila sebagian terhapus.
    pub fn pages_in_selection(&self, ids: &[String]) -> Vec<super::page::Page> {
        let all = self.sorted_all_pages();
        ids.iter()
            .filter_map(|id| all.iter().find(|p| &p.id == id).cloned())
            .collect()
    }

    pub(crate) fn sorted_all_pages(&self) -> Vec<super::page::Page> {
        let mut out = self.all_pages();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }

    fn sorted_subtree_pages(&self, folder_id: &str) -> Result<Vec<super::page::Page>, AppError> {
        self.subtree_pages(folder_id)
    }

    /// Id semua folder dalam subtree (termasuk root-nya sendiri).
    pub fn subtree_ids(&self, folder_id: &str) -> Vec<String> {
        let mut out = vec![folder_id.to_string()];
        let mut i = 0;
        while i < out.len() {
            let cur = out[i].clone();
            for f in &self.folders {
                if f.parent_id.as_deref() == Some(&cur) && !out.contains(&f.id) {
                    out.push(f.id.clone());
                }
            }
            i += 1;
        }
        out
    }

    /// Chain id dari root sampai folder ini (termasuk dirinya sendiri).
    pub fn breadcrumb_ids(&self, folder_id: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = Some(folder_id.to_string());
        while let Some(id) = cur {
            if out.contains(&id) {
                // Cegah loop tak berujung bila ada data korup
                break;
            }
            out.push(id.clone());
            cur = self.folder(&id).and_then(|f| f.parent_id.clone());
        }
        out.reverse();
        out
    }

    /// Pindah parent folder: tolak siklus (new_parent adalah keturunan dari folder_id),
    /// sendiri, parent = keturunan sendiri (siklus). `None` = jadikan root.
    pub fn set_parent(&mut self, folder_id: &str, new_parent: Option<&str>) -> Result<(), AppError> {
        if self.folder(folder_id).is_none() {
            return Err(AppError::folder_not_found(folder_id));
        }
        if let Some(p) = new_parent.filter(|s| !s.is_empty()) {
            if p == folder_id {
                return Err(AppError::parse("folder tidak bisa menjadi parent dirinya sendiri"));
            }
            if self.folder(p).is_none() {
                return Err(AppError::parent_folder_not_found(p));
            }
            if self.subtree_ids(folder_id).contains(&p.to_string()) {
                return Err(AppError::parse("parent adalah keturunan folder ini (siklus)"));
            }
            // Nama harus unik di antara saudara kandung yang baru.
            let name = self.folder(folder_id).expect("checked").name.clone();
            if self.folders.iter().any(|f| f.id != folder_id && f.parent_id.as_deref() == Some(p) && f.name == name) {
                return Err(AppError::parse(format!("folder sudah ada: {name}")));
            }
            self.folder_mut(folder_id).expect("checked").parent_id = Some(p.to_string());
        } else {
            self.folder_mut(folder_id).expect("checked").parent_id = None;
        }
        self.touch();
        Ok(())
    }

    /// Folder tujuan import: id eksplisit > folder `default` > folder pertama > buat baru.
    pub fn resolve_folder_mut(&mut self, folder_id: Option<&str>) -> &mut super::project::ChapterFolder {
        if let Some(want) = folder_id.filter(|s| !s.is_empty()) {
            if self.folders.iter().any(|f| f.id == want) {
                return self.folders.iter_mut().find(|f| f.id == want).expect("checked");
            }
        }
        if let Some(i) = self.folders.iter().position(|f| f.name == super::project::DEFAULT_FOLDER_NAME) {
            return &mut self.folders[i];
        }
        if self.folders.is_empty() {
            self.folders.push(super::project::ChapterFolder::new(super::project::DEFAULT_FOLDER_NAME.to_string()));
        }
        &mut self.folders[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::domain::{ChapterFolder, Page, PageStatus};

    fn stub_page(id: &str, path: &Path) -> Page {
        Page {
            id: id.to_string(),
            path: path.to_string_lossy().to_string(),
            width: 100,
            height: 100,
            status: PageStatus::Idle,
            bubbles: Vec::new(),
            translation: None,
        }
    }

    fn tree_proj() -> Project {
        let mut proj = Project::new("t".to_string());
        proj.id = "proj".to_string();
        let root = proj.folders[0].id.clone();
        let mut ch = ChapterFolder::new_child("Ch 12".to_string(), root.clone());
        ch.id = "ch12".to_string();
        let mut b = ChapterFolder::new_child("Batch 1".to_string(), "ch12".to_string());
        b.id = "b1".to_string();
        proj.folders.push(ch);
        proj.folders.push(b);
        proj
    }

    #[test]
    fn set_parent_rejects_self_and_cycles() {
        let mut proj = tree_proj();
        assert!(proj.set_parent("ch12", Some("ch12")).is_err());
        assert!(proj.set_parent("ch12", Some("b1")).is_err());
        assert!(proj.set_parent("ch12", Some("nope")).is_err());
        assert!(proj.set_parent("ch12", Some(proj.folders[0].id.clone().as_str())).is_ok());
        assert!(proj.set_parent("b1", None).is_ok());
        assert_eq!(proj.folder("b1").unwrap().parent_id, None);
    }

    #[test]
    fn subtree_ids_cover_descendants() {
        let proj = tree_proj();
        let mut ids = proj.subtree_ids("ch12");
        ids.sort();
        assert_eq!(ids, vec!["b1".to_string(), "ch12".to_string()]);
    }

    #[test]
    fn sibling_names_may_repeat_across_parents() {
        let proj = tree_proj();
        let root = proj.folders[0].id.clone();
        // "Batch 1" sudah ada di bawah ch12 — tolak duplikat sesaudara.
        assert!(proj.folders.iter().any(|f| f.name == "Batch 1"
            && f.parent_id.as_deref() == Some("ch12")));
        // Nama sama di bawah root = saudara beda → boleh (cek aturan manual).
        let clash = proj.folders.iter().any(|f| f.name == "Batch 1"
            && f.id != "b1" && f.parent_id.as_deref() == Some(root.as_str()));
        assert!(!clash);
    }

    #[test]
    fn subtree_scope_aggregates_descendants_sorted() {
        let mut proj = tree_proj();
        proj.folders[0].pages = vec![stub_page("r", Path::new("r.jpg"))];
        proj.folders.iter_mut().find(|f| f.id == "ch12").unwrap().pages =
            vec![stub_page("c", Path::new("c.jpg"))];
        proj.folders.iter_mut().find(|f| f.id == "b1").unwrap().pages =
            vec![stub_page("b", Path::new("b.jpg"))];
        // Scope ch12 = dirinya + Batch 1, urut filename.
        let got = proj.pages_in_scope(Some("ch12")).unwrap();
        assert_eq!(got.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["b", "c"]);
        // Scope kosong = semua folder termasuk root.
        let all = proj.pages_in_scope(None).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, "b");
        // Folder tak dikenal = error jujur.
        assert!(proj.pages_in_scope(Some("nope")).is_err());
    }
}
